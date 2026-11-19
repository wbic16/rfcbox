"""demo.py — two-KMF IKI exchange demonstration.

Spins up KMF "Agency A" and KMF "Agency B" on localhost (mutual TLS 1.3),
provisions a pairwise root secret to both (simulating the out-of-band
provisioning step real KMFs perform), creates AES-256 TEKs at A, runs the
IKI export A -> B, and proves:
  * B now holds byte-identical keys,
  * A holds B's signed receipt for the exchange,
  * both audit logs record the exchange and their hash chains verify.

Run:  .venv/bin/python demo.py        (exit code 0 iff every check passes)
"""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from iki_server import ALG_AES_256, AuditLog, KMF, make_ca

RUN_DIR = Path(__file__).parent / "run-demo"


def run_demo(base: Path = RUN_DIR, verbose: bool = True) -> dict:
    def say(msg: str):
        if verbose:
            print(msg)

    if base.exists():
        shutil.rmtree(base)
    base.mkdir(parents=True)

    ca = make_ca(base / "pki")
    a = KMF("Agency A", base / "kmf-a", ca)
    b = KMF("Agency B", base / "kmf-b", ca)
    results: dict = {"checks": []}

    def check(label: str, ok: bool):
        results["checks"].append((label, bool(ok)))
        say(f"  [{'PASS' if ok else 'FAIL'}] {label}")

    try:
        a.start()
        b.start()
        say(f"KMF A listening on 127.0.0.1:{a.port} (mutual TLS 1.3)")
        say(f"KMF B listening on 127.0.0.1:{b.port} (mutual TLS 1.3)")

        # Out-of-band pairwise provisioning (in production: KMF administrators
        # install the pairwise root through a controlled channel).
        root = os.urandom(32)
        a.provision_peer("Agency B", root)
        b.provision_peer("Agency A", root)
        say("Pairwise root provisioned to both KMFs (out-of-band step)")

        key_ids = [1, 2, 3]
        originals = {kid: a.create_key(kid) for kid in key_ids}
        say(f"Agency A created AES-256 TEKs with key_ids {key_ids} (alg 0x84)")

        outcome = a.export_keyset(b, key_ids)
        manifest, receipt = outcome["manifest"], outcome["receipt"]
        say(f"Exchange {manifest['exchange_id'][:8]}... : "
            f"{len(manifest['keys'])} wrapped keys sent A -> B")

        identical = all(
            b.store.get_key("Agency A", kid, ALG_AES_256) == originals[kid]
            for kid in key_ids)
        check("B holds byte-identical TEKs after unwrap", identical)

        stored = a.store.get_receipt(manifest["exchange_id"])
        check("A holds B's signed receipt",
              stored is not None
              and stored["receipt"]["status"] == "accepted"
              and stored["receipt"]["received_key_ids"] == key_ids)
        say(f"Receipt from B: status={receipt['status']}, "
            f"keys={receipt['received_key_ids']}")

        check("A audit chain verifies",
              AuditLog.verify_chain(a.workdir / "audit.jsonl"))
        check("B audit chain verifies",
              AuditLog.verify_chain(b.workdir / "audit.jsonl"))
        b_events = (b.workdir / "audit.jsonl").read_text()
        check("B audit log records the exchange",
              manifest["exchange_id"] in b_events
              and "keyset_import_accepted" in b_events)
    finally:
        a.stop()
        b.stop()

    results["ok"] = all(ok for _, ok in results["checks"])
    return results


def main() -> int:
    print("=== P25 IKI key server prototype — two-KMF demo ===")
    results = run_demo()
    print("DEMO RESULT:", "PASS" if results["ok"] else "FAIL")
    return 0 if results["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
