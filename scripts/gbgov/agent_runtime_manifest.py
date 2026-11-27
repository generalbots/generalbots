#!/usr/bin/env python3
"""Build and verify `agent-runtime-manifest.json` (General Bots Lite Agent mode).

The schema is documented in scripts/gbgov/README.md; `validate()` below is its
executable form. Standard library only.

Subcommands
-----------
asset   describe one built file as a manifest fragment (size, SHA-256, facts)
merge   combine the fragments into the manifest and a SHA256SUMS file
verify  check every asset in a directory against a manifest
--self-test  exercise the three commands on temporary files

Exit codes: 0 ok, 1 validation or verification failure, 2 usage error.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import tempfile
from pathlib import Path

SCHEMA_VERSION = 1
MANIFEST_NAME = "agent-runtime-manifest.json"
SUMS_NAME = "SHA256SUMS"
DISTRO_NAME = "GeneralBotsLite"
WORKSPACE_ALIAS = "gb-workspace-office"

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
FILE_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._+-]*$")
VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$")
SNAPSHOT_RE = re.compile(r"^[0-9]{8}T[0-9]{6}Z$")
BUILT_AT_RE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")

# role -> (kind, required string fields besides the common ones)
ROLES = {
    "wsl_rootfs": ("wsl-rootfs", ("distro_name", "compression", "debian_suite", "incus_version")),
    "workspace_image": ("incus-image", ("incus_fingerprint", "alias", "profile", "provisioned_marker", "format", "compression")),
}


class ManifestError(Exception):
    pass


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate(manifest: dict) -> list[str]:
    """Every schema violation, empty when the manifest is valid."""
    problems: list[str] = []
    if manifest.get("schema_version") != SCHEMA_VERSION:
        problems.append(f"schema_version must be {SCHEMA_VERSION}")
    if not VERSION_RE.match(str(manifest.get("runtime_version", ""))):
        problems.append("runtime_version must be major.minor.patch[-prerelease]")
    if not COMMIT_RE.match(str(manifest.get("commit", ""))):
        problems.append("commit must be a 40-character lowercase SHA")
    if not BUILT_AT_RE.match(str(manifest.get("built_at", ""))):
        problems.append("built_at must be UTC YYYY-MM-DDTHH:MM:SSZ")
    if not SNAPSHOT_RE.match(str(manifest.get("debian_snapshot", ""))):
        problems.append("debian_snapshot must be YYYYMMDDTHHMMSSZ")
    assets = manifest.get("assets")
    if not isinstance(assets, dict):
        return problems + ["assets must be an object keyed by role"]
    files: set[str] = set()
    for role, (kind, required) in ROLES.items():
        asset = assets.get(role)
        if not isinstance(asset, dict):
            problems.append(f"assets.{role} is missing")
            continue
        where = f"assets.{role}"
        if asset.get("kind") != kind:
            problems.append(f"{where}.kind must be {kind!r}")
        name = str(asset.get("file", ""))
        if not FILE_RE.match(name) or name in (MANIFEST_NAME, SUMS_NAME):
            problems.append(f"{where}.file must be a plain file name")
        elif name in files:
            problems.append(f"{where}.file {name!r} is used twice")
        files.add(name)
        size = asset.get("size")
        if not isinstance(size, int) or isinstance(size, bool) or size <= 0:
            problems.append(f"{where}.size must be a positive integer")
        if not SHA256_RE.match(str(asset.get("sha256", ""))):
            problems.append(f"{where}.sha256 must be 64 lowercase hex characters")
        if asset.get("arch") != "x86_64":
            problems.append(f"{where}.arch must be 'x86_64'")
        for field in required:
            if not isinstance(asset.get(field), str) or not asset[field]:
                problems.append(f"{where}.{field} must be a non-empty string")
    rootfs = assets.get("wsl_rootfs") or {}
    if isinstance(rootfs, dict):
        if rootfs.get("distro_name") not in (None, DISTRO_NAME):
            problems.append(f"assets.wsl_rootfs.distro_name must be {DISTRO_NAME!r}")
        if rootfs.get("compression") not in (None, "gzip"):
            problems.append("assets.wsl_rootfs.compression must be 'gzip'")
    image = assets.get("workspace_image") or {}
    if isinstance(image, dict):
        if image.get("format") not in (None, "unified"):
            problems.append("assets.workspace_image.format must be 'unified'")
        fingerprint = image.get("incus_fingerprint")
        if fingerprint is not None and not SHA256_RE.match(str(fingerprint)):
            problems.append("assets.workspace_image.incus_fingerprint must be 64 lowercase hex characters")
        elif fingerprint is not None and fingerprint != image.get("sha256"):
            problems.append("assets.workspace_image.incus_fingerprint must equal its sha256 (unified image)")
    return problems


def parse_pairs(pairs: list[str], as_int: bool) -> dict:
    out: dict = {}
    for pair in pairs:
        key, sep, value = pair.partition("=")
        if not sep or not re.match(r"^[a-z][a-z0-9_]*$", key):
            raise ManifestError(f"bad key=value pair: {pair!r}")
        if as_int:
            if not value.isdigit():
                raise ManifestError(f"{key} must be an integer, got {value!r}")
            out[key] = int(value)
        else:
            out[key] = value
    return out


def cmd_asset(args: argparse.Namespace) -> int:
    if args.role not in ROLES:
        raise ManifestError(f"unknown role {args.role!r} (known: {', '.join(ROLES)})")
    path = Path(args.file)
    if not path.is_file():
        raise ManifestError(f"no such file: {path}")
    fragment = {"role": args.role, "kind": ROLES[args.role][0], "file": path.name,
                "size": path.stat().st_size, "sha256": sha256_of(path), "arch": "x86_64"}
    extra = parse_pairs(args.set or [], as_int=False)
    extra.update(parse_pairs(args.set_int or [], as_int=True))
    clash = set(extra) & {"role", "kind", "file", "size", "sha256"}
    if clash:
        raise ManifestError(f"--set cannot override {sorted(clash)}")
    fragment.update(extra)
    Path(args.out).write_text(json.dumps(fragment, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(f"{fragment['file']}: {fragment['size']} bytes, sha256 {fragment['sha256']}")
    return 0


def cmd_merge(args: argparse.Namespace) -> int:
    assets: dict = {}
    for path in sorted(Path(args.fragments).glob("*.asset.json")):
        fragment = json.loads(path.read_text(encoding="utf-8"))
        role = fragment.pop("role", None)
        if role not in ROLES:
            raise ManifestError(f"{path}: unknown role {role!r}")
        if role in assets:
            raise ManifestError(f"two fragments for role {role!r}")
        assets[role] = fragment
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "runtime_version": args.version,
        "release_tag": args.tag,
        "commit": args.commit,
        "built_at": args.built_at,
        "debian_snapshot": args.snapshot,
        "assets": assets,
    }
    problems = validate(manifest)
    if problems:
        raise ManifestError("invalid manifest:\n  - " + "\n  - ".join(problems))
    out = Path(args.out)
    out.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    lines = [f"{a['sha256']}  {a['file']}" for a in sorted(assets.values(), key=lambda a: a["file"])]
    lines.append(f"{sha256_of(out)}  {out.name}")
    Path(args.sums).write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {out} and {args.sums} ({len(assets)} assets)")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
    problems = validate(manifest)
    directory = Path(args.dir)
    for role, asset in (manifest.get("assets") or {}).items():
        if not isinstance(asset, dict) or not FILE_RE.match(str(asset.get("file", ""))):
            continue
        path = directory / asset["file"]
        if not path.is_file():
            problems.append(f"{role}: {path} is missing")
            continue
        if path.stat().st_size != asset.get("size"):
            problems.append(f"{role}: {path} is {path.stat().st_size} bytes, manifest says {asset.get('size')}")
        elif sha256_of(path) != asset.get("sha256"):
            problems.append(f"{role}: {path} does not match its sha256")
    if problems:
        print("verification failed:\n  - " + "\n  - ".join(problems), file=sys.stderr)
        return 1
    print(f"ok: {len(manifest['assets'])} assets match {args.manifest}")
    return 0


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        rootfs = root / "GeneralBotsLite-agent-rootfs-1.0.0-x86_64.tar.gz"
        image = root / "GeneralBotsLite-agent-workspace-office-1.0.0-x86_64.tar.xz"
        rootfs.write_bytes(b"rootfs")
        image.write_bytes(b"image")
        fp = sha256_of(image)
        parser = build_parser()
        run = lambda argv: parser.parse_args(argv).func(parser.parse_args(argv))  # noqa: E731
        run(["asset", "--role", "wsl_rootfs", "--file", str(rootfs), "--out", str(root / "a.asset.json"),
             "--set", f"distro_name={DISTRO_NAME}", "--set", "compression=gzip",
             "--set", "debian_suite=trixie", "--set", "incus_version=6.0.4-2"])
        run(["asset", "--role", "workspace_image", "--file", str(image), "--out", str(root / "b.asset.json"),
             "--set", f"incus_fingerprint={fp}", "--set", f"alias={WORKSPACE_ALIAS}", "--set", "profile=office",
             "--set", "provisioned_marker=v7-office", "--set", "format=unified", "--set", "compression=xz"])
        manifest = root / MANIFEST_NAME
        run(["merge", "--fragments", str(root), "--version", "1.0.0", "--tag", "gbgov-agent-runtime-v1.0.0",
             "--commit", "0" * 40, "--built-at", "2026-11-26T00:00:00Z", "--snapshot", "20261120T000000Z",
             "--out", str(manifest), "--sums", str(root / SUMS_NAME)])
        assert run(["verify", "--manifest", str(manifest), "--dir", str(root)]) == 0
        image.write_bytes(b"imagx")  # same size: the sha256 must catch it
        assert run(["verify", "--manifest", str(manifest), "--dir", str(root)]) == 1, "tampering not caught"
        data = json.loads(manifest.read_text(encoding="utf-8"))
        data["assets"]["workspace_image"]["incus_fingerprint"] = "f" * 64
        assert validate(data), "fingerprint mismatch not caught"
        data["assets"]["wsl_rootfs"]["file"] = "../evil"
        assert any("plain file name" in p for p in validate(data)), "path traversal not caught"
        sums = (root / SUMS_NAME).read_text(encoding="utf-8").splitlines()
        assert len(sums) == 3 and sums[-1].endswith(MANIFEST_NAME), sums
    print("self-test ok")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true")
    sub = parser.add_subparsers(dest="command")
    asset = sub.add_parser("asset")
    asset.add_argument("--role", required=True)
    asset.add_argument("--file", required=True)
    asset.add_argument("--out", required=True)
    asset.add_argument("--set", action="append", metavar="KEY=VALUE")
    asset.add_argument("--set-int", action="append", metavar="KEY=INT")
    asset.set_defaults(func=cmd_asset)
    merge = sub.add_parser("merge")
    for name in ("--fragments", "--version", "--tag", "--commit", "--built-at", "--snapshot", "--out", "--sums"):
        merge.add_argument(name, required=True)
    merge.set_defaults(func=cmd_merge)
    verify = sub.add_parser("verify")
    verify.add_argument("--manifest", required=True)
    verify.add_argument("--dir", required=True)
    verify.set_defaults(func=cmd_verify)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if not args.command:
        parser.print_help()
        return 2
    try:
        return args.func(args)
    except ManifestError as error:
        print(f"agent_runtime_manifest: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
