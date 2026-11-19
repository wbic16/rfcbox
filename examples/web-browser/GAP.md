# GAP.md — RFC-Box gap record: basic web browser + phext streaming

**Gap IDs:** GAP-WEB-001 … GAP-WEB-005 · **Opened by:** RFC Box run 51.6.7/6.5.9/4.5.11 (2026-11-18)
**Severity for this build:** documented limitations — the browser is functional and honest about them; the gaps block any claim to be a *complete* or *standards-conformant* browser, not the lynx-class reader it sets out to be.

## GAP-WEB-001 (headline) — HTML is not in the corpus, and cannot be

**HTML — the very thing a browser renders — is a WHATWG living standard, not an IETF RFC.**

- ietf.phext contains IETF standards. The HTML specification is maintained by the WHATWG (the `html.spec.whatwg.org` Living Standard); the IETF published no HTML RFC in the modern era, and a corpus query for the rendering spec returns nothing — correctly.
- Consequently the renderer in `browser.py` — headings, paragraphs, lists, numbered links, `<pre>`, the `<title>` — is a small subset built from general knowledge of HTML, **not** composed from a corpus slice the way the fetching half of the program is. It does not implement the HTML parsing algorithm (no tree construction, no error recovery, no foster parenting); real-world malformed pages will render approximately, not per spec.
- **What closing it takes (REQ-WEB-1):** adopt the WHATWG HTML Living Standard as an explicitly out-of-corpus source (unlike the TIA-102 documents in the IKI run, it is openly published — this gap is about corpus scope, not availability), implement against its parsing and rendering model, and test against the shared `html5lib-tests` / web-platform-tests suites. Until then, every rendering behavior beyond the named subset is best-effort.

## GAP-WEB-002 — CSS and JavaScript are outside the corpus

- Cascading Style Sheets is W3C/WHATWG territory; JavaScript is ECMA-262 (Ecma International). Neither is an IETF RFC; neither is in ietf.phext.
- This browser applies no styling and executes no script — `<script>` and `<style>` contents are skipped outright. Pages that require JavaScript to produce their content render whatever static text they carry, which may be almost nothing.
- **REQ-WEB-2:** closing this gap means adopting two further out-of-corpus specifications and two large engines (a CSS layout engine; a JS runtime). That is a different program than a basic reader; the gap is recorded so the boundary is explicit, not discovered.

## GAP-WEB-003 — HTTP/2 and HTTP/3 are in the corpus but not implemented

- Unlike HTML, these are **not** corpus absences: the corpus holds HTTP/2 (RFC 9113, 1.1.1/1.2.1/1.1.9113, canonical implementation nghttp2), HTTP/3 (RFC 9114, 1.1.1/1.2.1/1.1.9114), and QUIC (RFC 9000, 1.1.1/1.2.1/1.1.9000), with implementation links. The gap is in this build: Python's standard library speaks HTTP/1.1 only, and the brief fixed the stdlib as the composition.
- Practical effect: a server that offers only h2/h3 with no HTTP/1.1 fallback is unreachable from this browser. In practice HTTP/1.1 fallback is near-universal, so the gap rarely bites — but it is a gap, not a non-issue.
- **REQ-WEB-3:** compose an h2/h3 stack from the corpus's recorded implementations (nghttp2 / nghttp3 / quiche) behind the same fetch interface, with ALPN negotiation selecting the transport.

## GAP-WEB-004 — HTTP caching (RFC 9111) deliberately not implemented

- RFC 9111 (HTTP Caching, 1.1.1/1.2.1/1.1.9111) is in the slice's near-miss list, not the slice: this reader keeps no cache, stores nothing between runs except session cookies, and sends no conditional requests (`If-None-Match` / `If-Modified-Since`). Every fetch is a full fresh fetch — wasteful, and exactly what a basic reader may honestly be.
- **REQ-WEB-4:** add a cache store honoring `Cache-Control`, `Expires`, `ETag`, and `Last-Modified` per RFC 9111, with the tests to match.

## GAP-WEB-005 — Phext over HTTP is standardized nowhere in the corpus

- **Phext itself is Will's format, not an IETF RFC** — the same class of honest gap as GAP-WEB-001. No RFC defines phext's delimiters, assigns it a media type, or profiles its transport over HTTP. The corpus can supply the transport (RFC 9112, incl. §7.1 chunked coding) but not the payload format.
- This build's phext conventions are therefore local and recorded in SLICE.md: delimiter bytes and the coordinate state machine follow the reference builders in `~/workspace/phext/` (the libphext conventions); a scroll exists iff it holds ≥ 1 byte; streaming means scrolls are split and listed as bytes arrive, never after a whole-file read.
- **REQ-WEB-5:** closing this gap is a specification act, not a coding one: a written phext specification and, if public interoperability is ever wanted, a media-type registration (`text/phext` or a vendor type) through the RFC 6838 process — which, fittingly, the corpus does hold. The decision is Will's; the corpus cannot make it.

## Smaller recorded limitations

- **Cookies:** RFC 6265 is implemented as a named subset (host-only session cookies). Domain/Path matching, expiry, and the `Secure`/`HttpOnly` attributes are parsed off and ignored. A multi-user or long-lived browser must implement 6265 in full.
- **Content encodings:** the client requests `Accept-Encoding: identity`; gzip/deflate bodies are not decoded.
- **Authentication, forms, POST:** out of scope for a reader; only GET is issued.

## Status

Gaps recorded and carried on task 51.6.7/6.5.9/4.5.11. Nothing in this build depends on closing them; every conformance statement beyond "a working lynx-class reader over a verified corpus slice" does.
