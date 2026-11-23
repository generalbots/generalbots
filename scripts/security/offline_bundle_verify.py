#!/usr/bin/env python3
"""Offline verification and feed parity checks for the #2720 bundle.

Kept separate from the builder so each file stays small. All operations are
local; no function in this module opens a socket.
"""

from __future__ import annotations

import io
import json
import re
import tarfile
import tempfile
from pathlib import Path
from typing import Any

from offline_bundle_lib import (
    BUNDLE_VERSION,
    ID_RE,
    METADATA,
    REV_RE,
    SHA_RE,
    SCHEMA,
    BundleError,
    canonical_sums,
    check_offline_html,
    digest,
    parse_sums,
    read_feed,
    run_template,
    safe_relpath,
)


def load_manifest(bundle_dir: Path) -> dict[str, Any]:
    try:
        manifest = json.loads((bundle_dir / "MANIFEST.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise BundleError(f"cannot read manifest in {bundle_dir}: {error}") from error
    if not isinstance(manifest, dict) or manifest.get("schema") != SCHEMA:
        raise BundleError(f"unsupported manifest schema in {bundle_dir}")
    for field in ("bundle", "bundle_sha256", "tree_digest", "source_commit", "publication_date"):
        if not isinstance(manifest.get(field), str) or not manifest[field]:
            raise BundleError(f"manifest in {bundle_dir} has no {field}")
    if not REV_RE.match(str(manifest["source_commit"])):
        raise BundleError("manifest source_commit is not an immutable commit SHA")
    for field in ("bundle_sha256", "tree_digest"):
        if not SHA_RE.match(str(manifest[field])):
            raise BundleError(f"manifest {field} is malformed")
    ids = manifest.get("advisory_ids")
    if not isinstance(ids, list) or not all(isinstance(item, str) and ID_RE.match(item) for item in ids):
        raise BundleError("manifest advisory_ids is not a list of advisory IDs")
    return manifest


def extract(bundle_dir: Path, manifest: dict[str, Any], dest: Path) -> list[tuple[str, bytes]]:
    name = str(manifest["bundle"])
    prefix_marker = f"generalbots-advisories-{BUNDLE_VERSION}-"
    if Path(name).name != name or not name.startswith(prefix_marker) or not name.endswith(".tar.gz"):
        raise BundleError(f"unsafe or non-conforming bundle name: {name!r}")
    prefix = name[len(prefix_marker):-len(".tar.gz")]
    if not re.match(r"^[0-9a-f]{16}$", prefix) or prefix != str(manifest["tree_digest"])[:16]:
        raise BundleError("bundle filename does not match the tree digest prefix")
    try:
        raw = (bundle_dir / name).read_bytes()
    except OSError as error:
        raise BundleError(f"cannot read bundle {name}: {error}") from error
    if digest(raw) != manifest["bundle_sha256"]:
        raise BundleError("bundle digest mismatch")
    dest.mkdir(parents=True, exist_ok=True)
    files: list[tuple[str, bytes]] = []
    try:
        with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as archive:
            for member in archive:
                if member.type != tarfile.REGTYPE:
                    raise BundleError(f"non-regular member in bundle: {member.name}")
                rel = safe_relpath(member.name)
                body = archive.extractfile(member)
                if body is None:
                    raise BundleError(f"cannot extract {rel}")
                data = body.read()
                target = dest / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
                files.append((rel, data))
    except tarfile.TarError as error:
        raise BundleError(f"invalid bundle tar: {error}") from error
    return sorted(files, key=lambda item: item[0])


def advisory_ids_from_files(files: list[tuple[str, bytes]]) -> list[str]:
    ids: set[str] = set()
    for rel, data in files:
        if not rel.lower().endswith(".json") or rel in METADATA:
            continue
        try:
            parsed = json.loads(data)
        except ValueError as error:
            raise BundleError(f"{rel}: invalid JSON in bundle: {error}") from error
        identifier = ""
        if isinstance(parsed, dict):
            tracking = parsed.get("document", {}).get("tracking", {})
            if isinstance(tracking, dict):
                identifier = str(tracking.get("id", ""))
            if not identifier:
                identifier = str(parsed.get("id", ""))
        if ID_RE.match(identifier):
            ids.add(identifier)
    return sorted(ids)


def check_index_links(files: list[tuple[str, bytes]]) -> None:
    """Prove every local link in the offline index resolves inside the bundle."""
    content = dict(files)
    index = content.get("index.html")
    if index is None:
        raise BundleError("bundle has no offline index")
    try:
        text = index.decode("utf-8")
    except UnicodeDecodeError as error:
        raise BundleError("offline index is not UTF-8") from error
    for href in re.findall(r'(?i)href\s*=\s*"([^"]+)"', text):
        if href.startswith(("#", "//", "http:", "https:", "mailto:")):
            continue
        if safe_relpath(href) not in content:
            raise BundleError(f"offline index links to a missing file: {href}")


def _check_published_metadata(bundle_dir: Path, manifest: dict[str, Any]) -> bytes:
    try:
        sums = (bundle_dir / "SHA256SUMS").read_bytes()
        manifest_raw = (bundle_dir / "MANIFEST.json").read_bytes()
        manifest_sum = (bundle_dir / "MANIFEST.sha256").read_text(encoding="utf-8")
        latest = (bundle_dir / "latest").read_text(encoding="utf-8")
        latest_raw = (bundle_dir / "latest.json").read_text(encoding="utf-8")
    except OSError as error:
        raise BundleError(f"cannot read published metadata: {error}") from error
    if canonical_sums(parse_sums(sums)) != sums:
        raise BundleError("SHA256SUMS is not canonical (order/format changed)")
    if digest(sums) != manifest["tree_digest"]:
        raise BundleError("SHA256SUMS does not match the signed tree digest")
    if manifest_sum.strip() != f"{digest(manifest_raw)}  MANIFEST.json":
        raise BundleError("MANIFEST.sha256 does not match MANIFEST.json")
    if latest.strip() != manifest["bundle"]:
        raise BundleError("latest pointer does not name the published bundle")
    try:
        pointer = json.loads(latest_raw)
    except ValueError as error:
        raise BundleError(f"latest.json is invalid: {error}") from error
    for field in ("bundle", "tree_digest", "source_commit"):
        if pointer.get(field) != manifest[field]:
            raise BundleError(f"latest.json {field} disagrees with the manifest")
    return sums


def verify(bundle_dir: Path, *, expected_revision: str | None,
           verify_command: str | None, require_signature: bool) -> dict[str, Any]:
    """Verify a published bundle without using the network. Fails closed."""
    manifest = load_manifest(bundle_dir)
    revision = str(manifest["source_commit"])
    if expected_revision is not None and revision != expected_revision:
        raise BundleError(f"source revision {revision} != expected {expected_revision}")
    sums = _check_published_metadata(bundle_dir, manifest)

    with tempfile.TemporaryDirectory(prefix="offline-bundle-verify-") as temp:
        files = extract(bundle_dir, manifest, Path(temp))
        content = [(rel, data) for rel, data in files if rel not in METADATA]
        if canonical_sums({rel: digest(data) for rel, data in content}) != sums:
            raise BundleError("extracted content does not match SHA256SUMS")
        if len(content) != manifest.get("file_count") or \
                sum(len(data) for _, data in content) != manifest.get("total_bytes"):
            raise BundleError("extracted content counts do not match the manifest")
        if dict(files).get("SHA256SUMS") != sums:
            raise BundleError("SHA256SUMS inside the bundle differs from the signed copy")
        check_offline_html(content)
        check_index_links(files)
        if advisory_ids_from_files(content) != sorted(str(item) for item in manifest["advisory_ids"]):
            raise BundleError("bundle advisories differ from the manifest advisory IDs")

    signature = str(manifest.get("signature_file", ""))
    if signature:
        sig_path = bundle_dir / signature
        if not sig_path.is_file():
            raise BundleError(f"signature file is missing: {signature}")
        if verify_command is not None:
            run_template(verify_command, sig=str(sig_path), file=str(bundle_dir / "SHA256SUMS"))
        elif require_signature:
            raise BundleError("signature present but --verify-command was not supplied")
    elif require_signature:
        raise BundleError("bundle is unsigned and --require-signature was set")
    return manifest


def check_parity(bundle_dir: Path, feed_dir: Path, expected_revision: str | None) -> dict[str, Any]:
    """Prove the bundle carries exactly the advisories in the feed snapshot."""
    manifest = load_manifest(bundle_dir)
    revision = str(manifest["source_commit"])
    if expected_revision is not None and revision != expected_revision:
        raise BundleError(f"source revision {revision} != expected {expected_revision}")
    feed_files, feed_ids = read_feed(feed_dir)
    if feed_ids != sorted(str(item) for item in manifest["advisory_ids"]):
        raise BundleError("advisory IDs in the feed snapshot differ from the bundle manifest")
    try:
        sums = parse_sums((bundle_dir / "SHA256SUMS").read_bytes())
    except OSError as error:
        raise BundleError(f"cannot read bundle SHA256SUMS: {error}") from error
    for rel, data in feed_files:
        if sums.get(rel) != digest(data):
            raise BundleError(f"feed file {rel} differs from the bundled revision")
    return {"revision": revision, "advisory_ids": feed_ids, "feed_files": len(feed_files)}
