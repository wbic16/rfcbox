"""iki_server.py — P25 IKI (Inter-KMF Interface) key server prototype.

The first manual "RFC Box run" over ietf.phext (task 40.10.11/9.10.12/9.7.10).
This module implements the IKI *function* — one Key Management Facility (KMF)
securely exporting traffic-encryption keys to a peer KMF run by another
agency — standing on the IETF substrate selected in SLICE.md:

    RFC 9846  TLS 1.3            mutual-TLS channel (corpus redirect from 8446)
    RFC 5280  X.509 PKI profile  KMF identity certificates
    RFC 3394  AES Key Wrap       TEKs wrapped under a pairwise KEK
    RFC 5652  CMS                signed exchange packages and signed receipts
    RFC 5116  AEAD (AES-256-GCM) key material encrypted at rest in the store
    RFC 5869  HKDF               directional KEKs derived from a pairwise root

The IKI wire format itself is defined by TIA-102 documents that are NOT in
the IETF corpus (see GAP.md). The message formats here therefore follow the
IETF substrate above, not the TIA wire format. This is a substrate
prototype, not a certified or TIA-conformant product.

P25-style identifiers: keys are addressed by (key_id, alg_id); key_id is a
16-bit value and alg_id 0x84 denotes AES-256, per P25 security-services
convention (authoritative assignments live in the TIA-102 documents —
see GAP.md).
"""

from __future__ import annotations

import base64
import datetime
import hashlib
import http.client
import ipaddress
import json
import os
import sqlite3
import ssl
import subprocess
import tempfile
import threading
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.primitives.keywrap import aes_key_wrap, aes_key_unwrap
from cryptography.hazmat.primitives.serialization import pkcs7
from cryptography.x509.oid import NameOID

# --------------------------------------------------------------------------
# Constants
# --------------------------------------------------------------------------

ALG_AES_256 = 0x84          # P25 algorithm identifier for AES-256
KEY_LEN = 32                # AES-256
AUDIT_GENESIS = "0" * 64    # hash-chain genesis for the audit log


def utcnow() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def canonical(obj) -> bytes:
    """Deterministic JSON encoding used for everything that gets signed."""
    return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()


def b64e(b: bytes) -> str:
    return base64.b64encode(b).decode()


def b64d(s: str) -> bytes:
    return base64.b64decode(s.encode())


# --------------------------------------------------------------------------
# Test PKI (RFC 5280 profile, certificates issued by a local test CA)
# --------------------------------------------------------------------------

def _name(cn: str) -> x509.Name:
    return x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, cn)])


def make_ca(workdir: Path, cn: str = "IKI Test CA") -> dict:
    """Create a self-signed test CA. Returns paths + objects."""
    workdir.mkdir(parents=True, exist_ok=True)
    key = ec.generate_private_key(ec.SECP256R1())
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(_name(cn)).issuer_name(_name(cn))
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(minutes=5))
        .not_valid_after(now + datetime.timedelta(days=30))
        .add_extension(x509.BasicConstraints(True, None), critical=True)
        .add_extension(
            x509.KeyUsage(True, False, False, False, False, True, True, False, False),
            critical=True)
        .sign(key, hashes.SHA256())
    )
    ca_pem = workdir / "ca.pem"
    ca_pem.write_bytes(cert.public_bytes(serialization.Encoding.PEM))
    return {"cert": cert, "key": key, "pem_path": ca_pem}


def make_kmf_identity(workdir: Path, name: str, ca: dict) -> dict:
    """Issue a KMF identity certificate (server + client + signing use)."""
    workdir.mkdir(parents=True, exist_ok=True)
    key = ec.generate_private_key(ec.SECP256R1())
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(_name(name)).issuer_name(ca["cert"].subject)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(minutes=5))
        .not_valid_after(now + datetime.timedelta(days=30))
        .add_extension(x509.BasicConstraints(False, None), critical=True)
        .add_extension(
            x509.ExtendedKeyUsage([
                x509.oid.ExtendedKeyUsageOID.SERVER_AUTH,
                x509.oid.ExtendedKeyUsageOID.CLIENT_AUTH,
                # EMAIL_PROTECTION is the EKU OpenSSL requires for CMS/S-MIME
                # signing — our KMF identity also signs packages and receipts.
                x509.oid.ExtendedKeyUsageOID.EMAIL_PROTECTION,
            ]), critical=False)
        .add_extension(
            x509.SubjectAlternativeName([
                x509.DNSName("localhost"),
                x509.IPAddress(ipaddress.IPv4Address("127.0.0.1")),
            ]), critical=False)
        .sign(ca["key"], hashes.SHA256())
    )
    cert_pem = workdir / f"{name}.cert.pem"
    key_pem = workdir / f"{name}.key.pem"
    cert_pem.write_bytes(cert.public_bytes(serialization.Encoding.PEM))
    key_pem.write_bytes(key.private_bytes(
        serialization.Encoding.PEM,
        serialization.PrivateFormat.PKCS8,
        serialization.NoEncryption()))
    os.chmod(key_pem, 0o600)
    return {"name": name, "cert": cert, "key": key,
            "cert_path": cert_pem, "key_path": key_pem}


# --------------------------------------------------------------------------
# CMS signatures (RFC 5652): sign with `cryptography`, verify with OpenSSL
# --------------------------------------------------------------------------

def cms_sign_detached(data: bytes, identity: dict) -> bytes:
    """Detached CMS SignedData (DER) over `data`, signed by the KMF identity."""
    return (
        pkcs7.PKCS7SignatureBuilder()
        .set_data(data)
        .add_signer(identity["cert"], identity["key"], hashes.SHA256())
        .sign(serialization.Encoding.DER,
              [pkcs7.PKCS7Options.Binary, pkcs7.PKCS7Options.DetachedSignature])
    )


def cms_verify_detached(data: bytes, der_sig: bytes, ca_pem: Path) -> bool:
    """Verify a detached CMS signature against the test CA using OpenSSL."""
    with tempfile.TemporaryDirectory() as d:
        sig_p, data_p = Path(d) / "sig.p7", Path(d) / "data.bin"
        sig_p.write_bytes(der_sig)
        data_p.write_bytes(data)
        r = subprocess.run(
            ["openssl", "cms", "-verify", "-binary", "-inform", "DER",
             "-in", str(sig_p), "-content", str(data_p),
             "-CAfile", str(ca_pem), "-out", os.devnull],
            capture_output=True, text=True)
        return r.returncode == 0


# --------------------------------------------------------------------------
# Audit log: append-only JSONL with a SHA-256 hash chain
# --------------------------------------------------------------------------

class AuditLog:
    def __init__(self, path: Path, actor: str):
        self.path = path
        self.actor = actor
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._last = AUDIT_GENESIS
        if self.path.exists():
            lines = self.path.read_text().strip().splitlines()
            if lines:
                self._last = json.loads(lines[-1])["sha256"]
        self._lock = threading.Lock()

    def append(self, event: str, details: dict) -> dict:
        with self._lock:
            entry = {"ts": utcnow(), "actor": self.actor, "event": event,
                     "details": details, "prev_sha256": self._last}
            entry["sha256"] = hashlib.sha256(
                self._last.encode() + canonical(entry)).hexdigest()
            with open(self.path, "a") as f:
                f.write(json.dumps(entry, sort_keys=True) + "\n")
            self._last = entry["sha256"]
            return entry

    @staticmethod
    def verify_chain(path: Path) -> bool:
        prev = AUDIT_GENESIS
        for line in Path(path).read_text().strip().splitlines():
            e = json.loads(line)
            body = {k: v for k, v in e.items() if k != "sha256"}
            if e["prev_sha256"] != prev:
                return False
            if hashlib.sha256(prev.encode() + canonical(body)).hexdigest() != e["sha256"]:
                return False
            prev = e["sha256"]
        return True


# --------------------------------------------------------------------------
# Key store (SQLite). Key material is encrypted at rest with AES-256-GCM
# (RFC 5116 AEAD) under a per-KMF local master key.
# --------------------------------------------------------------------------

class KeyStore:
    def __init__(self, db_path: Path, master_key: bytes):
        assert len(master_key) == KEY_LEN
        self._aead = AESGCM(master_key)
        self.db = sqlite3.connect(str(db_path), check_same_thread=False)
        self._lock = threading.Lock()
        self.db.executescript("""
            CREATE TABLE IF NOT EXISTS keys (
                owner TEXT, key_id INTEGER, alg_id INTEGER,
                blob BLOB, origin TEXT, created TEXT,
                PRIMARY KEY (owner, key_id, alg_id));
            CREATE TABLE IF NOT EXISTS peers (
                peer TEXT PRIMARY KEY, root_blob BLOB);
            CREATE TABLE IF NOT EXISTS receipts (
                exchange_id TEXT PRIMARY KEY, receipt TEXT, signature BLOB);
        """)
        self.db.commit()

    # -- at-rest protection -------------------------------------------------
    def _seal(self, plaintext: bytes, aad: bytes) -> bytes:
        nonce = os.urandom(12)
        return nonce + self._aead.encrypt(nonce, plaintext, aad)

    def _unseal(self, blob: bytes, aad: bytes) -> bytes:
        return self._aead.decrypt(blob[:12], blob[12:], aad)

    # -- traffic keys --------------------------------------------------------
    def put_key(self, owner: str, key_id: int, alg_id: int,
                key: bytes, origin: str) -> None:
        aad = f"key:{owner}:{key_id}:{alg_id}".encode()
        with self._lock:
            self.db.execute(
                "INSERT OR REPLACE INTO keys VALUES (?,?,?,?,?,?)",
                (owner, key_id, alg_id, self._seal(key, aad), origin, utcnow()))
            self.db.commit()

    def get_key(self, owner: str, key_id: int, alg_id: int) -> bytes | None:
        row = self.db.execute(
            "SELECT blob FROM keys WHERE owner=? AND key_id=? AND alg_id=?",
            (owner, key_id, alg_id)).fetchone()
        if row is None:
            return None
        return self._unseal(row[0], f"key:{owner}:{key_id}:{alg_id}".encode())

    def list_keys(self, owner: str) -> list[dict]:
        rows = self.db.execute(
            "SELECT key_id, alg_id, origin FROM keys WHERE owner=? ORDER BY key_id",
            (owner,)).fetchall()
        return [{"key_id": k, "alg_id": a, "origin": o} for k, a, o in rows]

    # -- pairwise provisioning ----------------------------------------------
    def provision_peer(self, peer: str, root_secret: bytes) -> None:
        with self._lock:
            self.db.execute(
                "INSERT OR REPLACE INTO peers VALUES (?,?)",
                (peer, self._seal(root_secret, f"peer-root:{peer}".encode())))
            self.db.commit()

    def directional_kek(self, peer: str, src: str, dst: str) -> bytes:
        """HKDF (RFC 5869): derive the src->dst wrap key from the pairwise root."""
        row = self.db.execute(
            "SELECT root_blob FROM peers WHERE peer=?", (peer,)).fetchone()
        if row is None:
            raise KeyError(f"peer not provisioned: {peer}")
        root = self._unseal(row[0], f"peer-root:{peer}".encode())
        return HKDF(algorithm=hashes.SHA256(), length=KEY_LEN, salt=None,
                    info=f"iki-kek:{src}>{dst}".encode()).derive(root)

    # -- receipts -------------------------------------------------------------
    def put_receipt(self, exchange_id: str, receipt: dict, signature: bytes) -> None:
        with self._lock:
            self.db.execute(
                "INSERT OR REPLACE INTO receipts VALUES (?,?,?)",
                (exchange_id, json.dumps(receipt, sort_keys=True), signature))
            self.db.commit()

    def get_receipt(self, exchange_id: str) -> dict | None:
        row = self.db.execute(
            "SELECT receipt, signature FROM receipts WHERE exchange_id=?",
            (exchange_id,)).fetchone()
        if row is None:
            return None
        return {"receipt": json.loads(row[0]), "signature": bytes(row[1])}


# --------------------------------------------------------------------------
# KMF node: identity + store + mutual-TLS (RFC 9846) HTTPS service
# --------------------------------------------------------------------------

class KMF:
    def __init__(self, name: str, workdir: Path, ca: dict):
        self.name = name
        self.workdir = Path(workdir)
        self.workdir.mkdir(parents=True, exist_ok=True)
        self.ca = ca
        self.identity = make_kmf_identity(self.workdir, name, ca)
        master_path = self.workdir / "master.key"
        if not master_path.exists():
            master_path.write_bytes(os.urandom(KEY_LEN))
            os.chmod(master_path, 0o600)
        self.store = KeyStore(self.workdir / "keys.db", master_path.read_bytes())
        self.audit = AuditLog(self.workdir / "audit.jsonl", actor=name)
        self._server: ThreadingHTTPServer | None = None
        self.port: int | None = None

    # -- key management (local KMF operations) -------------------------------
    def create_key(self, key_id: int, alg_id: int = ALG_AES_256) -> bytes:
        key = os.urandom(KEY_LEN)
        self.store.put_key(self.name, key_id, alg_id, key, origin="created-locally")
        self.audit.append("key_created",
                          {"key_id": key_id, "alg_id": alg_id})
        return key

    def provision_peer(self, peer_name: str, root_secret: bytes) -> None:
        self.store.provision_peer(peer_name, root_secret)
        self.audit.append("peer_provisioned", {"peer": peer_name})

    # -- the IKI exchange: this KMF as exporter --------------------------------
    def export_keyset(self, peer: "KMF", key_ids: list[int],
                      alg_id: int = ALG_AES_256) -> dict:
        """Wrap this KMF's keys for `peer`, sign the package (CMS), send it
        over mutual TLS, verify the peer's signed receipt, store it."""
        kek = self.store.directional_kek(peer.name, self.name, peer.name)
        entries = []
        for kid in key_ids:
            tek = self.store.get_key(self.name, kid, alg_id)
            if tek is None:
                raise KeyError(f"no local key {kid}/{alg_id:#x}")
            entries.append({"key_id": kid, "alg_id": alg_id,
                            "wrapped": b64e(aes_key_wrap(kek, tek))})
        manifest = {"type": "iki-keyset", "version": 1,
                    "from": self.name, "to": peer.name,
                    "exchange_id": uuid.uuid4().hex,
                    "created": utcnow(), "keys": entries}
        manifest_bytes = canonical(manifest)
        signature = cms_sign_detached(manifest_bytes, self.identity)
        self.audit.append("keyset_export_sent",
                          {"exchange_id": manifest["exchange_id"],
                           "to": peer.name, "key_ids": key_ids,
                           "package_sha256": hashlib.sha256(manifest_bytes).hexdigest()})
        resp = self._post(peer, "/iki/exchange",
                          {"manifest_b64": b64e(manifest_bytes),
                           "signature_b64": b64e(signature)})
        receipt_bytes = b64d(resp["receipt_b64"])
        receipt_sig = b64d(resp["receipt_signature_b64"])
        if not cms_verify_detached(receipt_bytes, receipt_sig, self.ca["pem_path"]):
            raise ValueError("receipt signature verification FAILED")
        receipt = json.loads(receipt_bytes)
        if receipt["exchange_id"] != manifest["exchange_id"]:
            raise ValueError("receipt exchange_id mismatch")
        if receipt["package_sha256"] != hashlib.sha256(manifest_bytes).hexdigest():
            raise ValueError("receipt package hash mismatch")
        self.store.put_receipt(manifest["exchange_id"], receipt, receipt_sig)
        self.audit.append("receipt_received",
                          {"exchange_id": manifest["exchange_id"],
                           "from": peer.name, "status": receipt["status"]})
        return {"manifest": manifest, "receipt": receipt}

    # -- the IKI exchange: this KMF as importer (server side) ------------------
    def handle_exchange(self, manifest_bytes: bytes, signature: bytes,
                        peer_cn: str) -> dict:
        if not cms_verify_detached(manifest_bytes, signature, self.ca["pem_path"]):
            raise ValueError("package CMS signature verification FAILED")
        manifest = json.loads(manifest_bytes)
        if manifest["from"] != peer_cn:
            raise ValueError("TLS peer identity does not match manifest.from")
        if manifest["to"] != self.name:
            raise ValueError("manifest addressed to a different KMF")
        src = manifest["from"]
        kek = self.store.directional_kek(src, src, self.name)
        received = []
        for e in manifest["keys"]:
            tek = aes_key_unwrap(kek, b64d(e["wrapped"]))  # raises on tamper
            self.store.put_key(src, e["key_id"], e["alg_id"], tek,
                               origin=f"iki-import:{manifest['exchange_id']}")
            received.append(e["key_id"])
        receipt = {"type": "iki-receipt", "exchange_id": manifest["exchange_id"],
                   "from": self.name, "to": src, "status": "accepted",
                   "received_key_ids": received,
                   "package_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
                   "created": utcnow()}
        receipt_bytes = canonical(receipt)
        receipt_sig = cms_sign_detached(receipt_bytes, self.identity)
        self.audit.append("keyset_import_accepted",
                          {"exchange_id": manifest["exchange_id"],
                           "from": src, "key_ids": received,
                           "package_sha256": receipt["package_sha256"]})
        return {"receipt_b64": b64e(receipt_bytes),
                "receipt_signature_b64": b64e(receipt_sig)}

    # -- mutual-TLS plumbing -----------------------------------------------------
    def _server_context(self) -> ssl.SSLContext:
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.minimum_version = ssl.TLSVersion.TLSv1_3
        ctx.load_cert_chain(self.identity["cert_path"], self.identity["key_path"])
        ctx.load_verify_locations(cafile=str(self.ca["pem_path"]))
        ctx.verify_mode = ssl.CERT_REQUIRED
        return ctx

    def _client_context(self) -> ssl.SSLContext:
        ctx = ssl.create_default_context(
            ssl.Purpose.SERVER_AUTH, cafile=str(self.ca["pem_path"]))
        ctx.minimum_version = ssl.TLSVersion.TLSv1_3
        ctx.load_cert_chain(self.identity["cert_path"], self.identity["key_path"])
        return ctx

    def _post(self, peer: "KMF", path: str, payload: dict) -> dict:
        conn = http.client.HTTPSConnection(
            "127.0.0.1", peer.port, context=self._client_context(), timeout=15)
        conn.request("POST", path, body=canonical(payload),
                     headers={"Content-Type": "application/json"})
        resp = conn.getresponse()
        body = resp.read()
        conn.close()
        if resp.status != 200:
            raise RuntimeError(f"peer returned HTTP {resp.status}: {body[:200]!r}")
        return json.loads(body)

    def start(self) -> None:
        kmf = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):  # keep test output clean
                pass

            def do_GET(self):
                if self.path == "/iki/health":
                    self._reply(200, {"kmf": kmf.name, "status": "up"})
                else:
                    self._reply(404, {"error": "not found"})

            def do_POST(self):
                if self.path != "/iki/exchange":
                    self._reply(404, {"error": "not found"})
                    return
                try:
                    length = int(self.headers["Content-Length"])
                    payload = json.loads(self.rfile.read(length))
                    peer_cn = _peer_cn(self.request.getpeercert())
                    result = kmf.handle_exchange(
                        b64d(payload["manifest_b64"]),
                        b64d(payload["signature_b64"]), peer_cn)
                    self._reply(200, result)
                except Exception as exc:  # rejection is a normal IKI outcome
                    kmf.audit.append("keyset_import_rejected",
                                     {"reason": f"{type(exc).__name__}: {exc}"})
                    self._reply(400, {"error": f"{type(exc).__name__}: {exc}"})

            def _reply(self, code: int, obj: dict):
                body = canonical(obj)
                self.send_response(code)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        server.socket = self._server_context().wrap_socket(
            server.socket, server_side=True)
        self._server = server
        self.port = server.server_address[1]
        threading.Thread(target=server.serve_forever, daemon=True).start()

    def stop(self) -> None:
        if self._server is not None:
            self._server.shutdown()
            self._server.server_close()
            self._server = None


def _peer_cn(peercert: dict) -> str:
    for rdn in peercert.get("subject", ()):
        for attr, value in rdn:
            if attr == "commonName":
                return value
    raise ValueError("peer certificate has no CN")
