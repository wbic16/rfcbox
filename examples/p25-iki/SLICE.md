# SLICE.md — the ietf.phext slice for a P25 IKI key server

**RFC Box run:** task 40.10.11/9.10.12/9.7.10 (first manual RFC Box run over ietf.phext, task 25.6.9/8.6.10/6.6.9)
**Date:** 2026-11-18 · **Corpus:** `~/workspace/phext/ietf-source.json` / `ietf.index.json` (artifact sha256 9ecd7946…)
**Requirement:** a KMF peer server that exports AES-256 traffic keys to a peer KMF run by another agency — the IKI function — with mutual authentication, wrapped key transport, signed packages and receipts, protected key storage, and an audit trail.

Every entry below was verified present in the corpus at the stated coordinate, with record status **True / VERIFIED**. `test_iki.py` re-checks these claims against the corpus on every run.

## The slice

| RFC | Title | ietf.phext coordinate | Corpus status | Role in this build | Implementation link recorded in the corpus |
|---|---|---|---|---|---|
| 9846 | The Transport Layer Security (TLS) Protocol Version 1.3 | 1.1.1/1.2.1/1.1.9846 | PROPOSED STANDARD, current | Mutual-TLS 1.3 channel between KMF peers (confidentiality + peer authentication) | **OpenSSL** — canonical, VERIFIED (Apache-2.0, https://github.com/openssl/openssl); also BoringSSL, Rustls, wolfSSL |
| 5280 | Internet X.509 PKI Certificate and CRL Profile | 1.1.1/1.2.1/1.1.5280 | PROPOSED STANDARD, current | KMF identity certificates issued by the test CA; TLS peer identity = certificate CN | **OpenSSL** — canonical, VERIFIED |
| 3394 | AES Key Wrap Algorithm | 1.1.1/1.2.1/1.1.3394 | INFORMATIONAL, current | Each TEK wrapped under the pairwise directional KEK for transport | none recorded — Tier 3, unmapped by design (see composition note) |
| 5652 | Cryptographic Message Syntax (CMS) | 1.1.1/1.2.1/1.1.5652 | INTERNET STANDARD, current | Detached CMS signatures over exchange packages and over receipts | none recorded — Tier 3, unmapped by design |
| 5116 | An Interface and Algorithms for Authenticated Encryption | 1.1.1/1.2.1/1.1.5116 | PROPOSED STANDARD, current | AES-256-GCM AEAD seals key material at rest in the key store | none recorded — Tier 3, unmapped by design |
| 5869 | HKDF (HMAC-based Extract-and-Expand KDF) | 1.1.1/1.2.1/1.1.5869 | INFORMATIONAL, current | Directional KEKs (A→B ≠ B→A) derived from the provisioned pairwise root secret | none recorded — Tier 3, unmapped by design |

## The composition

One library stack carries the whole slice: **OpenSSL**, reached through Python bindings — `ssl` for the TLS 1.3 channel and `cryptography` for key wrap, HKDF, AES-GCM, X.509 issuance, and CMS signing; CMS verification shells out to the OpenSSL CLI (`openssl cms -verify`).

- Where the corpus names a canonical implementation (RFC 9846, RFC 5280), the composition uses exactly that implementation: **OpenSSL**.
- Where the corpus records no implementation link (RFCs 3394, 5652, 5116, 5869 — all Tier 3, which the corpus build left unmapped by design), the composition choice is this build's own, recorded here rather than attributed to the corpus: the same OpenSSL stack, via the Python `cryptography` package. No Tier-3 gap blocked the build; the mapping gap is noted for the corpus's next implementation-mapping pass.

Flow: pairwise root provisioned out-of-band → HKDF derives a directional KEK per direction → exporter wraps each TEK with AES-KW (RFC 3394) → manifest signed as detached CMS (RFC 5652) → sent over mutual TLS 1.3 (RFC 9846) with X.509 peer identity (RFC 5280) → importer verifies signature and TLS/manifest identity match, unwraps, stores → importer returns a CMS-signed receipt → exporter verifies and stores it. All key material at rest is AES-256-GCM-sealed (RFC 5116); every event lands in a hash-chained append-only audit log.

## Followed redirect

- **RFC 8446** (TLS 1.3, coordinate 1.1.1/1.2.1/1.1.8446) is the number everyone cites — but the corpus records it **superseded**, obsoleted by RFC 9846 with successor coordinate 1.1.1/1.2.1/1.1.9846. Per MOAT rules the superseded entry is kept and annotated; the slice takes the successor, 9846. Same title, same canonical implementation.

## Near-misses — considered and rejected

| RFC | Coordinate | Why considered | Why not in the slice |
|---|---|---|---|
| 5246 (TLS 1.2) | 1.1.1/1.2.1/1.1.5246 | The previous TLS | Corpus records it obsoleted by 8446/9846. TLS 1.3 only. |
| 5649 (AES-KW with Padding) | 1.1.1/1.2.1/1.1.5649 | Wraps key material of any length | RFC 3394 covers the AES-256 keys this build moves (32-byte plaintext). 5649 becomes a requirement only if legacy short keys (e.g. 8-byte DES) must be wrapped — carried in GAP.md as a conditional requirement. |
| 7030 (EST) | 1.1.1/1.2.1/1.1.7030 | Standards-track certificate enrollment | The prototype provisions identities directly from a local test CA. EST is the production enrollment path — carried in GAP.md. |
| 7515 / 7516 (JWS / JWE) | 1.1.1/1.2.1/1.1.7515, …7516 | Signed/encrypted JSON packages | Would duplicate CMS + AES-KW in a second (JSON) stack; the KMF/PKI world this interoperates with is ASN.1-shaped. |
| 9180 (HPKE) | 1.1.1/1.2.1/1.1.9180 | Modern one-shot public-key key delivery | The IKI trust model is pre-provisioned pairwise KEKs between known KMFs; HPKE solves a problem this design does not have. |
| 7296 (IKEv2) + 4301 (IPsec) | 1.1.1/1.2.1/1.1.7296, …4301 | Key exchange + protected transport | Network-layer alternative; IKI is an application-layer KMF-to-KMF function, and the application-layer substrate (TLS) was selected. A deployment could still run the prototype over IPsec without changing it. |
| 6960 (OCSP) | 1.1.1/1.2.1/1.1.6960 | Certificate revocation status | Deferred: the test PKI uses short-lived certificates and a local trust store. Production revocation path — carried in GAP.md. |
| 3217 (Triple-DES / RC2 Key Wrapping) | 1.1.1/1.2.1/1.1.3217 | Historical CMS key wrap | DES-era cryptography; exactly what an AES-256 key service exists to replace. |

## What the slice does not contain

The IKI wire specification itself. IKI message formats are defined by the TIA-102 document family (Telecommunications Industry Association), which is paywalled and is **not in the IETF corpus** — ietf.phext holds IETF standards only. That absence is recorded as a gap, not papered over: see **GAP.md**.
