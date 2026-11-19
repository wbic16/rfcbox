# rfcbox-ftp-client — a basic FTP client

A manual **RFC Box run** over ietf.phext (task 51.6.7/6.5.9/4.5.11): a requirement went in, a slice of the corpus came out (SLICE.md), code was composed from the slice, and what the build could not cover was recorded as gaps (GAP.md) instead of being smoothed over.

**What it is:** an FTP client library + CLI standing on RFC 959 with the modern extension set — FEAT negotiation (2389), MLSD listings / SIZE / MDTM (3659), UTF-8 paths (2640), EPSV passive mode (2428), and explicit FTPS (2228 + 4217) running over TLS 1.3 (9846, with 8996's deprecations honoured). Targets may be given as hosts or `ftp://` URLs (3986 syntax; the ftp scheme is 1738's, which the corpus records superseded — see SLICE.md).

**What it is not:** a full-featured FTP client. No HOST command, no REST resume, no non-TLS 2228 mechanisms, and interop is proven against one local fixture only — see GAP.md.

## Files

| File | Contents |
|---|---|
| `SLICE.md` | The corpus slice: RFCs, ietf.phext coordinates, statuses, implementation links, composition, redirects followed, near-misses |
| `GAP.md` | The gap record: FTPS testing scope stated plainly, deferred requirements REQ-FTP-1…7 |
| `ftp_client.py` | The client: `FTPClient` class + CLI (stdlib `ftplib`/`ssl`/`urllib.parse`) |
| `test_ftp.py` | Non-interactive suite — re-verifies the slice against the corpus, then exercises plain FTP and FTPS against hermetic pyftpdlib fixtures |
| `requirements.txt` | Test-fixture dependencies (pyftpdlib, pyOpenSSL, cryptography) — the client itself is stdlib-only |

## Run

```sh
python3 ftp_client.py --help
python3 ftp_client.py ftp.example.com ls
python3 ftp_client.py --user alice ftp.example.com get report.txt
python3 ftp_client.py --tls ftps.example.com put backup.tar.gz
python3 ftp_client.py ftp://alice@ftp.example.com/pub ls
```

Commands: `ls`, `get`, `put`, `pwd`, `cd`, `size`, `mdtm`, `rm`, `mkdir`, `rmdir`, `features`. Flags: `--tls` (explicit FTPS), `--no-verify`, `--active`, `--no-epsv`, `--port`, `--user`, `--password`, `--timeout`. Server refusals print one line (`error: …`) and exit 1 — no tracebacks.

## Test

```sh
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt
.venv/bin/python test_ftp.py    # 33 checks, exit 0 iff all pass
```
