# SLICE.md — the ietf.phext slice for an FTP client

**RFC Box run:** task 51.6.7/6.5.9/4.5.11 (second manual RFC Box run over ietf.phext; sibling of the web-browser build under the same coordinate)
**Date:** 2026-11-18 · **Corpus:** `~/workspace/phext/ietf-source.json` / `ietf.index.json` (artifact sha256 9ecd7946…)
**Requirement:** a basic FTP client — connect (anonymous or user/pass), list, upload, download, navigate — passive by default, with an explicit-TLS (FTPS) mode.

Every entry below was verified present in the corpus at the stated coordinate, with record status **True / VERIFIED**. `test_ftp.py` re-checks these claims against the corpus on every run.

## The slice

| RFC | Title | ietf.phext coordinate | Corpus status | Role in this build | Implementation link recorded in the corpus |
|---|---|---|---|---|---|
| 959 | File Transfer Protocol | 1.1.1/1.2.1/1.1.959 | INTERNET STANDARD, current | The base protocol: control connection, USER/PASS, CWD/PWD, RETR/STOR/DELE, TYPE I | none recorded — Tier 3, unmapped by design (see composition note) |
| 2389 | Feature negotiation mechanism for FTP | 1.1.1/1.2.1/1.1.2389 | PROPOSED STANDARD, current | FEAT discovery after login; the client keys MLSD/SIZE/MDTM/UTF8 behaviour off the reply | none recorded — Tier 3, unmapped by design |
| 3659 | Extensions to FTP | 1.1.1/1.2.1/1.1.3659 | PROPOSED STANDARD, current (updates 959) | MLSD machine-readable listings, SIZE, MDTM | none recorded — Tier 3, unmapped by design |
| 2640 | Internationalization of FTP | 1.1.1/1.2.1/1.1.2640 | PROPOSED STANDARD, current (updates 959) | OPTS UTF8 ON when FEAT advertises UTF8; path names in UTF-8 | none recorded — Tier 3, unmapped by design |
| 2428 | FTP Extensions for IPv6 and NATs | 1.1.1/1.2.1/1.1.2428 | PROPOSED STANDARD, current | EPSV — the default passive mode (address-family-agnostic, NAT-friendlier); classic PASV and active PORT remain available by flag | Linux kernel — canonical; FreeBSD — also-implemented (the corpus maps this RFC to the OS substrate, not to an FTP client; see composition note) |
| 2228 | FTP Security Extensions | 1.1.1/1.2.1/1.1.2228 | PROPOSED STANDARD, current (updates 959) | The AUTH / PBSZ / PROT framework that FTPS profiles | none recorded — Tier 3, unmapped by design |
| 4217 | Securing FTP with TLS | 1.1.1/1.2.1/1.1.4217 | PROPOSED STANDARD, current (updated by 8996) | Explicit FTPS: AUTH TLS upgrade, PBSZ 0, PROT P — the `--tls` mode | none recorded — Tier 3, unmapped by design |
| 9846 | TLS Protocol Version 1.3 | 1.1.1/1.2.1/1.1.9846 | PROPOSED STANDARD, current | The TLS the FTPS mode actually runs (see redirect note) | **OpenSSL** — canonical, VERIFIED (also BoringSSL, Rustls, wolfSSL) |
| 8996 | Deprecating TLS 1.0 and TLS 1.1 | 1.1.1/1.2.1/1.1.8996 | BEST CURRENT PRACTICE (updates 4217) | Policy constraint on 4217: TLS below 1.2 is refused | none recorded — Tier 3, unmapped by design |
| 3986 | URI: Generic Syntax | 1.1.1/1.2.1/1.1.3986 | INTERNET STANDARD, current | Parsing of `ftp://` targets (see redirect note) | curl, CPython (stdlib) — also-implemented; this build uses the CPython one |

## The composition

One stack carries the whole slice: **Python 3 standard library** — `ftplib` for the FTP protocol machinery (959 + the 2389/3659/2640/2428 extensions + the 2228/4217 AUTH-TLS profile via `FTP_TLS`), `ssl` for TLS, `urllib.parse` for RFC 3986 syntax.

- Where the corpus names an implementation this build uses, the build uses exactly it: **OpenSSL** (corpus-canonical for RFC 9846) is what Python's `ssl` module binds, and **CPython (stdlib)** — a corpus-recorded implementer of RFC 3986 — supplies both `ftplib` and `urllib.parse`.
- Where the corpus records no implementation link (RFCs 959, 2389, 3659, 2640, 2228, 4217, 8996 — Tier 3, unmapped by design), the composition choice is this build's own, recorded here rather than attributed to the corpus: CPython's stdlib `ftplib`. RFC 2428's corpus links point at OS kernels (the IPv6 substrate); the EPSV behaviour itself likewise comes from `ftplib`. No Tier-3 gap blocked the build; the mapping gap is noted for the corpus's next implementation-mapping pass.

## Followed redirects

- **RFC 1738** (Uniform Resource Locators, coordinate 1.1.1/1.2.1/1.1.1738) defined the `ftp:` URL — but the corpus records it **superseded** (obsoleted-by entries point at the scheme-specific split-offs 4248/4266, and RFC 3986 **updates** it). Per MOAT rules the superseded entry is kept and annotated; the slice takes **3986** for URI syntax, keeping 1738's ftp-scheme semantics (user/password/host/port/path) as the annotated historical definition.
- **RFC 4217's TLS substrate moved.** 4217 was written against RFC 2246 (TLS 1.0). The corpus's current TLS record is **9846** (TLS 1.3, which obsoletes 2246's line), and **8996** updates 4217 itself to deprecate TLS 1.0/1.1. The build therefore runs FTPS over TLS 1.3 and refuses anything below TLS 1.2.

## Findings the build fed back

- **The FEAT token for MLSD is "MLST".** RFC 3659 advertises the MLST feature in FEAT to signal support for *both* MLST and MLSD; a client keying on a literal "MLSD" token silently falls back to LIST. The fixture server (pyftpdlib) advertises MLST only — caught by the test suite, fixed in the client.
- **stdlib `ftplib`'s MLSD rides `retrlines`, which flips the session to TYPE A** and does not restore it; a SIZE after a listing is then refused ("not allowed in ASCII mode"). The client defaults to TYPE I at connect and re-asserts TYPE I after every listing.
- **SIZE is only meaningful in binary mode** — one more reason the client's default transfer type is TYPE I, not the RFC 959 ASCII default.

## Near-misses — considered and rejected

| RFC | Coordinate | Why considered | Why not in the slice |
|---|---|---|---|
| 5797 | 1.1.1/1.2.1/1.1.5797 | FTP Command and Extension Registry | A registry, not wire behaviour the client implements; useful reference when adding commands. |
| 7151 | 1.1.1/1.2.1/1.1.7151 | FTP HOST command (virtual hosting) | Not implemented by this client; carried in GAP.md as REQ-FTP-1. |
| 2577 | 1.1.1/1.2.1/1.1.2577 | FTP security considerations | Guidance, not protocol; read, not sliced. |
| 2246 (TLS 1.0) | 1.1.1/1.2.1/1.1.2246 | 4217's original TLS reference | Obsoleted line; superseded by 9846 and deprecated by 8996. |
| — (implicit FTPS, port 990) | — | The other historical "FTPS" | Never standardized in an RFC; only the explicit RFC 4217 mode is implemented. |

## What the slice does not contain

A second server implementation to interoperate against, and any production FTPS endpoint: the test fixture is one local server (pyftpdlib). The interop and coverage limits are recorded in **GAP.md**, not smoothed over.
