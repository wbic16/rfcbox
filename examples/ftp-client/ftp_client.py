#!/usr/bin/env python3
"""ftp_client.py — a basic FTP client composed via the RFC Box pattern over ietf.phext.

Slice (see SLICE.md for coordinates and corpus records):
  RFC 959   File Transfer Protocol (base protocol)
  RFC 2389  FEAT — feature negotiation
  RFC 3659  MLSD/MLST listings, SIZE, MDTM
  RFC 2640  UTF-8 path names (OPTS UTF8 ON)
  RFC 2428  EPSV — extended passive mode (IPv6/NAT-safe)
  RFC 2228  FTP Security Extensions (AUTH / PBSZ / PROT framework)
  RFC 4217  Securing FTP with TLS — explicit FTPS (--tls)
  RFC 9846  TLS 1.3 — the TLS the corpus currently records (4217's TLS 1.0
            substrate was obsoleted; RFC 8996 deprecates TLS 1.0/1.1)
  RFC 3986  URI generic syntax — ftp:// target parsing (RFC 1738, which
            defined the ftp URL, is superseded in the corpus; 3986 updates it)

Composition: Python 3 standard library — ftplib for the protocol, ssl
(OpenSSL, the corpus's canonical TLS implementation) for FTPS, urllib.parse
for RFC 3986 syntax. See SLICE.md for what the corpus maps vs what this
build composes.

Ordinary server refusals (bad login, missing file, permission denied) are
reported as one-line errors on stderr with exit code 1 — never tracebacks.
"""

import argparse
import ftplib
import os
import ssl
import sys
from datetime import datetime, timezone
from urllib.parse import unquote, urlparse


class FTPClientError(Exception):
    """An ordinary, reportable failure (server refusal, network, usage)."""


def _clean(exc):
    """Turn an ftplib/socket failure into a one-line message."""
    if isinstance(exc, ftplib.error_perm):
        return f"server refused: {exc}"
    if isinstance(exc, ftplib.error_temp):
        return f"server temporary failure: {exc}"
    if isinstance(exc, ftplib.error_proto):
        return f"protocol error: {exc}"
    if isinstance(exc, (TimeoutError, ConnectionError, OSError)):
        return f"network error: {exc}"
    return f"{type(exc).__name__}: {exc}"


class FTPClient:
    def __init__(self, host, port=21, user="anonymous", password="anonymous@",
                 tls=False, timeout=30, passive=True, use_epsv=True,
                 tls_verify=True):
        self.host = host
        self.port = port
        self.user = user
        self.password = password
        self.use_tls = tls
        self.timeout = timeout
        self.passive = passive
        self.use_epsv = use_epsv
        self.tls_verify = tls_verify
        self.ftp = None
        self.features = {}

    # -- connection ----------------------------------------------------
    def connect(self):
        try:
            if self.use_tls:
                ctx = ssl.create_default_context()
                if not self.tls_verify:
                    ctx.check_hostname = False
                    ctx.verify_mode = ssl.CERT_NONE
                ftp = ftplib.FTP_TLS(context=ctx)
                ftp.connect(self.host, self.port, timeout=self.timeout)
                # RFC 4217: explicit AUTH TLS, then protect the data channel.
                ftp.auth()
                ftp.prot_p()
            else:
                ftp = ftplib.FTP()
                ftp.connect(self.host, self.port, timeout=self.timeout)
            ftp.login(self.user, self.password)
        except ftplib.all_errors + (OSError,) as exc:
            raise FTPClientError(_clean(exc)) from None
        ftp.set_pasv(self.passive)
        ftp.use_epsv = self.use_epsv  # RFC 2428 EPSV when passive (ftplib knob)
        # Binary transfers by default (RFC 959 TYPE I). This is also what
        # makes RFC 3659 SIZE meaningful: servers may refuse SIZE in
        # ASCII mode, where the byte count is not well defined.
        try:
            ftp.sendcmd("TYPE I")
        except ftplib.all_errors:
            pass
        self.ftp = ftp
        self.features = self._feat()
        # RFC 2640: switch path names to UTF-8 when the server offers it.
        if "UTF8" in self.features:
            try:
                ftp.sendcmd("OPTS UTF8 ON")
                ftp.encoding = "utf-8"
            except ftplib.all_errors:
                pass  # advertised but refused; stay in the default encoding
        return self

    def _feat(self):
        """RFC 2389 FEAT: returns {FEATURE: rest-of-line} for this server."""
        feats = {}
        try:
            resp = self.ftp.sendcmd("FEAT")
        except ftplib.all_errors:
            return feats
        for line in resp.splitlines()[1:]:  # first line is the 211 header
            line = line.strip()
            if not line or line.startswith("211"):
                continue
            parts = line.split(None, 1)
            feats[parts[0].upper()] = parts[1] if len(parts) > 1 else ""
        return feats

    def close(self):
        if self.ftp is not None:
            try:
                self.ftp.quit()
            except Exception:
                self.ftp.close()
            self.ftp = None

    def __enter__(self):
        return self.connect()

    def __exit__(self, *exc):
        self.close()

    def _guard(self, fn, *args, **kwargs):
        try:
            return fn(*args, **kwargs)
        except ftplib.all_errors + (OSError,) as exc:
            raise FTPClientError(_clean(exc)) from None

    # -- commands --------------------------------------------------------
    def pwd(self):
        return self._guard(self.ftp.pwd)

    def cd(self, path):
        return self._guard(self.ftp.cwd, path)

    def list(self, path=None):
        """Directory listing: MLSD (RFC 3659) when offered, LIST fallback.

        Returns a list of (name, facts-dict) for MLSD, or (line, {}) rows
        for a LIST fallback.
        """
        # RFC 3659: the FEAT token is "MLST"; its presence advertises
        # support for both MLST and MLSD.
        try:
            if "MLSD" in self.features or "MLST" in self.features:
                rows = []
                def collect():
                    for name, facts in self.ftp.mlsd(path or "", facts=["type", "size", "modify"]):
                        rows.append((name, facts))
                self._guard(collect)
                return rows, "MLSD"
            lines = []
            cmd = "LIST" + (f" {path}" if path else "")
            self._guard(self.ftp.retrlines, cmd, lines.append)
            return [(line, {}) for line in lines], "LIST"
        finally:
            # ftplib implements listings (MLSD included) over retrlines,
            # which switches the session to TYPE A (ASCII) and does not
            # restore it. Re-assert binary mode so a following SIZE or
            # transfer is not refused or mangled.
            try:
                self.ftp.sendcmd("TYPE I")
            except Exception:
                pass

    def get(self, remote, local=None):
        local = local or os.path.basename(remote.rstrip("/")) or "download"
        def run():
            with open(local, "wb") as fh:
                self.ftp.retrbinary(f"RETR {remote}", fh.write)
        try:
            self._guard(run)
        except FTPClientError:
            # A refused/failed RETR must not leave a zero-byte (or partial)
            # file behind masquerading as a download.
            try:
                os.unlink(local)
            except OSError:
                pass
            raise
        return local

    def get_bytes(self, remote):
        chunks = []
        self._guard(self.ftp.retrbinary, f"RETR {remote}", chunks.append)
        return b"".join(chunks)

    def put(self, local, remote=None):
        remote = remote or os.path.basename(local)
        def run():
            with open(local, "rb") as fh:
                self.ftp.storbinary(f"STOR {remote}", fh)
        self._guard(run)
        return remote

    def put_bytes(self, data, remote):
        import io
        self._guard(self.ftp.storbinary, f"STOR {remote}", io.BytesIO(data))
        return remote

    def size(self, path):
        """RFC 3659 SIZE: byte count of a remote file."""
        resp = self._guard(self.ftp.sendcmd, f"SIZE {path}")
        try:
            return int(resp.split()[-1])
        except (ValueError, IndexError):
            raise FTPClientError(f"unparsable SIZE reply: {resp!r}") from None

    def mdtm(self, path):
        """RFC 3659 MDTM: modification time of a remote file (UTC)."""
        resp = self._guard(self.ftp.sendcmd, f"MDTM {path}")
        try:
            stamp = resp.split()[-1].split(".")[0]
            return datetime.strptime(stamp, "%Y%m%d%H%M%S").replace(tzinfo=timezone.utc)
        except (ValueError, IndexError):
            raise FTPClientError(f"unparsable MDTM reply: {resp!r}") from None

    def delete(self, path):
        self._guard(self.ftp.delete, path)

    def mkdir(self, path):
        return self._guard(self.ftp.mkd, path)

    def rmdir(self, path):
        self._guard(self.ftp.rmd, path)


def parse_target(target, args):
    """Target is a host or an ftp:// URL (RFC 3986 syntax; the ftp scheme
    semantics — user/pass/host/port/path — are RFC 1738's, kept annotated)."""
    if "://" not in target:
        return target, args.port, args.user, args.password, ""
    u = urlparse(target)
    if u.scheme not in ("ftp", "ftps"):
        raise FTPClientError(f"unsupported URL scheme: {u.scheme!r}")
    host = u.hostname or ""
    port = u.port or args.port
    user = unquote(u.username) if u.username else args.user
    password = unquote(u.password) if u.password else args.password
    if u.scheme == "ftps":
        args.tls = True
    return host, port, user, password, unquote(u.path or "")


def main(argv=None):
    ap = argparse.ArgumentParser(prog="ftp_client.py", description=__doc__.splitlines()[0])
    ap.add_argument("target", help="host or ftp://[user[:pass]@]host[:port][/path]")
    ap.add_argument("command", help="ls | get | put | pwd | cd | size | mdtm | rm | mkdir | rmdir | features")
    ap.add_argument("cmdargs", nargs="*", help="command arguments")
    ap.add_argument("--port", type=int, default=21)
    ap.add_argument("--user", default="anonymous")
    ap.add_argument("--password", default="anonymous@")
    ap.add_argument("--tls", action="store_true", help="explicit FTPS (RFC 4217 AUTH TLS)")
    ap.add_argument("--no-verify", action="store_true", help="do not verify the FTPS server certificate")
    ap.add_argument("--active", action="store_true", help="active mode (PORT) instead of passive")
    ap.add_argument("--no-epsv", action="store_true", help="use classic PASV instead of EPSV (RFC 2428)")
    ap.add_argument("--timeout", type=int, default=30)
    args = ap.parse_args(argv)

    try:
        host, port, user, password, urlpath = parse_target(args.target, args)
        if not host:
            raise FTPClientError("no host in target")
        client = FTPClient(host, port=port, user=user, password=password,
                           tls=args.tls, timeout=args.timeout,
                           passive=not args.active, use_epsv=not args.no_epsv,
                           tls_verify=not args.no_verify)
        with client:
            cmd, rest = args.command, args.cmdargs
            if cmd == "features":
                for name in sorted(client.features):
                    print(f"{name} {client.features[name]}".rstrip())
            elif cmd == "pwd":
                print(client.pwd())
            elif cmd == "cd":
                if not rest:
                    raise FTPClientError("cd needs a directory")
                client.cd(rest[0])
                print(client.pwd())
            elif cmd == "ls":
                path = rest[0] if rest else (urlpath.lstrip("/") or None)
                rows, kind = client.list(path)
                for name, facts in rows:
                    if kind == "MLSD":
                        print(f"{facts.get('type', '?'):4} {facts.get('size', '-'):>10}  {name}")
                    else:
                        print(name)
            elif cmd == "get":
                remote = rest[0] if rest else urlpath.lstrip("/")
                if not remote:
                    raise FTPClientError("get needs a remote file")
                local = client.get(remote, rest[1] if len(rest) > 1 else None)
                print(f"downloaded {remote} -> {local} ({os.path.getsize(local)} bytes)")
            elif cmd == "put":
                if not rest:
                    raise FTPClientError("put needs a local file")
                if not os.path.isfile(rest[0]):
                    raise FTPClientError(f"no such local file: {rest[0]}")
                remote = client.put(rest[0], rest[1] if len(rest) > 1 else None)
                print(f"uploaded {rest[0]} -> {remote}")
            elif cmd == "size":
                if not rest:
                    raise FTPClientError("size needs a remote file")
                print(client.size(rest[0]))
            elif cmd == "mdtm":
                if not rest:
                    raise FTPClientError("mdtm needs a remote file")
                print(client.mdtm(rest[0]).isoformat())
            elif cmd == "rm":
                if not rest:
                    raise FTPClientError("rm needs a remote file")
                client.delete(rest[0])
                print(f"deleted {rest[0]}")
            elif cmd == "mkdir":
                if not rest:
                    raise FTPClientError("mkdir needs a directory name")
                print(client.mkdir(rest[0]))
            elif cmd == "rmdir":
                if not rest:
                    raise FTPClientError("rmdir needs a directory name")
                client.rmdir(rest[0])
                print(f"removed directory {rest[0]}")
            else:
                raise FTPClientError(f"unknown command: {cmd}")
    except FTPClientError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
