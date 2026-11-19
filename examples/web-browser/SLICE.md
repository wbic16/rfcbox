# SLICE.md — the ietf.phext slice for a basic web browser (+ phext streaming)

**RFC Box run:** task 51.6.7/6.5.9/4.5.11 (browser half; a sibling FTP build shares the coordinate)
**Date:** 2026-11-18 · **Corpus:** `~/workspace/phext/ietf-source.json` / `ietf.index.json` (artifact sha256 9ecd7946…)
**Requirement:** a basic terminal web browser — fetch a URL over HTTP/HTTPS, render HTML as readable text with numbered links, follow links and redirects, show the page title — plus, at Will's direction (2026-11-18 19:20 CST), **phext streaming over HTTP/1.1**: fetch a .phext with chunked transfer coding and split scrolls as bytes arrive.

Every entry below was verified present in the corpus at the stated coordinate, with record status **True / VERIFIED**. `test_browser.py` re-checks these claims against the corpus on every run.

## The slice

| RFC | Title | ietf.phext coordinate | Corpus status | Role in this build | Implementation link recorded in the corpus |
|---|---|---|---|---|---|
| 3986 | Uniform Resource Identifier (URI): Generic Syntax | 1.1.1/1.2.1/1.1.3986 | INTERNET STANDARD, current | URL parsing; relative-reference resolution for redirect `Location` headers and for every link on a page (`urljoin`) | **CPython (stdlib)** — also-implemented, ATTRIBUTED (https://github.com/python/cpython); also curl, VERIFIED |
| 9110 | HTTP Semantics | 1.1.1/1.2.1/1.1.9110 | INTERNET STANDARD, current | Methods, status codes (incl. the §15.4 redirect class), header fields, Content-Type — the meaning of everything the browser exchanges | **curl** — canonical, VERIFIED; also nginx, Apache HTTP Server, VERIFIED |
| 9112 | HTTP/1.1 | 1.1.1/1.2.1/1.1.9112 | INTERNET STANDARD, current | The wire protocol `http.client` speaks — and **§7.1 chunked transfer coding**, the mechanism phext streaming rides on | **curl** — canonical, VERIFIED; also nginx, Apache HTTP Server, VERIFIED |
| 9846 | The Transport Layer Security (TLS) Protocol Version 1.3 | 1.1.1/1.2.1/1.1.9846 | PROPOSED STANDARD, current | HTTPS: encrypted, server-authenticated fetch via the system trust store | **OpenSSL** — canonical, VERIFIED (Apache-2.0, https://github.com/openssl/openssl); also BoringSSL, Rustls, wolfSSL |
| 1035 | Domain names — implementation and specification | 1.1.1/1.2.1/1.1.1035 | INTERNET STANDARD, current | Hostname resolution. The browser delegates to the system resolver (via the socket layer) rather than speaking the DNS wire format itself; RFC 1034 is its concepts companion | **BIND 9** — canonical, VERIFIED; also Unbound, Knot DNS, VERIFIED |
| 6265 | HTTP State Management Mechanism | 1.1.1/1.2.1/1.1.6265 | PROPOSED STANDARD, current | Cookies — a deliberate subset: host-only session cookies (`Set-Cookie` stored per host, sent back to that host; Path/Domain/Expires attributes parsed off and ignored) | **curl** — canonical, VERIFIED |
| 2045 | MIME Part One: Format of Internet Message Bodies | 1.1.1/1.2.1/1.1.2045 | DRAFT STANDARD, current | The `Content-Type` header's grammar, incl. the `charset` parameter used to decode bodies | **CPython (stdlib)** — also-implemented; also Apache James Mime4j |
| 2046 | MIME Part Two: Media Types | 1.1.1/1.2.1/1.1.2046 | DRAFT STANDARD, current | Media-type dispatch: `text/html` goes to the renderer, `text/plain` is shown as-is | **CPython (stdlib)** — also-implemented; also Apache James Mime4j |
| 3629 | UTF-8, a transformation format of ISO 10646 | 1.1.1/1.2.1/1.1.3629 | INTERNET STANDARD, current | The default body charset when Content-Type names none | **ICU** — canonical (the build decodes via CPython codecs; see composition) |

## The composition

**Python 3 standard library only** — no pip dependencies. Where the corpus itself records CPython (stdlib) as an implementation, the build uses exactly that recorded implementation: `urllib.parse` for RFC 3986, the `email.message` header model behind `http.client` for RFC 2045/2046. Where the corpus names a canonical implementation the stdlib wraps, the build uses it: the `ssl` module **is** OpenSSL (RFC 9846's canonical, VERIFIED). Where the corpus's canonical is a standalone program this build does not embed — curl for HTTP (9110/9112/6265), BIND 9 for DNS (1035), ICU for UTF-8 (3629) — the composition choice is the stdlib equivalent (`http.client`, the system resolver, CPython codecs), recorded here as this build's own choice rather than attributed to the corpus. No slice gap blocked the build; HTTP/2 and HTTP/3 *are* in the corpus and were left unimplemented — that is a build-scope gap, recorded in GAP.md.

Flow: URL parsed (3986) → host resolved (1035, via the OS resolver) → GET over HTTP/1.1 (9112) with 9110 semantics — redirects resolved against the current URL (3986 §5), cookies stored/sent (6265 subset) → HTTPS where the scheme demands it (9846 via OpenSSL, system trust store) → body decoded by Content-Type charset (2045), defaulting to UTF-8 (3629) → dispatched by media type (2046): HTML to the renderer, plain text as-is.

## Phext over HTTP/1.1 (the added requirement)

Phext streaming is this slice applied to a payload the corpus does not contain:

- **Transport:** plain HTTP/1.1 (9112), body delivered with **chunked transfer coding (§7.1)**. The client de-chunks incrementally (`HTTPResponse.read1()` — one network read at a time; a filling `read(n)` would silently defeat streaming, a defect the test suite caught during this build) and feeds each piece to the splitter immediately. The whole file is never held as one string before first output.
- **Payload format:** phext. The 9 dimension delimiter bytes — `[0x01, 0x1F, 0x1E, 0x1D, 0x1C, 0x1A, 0x19, 0x18, 0x17]`, library..scroll — and the coordinate state machine (a delimiter increments its dimension and resets all deeper dimensions to 1) follow the reference builders in `~/workspace/phext/` (`build_tasks_phext.py` et al., the libphext conventions), **not** anything in the IETF corpus. Delimiters are single bytes below 0x20, so a scroll boundary split across chunk boundaries is handled by the same byte-at-a-time state machine — the tests split the fixture into 5-byte chunks to prove it.
- **Scroll existence:** a scroll exists when it holds at least one byte (the Incipit convention). Empty segments produced while advancing several dimensions are transit and are not listed in the TOC.
- **Operations:** as each scroll completes, its coordinate is listed (the TOC as it streams); `--select COORD` prints just that scroll's text and stops reading once it lands.

## Followed redirect

- **RFC 8446** (TLS 1.3, coordinate 1.1.1/1.2.1/1.1.8446) — the number everyone cites, recorded by the corpus **superseded**, obsoleted by RFC 9846. The slice takes the successor, 9846. Same title, same canonical implementation. (Consistent with the P25 IKI run, task 40.10.11/9.10.12/9.7.10, which hit the same redirect independently.)

## Checked — no redirect exists

- **RFC 6265** (cookies): the corpus records no successor — `obsoleted_by` is empty. The 6265bis revision had not become an RFC when the corpus was built, so 6265 itself is the slice entry, and the build implements a named subset of it.
- **RFC 1035** (DNS): updated by many later RFCs, obsoleted by none; current.
- **RFC 3986** (URI): current INTERNET STANDARD, no successor.

## Near-misses — considered and rejected

| RFC | Coordinate | Why considered | Why not in the slice |
|---|---|---|---|
| 9111 (HTTP Caching) | 1.1.1/1.2.1/1.1.9111 | A browser-shaped component | Deliberately not implemented: this reader keeps no cache and sends no conditional requests; every fetch is fresh. Recorded as a gap (GAP.md), not silently omitted. |
| 2978 (IANA Charset Registration Procedures) | 1.1.1/1.2.1/1.1.2978 | Charset-adjacent | It standardizes how charset labels get *registered*, not how a consumer decodes them. The build consumes labels (2045's charset parameter, 3629 default) via CPython codecs. Tier 3, unmapped in the corpus. |
| 7766 / 7858 / 8484 (DNS over TCP / DoT / DoH) | 1.1.1/1.2.1/1.1.7766, …7858, …8484 | Modern DNS transports | Resolution is delegated to the system resolver; which transport it uses is the OS's configuration, not this program's. |
| 9113 (HTTP/2), 9114 (HTTP/3), 9000 (QUIC) | 1.1.1/1.2.1/1.1.9113, …9114, …9000 | The newer HTTP transports | In the corpus, with implementation links (nghttp2; quiche/nghttp3) — but Python's stdlib speaks HTTP/1.1 only. Not implemented in this basic build: GAP.md. |

## What the slice does not contain

HTML. The rendering specification is a WHATWG living standard, not an IETF RFC, so no corpus query can return it — the headline gap, recorded in **GAP.md**. CSS and JavaScript are likewise outside the corpus (W3C/ECMA), and phext itself is Will's format, not an IETF standard: GAP.md records each absence as a gap rather than papering over it.
