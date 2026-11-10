#!/usr/bin/env python3
"""Compute the next product version from Conventional Commits (#2118).

``scripts/release_version.py`` answers *"what is the version"* and *"write this
version everywhere"*.  This script answers the question before that one: *"what
should the next version be?"*  The bump source is the commit history, so nobody
types a version by hand:

===============  =========================================================
commit subject   bump
===============  =========================================================
``feat!: ...``   major -- also ``feat(scope)!:`` and a ``BREAKING CHANGE:``
``BREAKING-CHANGE:`` footer anywhere in the commit body
``feat: ...``    minor -- also ``feat(scope):``
anything else    patch -- ``fix:``, ``perf:``, ``chore:``, ``docs:``, merges
===============  =========================================================

The highest bump found wins.  Only commits after the newest product tag are
considered (``git log <tag>..HEAD --pretty=%s``), so a merge that carries a
single ``feat`` raises the minor once, not once per merge.

Tag handling
------------
* Product tags are ``vMAJOR.MINOR.PATCH``, optionally with a pre-release
  suffix (``v7.1.0-rc.1``).  Tags that are not a version (``verify-splits``)
  are ignored.
* A pre-release tag names the release being prepared.  With no commits after
  it, the next version is its base (``v7.1.0-rc.1`` -> ``7.1.0``: promotion);
  with commits after it the usual bump applies to the base.
* With no product tag at all there is no release boundary to measure from, so
  the workspace version in ``Cargo.toml`` is used unchanged.

Artifacts
---------
The version lives in ``Cargo.toml`` (``[workspace.package]``) and is mirrored
into every other file that carries it.  The list is discovered, not hardcoded
per release: a new ``package.json``, ``tauri*.conf.json`` or ``.product`` with
a ``version`` key is picked up automatically, so a new artifact can never be
silently left behind.

Commands
--------
``next [--bump auto|patch|minor|major]``
    Print the next product tag (``vX.Y.Z``) and, on stderr, the commits that
    drove the decision.
``--write``
    Sync the computed version into every artifact, then print the tag.
``--check``
    Fail (exit 1) when any artifact disagrees with ``Cargo.toml``.
``--self-test``
    Run ``scripts/tests/test_next_version.py``.

Standard library only: the release workflow must not need Node or npm.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import release_version as rv  # noqa: E402  (sibling script, stdlib only)

# v7.1.0 and v7.1.0-rc.1 are product tags; v7.0.2-test2 is a pre-release too.
TAG_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z][0-9A-Za-z.-]*))?$")
CONVENTIONAL_RE = re.compile(r"^([A-Za-z]+)(?:\([^()\r\n]*\))?(!)?\s*:")
BREAKING_FOOTER_RE = re.compile(r"(?mi)^BREAKING[ -]CHANGE\s*:")

KINDS = ("patch", "minor", "major")
KIND_RANK = {kind: rank for rank, kind in enumerate(KINDS)}

# Files that carry a version of their own, not the product version. Every entry
# needs a reason: anything else with a version is an artifact and must be
# synced, so a new one can never be silently left behind.
NON_PRODUCT_VERSION_FILES = {
    "botui/package.json": (
        "internal npm package for the UI server tests; its own 1.0.0 is not the "
        "product version (that is served by /api/product from Cargo.toml)"
    ),
}


@dataclass(frozen=True)
class Tag:
    """A parsed product tag."""

    name: str
    version: tuple[int, int, int]
    prerelease: str | None

    def sort_key(self) -> tuple[int, int, int, int, str]:
        # A final release sorts above every pre-release of the same base.
        return (*self.version, 1 if self.prerelease is None else 0, self.prerelease or "")


@dataclass(frozen=True)
class Artifact:
    """A file that mirrors the product version, described by one regex."""

    path: str
    pattern: str
    expected: int = 1


def parse_tag(name: str) -> Tag | None:
    match = TAG_RE.match(name.strip())
    if not match:
        return None
    major, minor, patch, prerelease = match.groups()
    return Tag(
        name=name.strip(),
        version=(int(major), int(minor), int(patch)),
        prerelease=prerelease,
    )


def latest_tag(tags: list[Tag]) -> Tag | None:
    return max(tags, key=lambda tag: tag.sort_key()) if tags else None


def git_product_tags(root: Path) -> list[Tag]:
    try:
        output = subprocess.check_output(
            ["git", "tag", "--list", "v*"],
            cwd=root,
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        return []
    parsed = [parse_tag(line) for line in output.splitlines()]
    return [tag for tag in parsed if tag is not None]


def git_log(root: Path, revision_range: str | None, pretty: str) -> str:
    command = ["git", "log"]
    if revision_range:
        command.append(revision_range)
    command.append(f"--pretty={pretty}")
    try:
        return subprocess.check_output(
            command, cwd=root, text=True, stderr=subprocess.DEVNULL
        )
    except (OSError, subprocess.CalledProcessError):
        return ""


def git_subjects(root: Path, since: Tag | None) -> list[str]:
    """Commit subjects after ``since``: ``git log <tag>..HEAD --pretty=%s``."""
    revision_range = f"{since.name}..HEAD" if since else None
    return [line for line in git_log(root, revision_range, "%s").splitlines() if line.strip()]


def git_bodies(root: Path, since: Tag | None) -> list[str]:
    """Full commit messages, used only for the ``BREAKING CHANGE`` footer."""
    revision_range = f"{since.name}..HEAD" if since else None
    text = git_log(root, revision_range, "%B%x00")
    return [block.strip() for block in text.split("\x00") if block.strip()]


def bump_kind(subjects: list[str], bodies: list[str]) -> str:
    """Highest Conventional-Commits bump present in the history."""
    kind = "patch"
    for subject in subjects:
        match = CONVENTIONAL_RE.match(subject)
        if not match:
            continue
        commit_type, bang = match.group(1).lower(), match.group(2)
        if bang:
            return "major"
        if commit_type == "feat" and KIND_RANK["minor"] > KIND_RANK[kind]:
            kind = "minor"
    if any(BREAKING_FOOTER_RE.search(body) for body in bodies):
        return "major"
    return kind


def compute_next(
    workspace: tuple[int, int, int],
    latest: Tag | None,
    subjects: list[str],
    bodies: list[str],
    bump: str = "auto",
) -> tuple[tuple[int, int, int], str]:
    """Return ``(version, reason)`` for the next product release."""
    if latest is None:
        return workspace, "no product tag yet: keeping the Cargo.toml version"

    kind = bump_kind(subjects, bodies) if bump == "auto" else bump

    if latest.prerelease and not subjects:
        return latest.version, (
            f"{latest.name} has no commits after it: promoting to the final release"
        )

    version = rv.bump_version(latest.version, kind)
    source = "auto-detected" if bump == "auto" else "requested"
    return version, f"{kind} bump ({source}) from {latest.name}"


def artifacts(root: Path) -> list[Artifact]:
    """Every file that mirrors the version, discovered from the tree."""
    found = [
        Artifact("botapp/tauri.conf.json", r'"version"\s*:\s*"(\d+\.\d+\.\d+)"'),
        Artifact("README.md", r"version-(\d+\.\d+\.\d+)-informational"),
        Artifact(
            "botbook/src/README.md",
            r"(?m)^(?:This documentation describes General Bots version |"
            r"The current version is )(\d+\.\d+\.\d+)",
            expected=2,
        ),
        Artifact("botbook/src/introduction.md", r"(?m)^- \*\*Version\*\*: (\d+\.\d+\.\d+)$"),
        Artifact("botbook/src/features.md", r"(?m)^\*\*Version:\*\* (\d+\.\d+\.\d+)$"),
    ]
    for candidate in ("package.json", "botapp/package.json", "botui/package.json"):
        if candidate in NON_PRODUCT_VERSION_FILES:
            continue
        path = root / candidate
        if path.is_file() and re.search(
            r'"version"\s*:\s*"\d+\.\d+\.\d+"', rv.read_text(path)
        ):
            found.append(Artifact(candidate, r'"version"\s*:\s*"(\d+\.\d+\.\d+)"'))
    for product in sorted((root / "botserver").glob(".product*")):
        if product.is_file() and re.search(
            r"(?m)^version\s*=\s*\d+\.\d+\.\d+\s*$", rv.read_text(product)
        ):
            found.append(
                Artifact(
                    product.relative_to(root).as_posix(),
                    r"(?m)^version\s*=\s*(\d+\.\d+\.\d+)\s*$",
                )
            )
    return [artifact for artifact in found if (root / artifact.path).is_file()]


def artifact_versions(root: Path, artifact: Artifact) -> list[str]:
    text = rv.read_text(root / artifact.path)
    return [match.group(1) for match in re.finditer(artifact.pattern, text)]


def check(root: Path) -> list[str]:
    """Every disagreement between an artifact and ``Cargo.toml``."""
    cargo = rv.read_text(root / "Cargo.toml")
    expected = ".".join(str(part) for part in rv.workspace_version(cargo))
    problems: list[str] = []

    lock_path = root / "Cargo.lock"
    if lock_path.exists():
        lock = rv.read_text(lock_path)
        if rv.update_cargo_lock(lock, rv.versioned_members(root, cargo), rv.parse_version(expected)) != lock:
            problems.append(f"Cargo.lock disagrees with Cargo.toml ({expected})")

    for artifact in artifacts(root):
        versions = artifact_versions(root, artifact)
        if len(versions) != artifact.expected:
            problems.append(
                f"{artifact.path}: expected {artifact.expected} version claim(s), "
                f"found {len(versions)} (is the pattern still right?)"
            )
            continue
        for version in versions:
            if version != expected:
                problems.append(
                    f"{artifact.path} claims {version}, Cargo.toml has {expected}"
                )
    return problems


def sync(root: Path, version: tuple[int, int, int]) -> list[str]:
    """Write ``version`` into every artifact; return the changed paths."""
    changed: list[str] = []
    new = ".".join(str(part) for part in version)

    cargo_path = root / "Cargo.toml"
    cargo = rv.read_text(cargo_path)
    updated = rv.update_cargo_toml(cargo, version)
    if rv.write_text(cargo_path, updated):
        changed.append("Cargo.toml")

    lock_path = root / "Cargo.lock"
    if lock_path.exists():
        lock = rv.read_text(lock_path)
        updated_lock = rv.update_cargo_lock(lock, rv.versioned_members(root, cargo), version)
        if rv.write_text(lock_path, updated_lock):
            changed.append("Cargo.lock")

    for artifact in artifacts(root):
        path = root / artifact.path
        text = rv.read_text(path)
        versions = artifact_versions(root, artifact)
        if len(versions) != artifact.expected:
            raise SystemExit(
                f"{artifact.path}: expected {artifact.expected} version claim(s), "
                f"found {len(versions)}; update the pattern in scripts/next_version.py"
            )
        def replace_claim(match: re.Match[str], new_version: str = new) -> str:
            start, end = match.span(1)
            base = match.start(0)
            text_part = match.group(0)
            return text_part[: start - base] + new_version + text_part[end - base :]

        updated = re.sub(artifact.pattern, replace_claim, text)
        if rv.write_text(path, updated):
            changed.append(artifact.path)
    return changed


def command_next(args: argparse.Namespace) -> int:
    root = rv.repo_root(args.root)
    cargo = rv.read_text(root / "Cargo.toml")
    workspace = rv.workspace_version(cargo)
    tags = git_product_tags(root)
    latest = latest_tag(tags)
    since = parse_tag(args.since) if args.since else latest
    if args.since and since is None:
        raise SystemExit(f"--since must be a product tag like v7.0.0 (got {args.since!r})")

    subjects = git_subjects(root, since)
    bodies = git_bodies(root, since)
    version, reason = compute_next(workspace, latest, subjects, bodies, args.bump)

    if args.write:
        for path in sync(root, version):
            print(f"updated {path}", file=sys.stderr)
        problems = check(root)
        if problems:
            for problem in problems:
                print(f"version drift: {problem}", file=sys.stderr)
            return 1

    print(f"{len(subjects)} commit(s) since {since.name if since else 'the first commit'}", file=sys.stderr)
    print(reason, file=sys.stderr)
    print(rv.format_tag(version))
    return 0


def command_check(args: argparse.Namespace) -> int:
    root = rv.repo_root(args.root)
    problems = check(root)
    if problems:
        for problem in problems:
            print(f"version drift: {problem}", file=sys.stderr)
        print(
            "run: python3 scripts/next_version.py --write", file=sys.stderr
        )
        return 1
    current = ".".join(str(part) for part in rv.workspace_version(rv.read_text(root / "Cargo.toml")))
    files = len(artifacts(root)) + 1 + (1 if (root / "Cargo.lock").exists() else 0)
    print(f"versions agree: {current} ({files} artifact(s))")
    return 0


def command_self_test(_args: argparse.Namespace) -> int:
    import unittest

    tests = Path(__file__).resolve().parent / "tests"
    if not tests.is_dir():
        print("self-test: scripts/tests is missing", file=sys.stderr)
        return 1
    sys.path.insert(0, str(tests))
    suite = unittest.TestLoader().discover(str(tests), pattern="test_next_version.py")
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    return 0 if result.wasSuccessful() else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", help="repository root (defaults to the parent of this script)")
    parser.add_argument("--bump", choices=("auto", "patch", "minor", "major"), default="auto")
    parser.add_argument("--since", help="compute from this tag instead of the newest one")
    parser.add_argument("--write", action="store_true", help="sync the computed version into every artifact")
    parser.add_argument("--check", action="store_true", help="fail when an artifact disagrees with Cargo.toml")
    parser.add_argument("--self-test", action="store_true", help="run scripts/tests/test_next_version.py")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.self_test:
        return command_self_test(args)
    if args.check:
        return command_check(args)
    return command_next(args)


if __name__ == "__main__":
    sys.exit(main())
