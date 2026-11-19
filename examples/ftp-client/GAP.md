# GAP.md — RFC-Box gap record: FTP client

**Gap ID:** GAP-FTP-001 · **Opened by:** RFC Box run 51.6.7/6.5.9/4.5.11 (2026-11-18)
**Severity for this build:** documented limitations — the client is functional and tested against a local fixture; the gaps below bound its *interop and coverage* claims, not the substrate demonstration.

## Headline: what FTPS testing does and does not cover

**FTPS was tested — plainly: against a local fixture, not against the world.**
The full client exercise (login, FEAT, MLSD upload/list, SIZE, MDTM, byte-identical download, delete) passes over explicit FTPS (RFC 4217: AUTH TLS, PBSZ 0, PROT P) against a pyftpdlib TLS fixture with a self-signed certificate, TLS 1.3 via OpenSSL. The fixture run uses `--no-verify` semantics (verification disabled) because the fixture certificate is self-signed. What has **not** been tested: a production FTPS server, a CA-issued server certificate through the client's default verification path, client-certificate (mutual) FTPS, or any FTPS server implementation other than pyftpdlib.

## The gaps (requirements)

1. **REQ-FTP-1 — RFC 7151 HOST command.** Not implemented. Servers that virtual-host multiple FTP sites behind one address may require HOST before login; this client cannot select a virtual host. Add HOST (sent after connect, before USER) when such a server is in scope.
2. **REQ-FTP-2 — RFC 2228 beyond the TLS profile.** Only the RFC 4217 TLS profile of the 2228 framework is implemented. AUTH GSSAPI/Kerberos and the CONF/ENC/MIC protection levels are not implemented, and CCC (clear command channel after AUTH) is deliberately not offered — downgrading the control channel is a footgun this client declines to carry.
3. **REQ-FTP-3 — Active mode is implemented but untested, and NAT-hostile.** `--active` (PORT/EPRT via stdlib) exists but the test suite exercises only passive modes (EPSV default, which is the tested path). Behind NAT, active mode asks the server to open a connection *to* the client and typically fails; passive data connections to real-world servers with firewalled passive port ranges are likewise untested outside the loopback fixture.
4. **REQ-FTP-4 — Restart / resume (REST).** RFC 3659's slice-mates include stream-mode restart; this client does not implement REST offsets, so interrupted transfers restart from byte zero.
5. **REQ-FTP-5 — MLST and MFMT not exposed.** Listings use MLSD (with LIST fallback); single-object MLST queries and MFMT (set remote mtime) are not surfaced in the CLI or API.
6. **REQ-FTP-6 — Interop breadth.** One fixture implementation (pyftpdlib 2.2.0). No testing yet against vsftpd, ProFTPD, FileZilla Server, or IIS FTP. The LIST fallback path is implemented but the fixture always advertises MLST, so the fallback is **untested code** — it should be exercised against a LIST-only server before it is relied on.
7. **REQ-FTP-7 — Corpus mapping debt (upstream).** The FTP core (RFCs 959, 2389, 3659, 2640, 2228, 4217) is Tier 3 in ietf.phext — present and verified as records, but with no canonical-implementation links. This build composed CPython stdlib `ftplib` on its own judgement (recorded in SLICE.md). A future corpus implementation-mapping pass over the FTP family would let the next RFC Box run *derive* that choice instead of making it.

## Status

Gap recorded and carried on task 51.6.7/6.5.9/4.5.11. Nothing in this build depends on closing these; every interop statement beyond "tested against the pyftpdlib fixture, plain and FTPS" does.
