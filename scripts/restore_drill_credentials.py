#!/usr/bin/env python3
"""Credential hygiene for the restore drill (#2679).

The full PostgreSQL DSN carries a password. Passing it as an argv element to
psql/pg_dump/pg_restore (or to the checker's own subprocesses) exposes it to
every local user through `ps`. This module strips credentials from a DSN for
argv and writes them to a 0600 env file that libpq tools inherit instead.
"""

from __future__ import annotations

import os
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import parse_qsl, unquote, urlencode, urlsplit, urlunsplit

SECRET_KEYS = ("password", "passfile", "pgpassfile")


def split_credentials(dsn: str) -> tuple[str, str, str]:
    """Return (safe_dsn, user, password) — no credential remains in the URL.

    Scheme, host, port, database and non-secret parameters (e.g. sslmode) are
    preserved so connection behaviour is unchanged; only userinfo and any
    password query parameter are removed.
    """
    parts = urlsplit(dsn)
    user, password, netloc = "", "", parts.netloc
    if "@" in netloc:
        userinfo, netloc = netloc.rsplit("@", 1)
        user, separator, raw_password = userinfo.partition(":")
        user = unquote(user)
        if separator:
            password = unquote(raw_password)
    query = []
    for key, value in parse_qsl(parts.query, keep_blank_values=True):
        if key.lower() == "password":
            password = unquote(value)
        elif key.lower() not in SECRET_KEYS:
            query.append((key, value))
    safe = urlunsplit((parts.scheme, netloc, parts.path, urlencode(query), parts.fragment))
    return safe, user, password


def write_pg_env(dsn: str, env_path: str | Path) -> str:
    """Write PGUSER/PGPASSWORD exports to a 0600 file; return the safe DSN.

    The shell sources this file, so the credentials are inherited by psql,
    pg_dump, pg_restore and the checker's own psql subprocesses — never argv.
    """
    safe, user, password = split_credentials(dsn)
    path = Path(env_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    if user:
        lines.append(f"export PGUSER={shlex.quote(user)}")
    if password:
        lines.append(f"export PGPASSWORD={shlex.quote(password)}")
    path.write_text("\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
    os.chmod(path, 0o600)
    return safe


def credential_self_test(failures: list[str]) -> None:
    """Offline proof that credentials leave neither argv nor a world-readable file."""
    cases = (
        ("postgres://alice:S3cr3t%21@db.example:5432/botserver?sslmode=disable", "S3cr3t!", "alice"),
        ("postgres://db.example:5432/botserver?password=leak&sslmode=disable", "leak", ""),
    )
    with tempfile.TemporaryDirectory() as tmp:
        for fake, secret, user in cases:
            path = Path(tmp) / "pg.env"
            safe = write_pg_env(fake, path)
            if secret in safe or user and user in safe or "password=" in safe:
                failures.append(f"safe DSN still carries credential material: {safe}")
            mode = path.stat().st_mode & 0o777
            if mode != 0o600:
                failures.append(f"credential env file mode is {mode:o}, expected 600")
            content = path.read_text(encoding="utf-8")
            if "PGPASSWORD=" not in content or secret not in content:
                failures.append("credential env file did not carry the decoded password")
            if user and f"PGUSER={shlex.quote(user)}" not in content:
                failures.append("credential env file did not carry the decoded user")
            path.unlink()

            # A spawned process must not expose the credential in /proc argv.
            if Path("/proc/self/cmdline").is_file():
                proc = subprocess.Popen(
                    [sys.executable, "-c", "import time; time.sleep(30)", safe],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
                try:
                    argv = Path(f"/proc/{proc.pid}/cmdline").read_bytes()
                finally:
                    proc.terminate()
                    proc.wait(timeout=10)
                if secret.encode() in argv or user and user.encode() in argv:
                    failures.append(f"credential leaked into a process argument list: {safe}")
