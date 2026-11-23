#!/usr/bin/env python3
"""Vendored, offline static server for `cargo tauri dev` (issue #2695).

Serves ``../botui/ui`` on 127.0.0.1:3000 using only the Python 3 standard
library. No npm, no npx, no CDN and no network fetch: dev mode stays fully
offline-first. The bind address is restricted to loopback, and a port that is
already in use makes the process exit non-zero so Tauri fails loudly instead of
loading a stale server on :3000.
"""
from __future__ import annotations

import argparse
import functools
import http.server
import ipaddress
import sys
from pathlib import Path

DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 3000
UI_DIR = Path(__file__).resolve().parent.parent / "botui" / "ui"


def is_loopback(host: str) -> bool:
    """Return True only for loopback addresses (never bind a public iface)."""
    if host == "localhost":
        return True
    try:
        return ipaddress.ip_address(host).is_loopback
    except ValueError:
        return False


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--host",
        default=DEFAULT_HOST,
        help=f"bind address, loopback only (default: {DEFAULT_HOST})",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=DEFAULT_PORT,
        help=f"listen port (default: {DEFAULT_PORT})",
    )
    parser.add_argument(
        "--directory",
        type=Path,
        default=UI_DIR,
        help="static root (default: ../botui/ui)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)

    if not is_loopback(args.host):
        print(
            f"dev-static-server: refusing non-loopback host {args.host!r}",
            file=sys.stderr,
        )
        return 2

    root = args.directory.resolve()
    if not root.is_dir():
        print(f"dev-static-server: directory not found: {root}", file=sys.stderr)
        return 2

    handler = functools.partial(
        http.server.SimpleHTTPRequestHandler,
        directory=str(root),
    )
    try:
        with http.server.ThreadingHTTPServer((args.host, args.port), handler) as server:
            print(
                f"dev-static-server: serving {root} at http://{args.host}:{args.port}",
                flush=True,
            )
            server.serve_forever()
    except OSError as exc:
        print(
            f"dev-static-server: cannot bind {args.host}:{args.port}: {exc}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
