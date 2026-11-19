# p25-iki — P25 IKI key server prototype

The first manual **RFC Box run** over ietf.phext (task 40.10.11/9.10.12/9.7.10): a requirement went in, a slice of the corpus came out (SLICE.md), code was composed from the slice's canonical implementations, and what the corpus could not supply was recorded as a gap (GAP.md) instead of being improvised.

**What it is:** two KMF (Key Management Facility) peers exchanging AES-256 traffic keys — the IKI function — over mutual TLS 1.3, keys wrapped with AES Key Wrap under HKDF-derived pairwise KEKs, packages and receipts CMS-signed, key material AES-256-GCM-sealed at rest, every event in a hash-chained audit log.

**What it is not:** a TIA-102-conformant IKI implementation or a certified product. The IKI wire spec is a paywalled TIA document and is not in the IETF corpus — see GAP.md.

## Files

| File | Contents |
|---|---|
| `SLICE.md` | The corpus slice: RFCs, ietf.phext coordinates, statuses, implementation links, composition, rejected near-misses |
| `GAP.md` | The gap record: the missing TIA-102 IKI specification and the requirements to close it |
| `iki_server.py` | The prototype: test PKI, key store, KMF server + exchange logic |
| `demo.py` | Two-KMF demonstration (Agency A → Agency B) with PASS/FAIL checks |
| `test_iki.py` | Non-interactive test suite — re-verifies the slice against the corpus, the crypto, and the full exchange |
| `requirements.txt` | Python dependency (`cryptography`) |

## Run

```sh
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt
.venv/bin/python demo.py        # two-KMF exchange, prints PASS/FAIL
.venv/bin/python test_iki.py    # full suite, exit 0 iff all checks pass
```

CMS signature verification shells out to the `openssl` CLI (OpenSSL 3.x).
Demo artifacts (test CA, per-KMF stores, audit logs) land in `run-demo/`.
