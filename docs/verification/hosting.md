# ietf.phext — primary hosting record (Watch, PH: DevSecOps)

Task 25.6.9/8.6.10/6.6.9 · One Day Build, 2026-11-18 · Reporter: Watch (PH: DevSecOps), under Orion (PH: R&D)

## Endpoint

- **Base:** `http://100.107.204.62:1338/api/v2` (logos-prime.pizzly-halfmoon.ts.net:1338), SQ v0.6.0 REST
- **Tenant:** every call carries `p=ietf` — e.g. `/status?p=ietf`, `/toc?p=ietf`, `/select?p=ietf&c=<coordinate>`
- **Host node:** logos-prime (100.107.204.62), chosen over halycon-vector on survey evidence (load 0.01/0.02/0.00 vs 0.12/0.05/0.01; 1.6 TB disk free, ~11 GB RAM on both). Port 1338 was surveyed free; the node's pre-existing SQ tenant on port 1337 was not disturbed at any step (same pid throughout).
- **Mirror:** free-fall (100.74.202.8), held by Rogue (PH: Evolution) — separate leg, not covered here.
- **Artifact:** `~/ietf-phext/ietf.phext` on logos-prime, 564,160,499 bytes, SHA-256 `5c576b3737c65d12b24bc1dbb8329639326838c986ebf8882bdbb095541ac045`. Transfer hash-verified on-node before hosting; a full `get` round-trip after hosting reproduced the artifact byte-for-byte. Corpus shape: 1,014 scrolls (10,052 RFC entries, RFC 1–10094); ~26.8 MB of scroll content, the remainder coordinate-delimiter padding, preserved intact by hosting.

## Gating

- The host runs with an API key (`sq host 1338 --key`). Unauthenticated `/status`, `/toc`, `/select` return **HTTP 401 "Unauthorized"** (verified).
- Key presentation (tested, not assumed): the **`Authorization` header** — `Authorization: Bearer <key>` succeeds; raw `Authorization: <key>` also succeeds. Query params (`?key=`, `?k=`) and other headers (`X-API-Key`) fail with 401.
- Key custody: the key lives only on logos-prime (`~/ietf-phext/.sq-key`, mode 600) and in Watch's DevSecOps records (mode 600). It is not in this repo, not in any access doc — Reed's access doc states "key required, obtain from Watch/Orion." Key fingerprint, for identity checks only: SHA-256(key) first 12 hex = `0acc828cf579`. The fingerprint identifies the key; it cannot be used as the key.
- **Residual caveat:** SQ v0.6.0 accepts the key only via the `--key` command-line flag, so the key is visible in the host process's argv to any local user running `ps` on logos-prime. The key file (600) and its launcher (700) are tight; the argv exposure is inherent to SQ's interface — SQ has no file-based key option. Recorded openly; accepted for a single-operator ranch node.

## Persistence (systemd)

- System units on logos-prime, both **enabled + active**, `User=wbic16`, `Restart=on-failure`:
  - `sq-ietf-share.service` — the share daemon; its `ExecStartPre` removes the stale `.sq` shared-memory directory before start (the known SQ wedge guard).
  - `sq-ietf-host.service` — the HTTP host; `Requires`/`After`/`PartOf` the share unit, with a settle delay. Its ExecStart is a wrapper (`~/ietf-phext/run-host.sh`, mode 700) that reads the key from the 600 key file at exec time — no key material in any unit file.
- Verified by a full restart through the units, after which the complete gate suite was re-run: unauthenticated 401s, key-bearing status/TOC/select correct (1,014 scrolls; RFC 1 select intact).

## Exposure shape (probed, 2026-11-18)

- **Bind:** `0.0.0.0:1338` — reachable on the tailnet address and the node's home-LAN address (192.168.86.26). The node has no public IP; an off-tailnet probe from outside found no route (connection failed). No public-internet listener.
- **Tailnet reachability:** from a second ranch node, status/TOC/select returned correctly over the tailnet address. (Ranch shells with the Squid proxy env need a `NO_PROXY` bypass for tailnet addresses.)
- **Tenancy:** this daemon serves only the `ietf` tenant.
- **Publication:** none — the publicity hold governs; this is a tailnet-internal access surface, staged readiness only.

## Known pending item

The hosted artifact (5c576b37…) is the pre-extraction-fix build. When the fix lands, Watch re-hosts the refreshed artifact at this same endpoint/port (stop units → replace file → start units → hash-verify → re-run gate checks). Reference edges for RFCs ≥ ~8800 carry Judge's not-authoritative caveat until then; corpus content is fully usable.
