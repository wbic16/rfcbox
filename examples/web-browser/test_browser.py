#!/usr/bin/env python3
"""test_browser.py — non-interactive suite for browser.py.

Re-verifies the SLICE.md claims against the corpus (ietf-source.json),
exercises the browser against local fixtures (an http.server site with an
index, links, a redirect, a cookie, a 404), and exercises phext streaming
against a raw-socket HTTP/1.1 chunked server that splits the fixture phext
into 5-byte chunks — scroll boundaries land wherever they land — with a
pause mid-stream to prove the client yields the first scroll before the
last byte arrives.

Exit 0 iff every check passes. The live example.com check skips (does not
fail) when outbound network is unavailable.
"""

import json
import os
import socket
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import browser  # noqa: E402

CORPUS = os.path.expanduser("~/workspace/phext/ietf-source.json")

FAILURES = []


def check(label, condition, detail=""):
    if condition:
        print(f"PASS: {label}")
    else:
        print(f"FAIL: {label} {detail}")
        FAILURES.append(label)


# ---------------------------------------------------------------------------
# 1. Slice verification against the corpus
# ---------------------------------------------------------------------------

SLICE = {  # rfc -> expected ietf.phext coordinate (SLICE.md)
    3986: "1.1.1/1.2.1/1.1.3986",
    9110: "1.1.1/1.2.1/1.1.9110",
    9112: "1.1.1/1.2.1/1.1.9112",
    9846: "1.1.1/1.2.1/1.1.9846",
    1035: "1.1.1/1.2.1/1.1.1035",
    6265: "1.1.1/1.2.1/1.1.6265",
    2045: "1.1.1/1.2.1/1.1.2045",
    2046: "1.1.1/1.2.1/1.1.2046",
    3629: "1.1.1/1.2.1/1.1.3629",
}


def test_slice():
    with open(CORPUS) as fh:
        corpus = json.load(fh)
    by_rfc = {e["rfc"]: e for e in corpus["entries"]}
    for rfc, coord in SLICE.items():
        entry = by_rfc.get(rfc)
        check(f"slice RFC {rfc} present in corpus at {coord}",
              entry is not None and entry["coordinate"] == coord
              and entry["record_status"] == "True",
              f"(got {entry and entry['coordinate']})")
    e8446 = by_rfc[8446]
    check("corpus marks RFC 8446 superseded, redirecting to 9846",
          e8446["superseded"] and e8446["obsoleted_by"] == [9846])
    check("corpus canonical implementation for TLS 1.3 is OpenSSL",
          any(i["name"] == "OpenSSL" and i["role"] == "canonical"
              for i in by_rfc[9846]["implementations"]))
    check("corpus records CPython (stdlib) as an implementation of RFC 3986",
          any(i["name"] == "CPython (stdlib)"
              for i in by_rfc[3986]["implementations"]))
    check("RFC 6265 is current in the corpus (no successor recorded)",
          by_rfc[6265]["obsoleted_by"] == [])


# ---------------------------------------------------------------------------
# 2. HTTP fixture site
# ---------------------------------------------------------------------------

INDEX_HTML = """<!DOCTYPE html>
<html><head><title>Fixture Home</title></head><body>
<h1>Fixture Home Page</h1>
<p>Welcome to the fixture index.</p>
<ul>
<li><a href="/page2">Second Page</a></li>
<li><a href="/redir">Redirecting Link</a></li>
</ul>
<ol><li>First numbered item</li><li>Second numbered item</li></ol>
</body></html>"""

PAGE2_HTML = """<!DOCTYPE html>
<html><head><title>Second Page</title></head><body>
<h1>Second Page</h1>
<p>You have arrived at page two.</p>
</body></html>"""

NOTFOUND_HTML = """<!DOCTYPE html>
<html><head><title>Not Found</title></head><body>
<p>Nothing here.</p>
</body></html>"""


class FixtureHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *args):
        pass

    def _send(self, status, body, content_type="text/html", extra=None):
        data = body.encode()
        self.send_response(status)
        self.send_header("Content-Type", f"{content_type}; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        for key, value in (extra or {}).items():
            self.send_header(key, value)
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path == "/":
            self._send(200, INDEX_HTML, extra={"Set-Cookie": "session=abc123"})
        elif self.path == "/page2":
            self._send(200, PAGE2_HTML)
        elif self.path == "/redir":
            self.send_response(302)
            self.send_header("Location", "/page2")
            self.send_header("Content-Length", "0")
            self.end_headers()
        elif self.path == "/echo-cookie":
            cookie = self.headers.get("Cookie") or "none"
            self._send(200, f"cookie: {cookie}", content_type="text/plain")
        else:
            self._send(404, NOTFOUND_HTML)


def test_http(base):
    jar = browser.CookieJar()
    page = browser.fetch_page(base + "/", jar=jar)
    check("fetch index: status 200", page.status == 200, f"(got {page.status})")
    check("fetch index: title shown", page.title == "Fixture Home",
          f"(got {page.title!r})")
    check("render: heading rendered", "Fixture Home Page" in page.text)
    check("render: paragraph text present",
          "Welcome to the fixture index." in page.text)
    check("render: ordered list numbered",
          "1. First numbered item" in page.text
          and "2. Second numbered item" in page.text)
    check("links: two links extracted", len(page.links) == 2,
          f"(got {page.links})")
    check("links: numbered in rendered text", "[1]" in page.text
          and "[2]" in page.text)
    check("links: first link resolves to /page2 (RFC 3986 resolution)",
          page.links and page.links[0][1] == base + "/page2",
          f"(got {page.links and page.links[0][1]})")

    page2 = browser.follow(page, 1)
    check("follow link 1: lands on page two",
          page2.status == 200 and "You have arrived at page two." in page2.text)

    via_redirect = browser.follow(page, 2)
    check("follow link 2: redirect followed to /page2",
          via_redirect.url == base + "/page2"
          and "You have arrived at page two." in via_redirect.text,
          f"(final url {via_redirect.url})")
    check("redirect: hop recorded in history",
          any(status == 302 for status, _ in via_redirect.result.history))

    direct = browser.fetch_page(base + "/redir")
    check("fetch /redir directly: followed, final URL is /page2",
          direct.url == base + "/page2" and direct.status == 200)

    missing = browser.fetch_page(base + "/missing")
    check("404: reported, not crashed",
          missing.status == 404 and "Nothing here." in missing.text,
          f"(got status {missing.status})")

    echo = browser.fetch_page(base + "/echo-cookie", jar=jar)
    check("cookies (RFC 6265 subset): Set-Cookie stored and sent back",
          "session=abc123" in echo.text, f"(echo said: {echo.text!r})")


# ---------------------------------------------------------------------------
# 3. Phext streaming over HTTP/1.1 chunked transfer coding (RFC 9112 §7.1)
# ---------------------------------------------------------------------------

LEVEL_BYTES = [0x01, 0x1F, 0x1E, 0x1D, 0x1C, 0x1A, 0x19, 0x18, 0x17]


def _parse_coord(text):
    return tuple(int(p) for p in text.replace("/", ".").split("."))


def _implode(scrolls):
    """The reference implode state machine (~/workspace/phext builders)."""
    items = sorted((_parse_coord(k), v) for k, v in scrolls.items())
    out = bytearray()
    state = [1] * 9
    for target, text in items:
        for lvl in range(9):
            while state[lvl] < target[lvl]:
                out.append(LEVEL_BYTES[lvl])
                state[lvl] += 1
                for j in range(lvl + 1, 9):
                    state[j] = 1
        out += text.encode()
    return bytes(out)


FIXTURE_SCROLLS = {
    "1.1.1/1.1.1/1.1.1": "Home scroll\nWelcome to the fixture phext.\n",
    "1.1.1/1.1.1/1.1.2": "Second scroll\nContents of scroll two.\n",
    "1.1.1/1.2.1/1.1.1": "Chapter scroll\nA new chapter begins here.\n",
}


class ChunkedPhextServer(threading.Thread):
    """A raw-socket HTTP/1.1 server that speaks chunked transfer coding
    itself — http.server cannot be made to, and doing it by hand proves
    the client is really consuming a chunked stream. Sends the first
    scroll, pauses, then dribbles the rest in 5-byte chunks so scroll
    boundaries fall across chunk boundaries."""

    def __init__(self, payload, first_chunk_len, pause=0.5):
        super().__init__(daemon=True)
        self.payload = payload
        self.first_chunk_len = first_chunk_len
        self.pause = pause
        self.finished = threading.Event()
        self._stopped = False
        self.sock = socket.socket()
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind(("127.0.0.1", 0))
        self.sock.listen(5)
        self.port = self.sock.getsockname()[1]

    def run(self):
        while not self._stopped:
            try:
                self.sock.settimeout(0.2)
                conn, _ = self.sock.accept()
            except socket.timeout:
                continue
            except OSError:
                break
            threading.Thread(target=self._serve, args=(conn,),
                             daemon=True).start()

    def _serve(self, conn):
        try:
            request = b""
            while b"\r\n\r\n" not in request:
                data = conn.recv(4096)
                if not data:
                    return
                request += data
            conn.sendall(
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Type: text/plain; charset=utf-8\r\n"
                b"Transfer-Encoding: chunked\r\n"
                b"Connection: close\r\n\r\n")

            def send_chunk(data):
                conn.sendall(b"%x\r\n" % len(data) + data + b"\r\n")

            send_chunk(self.payload[:self.first_chunk_len])
            time.sleep(self.pause)
            rest = self.payload[self.first_chunk_len:]
            for i in range(0, len(rest), 5):
                send_chunk(rest[i:i + 5])
            conn.sendall(b"0\r\n\r\n")
            self.finished.set()
        except OSError:
            pass
        finally:
            conn.close()

    def stop(self):
        self._stopped = True
        try:
            self.sock.close()
        except OSError:
            pass


def test_phext(port, server):
    url = f"http://127.0.0.1:{port}/fixture.phext"

    # Streaming proof: pull the first scroll off the generator and confirm
    # the server has NOT finished sending yet.
    stream = browser.iter_phext_scrolls(url)
    first_coord, first_scroll = next(stream)
    check("phext streaming: first scroll yielded before the stream finished",
          first_coord == "1.1.1/1.1.1/1.1.1"
          and first_scroll.decode() == FIXTURE_SCROLLS["1.1.1/1.1.1/1.1.1"]
          and not server.finished.is_set())
    remaining = list(stream)
    expected_rest = ["1.1.1/1.1.1/1.1.2", "1.1.1/1.2.1/1.1.1"]
    check("phext streaming: remaining scrolls arrive in coordinate order",
          [c for c, _ in remaining] == expected_rest,
          f"(got {[c for c, _ in remaining]})")

    toc = browser.phext_toc(url)
    expected_toc = [(coord, len(text.encode()))
                    for coord, text in FIXTURE_SCROLLS.items()]
    check("phext TOC: coordinates and sizes match the fixture",
          toc == expected_toc, f"(got {toc})")

    selected = browser.phext_select(url, "1.1.1/1.1.1/1.1.2")
    check("phext select: exact scroll bytes returned",
          selected is not None
          and selected.decode() == FIXTURE_SCROLLS["1.1.1/1.1.1/1.1.2"],
          f"(got {selected!r})")
    missing = browser.phext_select(url, "9.9.9/9.9.9/9.9.9")
    check("phext select: unknown coordinate returns None, not a crash",
          missing is None)


# ---------------------------------------------------------------------------
# 4. Live check (skips, never fails, when the network blocks it)
# ---------------------------------------------------------------------------

def test_live():
    try:
        page = browser.fetch_page("https://example.com/")
    except Exception as exc:  # network blocked from this VM
        print(f"SKIP: live https://example.com/ check — network unavailable ({exc})")
        return
    check("live: https://example.com/ fetched over TLS, known text present",
          page.status == 200 and "Example Domain" in page.text,
          f"(status {page.status})")


def main():
    test_slice()

    server = ThreadingHTTPServer(("127.0.0.1", 0), FixtureHandler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    try:
        test_http(base)
    finally:
        server.shutdown()

    payload = _implode(FIXTURE_SCROLLS)
    first_chunk_len = len(FIXTURE_SCROLLS["1.1.1/1.1.1/1.1.1"].encode()) + 1
    phext_server = ChunkedPhextServer(payload, first_chunk_len)
    phext_server.start()
    try:
        test_phext(phext_server.port, phext_server)
    finally:
        phext_server.stop()

    test_live()

    if FAILURES:
        print(f"\nTEST RESULT: FAIL ({len(FAILURES)} check(s): {FAILURES})")
        return 1
    print("\nTEST RESULT: PASS (all checks)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
