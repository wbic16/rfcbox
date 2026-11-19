"""test_iki.py — non-interactive test suite for the IKI prototype.

Run:  .venv/bin/python test_iki.py     (exit code 0 iff all tests pass)

Covers: the SLICE claims against the corpus itself, the cryptographic
primitives (AES-KW tamper evidence, HKDF direction separation), the key
store's at-rest protection, CMS sign/verify including tamper rejection,
and the full two-KMF exchange from demo.py.
"""

from __future__ import annotations

import json
import os
import sqlite3
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from cryptography.hazmat.primitives.keywrap import (  # noqa: E402
    InvalidUnwrap, aes_key_unwrap, aes_key_wrap)

import iki_server as iki  # noqa: E402
from demo import run_demo  # noqa: E402

CORPUS = Path("/home/hatch/workspace/phext/ietf-source.json")

FAILURES: list[str] = []


def check(label: str, ok: bool, extra: str = ""):
    print(f"{'PASS' if ok else 'FAIL'}: {label}" + (f" ({extra})" if extra else ""))
    if not ok:
        FAILURES.append(label)


# The slice this build claims, as recorded in ietf.phext (see SLICE.md).
EXPECTED_SLICE = {
    9846: "1.1.1/1.2.1/1.1.9846",   # TLS 1.3 (current)
    5280: "1.1.1/1.2.1/1.1.5280",   # X.509 PKI profile
    3394: "1.1.1/1.2.1/1.1.3394",   # AES Key Wrap
    5652: "1.1.1/1.2.1/1.1.5652",   # CMS
    5116: "1.1.1/1.2.1/1.1.5116",   # AEAD
    5869: "1.1.1/1.2.1/1.1.5869",   # HKDF
}


def test_slice_against_corpus():
    src = json.loads(CORPUS.read_text())
    by = {e["rfc"]: e for e in src["entries"]}
    for num, coord in EXPECTED_SLICE.items():
        e = by.get(num)
        check(f"slice RFC {num} present in corpus at {coord}",
              e is not None and e["coordinate"] == coord
              and e["record_status"] == "True" and e["record_basis"] == "VERIFIED")
    e8446 = by[8446]
    check("corpus marks RFC 8446 superseded, redirecting to 9846",
          e8446["superseded"] is True and e8446["obsoleted_by"] == [9846]
          and e8446["successor_coordinates"] == ["1.1.1/1.2.1/1.1.9846"])
    check("corpus canonical implementation for TLS 1.3 is OpenSSL",
          any(i["name"] == "OpenSSL" and i["role"] == "canonical"
              for i in by[9846]["implementations"]))


def test_keywrap():
    kek, tek = os.urandom(32), os.urandom(32)
    wrapped = aes_key_wrap(kek, tek)
    check("RFC 3394 AES-KW round-trip", aes_key_unwrap(kek, wrapped) == tek)
    tampered = bytearray(wrapped)
    tampered[5] ^= 0x01
    try:
        aes_key_unwrap(kek, bytes(tampered))
        check("RFC 3394 AES-KW rejects tampered ciphertext", False)
    except InvalidUnwrap:
        check("RFC 3394 AES-KW rejects tampered ciphertext", True)


def test_hkdf_directions():
    with tempfile.TemporaryDirectory() as d:
        store = iki.KeyStore(Path(d) / "k.db", os.urandom(32))
        root = os.urandom(32)
        store.provision_peer("Peer", root)
        ab = store.directional_kek("Peer", "A", "B")
        ba = store.directional_kek("Peer", "B", "A")
        ab2 = store.directional_kek("Peer", "A", "B")
        check("RFC 5869 HKDF directional KEKs differ by direction", ab != ba)
        check("RFC 5869 HKDF derivation is deterministic", ab == ab2)


def test_store_at_rest():
    with tempfile.TemporaryDirectory() as d:
        db = Path(d) / "k.db"
        store = iki.KeyStore(db, os.urandom(32))
        tek = os.urandom(32)
        store.put_key("Agency A", 7, iki.ALG_AES_256, tek, origin="test")
        check("key store round-trip",
              store.get_key("Agency A", 7, iki.ALG_AES_256) == tek)
        raw = sqlite3.connect(str(db)).execute(
            "SELECT blob FROM keys").fetchone()[0]
        check("RFC 5116 AEAD: stored blob is not the plaintext key",
              bytes(raw) != tek and tek not in bytes(raw))


def test_cms_sign_verify():
    with tempfile.TemporaryDirectory() as d:
        base = Path(d)
        ca = iki.make_ca(base / "pki")
        ident = iki.make_kmf_identity(base / "kmf", "Agency A", ca)
        data = b"iki test package"
        sig = iki.cms_sign_detached(data, ident)
        check("RFC 5652 CMS signature verifies",
              iki.cms_verify_detached(data, sig, ca["pem_path"]))
        check("RFC 5652 CMS rejects tampered content",
              not iki.cms_verify_detached(data + b"x", sig, ca["pem_path"]))


def test_full_exchange():
    with tempfile.TemporaryDirectory() as d:
        results = run_demo(base=Path(d) / "run", verbose=False)
        for label, ok in results["checks"]:
            check(f"demo: {label}", ok)
        check("demo overall", results["ok"])


def main() -> int:
    test_slice_against_corpus()
    test_keywrap()
    test_hkdf_directions()
    test_store_at_rest()
    test_cms_sign_verify()
    test_full_exchange()
    print()
    if FAILURES:
        print(f"TEST RESULT: FAIL ({len(FAILURES)} failing checks)")
        return 1
    print("TEST RESULT: PASS (all checks)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
