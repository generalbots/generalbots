#!/usr/bin/env python3
"""Build, verify and mirror the signed offline advisory bundle (#2720).

The bundle is a deterministic tarball of the online advisory feed snapshot
(CSAF + OSV JSON), a static HTML index with no external assets, and offline
verification instructions. A manifest records the publication date, advisory
IDs and source commit; SHA-256 checksums and an optional detached signature are
published next to the tarball, together with a `latest` pointer.

Everything runs on local files with the Python standard library. `verify` and
`check-parity` make no network calls, so they work on an air-gapped machine.

Usage:
    python3 scripts/security/build_offline_bundle.py build \
        --feed-dir /path/to/feed --static-dir security/offline \
        --template security/offline/index.template.html --out /tmp/bundle \
        --source-commit <40-hex-sha> [--publication-date YYYY-MM-DD] \
        [--sign-command 'cosign sign-blob --yes --output-signature {sig} {file}'] \
        [--public-key keys/release.pub.asc] [--key-fingerprint <hex>]
    python3 scripts/security/build_offline_bundle.py verify --bundle /tmp/bundle \
        [--expected-revision <sha>] [--require-signature] \
        [--verify-command 'gpg --verify {sig} {file}']
    python3 scripts/security/build_offline_bundle.py check-parity \
        --bundle /tmp/bundle --feed-dir /path/to/feed
    python3 scripts/security/build_offline_bundle.py --self-test

Exit codes: 0 clean, 1 integrity/policy failure, 2 bad arguments.
"""

from __future__ import annotations

import argparse
import json
import shutil
import socket
import sys
import tempfile
from pathlib import Path
from typing import Any

from offline_bundle_lib import BundleError, build
from offline_bundle_verify import check_parity, verify

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_STATIC = ROOT / "security" / "offline"
DEFAULT_TEMPLATE = DEFAULT_STATIC / "index.template.html"


class NoNetwork:
    """Context manager that proves an operation needs no networking."""

    def __enter__(self) -> None:
        self._original = socket.socket

        def blocked(*_args: Any, **_kwargs: Any) -> None:
            raise BundleError("network access is disabled for the offline bundle tool")

        socket.socket = blocked  # type: ignore[assignment]

    def __exit__(self, *_exc: Any) -> None:
        socket.socket = self._original  # type: ignore[assignment]


def _write_fixture(feed: Path) -> None:
    (feed / "csaf" / "2026").mkdir(parents=True)
    (feed / "osv").mkdir(parents=True)
    for number in ("0001", "0002"):
        advisory = f"GBSA-2026-{number}"
        csaf = {
            "document": {
                "tracking": {"id": advisory, "current_release_date": "2026-11-23T00:00:00Z"},
                "title": f"Example advisory {number}",
            }
        }
        osv = {"id": advisory, "summary": f"Example advisory {number}"}
        (feed / "csaf" / "2026" / f"{advisory}.json").write_text(
            json.dumps(csaf, indent=2), encoding="utf-8"
        )
        (feed / "osv" / f"{advisory}.json").write_text(json.dumps(osv, indent=2), encoding="utf-8")
    (feed / "csaf" / "changes.csv").write_text(
        "path,date\n2026/GBSA-2026-0001.json,2026-11-23\n", encoding="utf-8"
    )


def _write_mock_signer(temp: Path) -> tuple[str, str]:
    signer = temp / "mock_sign.py"
    verifier = temp / "mock_verify.py"
    signer.write_text(
        "import hashlib,sys\n"
        "data=open(sys.argv[1],'rb').read()\n"
        "open(sys.argv[2],'w').write('TESTSIG:'+hashlib.sha256(b'key'+data).hexdigest()+'\\n')\n",
        encoding="utf-8",
    )
    verifier.write_text(
        "import hashlib,sys\n"
        "sig=open(sys.argv[1]).read().strip()\n"
        "data=open(sys.argv[2],'rb').read()\n"
        "ok=sig=='TESTSIG:'+hashlib.sha256(b'key'+data).hexdigest()\n"
        "sys.exit(0 if ok else 1)\n",
        encoding="utf-8",
    )
    return (f"python3 {signer} {{file}} {{sig}}", f"python3 {verifier} {{sig}} {{file}}")


def self_test() -> int:
    """Offline proof: determinism, signing, tamper detection and feed parity."""
    revision = "a" * 40
    with NoNetwork(), tempfile.TemporaryDirectory(prefix="offline-bundle-selftest-") as folder:
        root = Path(folder)
        feed = root / "feed"
        _write_fixture(feed)
        sign_cmd, verify_cmd = _write_mock_signer(root)
        key = root / "release.pub.asc"
        key.write_bytes(b"-----BEGIN PGP PUBLIC KEY BLOCK-----\ntest\n-----END PGP PUBLIC KEY BLOCK-----\n")
        out1, out2 = root / "out1", root / "out2"
        common = dict(
            feed_dir=feed, static_dir=DEFAULT_STATIC, template=DEFAULT_TEMPLATE,
            source_commit=revision, publication_date="2026-11-23",
            sign_command=sign_cmd, public_key=key, key_fingerprint="ABCDEF0123456789",
        )
        first = build(out_dir=out1, **common)
        second = build(out_dir=out2, **common)
        if first != second:
            raise BundleError("self-test: builds are not deterministic")
        verify(out1, expected_revision=revision, verify_command=verify_cmd, require_signature=True)
        parity = check_parity(out1, feed, revision)
        if parity["advisory_ids"] != ["GBSA-2026-0001", "GBSA-2026-0002"]:
            raise BundleError("self-test: wrong advisory IDs")

        # An unsigned build must work, but cannot satisfy --require-signature.
        unsigned = root / "unsigned"
        build(out_dir=unsigned, **{**common, "sign_command": None})
        verify(unsigned, expected_revision=revision, verify_command=None, require_signature=False)
        try:
            verify(unsigned, expected_revision=revision, verify_command=None, require_signature=True)
        except BundleError:
            pass
        else:
            raise BundleError("self-test: unsigned bundle passed --require-signature")

        # A remote asset in the feed must be rejected at build time.
        bad_feed = root / "bad-feed"
        shutil.copytree(feed, bad_feed)
        (bad_feed / "notice.html").write_text(
            '<link rel="stylesheet" href="https://cdn.example.org/x.css">', encoding="utf-8"
        )
        try:
            build(out_dir=root / "bad-out", **{**common, "feed_dir": bad_feed})
        except BundleError:
            pass
        else:
            raise BundleError("self-test: remote HTML asset was accepted")

        # Tampering with any published file must fail closed.
        for name in ("SHA256SUMS", "MANIFEST.json", "latest", "SHA256SUMS.sig"):
            tampered = root / f"tamper-{name}"
            shutil.copytree(out1, tampered)
            target = tampered / name
            raw = target.read_bytes()
            target.write_bytes(raw.replace(b"a", b"b", 1) if name != "SHA256SUMS.sig"
                               else b"BADSIG\n")
            try:
                verify(tampered, expected_revision=revision, verify_command=verify_cmd,
                       require_signature=True)
            except BundleError:
                pass
            else:
                raise BundleError(f"self-test: tampered {name} was accepted")

        # Feed drift must be caught by the parity check.
        drifted = root / "drifted-feed"
        shutil.copytree(feed, drifted)
        (drifted / "osv" / "GBSA-2026-0003.json").write_text(
            json.dumps({"id": "GBSA-2026-0003"}), encoding="utf-8"
        )
        try:
            check_parity(out1, drifted, revision)
        except BundleError:
            pass
        else:
            raise BundleError("self-test: feed drift was accepted")
    print("offline bundle self-test: ok (offline, deterministic, signed, tamper-evident, parity)")
    return 0


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true", help="run the offline proof suite")
    parser.add_argument("--offline", action="store_true",
                        help="block all socket creation; proves the operation needs no network")
    commands = parser.add_subparsers(dest="command")

    builder = commands.add_parser("build", help="build a bundle from a feed snapshot")
    builder.add_argument("--feed-dir", required=True, type=Path)
    builder.add_argument("--static-dir", type=Path, default=DEFAULT_STATIC)
    builder.add_argument("--template", type=Path, default=DEFAULT_TEMPLATE)
    builder.add_argument("--out", required=True, type=Path)
    builder.add_argument("--source-commit", required=True)
    builder.add_argument("--publication-date", default="")
    builder.add_argument("--sign-command", default=None)
    builder.add_argument("--public-key", type=Path, default=None)
    builder.add_argument("--key-fingerprint", default=None)

    verifier = commands.add_parser("verify", help="verify a published bundle offline")
    verifier.add_argument("--bundle", required=True, type=Path)
    verifier.add_argument("--expected-revision", default=None)
    verifier.add_argument("--verify-command", default=None)
    verifier.add_argument("--require-signature", action="store_true")

    parity = commands.add_parser("check-parity", help="compare a bundle with a feed snapshot")
    parity.add_argument("--bundle", required=True, type=Path)
    parity.add_argument("--feed-dir", required=True, type=Path)
    parity.add_argument("--expected-revision", default=None)
    return parser


def _dispatch(args: argparse.Namespace) -> int:
    try:
        if args.self_test:
            return self_test()
        if args.command == "verify":
            manifest = verify(args.bundle, expected_revision=args.expected_revision,
                              verify_command=args.verify_command,
                              require_signature=args.require_signature)
            print(f"verified {manifest['bundle']} at revision {manifest['source_commit']}")
            if manifest.get("signature_file") and not args.verify_command:
                print("warning: signature present but not verified; pass --verify-command "
                      "or use --require-signature", file=sys.stderr)
            return 0
        if args.command == "check-parity":
            result = check_parity(args.bundle, args.feed_dir, args.expected_revision)
            print(json.dumps(result, sort_keys=True))
            return 0
        if args.command == "build":
            publication_date = args.publication_date
            if not publication_date:
                from datetime import datetime, timezone

                publication_date = datetime.now(timezone.utc).date().isoformat()
            manifest = build(
                feed_dir=args.feed_dir, static_dir=args.static_dir, template=args.template,
                out_dir=args.out, source_commit=args.source_commit,
                publication_date=publication_date, sign_command=args.sign_command,
                public_key=args.public_key, key_fingerprint=args.key_fingerprint,
            )
            print(json.dumps(manifest, sort_keys=True))
            return 0
        make_parser().print_help()
        return 2
    except BundleError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


def main(argv: list[str] | None = None) -> int:
    from contextlib import nullcontext

    args = make_parser().parse_args(argv)
    with NoNetwork() if args.offline else nullcontext():
        return _dispatch(args)


if __name__ == "__main__":
    raise SystemExit(main())
