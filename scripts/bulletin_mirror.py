#!/usr/bin/env python3
"""Build and verify the immutable security-bulletin mirror (#2726).

The advisory site is a static tree with no server-side code and no admin
panel (see the security-bulletin epic #2712). This tool turns that tree into a
content-addressed, immutable bundle that can be published to independent
surfaces and verified byte-for-byte by any consumer:

  * SHA256SUMS    one line per tree file, sorted, `sha256sum` format;
  * manifest.json tree digest = SHA-256 of the canonical SHA256SUMS bytes,
                  plus the source revision, counts and the bundle digest;
  * <bundle>.tar.gz deterministic tar (fixed mode/uid/mtime, sorted entries,
                  gzip with mtime=0) whose name carries the tree digest.

A consumer recomputes the tree digest from the extracted files. Any added,
removed or modified file changes it, so a single published digest seals the
whole tree. The detached signature over SHA256SUMS is produced in CI by cosign
(`cosign sign-blob`), not by this script.

Commands
--------
  build        --source DIR --revision <40-hex commit> --out DIR
  verify       --bundle DIR
  check-mirrors (--url URL | --dir DIR) ... [--expected-revision REV]
  --self-test

`check-mirrors` accepts local directories for offline testing and HTTPS URLs
in CI. It proves that two surfaces serve the *same* revision and tree digest;
it never follows a redirect that downgrades to plaintext.

Usage:
    python3 scripts/bulletin_mirror.py --self-test
    python3 scripts/bulletin_mirror.py build --source site --revision <sha> --out /tmp/mirror
    python3 scripts/bulletin_mirror.py verify --bundle /tmp/mirror
    python3 scripts/bulletin_mirror.py check-mirrors --url https://a/ --url https://b/

Exit codes: 0 clean, 1 integrity/policy failure, 2 bad arguments.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import os
import re
import sys
import tarfile
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

SCHEMA = 1
REVISION_RE = re.compile(r"^[0-9a-f]{40}$")
SHA_RE = re.compile(r"^[0-9a-f]{64}$")
MAX_METADATA = 4 * 1024 * 1024
MAX_BUNDLE = 512 * 1024 * 1024
MAX_FILE = 64 * 1024 * 1024
TIMEOUT = 20
USER_AGENT = "generalbots-bulletin-verifier/1.0"


class MirrorError(Exception):
    """Any integrity or policy failure; always fails closed."""


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_relpath(name: str) -> str:
    """Normalise a tree path or reject it. No symlinks, traversal or NULs."""
    if not name or "\x00" in name or "\\" in name:
        raise MirrorError(f"unsafe path: {name!r}")
    path = Path(name)
    if path.is_absolute() or any(part in ("", ".", "..") for part in name.split("/")):
        raise MirrorError(f"unsafe path: {name!r}")
    if name.endswith("/"):
        raise MirrorError(f"not a regular file: {name!r}")
    return name


def read_tree(root: Path) -> list[tuple[str, bytes, int]]:
    """Return (relative path, bytes, mode) for every regular file, sorted."""
    files: list[tuple[str, bytes, int]] = []
    for path in sorted(root.rglob("*"), key=lambda p: p.as_posix()):
        if path.is_symlink():
            raise MirrorError(f"symlink in bulletin tree: {path.name}")
        if not path.is_file():
            continue
        rel = safe_relpath(path.relative_to(root).as_posix())
        data = path.read_bytes()
        if len(data) > MAX_FILE:
            raise MirrorError(f"file too large: {rel}")
        stat = path.stat()
        mode = 0o755 if stat.st_mode & 0o111 else 0o644
        files.append((rel, data, mode))
    if not files:
        raise MirrorError("bulletin tree is empty")
    return files


def canonical_sums(parsed: dict[str, str]) -> bytes:
    """Serialise a {path: digest} map as canonical SHA256SUMS bytes."""
    return "".join(f"{parsed[rel]}  {rel}\n" for rel in sorted(parsed)).encode("utf-8")


def sums_bytes(files: list[tuple[str, bytes, int]]) -> bytes:
    """Canonical SHA256SUMS: sorted paths, two spaces, LF, UTF-8."""
    return canonical_sums({rel: digest(data) for rel, data, _ in files})


def parse_sums(raw: bytes) -> dict[str, str]:
    """Parse a SHA256SUMS blob into {path: digest}, rejecting malformed lines."""
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise MirrorError("SHA256SUMS is not UTF-8") from error
    result: dict[str, str] = {}
    for line in text.splitlines():
        if not line:
            continue
        pair = line.split("  ", 1)
        if len(pair) != 2 or not SHA_RE.match(pair[0]):
            raise MirrorError(f"malformed SHA256SUMS line: {line!r}")
        rel = safe_relpath(pair[1])
        if rel in result:
            raise MirrorError(f"duplicate SHA256SUMS entry: {rel}")
        result[rel] = pair[0]
    if not result:
        raise MirrorError("SHA256SUMS is empty")
    return result


def make_tar(files: list[tuple[str, bytes, int]]) -> bytes:
    """Reproducible tar.gz: sorted entries, fixed metadata, no timestamps."""
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.USTAR_FORMAT) as archive:
        for rel, data, mode in files:
            info = tarfile.TarInfo(name=rel)
            info.size = len(data)
            info.mode = mode
            info.mtime = 0
            info.uid = 0
            info.gid = 0
            info.uname = ""
            info.gname = ""
            info.type = tarfile.REGTYPE
            archive.addfile(info, io.BytesIO(data))
    gz = io.BytesIO()
    with gzip.GzipFile(filename="", mode="wb", fileobj=gz, mtime=0, compresslevel=9) as stream:
        stream.write(raw.getvalue())
    return gz.getvalue()


def write_bundle(source: Path, revision: str, out: Path) -> dict[str, object]:
    if not REVISION_RE.match(revision):
        raise MirrorError("--revision must be a 40-character lowercase commit SHA")
    if not source.is_dir():
        raise MirrorError(f"source directory not found: {source}")
    files = read_tree(source)
    sums = sums_bytes(files)
    tree = digest(sums)
    name = f"bulletin-{tree[:16]}.tar.gz"
    bundle = make_tar(files)
    out.mkdir(parents=True, exist_ok=True)
    (out / "SHA256SUMS").write_bytes(sums)
    (out / name).write_bytes(bundle)
    manifest = {
        "schema": SCHEMA,
        "revision": revision,
        "tree_digest": tree,
        "file_count": len(files),
        "total_bytes": sum(len(data) for _, data, _ in files),
        "bundle": name,
        "bundle_sha256": digest(bundle),
    }
    (out / "manifest.json").write_bytes(
        (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
    )
    return manifest


def load_manifest(folder: Path) -> dict[str, object]:
    try:
        manifest = json.loads((folder / "manifest.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise MirrorError(f"cannot read manifest.json in {folder}: {error}") from error
    if not isinstance(manifest, dict) or manifest.get("schema") != SCHEMA:
        raise MirrorError(f"unsupported manifest schema in {folder}")
    for field in ("revision", "tree_digest", "bundle", "bundle_sha256"):
        if not isinstance(manifest.get(field), str) or not manifest[field]:
            raise MirrorError(f"manifest in {folder} has no {field}")
    if not REVISION_RE.match(str(manifest["revision"])):
        raise MirrorError(f"manifest in {folder} has a non-immutable revision")
    if not SHA_RE.match(str(manifest["tree_digest"])) or not SHA_RE.match(str(manifest["bundle_sha256"])):
        raise MirrorError(f"manifest in {folder} has a malformed digest")
    return manifest


def extract_verified(folder: Path, manifest: dict[str, object], dest: Path) -> list[tuple[str, bytes, int]]:
    """Unpack the bundle with traversal guards, then re-derive SHA256SUMS."""
    bundle_name = str(manifest["bundle"])
    if Path(bundle_name).name != bundle_name or not bundle_name.endswith(".tar.gz"):
        raise MirrorError(f"unsafe bundle name: {bundle_name!r}")
    bundle_path = folder / bundle_name
    try:
        bundle = bundle_path.read_bytes()
    except OSError as error:
        raise MirrorError(f"cannot read bundle {bundle_name}: {error}") from error
    if digest(bundle) != manifest["bundle_sha256"]:
        raise MirrorError(f"bundle digest mismatch: {bundle_name}")
    dest.mkdir(parents=True, exist_ok=True)
    files: list[tuple[str, bytes, int]] = []
    try:
        with tarfile.open(fileobj=io.BytesIO(bundle), mode="r:gz") as archive:
            for member in archive:
                if member.type != tarfile.REGTYPE:
                    raise MirrorError(f"non-regular member in bundle: {member.name}")
                rel = safe_relpath(member.name)
                body = archive.extractfile(member)
                if body is None:
                    raise MirrorError(f"cannot extract {rel}")
                data = body.read()
                (dest / rel).parent.mkdir(parents=True, exist_ok=True)
                (dest / rel).write_bytes(data)
                files.append((rel, data, 0o755 if member.mode & 0o111 else 0o644))
    except tarfile.TarError as error:
        raise MirrorError(f"invalid bundle tar: {error}") from error
    return sorted(files, key=lambda item: item[0])


def verify_bundle(folder: Path) -> dict[str, object]:
    """Verify a published bundle directory. Raises MirrorError on tampering."""
    manifest = load_manifest(folder)
    try:
        published = (folder / "SHA256SUMS").read_bytes()
    except OSError as error:
        raise MirrorError(f"cannot read SHA256SUMS in {folder}: {error}") from error
    if published != canonical_sums(parse_sums(published)):
        raise MirrorError("SHA256SUMS is not canonical (order/format changed)")
    if digest(published) != manifest["tree_digest"]:
        raise MirrorError("SHA256SUMS does not match the signed tree digest")
    with tempfile.TemporaryDirectory(prefix="bulletin-verify-") as temp:
        files = extract_verified(folder, manifest, Path(temp))
    if sums_bytes(files) != published:
        raise MirrorError("extracted tree does not match SHA256SUMS")
    if len(files) != manifest.get("file_count") or sum(len(d) for _, d, _ in files) != manifest.get("total_bytes"):
        raise MirrorError("extracted tree counts do not match manifest")
    return manifest


class NoDowngrade(urllib.request.HTTPRedirectHandler):
    """Refuse redirects from HTTPS to plaintext HTTP."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        if not newurl.lower().startswith("https://"):
            raise MirrorError(f"refusing redirect to non-HTTPS URL: {newurl}")
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def fetch(url: str, limit: int, timeout: int) -> bytes:
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https" or not parsed.netloc:
        raise MirrorError(f"mirror URL must be HTTPS: {url}")
    opener = urllib.request.build_opener(NoDowngrade())
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with opener.open(request, timeout=timeout) as response:
            body = response.read(limit + 1)
    except (urllib.error.URLError, OSError) as error:
        raise MirrorError(f"cannot fetch {url}: {error}") from error
    if len(body) > limit:
        raise MirrorError(f"response from {url} exceeds {limit} bytes")
    return body


def mirror_state(source: str, timeout: int) -> tuple[str, bytes, dict[str, object]]:
    """Return (label, SHA256SUMS bytes, manifest) for one published surface."""
    if source.startswith("https://"):
        base = source.rstrip("/") + "/"
        sums = fetch(base + "SHA256SUMS", MAX_METADATA, timeout)
        manifest_raw = fetch(base + "manifest.json", MAX_METADATA, timeout)
    else:
        folder = Path(source)
        try:
            sums = (folder / "SHA256SUMS").read_bytes()
            manifest_raw = (folder / "manifest.json").read_bytes()
        except OSError as error:
            raise MirrorError(f"cannot read mirror directory {folder}: {error}") from error
    try:
        manifest = json.loads(manifest_raw)
    except ValueError as error:
        raise MirrorError(f"invalid manifest.json at {source}: {error}") from error
    if not isinstance(manifest, dict) or manifest.get("schema") != SCHEMA:
        raise MirrorError(f"unsupported manifest schema at {source}")
    parsed = parse_sums(sums)
    if canonical_sums(parsed) != sums or digest(sums) != manifest.get("tree_digest"):
        raise MirrorError(f"SHA256SUMS does not match tree digest at {source}")
    return source, sums, manifest


def check_mirrors(sources: list[str], timeout: int, expected: str | None) -> dict[str, object]:
    if len(sources) < 2:
        raise MirrorError("at least two independent surfaces are required")
    if len(set(sources)) != len(sources):
        raise MirrorError("mirror surfaces must be distinct URLs/paths")
    states = [mirror_state(src, timeout) for src in sources]
    reference = states[0]
    for label, sums, manifest in states[1:]:
        if manifest.get("revision") != reference[2].get("revision"):
            raise MirrorError(f"revision drift: {reference[0]}={reference[2].get('revision')} vs {label}={manifest.get('revision')}")
        if manifest.get("tree_digest") != reference[2].get("tree_digest") or sums != reference[1]:
            raise MirrorError(f"content drift between {reference[0]} and {label}")
    revision = str(reference[2].get("revision"))
    if expected and revision != expected:
        raise MirrorError(f"published revision {revision} != expected {expected}")
    if not REVISION_RE.match(revision):
        raise MirrorError(f"published revision is not an immutable commit SHA: {revision}")
    return {"revision": revision, "tree_digest": reference[2].get("tree_digest"), "surfaces": len(states)}


def self_test() -> int:
    """Offline proof: determinism, tamper detection and two-surface agreement."""
    with tempfile.TemporaryDirectory(prefix="bulletin-selftest-") as temp:
        root = Path(temp)
        revision = "a" * 40
        sources = []
        for index in (1, 2):
            src = root / f"site{index}"
            (src / "security").mkdir(parents=True)
            (src / "security" / "index.html").write_text("<h1>Bulletin</h1>\n", encoding="utf-8")
            (src / ".well-known").mkdir()
            (src / ".well-known" / "security.txt").write_text("Contact: mailto:security@example.org\n", encoding="utf-8")
            os.utime(src / "security" / "index.html", (index, index))
            sources.append(src)
        out1, out2 = root / "mirror1", root / "mirror2"
        first = write_bundle(sources[0], revision, out1)
        second = write_bundle(sources[1], revision, out2)
        if first != second or verify_bundle(out1) != verify_bundle(out2):
            raise MirrorError("self-test: builds are not deterministic")
        agreed = check_mirrors([str(out1), str(out2)], TIMEOUT, revision)
        if agreed["tree_digest"] != first["tree_digest"]:
            raise MirrorError("self-test: mirror agreement reported the wrong digest")
        # Tamper: flip one byte of published SHA256SUMS.
        tampered = root / "tampered"
        tampered.mkdir()
        for item in out1.iterdir():
            tampered.joinpath(item.name).write_bytes(item.read_bytes())
        sums_path = tampered / "SHA256SUMS"
        original = sums_path.read_text(encoding="utf-8")
        sums_path.write_text(original.replace("a", "b", 1), encoding="utf-8")
        try:
            verify_bundle(tampered)
        except MirrorError:
            pass
        else:
            raise MirrorError("self-test: tampered SHA256SUMS was accepted")
        # Tamper: drift one surface to a different revision.
        (out2 / "manifest.json").write_text(
            json.dumps({**second, "revision": "b" * 40}, sort_keys=True), encoding="utf-8"
        )
        try:
            check_mirrors([str(out1), str(out2)], TIMEOUT, None)
        except MirrorError:
            pass
        else:
            raise MirrorError("self-test: revision drift was accepted")
    print("bulletin_mirror self-test: ok (deterministic, tamper-evident, drift detected)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true", help="run offline integrity self-tests")
    commands = parser.add_subparsers(dest="command")

    build = commands.add_parser("build", help="build an immutable bundle from a static tree")
    build.add_argument("--source", required=True, type=Path)
    build.add_argument("--revision", required=True)
    build.add_argument("--out", required=True, type=Path)

    verify = commands.add_parser("verify", help="verify a published bundle directory")
    verify.add_argument("--bundle", required=True, type=Path)

    check = commands.add_parser("check-mirrors", help="prove two surfaces serve identical content")
    check.add_argument("--url", action="append", default=[])
    check.add_argument("--dir", action="append", default=[])
    check.add_argument("--expected-revision")
    check.add_argument("--timeout", type=int, default=TIMEOUT)

    args = parser.parse_args(argv)
    try:
        if args.self_test:
            return self_test()
        if args.command == "build":
            result = write_bundle(args.source, args.revision, args.out)
            print(json.dumps(result, sort_keys=True))
            return 0
        if args.command == "verify":
            result = verify_bundle(args.bundle)
            print(f"verified tree {result['tree_digest']} at revision {result['revision']}")
            return 0
        if args.command == "check-mirrors":
            sources = [*args.url, *args.dir]
            result = check_mirrors(sources, args.timeout, args.expected_revision)
            print(json.dumps(result, sort_keys=True))
            return 0
        parser.print_help()
        return 2
    except MirrorError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
