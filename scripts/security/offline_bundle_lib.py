#!/usr/bin/env python3
"""Core library for the offline advisory bundle builder (#2720).

Pure stdlib. Every operation is local and fails closed; no code path in this
module opens a socket, follows a URL or reads outside the supplied directories.
The CLI in build_offline_bundle.py owns argument parsing and reporting.
"""

from __future__ import annotations

import hashlib
import io
import json
import re
import shlex
import subprocess
import tarfile
from pathlib import Path
from typing import Any

SCHEMA = 1
BUNDLE_VERSION = "v1"
SHA_RE = re.compile(r"^[0-9a-f]{64}$")
REV_RE = re.compile(r"^[0-9a-f]{40}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
ID_RE = re.compile(r"^GBSA-\d{4}-\d{4}$")
MAX_FILE = 64 * 1024 * 1024
METADATA = {"SHA256SUMS", "MANIFEST.json"}
RESERVED = {*METADATA, "index.html", "README.md", "VERIFYING.md", "keys/README.md"}


class BundleError(Exception):
    """Any integrity, offline-safety or policy failure; always fails closed."""


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_relpath(name: str) -> str:
    """Normalise a bundle path or reject it. No traversal, NULs or absolute paths."""
    if not name or "\x00" in name or "\\" in name:
        raise BundleError(f"unsafe path: {name!r}")
    if Path(name).is_absolute() or any(part in ("", ".", "..") for part in name.split("/")):
        raise BundleError(f"unsafe path: {name!r}")
    if name.endswith("/"):
        raise BundleError(f"not a regular file: {name!r}")
    return name


def canonical_sums(parsed: dict[str, str]) -> bytes:
    """Serialise a {path: digest} map as canonical SHA256SUMS bytes."""
    return "".join(f"{parsed[rel]}  {rel}\n" for rel in sorted(parsed)).encode("utf-8")


def parse_sums(raw: bytes) -> dict[str, str]:
    """Parse a SHA256SUMS blob into {path: digest}, rejecting malformed lines."""
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise BundleError("SHA256SUMS is not UTF-8") from error
    result: dict[str, str] = {}
    for line in text.splitlines():
        if not line:
            continue
        pair = line.split("  ", 1)
        if len(pair) != 2 or not SHA_RE.match(pair[0]):
            raise BundleError(f"malformed SHA256SUMS line: {line!r}")
        rel = safe_relpath(pair[1])
        if rel in result:
            raise BundleError(f"duplicate SHA256SUMS entry: {rel}")
        result[rel] = pair[0]
    if not result:
        raise BundleError("SHA256SUMS is empty")
    return result


def make_tar(files: list[tuple[str, bytes]]) -> bytes:
    """Reproducible tar.gz: sorted entries, fixed metadata, no timestamps."""
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.USTAR_FORMAT) as archive:
        for rel, data in sorted(files, key=lambda item: item[0]):
            info = tarfile.TarInfo(name=safe_relpath(rel))
            info.size = len(data)
            info.mode = 0o644
            info.mtime = 0
            info.uid = 0
            info.gid = 0
            info.uname = ""
            info.gname = ""
            info.type = tarfile.REGTYPE
            archive.addfile(info, io.BytesIO(data))
    gz = io.BytesIO()
    with gzip_stream(gz) as stream:
        stream.write(raw.getvalue())
    return gz.getvalue()


def gzip_stream(target: io.BytesIO) -> Any:
    import gzip

    return gzip.GzipFile(filename="", mode="wb", fileobj=target, mtime=0, compresslevel=9)


def check_offline_html(files: list[tuple[str, bytes]]) -> None:
    """Reject HTML that would load anything from the network at read time."""
    asset_patterns = (
        re.compile(r"(?is)<link\b[^>]*\bhref\s*=\s*[\"']?(?:https?:)?//"),
        re.compile(r"(?is)<script\b[^>]*\bsrc\s*="),
        re.compile(r"(?is)<img\b[^>]*\bsrc\s*=\s*[\"']?(?:https?:)?//"),
        re.compile(r"(?is)@import\b"),
        re.compile(r"(?is)@font-face\b"),
        re.compile(r"(?is)url\(\s*[\"']?(?:https?:)?//"),
        re.compile(r"(?i)fonts\.(?:googleapis|gstatic)\.com"),
        re.compile(r"(?i)(?:cdn|unpkg|jsdelivr)\."),
    )
    for rel, data in files:
        if not rel.lower().endswith((".html", ".htm")):
            continue
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError as error:
            raise BundleError(f"{rel}: HTML is not UTF-8") from error
        for pattern in asset_patterns:
            if pattern.search(text):
                raise BundleError(f"{rel}: offline HTML contains a remote asset: {pattern.pattern}")


def read_feed(feed_dir: Path) -> tuple[list[tuple[str, bytes]], list[str]]:
    """Read the online-feed snapshot and derive its advisory IDs, offline."""
    if not feed_dir.is_dir():
        raise BundleError(f"feed directory not found: {feed_dir}")
    files: list[tuple[str, bytes]] = []
    ids: set[str] = set()
    for path in sorted(feed_dir.rglob("*"), key=lambda item: item.as_posix()):
        if path.is_symlink():
            raise BundleError(f"symlink in feed tree: {path.name}")
        if not path.is_file():
            continue
        rel = safe_relpath(path.relative_to(feed_dir).as_posix())
        data = path.read_bytes()
        if len(data) > MAX_FILE:
            raise BundleError(f"feed file too large: {rel}")
        files.append((rel, data))
        if rel.lower().endswith(".json"):
            try:
                parsed = json.loads(data)
            except ValueError as error:
                raise BundleError(f"{rel}: invalid JSON in feed: {error}") from error
            identifier = ""
            if isinstance(parsed, dict):
                tracking = parsed.get("document", {}).get("tracking", {})
                if isinstance(tracking, dict):
                    identifier = str(tracking.get("id", ""))
                if not identifier:
                    identifier = str(parsed.get("id", ""))
            if ID_RE.match(identifier):
                ids.add(identifier)
    if not files:
        raise BundleError(f"feed directory is empty: {feed_dir}")
    if not ids:
        raise BundleError("no CSAF/OSV advisory IDs found in the feed snapshot")
    return files, sorted(ids)


def advisory_paths(files: list[tuple[str, bytes]]) -> dict[str, dict[str, str]]:
    """Map an advisory ID to the real CSAF/OSV paths in the feed snapshot."""
    found: dict[str, dict[str, str]] = {}
    for rel, data in files:
        if rel in METADATA or not rel.lower().endswith(".json"):
            continue
        kind = "csaf" if rel.startswith("csaf/") else "osv" if rel.startswith("osv/") else ""
        if not kind:
            continue
        try:
            parsed = json.loads(data)
        except ValueError as error:
            raise BundleError(f"{rel}: invalid JSON in feed: {error}") from error
        identifier = ""
        if isinstance(parsed, dict):
            tracking = parsed.get("document", {}).get("tracking", {})
            if isinstance(tracking, dict):
                identifier = str(tracking.get("id", ""))
            if not identifier:
                identifier = str(parsed.get("id", ""))
        if ID_RE.match(identifier):
            found.setdefault(identifier, {})[kind] = rel
    return found


def render_index(template: Path, advisory_ids: list[str], published: str,
                 revision: str, paths: dict[str, dict[str, str]]) -> bytes:
    """Render the static offline index; only relative, local links are emitted."""
    try:
        text = template.read_text(encoding="utf-8")
    except OSError as error:
        raise BundleError(f"cannot read index template {template}: {error}") from error
    rows = "\n".join(
        f'        <li><a href="{paths.get(aid, {}).get("csaf", "csaf/" + aid + ".json")}">{aid}</a> '
        f'(<a href="{paths.get(aid, {}).get("osv", "osv/" + aid + ".json")}">OSV</a>)</li>'
        for aid in advisory_ids
    )
    replacements = {
        "{{PUBLICATION_DATE}}": published,
        "{{SOURCE_COMMIT}}": revision,
        "{{ADVISORY_COUNT}}": str(len(advisory_ids)),
        "{{ADVISORY_ROWS}}": rows,
    }
    for key, value in replacements.items():
        text = text.replace(key, value)
    if "{{" in text:
        raise BundleError("index template has unrendered placeholders")
    return text.encode("utf-8")


def read_static(static_dir: Path) -> dict[str, bytes]:
    """Read the fixed documentation shipped inside every bundle."""
    if not static_dir.is_dir():
        raise BundleError(f"static asset directory not found: {static_dir}")
    result: dict[str, bytes] = {}
    for name in ("VERIFYING.md", "README.md", "keys/README.md"):
        path = static_dir / name
        if not path.is_file():
            raise BundleError(f"missing static bundle asset: {path}")
        result[name] = path.read_bytes()
    return result


def run_template(template: str, **tokens: str) -> None:
    """Run an external signer/verifier with placeholders; never uses a shell."""
    try:
        argv = [part.format(**tokens) for part in shlex.split(template)]
    except (KeyError, ValueError) as error:
        raise BundleError(f"invalid command template: {error}") from error
    if not argv:
        raise BundleError("empty command template")
    try:
        completed = subprocess.run(argv, check=False, capture_output=True, text=True, timeout=120)
    except (OSError, subprocess.SubprocessError) as error:
        raise BundleError(f"command failed to run: {error}") from error
    if completed.returncode != 0:
        raise BundleError(f"command exited {completed.returncode}: {completed.stderr.strip()}")


def build(*, feed_dir: Path, static_dir: Path, template: Path, out_dir: Path,
          source_commit: str, publication_date: str, sign_command: str | None,
          public_key: Path | None, key_fingerprint: str | None) -> dict[str, Any]:
    """Build a deterministic, immutable, optionally signed offline bundle."""
    if not REV_RE.match(source_commit):
        raise BundleError("--source-commit must be a 40-character lowercase commit SHA")
    if not DATE_RE.match(publication_date):
        raise BundleError("--publication-date must be YYYY-MM-DD")

    feed_files, advisory_ids = read_feed(feed_dir)
    for rel, _ in feed_files:
        if rel in RESERVED:
            raise BundleError(f"feed file collides with a generated bundle file: {rel}")
    content: dict[str, bytes] = dict(feed_files)
    content.update(read_static(static_dir))
    content["index.html"] = render_index(
        template, advisory_ids, publication_date, source_commit, advisory_paths(feed_files)
    )
    if public_key is not None:
        if not public_key.is_file():
            raise BundleError(f"public key not found: {public_key}")
        content["keys/release.pub.asc"] = public_key.read_bytes()
    for rel, data in content.items():
        if len(data) > MAX_FILE:
            raise BundleError(f"bundle file too large: {rel}")
    check_offline_html(list(content.items()))

    sums = canonical_sums({rel: digest(data) for rel, data in content.items()})
    tree = digest(sums)
    prefix = tree[:16]
    bundle_name = f"generalbots-advisories-{BUNDLE_VERSION}-{prefix}.tar.gz"
    bundle = make_tar([*content.items(), ("SHA256SUMS", sums)])
    bundle_digest = digest(bundle)

    manifest: dict[str, Any] = {
        "schema": SCHEMA,
        "bundle_version": BUNDLE_VERSION,
        "publication_date": publication_date,
        "source_commit": source_commit,
        "advisory_ids": advisory_ids,
        "tree_digest": tree,
        "file_count": len(content),
        "total_bytes": sum(len(data) for data in content.values()),
        "bundle": bundle_name,
        "bundle_sha256": bundle_digest,
        "signature_file": "",
        "key_fingerprint": key_fingerprint or "",
    }
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / bundle_name).write_bytes(bundle)
    (out_dir / "SHA256SUMS").write_bytes(sums)
    write_manifest(out_dir, manifest)

    if sign_command is not None:
        sig_path = out_dir / "SHA256SUMS.sig"
        run_template(sign_command, file=str(out_dir / "SHA256SUMS"), sig=str(sig_path))
        if not sig_path.is_file():
            raise BundleError("signing command did not create SHA256SUMS.sig")
        manifest["signature_file"] = sig_path.name
        write_manifest(out_dir, manifest)

    (out_dir / "latest").write_text(bundle_name + "\n", encoding="utf-8")
    (out_dir / "latest.json").write_bytes(
        (json.dumps({
            "schema": SCHEMA,
            "bundle": bundle_name,
            "tree_digest": tree,
            "source_commit": source_commit,
            "publication_date": publication_date,
            "advisory_count": len(advisory_ids),
        }, indent=2, sort_keys=True) + "\n").encode("utf-8")
    )
    return manifest


def write_manifest(out_dir: Path, manifest: dict[str, Any]) -> None:
    raw = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
    (out_dir / "MANIFEST.json").write_bytes(raw)
    (out_dir / "MANIFEST.sha256").write_text(digest(raw) + "  MANIFEST.json\n", encoding="utf-8")

