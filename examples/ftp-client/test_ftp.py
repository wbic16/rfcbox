#!/usr/bin/env python3
"""test_ftp.py — non-interactive suite for ftp_client.py.

Three layers, mirroring the RFC Box pattern:
  1. The slice is re-verified against ietf.phext's source records.
  2. The client is exercised against a hermetic local FTP server
     (pyftpdlib fixture, ephemeral port, temp directory).
  3. The same exercise runs over explicit FTPS (RFC 4217) against a
     TLS fixture with a self-signed certificate.

Exit code 0 iff every check passes. Prints one PASS/FAIL line per check.
"""

import json
import os
import subprocess
import sys
import tempfile
import threading
from datetime import datetime, timedelta, timezone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ftp_client import FTPClient, FTPClientError  # noqa: E402

CORPUS = os.path.expanduser("~/workspace/phext/ietf-source.json")
FAILURES = []


def check(label, cond, detail=""):
    if cond:
        print(f"PASS: {label}")
    else:
        print(f"FAIL: {label}" + (f" — {detail}" if detail else ""))
        FAILURES.append(label)


# ---------------------------------------------------------------- slice
def test_slice():
    with open(CORPUS) as fh:
        entries = {e["rfc"]: e for e in json.load(fh)["entries"]}
    slice_rfcs = {
        959: "1.1.1/1.2.1/1.1.959",
        2389: "1.1.1/1.2.1/1.1.2389",
        3659: "1.1.1/1.2.1/1.1.3659",
        2640: "1.1.1/1.2.1/1.1.2640",
        2428: "1.1.1/1.2.1/1.1.2428",
        2228: "1.1.1/1.2.1/1.1.2228",
        4217: "1.1.1/1.2.1/1.1.4217",
        9846: "1.1.1/1.2.1/1.1.9846",
        3986: "1.1.1/1.2.1/1.1.3986",
    }
    for num, coord in slice_rfcs.items():
        e = entries.get(num)
        check(f"slice RFC {num} present in corpus at {coord}",
              e is not None and e["coordinate"] == coord
              and e["record_status"] == "True" and e["record_basis"] == "VERIFIED")
    e1738 = entries[1738]
    check("corpus marks RFC 1738 superseded; RFC 3986 updates it",
          e1738["superseded"] and 3986 in e1738["updated_by"])
    check("corpus records RFC 4217 updated by RFC 8996 (TLS 1.0/1.1 deprecated)",
          8996 in entries[4217]["updated_by"])
    impls = {i["name"]: i.get("role") for i in entries[9846]["implementations"]}
    check("corpus canonical implementation for TLS 1.3 (9846) is OpenSSL",
          impls.get("OpenSSL") == "canonical")


# ------------------------------------------------------------- fixtures
def make_cert(tmp):
    from cryptography import x509
    from cryptography.hazmat.primitives import hashes, serialization
    from cryptography.hazmat.primitives.asymmetric import rsa
    from cryptography.x509.oid import NameOID
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "localhost")])
    cert = (x509.CertificateBuilder()
            .subject_name(name).issuer_name(name)
            .public_key(key.public_key())
            .serial_number(x509.random_serial_number())
            .not_valid_before(datetime.now(timezone.utc) - timedelta(days=1))
            .not_valid_after(datetime.now(timezone.utc) + timedelta(days=30))
            .add_extension(x509.SubjectAlternativeName([x509.DNSName("localhost")]), False)
            .sign(key, hashes.SHA256()))
    cert_p = os.path.join(tmp, "cert.pem")
    key_p = os.path.join(tmp, "key.pem")
    with open(cert_p, "wb") as fh:
        fh.write(cert.public_bytes(serialization.Encoding.PEM))
    with open(key_p, "wb") as fh:
        fh.write(key.private_bytes(serialization.Encoding.PEM,
                                   serialization.PrivateFormat.TraditionalOpenSSL,
                                   serialization.NoEncryption()))
    return cert_p, key_p


def start_server(root, tls=None):
    from pyftpdlib.authorizers import DummyAuthorizer
    from pyftpdlib.handlers import FTPHandler, TLS_FTPHandler
    from pyftpdlib.servers import FTPServer
    auth = DummyAuthorizer()
    auth.add_anonymous(root, perm="elradfmwMT")
    auth.add_user("alice", "secret", root, perm="elradfmwMT")
    if tls:
        cert_p, key_p = tls
        handler = TLS_FTPHandler
        handler.certfile = cert_p
        handler.keyfile = key_p
        handler.tls_control_required = True
        handler.tls_data_required = True
    else:
        handler = FTPHandler
    handler.authorizer = auth
    handler.banner = "rfcbox fixture"
    server = FTPServer(("127.0.0.1", 0), handler)
    port = server.socket.getsockname()[1]
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    return server, port


# -------------------------------------------------------------- exercise
def exercise(port, tls, label):
    payload = bytes(range(256)) * 64 + b"\nrfcbox-ftp-client fixture payload\n"
    with FTPClient("127.0.0.1", port=port, tls=tls, tls_verify=False,
                   timeout=15) as c:
        check(f"{label}: login (anonymous)", True)
        check(f"{label}: FEAT advertises MLST/MLSD (RFC 2389/3659)",
              "MLST" in c.features or "MLSD" in c.features,
              f"features={sorted(c.features)}")
        c.put_bytes(payload, "hello.bin")
        rows, kind = c.list()
        names = [n for n, _ in rows]
        check(f"{label}: upload then MLSD list shows the file",
              kind == "MLSD" and "hello.bin" in names, f"kind={kind} names={names}")
        check(f"{label}: SIZE parsed (RFC 3659)",
              c.size("hello.bin") == len(payload))
        mdtm = c.mdtm("hello.bin")
        check(f"{label}: MDTM parsed as UTC timestamp (RFC 3659)",
              isinstance(mdtm, datetime) and mdtm.tzinfo is not None
              and abs((datetime.now(timezone.utc) - mdtm).total_seconds()) < 3600)
        back = c.get_bytes("hello.bin")
        check(f"{label}: download is byte-identical", back == payload)
        sub = c.mkdir("sub")
        c.cd("sub")
        check(f"{label}: cd/pwd round trip", c.pwd().endswith("/sub"))
        c.cd("/")
        c.rmdir("sub")
        c.delete("hello.bin")
        names = [n for n, _ in c.list()[0]]
        check(f"{label}: delete cleans up", "hello.bin" not in names)


def test_errors(port):
    try:
        with FTPClient("127.0.0.1", port=port, user="alice",
                       password="wrong", timeout=10):
            pass
        check("errors: bad password raises FTPClientError (no traceback)", False)
    except FTPClientError:
        check("errors: bad password raises FTPClientError (no traceback)", True)
    except Exception as exc:  # noqa: BLE001
        check("errors: bad password raises FTPClientError (no traceback)", False,
              f"got {type(exc).__name__}")


def test_cli(port, tmp):
    cli = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ftp_client.py")
    local = os.path.join(tmp, "cli.txt")
    with open(local, "wb") as fh:
        fh.write(b"via the CLI\n")
    run = subprocess.run([sys.executable, cli, "--port", str(port), "127.0.0.1",
                          "put", local, "cli.txt"], capture_output=True, text=True)
    check("CLI: put exits 0", run.returncode == 0, run.stderr.strip())
    run = subprocess.run([sys.executable, cli, "--port", str(port), "127.0.0.1",
                          "ls"], capture_output=True, text=True)
    check("CLI: ls shows uploaded file", run.returncode == 0 and "cli.txt" in run.stdout,
          run.stderr.strip())
    run = subprocess.run([sys.executable, cli, "--port", str(port), "127.0.0.1",
                          "get", "no-such-file.bin"], capture_output=True, text=True)
    check("CLI: missing file is a clean one-line error, exit 1, no traceback",
          run.returncode == 1 and "error:" in run.stderr and "Traceback" not in run.stderr,
          run.stderr.strip()[:200])


def main():
    test_slice()
    with tempfile.TemporaryDirectory() as tmp:
        plain_root = os.path.join(tmp, "plain")
        tls_root = os.path.join(tmp, "tls")
        os.mkdir(plain_root)
        os.mkdir(tls_root)
        plain, pport = start_server(plain_root)
        try:
            exercise(pport, tls=False, label="ftp")
            test_errors(pport)
            test_cli(pport, tmp)
        finally:
            plain.close_all()
        cert = make_cert(tmp)
        secure, sport = start_server(tls_root, tls=cert)
        try:
            exercise(sport, tls=True, label="ftps")
        finally:
            secure.close_all()
    if FAILURES:
        print(f"\nTEST RESULT: FAIL ({len(FAILURES)} check(s): {FAILURES})")
        return 1
    print("\nTEST RESULT: PASS (all checks)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
