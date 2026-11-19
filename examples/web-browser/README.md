# rfcbox-web-browser — a basic terminal web browser, plus phext streaming over HTTP/1.1

An **RFC Box run** over ietf.phext (task 51.6.7/6.5.9/4.5.11): a requirement went in, a slice of the corpus came out (SLICE.md), code was composed from the slice using Python's standard library only, and what the corpus could not supply was recorded as gaps (GAP.md) instead of being improvised.

**What it is:** a lynx-class reader. Give it a URL and it fetches the page over HTTP/1.1 (or HTTPS via TLS 1.3 / OpenSSL and the system trust store), follows redirects, keeps session cookies, renders the HTML as readable terminal text — headings, paragraphs, bulleted and numbered lists, links numbered for following — and shows the page title. At Will's direction it also speaks **phext over HTTP/1.1**: point it at a .phext served with chunked transfer coding and it splits scrolls on the phext dimension delimiters *as the bytes arrive*, listing each scroll's coordinate (the TOC) as it completes, and can print a single scroll selected by coordinate.

**What it is not:** a layout engine or a conformant browser. HTML is a WHATWG living standard, not an IETF RFC, so the corpus cannot supply the rendering spec; there is no CSS, no JavaScript, no HTTP/2 or HTTP/3, and no cache. Each absence is a named gap in GAP.md, not a silent omission.

## Files

| File | Contents |
|---|---|
| `SLICE.md` | The corpus slice: RFCs, ietf.phext coordinates, statuses, implementation links, composition, the followed 8446→9846 redirect, rejected near-misses |
| `GAP.md` | The gap record: HTML/WHATWG (headline), CSS/JS, HTTP/2+3, caching, and phext-over-HTTP's lack of any IETF standardization |
| `browser.py` | The browser + phext streamer (stdlib only: `http.client`, `ssl`, `html.parser`, `urllib.parse`) |
| `test_browser.py` | Non-interactive suite: re-verifies the slice against the corpus, exercises local HTTP fixtures (links, redirect, cookie, 404), and streams a fixture phext from a raw-socket chunked server in 5-byte chunks |

## Run

```sh
python3 browser.py https://example.com/          # fetch + render
python3 browser.py https://example.com/ --link 1 # …then follow link 1
python3 browser.py https://example.com/          # on a terminal: interactive
                                                 #   [link # / url / q] prompt
python3 browser.py phext http://host/book.phext              # streaming TOC
python3 browser.py phext http://host/book.phext --select 1.1.1/1.2.1/1.1.1
                                                 # print just that scroll
python3 test_browser.py                          # full suite, exit 0 iff PASS
```

No dependencies beyond Python 3 itself. The suite's one live check (fetching
https://example.com/) skips with a note when outbound network is unavailable;
everything else runs against localhost fixtures.
