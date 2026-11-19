#!/usr/bin/env python3
"""browser.py — a basic terminal web browser, plus a phext-over-HTTP streamer.

An RFC Box run over ietf.phext (task 51.6.7/6.5.9/4.5.11): the requirement went
in, a slice of the corpus came out (SLICE.md), and this program is composed
from that slice using Python's standard library only — the corpus records
CPython (stdlib) as an implementation of RFC 3986 and RFC 2045/2046, and the
ssl module is OpenSSL, the corpus's canonical TLS implementation (RFC 9846).

Two modes:

  browser.py <url> [--link N]   Fetch a page, render it as terminal text
                                (headings, paragraphs, lists, numbered links),
                                show its title. --link N then follows link N.
                                Without --link on a terminal, an interactive
                                loop offers link numbers / new URLs / q.

  browser.py phext <url> [--select COORD]
                                Fetch a .phext over HTTP/1.1 and process it
                                AS A STREAM: scrolls are split on the phext
                                dimension delimiters as bytes arrive (chunked
                                transfer coding, RFC 9112 section 7.1), and
                                each completed scroll's coordinate is listed
                                (the table of contents) as it lands. --select
                                prints just that scroll's text.

Phext format note: the 9 dimension delimiter bytes and the coordinate state
machine follow the reference builders in ~/workspace/phext/ (LEVEL_BYTES,
library..scroll). Phext is Will's format, not an IETF RFC — see GAP.md.

Standard library only: http.client, ssl, html.parser, urllib.parse.
"""

import ssl
import sys
from html.parser import HTMLParser
from urllib.parse import urljoin, urlsplit, urlunsplit

USER_AGENT = "rfcbox-browser/0.1 (RFC Box run 51.6.7/6.5.9/4.5.11; stdlib)"
MAX_REDIRECTS = 10
TIMEOUT = 15

REDIRECT_STATUSES = (301, 302, 303, 307, 308)


class BrowserError(Exception):
    pass


# --------------------------------------------------------------------------
# Cookies — RFC 6265, deliberately small subset: host-only session cookies.
# Set-Cookie name=value pairs are stored per host and sent back to that host.
# Path/Domain/Expires/Secure attributes are parsed off and ignored; this is
# a reader, not a session manager. The subset is recorded in SLICE.md.
# --------------------------------------------------------------------------

class CookieJar:
    def __init__(self):
        self._by_host = {}  # host -> {name: value}

    def extract(self, url, msg):
        host = urlsplit(url).hostname
        if not host:
            return
        for header in msg.get_all("Set-Cookie", []):
            pair = header.split(";", 1)[0].strip()
            if "=" not in pair:
                continue
            name, value = pair.split("=", 1)
            self._by_host.setdefault(host, {})[name.strip()] = value.strip()

    def header_for(self, url):
        host = urlsplit(url).hostname
        cookies = self._by_host.get(host or "", {})
        if not cookies:
            return None
        return "; ".join(f"{k}={v}" for k, v in cookies.items())


# --------------------------------------------------------------------------
# Fetching — RFC 9110 (semantics) over RFC 9112 (HTTP/1.1), RFC 3986
# (URI parsing and reference resolution), RFC 9846 (TLS 1.3 via OpenSSL),
# RFC 1035 (name resolution, delegated to the system resolver).
# --------------------------------------------------------------------------

def _open(url, jar):
    """Issue one GET. Returns (conn, response, final_url_after_this_hop)."""
    parts = urlsplit(url)
    if parts.scheme not in ("http", "https"):
        raise BrowserError(f"unsupported scheme: {parts.scheme!r} (http/https only)")
    if not parts.hostname:
        raise BrowserError(f"URL has no host: {url}")
    path = urlunsplit(("", "", parts.path or "/", parts.query, ""))
    headers = {
        "User-Agent": USER_AGENT,
        "Accept": "text/html, text/plain, */*",
        "Accept-Encoding": "identity",
        "Connection": "close",
    }
    if jar is not None:
        cookie = jar.header_for(url)
        if cookie:
            headers["Cookie"] = cookie
    if parts.scheme == "https":
        import http.client
        context = ssl.create_default_context()  # system trust store
        conn = http.client.HTTPSConnection(
            parts.hostname, parts.port or 443, timeout=TIMEOUT, context=context)
    else:
        import http.client
        conn = http.client.HTTPConnection(
            parts.hostname, parts.port or 80, timeout=TIMEOUT)
    conn.request("GET", path, headers=headers)
    return conn, conn.getresponse()


class FetchResult:
    def __init__(self, url, status, reason, headers, body, history):
        self.url = url                # final URL after redirects
        self.status = status
        self.reason = reason
        self.headers = headers        # email.message.HTTPMessage
        self.body = body              # bytes
        self.history = history        # [(status, url)] redirect hops

    @property
    def content_type(self):
        return self.headers.get_content_type() if self.headers else ""

    def text(self):
        """Decode the body: MIME charset parameter (RFC 2045) if given,
        else UTF-8 (RFC 3629)."""
        charset = None
        if self.headers is not None:
            charset = self.headers.get_param("charset")
        return self.body.decode(charset or "utf-8", errors="replace")


def fetch(url, jar=None):
    """GET with redirect following (RFC 9110 section 15.4). HTTP error
    statuses (404 etc.) are returned, not raised — a browser reports them."""
    history = []
    current = url
    for _ in range(MAX_REDIRECTS + 1):
        conn, resp = _open(current, jar)
        try:
            if jar is not None:
                jar.extract(current, resp.msg)
            if resp.status in REDIRECT_STATUSES:
                location = resp.getheader("Location")
                if location:
                    resp.read()
                    history.append((resp.status, current))
                    current = urljoin(current, location)  # RFC 3986 section 5
                    continue
            body = resp.read()
            return FetchResult(current, resp.status, resp.reason,
                               resp.msg, body, history)
        finally:
            conn.close()
    raise BrowserError(f"too many redirects (>{MAX_REDIRECTS}) from {url}")


def open_stream(url, jar=None):
    """Like fetch(), but returns (final_url, response, conn) with the body
    unread, so the caller can consume it incrementally. Used by phext mode:
    with Transfer-Encoding: chunked (RFC 9112 section 7.1), http.client
    de-chunks as read() is called — the body is never held whole."""
    current = url
    for _ in range(MAX_REDIRECTS + 1):
        conn, resp = _open(current, jar)
        if jar is not None:
            jar.extract(current, resp.msg)
        if resp.status in REDIRECT_STATUSES:
            location = resp.getheader("Location")
            if location:
                resp.read()
                conn.close()
                current = urljoin(current, location)
                continue
        if resp.status != 200:
            resp.read()
            conn.close()
            raise BrowserError(f"HTTP {resp.status} {resp.reason} for {current}")
        return current, resp, conn
    raise BrowserError(f"too many redirects (>{MAX_REDIRECTS}) from {url}")


# --------------------------------------------------------------------------
# Rendering — HTML to terminal text. HTML itself is a WHATWG living
# standard, NOT an IETF RFC; this renderer is a small, honest subset
# (headings, paragraphs, lists, links, pre) — see GAP.md.
# --------------------------------------------------------------------------

_BLOCK_TAGS = {"p", "div", "section", "article", "header", "footer",
               "blockquote", "tr", "table", "form", "main", "nav"}
_HEADING_TAGS = {"h1", "h2", "h3", "h4", "h5", "h6"}
_SKIP_TAGS = {"script", "style"}


class Renderer(HTMLParser):
    def __init__(self, base_url):
        super().__init__(convert_charrefs=True)
        self.base_url = base_url
        self.title = None
        self.lines = []
        self.links = []          # [(link_text, absolute_url)]
        self._buf = []
        self._prefix = ""        # pending line prefix (heading/list marker)
        self._in_title = False
        self._title_buf = []
        self._skip_depth = 0
        self._list_stack = []    # ["ul"] or ["ol", next_number]
        self._link_href = None
        self._link_buf_start = 0
        self._in_pre = False

    # -- helpers ---------------------------------------------------------
    def _flush(self):
        text = "".join(self._buf)
        self._buf = []
        if self._in_pre:
            for line in text.split("\n"):
                self.lines.append(line.rstrip())
        else:
            collapsed = " ".join(text.split())
            if collapsed:
                self.lines.append(self._prefix + collapsed)
        self._prefix = ""

    def _raw_text(self):
        return " ".join("".join(self._buf).split())

    # -- parser callbacks --------------------------------------------------
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag in _SKIP_TAGS:
            self._skip_depth += 1
        elif tag == "title":
            self._in_title = True
        elif tag in _HEADING_TAGS:
            self._flush()
            self._prefix = "#" * int(tag[1]) + " "
        elif tag in _BLOCK_TAGS:
            self._flush()
        elif tag == "br":
            self._flush()
        elif tag == "ul":
            self._flush()
            self._list_stack.append(["ul"])
        elif tag == "ol":
            self._flush()
            self._list_stack.append(["ol", 1])
        elif tag == "li":
            self._flush()
            if self._list_stack and self._list_stack[-1][0] == "ol":
                top = self._list_stack[-1]
                self._prefix = f"  {top[1]}. "
                top[1] += 1
            else:
                self._prefix = "  • "
        elif tag == "a":
            self._link_href = attrs.get("href")
            self._link_buf_start = len(self._buf)
        elif tag == "img":
            alt = attrs.get("alt")
            if alt:
                self._buf.append(f"[image: {alt}]")
        elif tag == "pre":
            self._flush()
            self._in_pre = True

    def handle_endtag(self, tag):
        if tag in _SKIP_TAGS:
            self._skip_depth = max(0, self._skip_depth - 1)
        elif tag == "title":
            self._in_title = False
            self.title = " ".join("".join(self._title_buf).split()) or None
        elif tag in _HEADING_TAGS or tag in _BLOCK_TAGS:
            self._flush()
        elif tag in ("ul", "ol"):
            self._flush()
            if self._list_stack:
                self._list_stack.pop()
        elif tag == "li":
            self._flush()
        elif tag == "a":
            if self._link_href:
                raw = "".join(self._buf[self._link_buf_start:])
                text = " ".join(raw.split()) or self._link_href
                absolute = urljoin(self.base_url, self._link_href)
                self.links.append((text, absolute))
                self._buf.append(f"[{len(self.links)}]")
            self._link_href = None
        elif tag == "pre":
            self._flush()
            self._in_pre = False

    def handle_data(self, data):
        if self._skip_depth:
            return
        if self._in_title:
            self._title_buf.append(data)
        elif self._in_pre:
            self._buf.append(data)
        else:
            self._buf.append(data)

    def rendered_text(self):
        self._flush()
        return "\n".join(self.lines)


class Page:
    def __init__(self, result):
        self.result = result
        self.url = result.url
        self.status = result.status
        if result.content_type == "text/html":
            renderer = Renderer(result.url)
            renderer.feed(result.text())
            renderer.close()
            self.title = renderer.title
            self.text = renderer.rendered_text()
            self.links = renderer.links
        else:
            self.title = None
            self.text = result.text()
            self.links = []


def fetch_page(url, jar=None):
    return Page(fetch(url, jar=jar))


def follow(page, number):
    """Follow link `number` (1-based, as printed) from a rendered page."""
    if not 1 <= number <= len(page.links):
        raise BrowserError(
            f"link {number} out of range (page has {len(page.links)} links)")
    return fetch_page(page.links[number - 1][1])


def display(page, out=None):
    out = out or sys.stdout
    title = page.title or "(no title)"
    print(f"--- {page.status} · {page.url}", file=out)
    print(f"Title: {title}", file=out)
    if page.result.history:
        hops = " -> ".join(f"{s} {u}" for s, u in page.result.history)
        print(f"Redirects: {hops}", file=out)
    print(file=out)
    print(page.text, file=out)
    if page.links:
        print("\nLinks:", file=out)
        for i, (text, url) in enumerate(page.links, 1):
            print(f"  [{i}] {text} — {url}", file=out)


# --------------------------------------------------------------------------
# Phext streaming — dimension delimiters and the coordinate state machine
# follow the reference builders in ~/workspace/phext/ (build_tasks_phext.py:
# LEVEL_BYTES, library..scroll; implode/explode state machine). A scroll
# exists when it holds at least one byte; empty segments produced while
# advancing several dimensions are transit, not scrolls.
# --------------------------------------------------------------------------

LEVEL_BYTES = [0x01, 0x1F, 0x1E, 0x1D, 0x1C, 0x1A, 0x19, 0x18, 0x17]  # library..scroll
_LEVEL_INDEX = {b: i for i, b in enumerate(LEVEL_BYTES)}


def coord_str(state):
    s = state
    return f"{s[0]}.{s[1]}.{s[2]}/{s[3]}.{s[4]}.{s[5]}/{s[6]}.{s[7]}.{s[8]}"


class PhextStreamParser:
    """Incremental phext splitter. Feed bytes in any chunking — a delimiter
    arriving at a chunk edge, or a scroll spanning many chunks, is handled
    identically, because splitting is a byte-at-a-time state machine over
    the stream, never over a whole-file string."""

    def __init__(self):
        self.state = [1] * 9
        self._buf = bytearray()

    def feed(self, data):
        """Returns [(coordinate, scroll_bytes)] completed by this data."""
        completed = []
        for byte in data:
            level = _LEVEL_INDEX.get(byte)
            if level is None:
                self._buf.append(byte)
            else:
                completed.append((coord_str(self.state), bytes(self._buf)))
                self._buf.clear()
                self.state[level] += 1
                for j in range(level + 1, 9):
                    self.state[j] = 1
        return completed

    def finish(self):
        """The final scroll: (coordinate, bytes) for whatever remains."""
        return coord_str(self.state), bytes(self._buf)


def iter_phext_scrolls(url, jar=None, chunk_size=4096):
    """Stream a .phext from an HTTP/1.1 endpoint, yielding
    (coordinate, scroll_bytes) for each non-empty scroll AS IT COMPLETES —
    the first scroll is yielded before the last byte arrives."""
    _final_url, resp, conn = open_stream(url, jar=jar)
    parser = PhextStreamParser()
    # read1(), not read(): read(n) on a chunked response keeps reading
    # until n bytes or EOF, which would defeat streaming. read1() returns
    # as soon as one chunk (or part of one) is available — one network
    # read, de-chunked — so scrolls complete mid-stream, as they should.
    read_some = getattr(resp, "read1", None) or resp.read
    try:
        while True:
            chunk = read_some(chunk_size)
            if not chunk:
                break
            for coord, scroll in parser.feed(chunk):
                if scroll:
                    yield coord, scroll
        coord, scroll = parser.finish()
        if scroll:
            yield coord, scroll
    finally:
        conn.close()


def phext_toc(url, jar=None):
    """The table of contents, as it streams: [(coordinate, byte_count)]."""
    return [(coord, len(scroll)) for coord, scroll in iter_phext_scrolls(url, jar=jar)]


def phext_select(url, wanted, jar=None):
    """Stream until the scroll at `wanted` completes; return its bytes
    (or None). Stops reading as soon as the scroll lands — later scrolls
    are never fetched."""
    for coord, scroll in iter_phext_scrolls(url, jar=jar):
        if coord == wanted:
            return scroll
    return None


def _first_line(scroll_bytes):
    text = scroll_bytes.decode("utf-8", errors="replace")
    for line in text.splitlines():
        if line.strip():
            return line.strip()[:60]
    return ""


def run_phext(url, select=None, jar=None):
    if select:
        scroll = phext_select(url, select, jar=jar)
        if scroll is None:
            print(f"scroll not found: {select}", file=sys.stderr)
            return 1
        sys.stdout.write(scroll.decode("utf-8", errors="replace"))
        return 0
    print(f"Streaming phext from {url} — table of contents:")
    for coord, scroll in iter_phext_scrolls(url, jar=jar):
        print(f"  {coord}  ({len(scroll)} bytes)  {_first_line(scroll)}")
    return 0


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------

def _interactive(start_url, jar):
    page = fetch_page(start_url, jar=jar)
    while True:
        display(page)
        try:
            command = input("\n[link # / url / q] > ").strip()
        except EOFError:
            return 0
        if command.lower() in ("q", "quit", "exit"):
            return 0
        if not command:
            continue
        try:
            if command.isdigit():
                page = follow(page, int(command))
            else:
                url = command if "://" in command else "https://" + command
                page = fetch_page(url, jar=jar)
        except (BrowserError, OSError) as exc:
            print(f"error: {exc}", file=sys.stderr)


def main(argv):
    args = argv[1:]
    jar = CookieJar()
    if args and args[0] == "phext":
        rest = args[1:]
        if not rest:
            print("usage: browser.py phext <url> [--select COORD]", file=sys.stderr)
            return 2
        url = rest[0]
        select = None
        if "--select" in rest:
            select = rest[rest.index("--select") + 1]
        try:
            return run_phext(url, select=select, jar=jar)
        except (BrowserError, OSError) as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 1
    if not args:
        print("usage: browser.py <url> [--link N]\n"
              "       browser.py phext <url> [--select COORD]", file=sys.stderr)
        return 2
    url = args[0]
    if "://" not in url:
        url = "https://" + url
    try:
        page = fetch_page(url, jar=jar)
        if "--link" in args:
            number = int(args[args.index("--link") + 1])
            page = follow(page, number)
            display(page)
            return 0 if page.status < 400 else 1
        if sys.stdin.isatty():
            return _interactive(url, jar)
        display(page)
        return 0 if page.status < 400 else 1
    except (BrowserError, OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
