# GAP.md — RFC-Box gap record: P25 IKI key server

**Gap ID:** GAP-IKI-001 · **Opened by:** RFC Box run 40.10.11/9.10.12/9.7.10 (2026-11-18)
**Severity for this build:** documented limitation — the prototype is functional and honest about it; the gap blocks any *conformance* claim, not the substrate demonstration.

## The gap

**The TIA-102 Inter-KMF Interface (IKI) wire specification is not openly available and is not in ietf.phext.**

- IKI — encrypted interoperability between radios managed by different Key Management Facilities — is specified in the TIA-102 document family published by the Telecommunications Industry Association. Those documents are sold, not published openly.
- ietf.phext contains IETF standards. A TIA document cannot be in it, and a corpus query for the IKI wire format returns nothing — correctly.
- Consequently the on-the-wire message formats in this prototype follow the **IETF substrate** selected in SLICE.md (mutual TLS 1.3 + AES Key Wrap + CMS-signed JSON manifests and receipts), **not** the TIA IKI wire format. A production IKI peer built from this prototype would not interoperate with a vendor KMF, and this prototype does not claim otherwise.
- The same dependence applies to details this prototype takes from public P25 convention — e.g. algorithm identifier 0x84 for AES-256 and 16-bit key IDs. The authoritative registries and semantics (keysets, key addressing, activation and erasure procedures) live in the TIA-102 documents.

## What a conformance implementation would need (requirements)

1. **REQ-IKI-1 — The specification.** Acquire the current TIA-102 document(s) defining the Inter-KMF Interface, plus the companion P25 security-services documents covering algorithm/key-identifier registries and KMF procedures, under TIA license. Owner: Will / agency partner with a TIA license. Blocks: wire-format implementation.
2. **REQ-IKI-2 — Wire format re-implementation.** Re-encode the exchange (keyset delivery, acknowledgement/receipt semantics, error handling, keyset activation and zeroization) in the TIA message formats, replacing this prototype's JSON manifests while keeping the SLICE.md substrate where the TIA spec itself profiles IETF mechanisms.
3. **REQ-IKI-3 — Test vectors.** Obtain or generate IKI conformance test vectors from the licensed documents; add them to `test_iki.py` as known-answer tests before any interop claim.
4. **REQ-IKI-4 — Reference interop + CAP.** Test against at least one reference/vendor KMF implementation, and plan for P25 Compliance Assessment Program (CAP) testing — the recognized route to a P25 conformance claim.
5. **REQ-IKI-5 — Conditional slice additions.** If legacy key material must be transported, add RFC 5649 (AES Key Wrap with Padding, coordinate 1.1.1/1.2.1/1.1.5649) for key plaintexts RFC 3394 cannot wrap. For production PKI operations, add RFC 7030 (EST enrollment, 1.1.1/1.2.1/1.1.7030) and RFC 6960 (OCSP revocation, 1.1.1/1.2.1/1.1.6960).
6. **REQ-IKI-6 — Validated cryptography.** A production KMF handling real TEKs must run inside a FIPS 140-3 validated cryptographic module boundary. This prototype uses OpenSSL via Python `cryptography` outside any validated boundary — acceptable for a substrate prototype, not for field key material.

## Status

Gap recorded and carried on task 40.10.11/9.10.12/9.7.10. Nothing in this build depends on closing it; every conformance statement does.
