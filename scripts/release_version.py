#!/usr/bin/env python3
"""Product release version helpers (issue #1804).

The product release line is a plain semantic version tag, ``vMAJOR.MINOR.PATCH``.
Component builds keep their own tags (``alm-*``, ``pythonbox-*``,
``drive-bin-*``) and are published as pre-releases, so ``/releases/latest``
always resolves to a product release that carries the desktop assets.

Commands
--------
``current``
    Print the version declared in ``Cargo.toml`` (``[workspace.package]``).
``next [--bump patch|minor|major]``
    Print the next product tag.  A workspace version ahead of every tag is a
    deliberate bump and is published as-is; otherwise the requested component
    advances from the newest ``vMAJOR.MINOR.PATCH`` tag.  A release can be
    retried safely after a failed bump commit.
``sync <version>``
    Write one version into every file that carries it: ``Cargo.toml``,
    ``Cargo.lock`` (workspace members only), ``botapp/tauri.conf.json`` and the
    README version badge.  ``v`` prefixes are accepted on the command line.
``check`` / ``--check``
    Fail (exit 1) when any version-bearing file disagrees with ``Cargo.toml``.
    CI runs this so a hand-edited version cannot reach a release (#2103).
``selftest``
    Exercise ``sync`` and ``check`` against a throw-away tree: a simulated tag
    ``v9.8.7`` must land as ``9.8.7`` in every file.

The script only uses the Python standard library so the release workflow does
not need Node, npm or a semantic-release installation.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

PRODUCT_TAG_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def repo_root(explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    return Path(__file__).resolve().parents[1]


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_text(path: Path, text: str) -> bool:
    old = path.read_text(encoding="utf-8")
    if old == text:
        return False
    path.write_text(text, encoding="utf-8")
    return True


def workspace_package_block(cargo_toml: str) -> re.Match[str]:
    match = re.search(
        r"(?ms)^\[workspace\.package\]\s*$(.*?)(?=^\[|\Z)", cargo_toml
    )
    if not match:
        raise SystemExit("Cargo.toml has no [workspace.package] section")
    return match


def workspace_version(cargo_toml: str) -> tuple[int, int, int]:
    block = workspace_package_block(cargo_toml).group(1)
    match = re.search(r'^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', block, re.M)
    if not match:
        raise SystemExit("Cargo.toml [workspace.package] has no numeric version")
    return tuple(int(part) for part in match.groups())  # type: ignore[return-value]


def workspace_member_paths(cargo_toml: str) -> list[str]:
    match = re.search(r"(?ms)^members\s*=\s*\[(.*?)\]", cargo_toml)
    if not match:
        raise SystemExit("Cargo.toml has no [workspace] members list")
    return [member.rstrip("/") for member in re.findall(r'"([^"]+)"', match.group(1))]


def versioned_members(root: Path, cargo_toml: str) -> set[str]:
    """Members whose version is inherited from ``[workspace.package]``.

    A member that declares its own ``version = "0.1.0"`` keeps it; rewriting
    that entry in ``Cargo.lock`` would make the lockfile disagree with its
    manifest and break the next ``cargo build``.
    """
    names: set[str] = set()
    for member in workspace_member_paths(cargo_toml):
        manifest = root / member / "Cargo.toml"
        if not manifest.is_file():
            continue
        if re.search(r"(?m)^\s*version\.workspace\s*=\s*true", read_text(manifest)):
            names.add(member.split("/")[-1])
    return names


def git_product_tags(root: Path) -> list[tuple[int, int, int]]:
    try:
        output = subprocess.check_output(
            ["git", "tag", "--list", "v*"], cwd=root, text=True, stderr=subprocess.DEVNULL
        )
    except (OSError, subprocess.CalledProcessError):
        return []
    versions: list[tuple[int, int, int]] = []
    for line in output.splitlines():
        match = PRODUCT_TAG_RE.match(line.strip())
        if match:
            versions.append(tuple(int(part) for part in match.groups()))  # type: ignore[arg-type]
    return versions


def parse_version(raw: str) -> tuple[int, int, int]:
    value = raw.strip()
    if value.startswith("v"):
        value = value[1:]
    match = SEMVER_RE.match(value)
    if not match:
        raise SystemExit(f"not a MAJOR.MINOR.PATCH version: {raw!r}")
    return tuple(int(part) for part in match.groups())  # type: ignore[return-value]


def bump_version(base: tuple[int, int, int], kind: str) -> tuple[int, int, int]:
    major, minor, patch = base
    if kind == "major":
        return major + 1, 0, 0
    if kind == "minor":
        return major, minor + 1, 0
    return major, minor, patch + 1


def format_tag(version: tuple[int, int, int]) -> str:
    return "v{}.{}.{}".format(*version)


def update_cargo_toml(text: str, new: tuple[int, int, int]) -> str:
    match = workspace_package_block(text)
    block = match.group(1)
    updated_block, count = re.subn(
        r'^(version\s*=\s*")\d+\.\d+\.\d+(")',
        r"\g<1>{}.{}.{}\g<2>".format(*new),
        block,
        count=1,
        flags=re.M,
    )
    if count != 1:
        raise SystemExit("could not update [workspace.package] version")
    return text[: match.start(1)] + updated_block + text[match.end(1) :]


def update_cargo_lock(
    text: str, members: set[str], new: tuple[int, int, int]
) -> str:
    package_re = re.compile(r"(?ms)^\[\[package\]\]\n.*?(?=^\[\[package\]\]\n|\Z)")
    new_string = "{}.{}.{}".format(*new)

    def replace_package(match: re.Match[str]) -> str:
        block = match.group(0)
        name = re.search(r'^name = "([^"]+)"', block, re.M)
        version = re.search(r'^version = "([^"]+)"', block, re.M)
        if name and version and name.group(1) in members:
            return block.replace(
                'version = "{}"'.format(version.group(1)),
                'version = "{}"'.format(new_string),
                1,
            )
        return block

    return package_re.sub(replace_package, text)


def update_tauri_config(text: str, new: tuple[int, int, int]) -> str:
    version = "{}.{}.{}".format(*new)
    updated, count = re.subn(
        r'("version"\s*:\s*")\d+\.\d+\.\d+(")',
        r"\g<1>{}\g<2>".format(version),
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit("could not update botapp/tauri.conf.json version")
    return updated


def update_readme_badge(text: str, new: tuple[int, int, int]) -> str:
    version = "{}.{}.{}".format(*new)
    updated, count = re.subn(
        r"(version-)\d+\.\d+\.\d+(-informational)",
        r"\g<1>{}\g<2>".format(version),
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit("could not update the README version badge")
    return updated


def command_current(args: argparse.Namespace) -> int:
    root = repo_root(args.root)
    version = workspace_version(read_text(root / "Cargo.toml"))
    print(".".join(str(part) for part in version))
    return 0


def command_next(args: argparse.Namespace) -> int:
    root = repo_root(args.root)
    cargo = read_text(root / "Cargo.toml")
    workspace = workspace_version(cargo)
    tags = git_product_tags(root)
    latest_tag = max(tags) if tags else None

    # A workspace version ahead of every tag is a deliberate bump (for example
    # a minor release prepared by hand): publish that version as-is. Otherwise
    # advance the requested component from the newest product tag.
    if latest_tag is None or workspace > latest_tag:
        print(format_tag(workspace))
    else:
        print(format_tag(bump_version(latest_tag, args.bump)))
    return 0


def command_check(args: argparse.Namespace) -> int:
    root = repo_root(args.root)
    cargo = read_text(root / "Cargo.toml")
    expected = workspace_version(cargo)
    drift = drifted_files(root, cargo, expected)
    if drift:
        for path in drift:
            print(f"version drift: {path} disagrees with Cargo.toml", file=sys.stderr)
        print(
            "run: python3 scripts/release_version.py sync "
            + ".".join(str(part) for part in expected),
            file=sys.stderr,
        )
        return 1
    print("versions agree: " + ".".join(str(part) for part in expected))
    return 0


def drifted_files(
    root: Path, cargo: str, expected: tuple[int, int, int]
) -> list[str]:
    """Every version-bearing file whose content would change on ``sync``."""
    drift: list[str] = []
    lock_path = root / "Cargo.lock"
    if lock_path.exists():
        lock = read_text(lock_path)
        members = versioned_members(root, cargo)
        if update_cargo_lock(lock, members, expected) != lock:
            drift.append("Cargo.lock")
    tauri_path = root / "botapp" / "tauri.conf.json"
    if tauri_path.exists():
        tauri = read_text(tauri_path)
        if update_tauri_config(tauri, expected) != tauri:
            drift.append("botapp/tauri.conf.json")
    readme_path = root / "README.md"
    if readme_path.exists():
        readme = read_text(readme_path)
        if update_readme_badge(readme, expected) != readme:
            drift.append("README.md")
    return drift


def command_selftest(args: argparse.Namespace) -> int:
    """Simulate a ``v9.8.7`` tag in a temporary tree (no repository writes)."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "botapp").mkdir()
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["botapp"]\n\n'
            '[workspace.package]\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (root / "Cargo.lock").write_text(
            '[[package]]\nname = "botapp"\nversion = "0.1.0"\n\n'
            '[[package]]\nname = "serde"\nversion = "1.0.0"\n',
            encoding="utf-8",
        )
        (root / "botapp" / "tauri.conf.json").write_text(
            '{\n  "version": "0.1.0"\n}\n', encoding="utf-8"
        )
        (root / "botapp" / "Cargo.toml").write_text(
            '[package]\nname = "botapp"\nversion.workspace = true\n',
            encoding="utf-8",
        )
        (root / "README.md").write_text(
            "[![Version](https://img.shields.io/badge/version-0.1.0-informational)](x)\n",
            encoding="utf-8",
        )

        sync_args = argparse.Namespace(root=str(root), version="v9.8.7")
        if command_sync(sync_args) != 0:
            print("selftest: sync failed", file=sys.stderr)
            return 1

        expected = "9.8.7"
        checks = {
            "Cargo.toml": read_text(root / "Cargo.toml"),
            "Cargo.lock": read_text(root / "Cargo.lock"),
            "botapp/tauri.conf.json": read_text(root / "botapp" / "tauri.conf.json"),
            "README.md": read_text(root / "README.md"),
        }
        for path, text in checks.items():
            if expected not in text:
                print(f"selftest: {path} did not receive {expected}", file=sys.stderr)
                return 1
        # A dependency version must survive the sync untouched.
        if 'name = "serde"\nversion = "1.0.0"' not in checks["Cargo.lock"]:
            print("selftest: sync rewrote a non-workspace dependency", file=sys.stderr)
            return 1

        if command_check(argparse.Namespace(root=str(root))) != 0:
            print("selftest: check rejected a synced tree", file=sys.stderr)
            return 1

        # Drift must be detected: hand-edit the Tauri config back.
        tauri = root / "botapp" / "tauri.conf.json"
        tauri.write_text('{\n  "version": "9.8.6"\n}\n', encoding="utf-8")
        if command_check(argparse.Namespace(root=str(root))) == 0:
            print("selftest: check missed version drift", file=sys.stderr)
            return 1

    print("selftest: ok (v9.8.7 -> 9.8.7, drift detected)")
    return 0


def command_sync(args: argparse.Namespace) -> int:
    root = repo_root(args.root)
    new = parse_version(args.version)
    cargo_path = root / "Cargo.toml"
    cargo = read_text(cargo_path)
    old = workspace_version(cargo)
    if old == new:
        print("Cargo.toml already at {}.{}.{}".format(*new))
    else:
        write_text(cargo_path, update_cargo_toml(cargo, new))

    # Always normalize the lockfile: a previous run may have updated the
    # manifests but failed before writing Cargo.lock.
    lock_path = root / "Cargo.lock"
    if lock_path.exists():
        write_text(
            lock_path,
            update_cargo_lock(read_text(lock_path), versioned_members(root, cargo), new),
        )

    tauri_path = root / "botapp" / "tauri.conf.json"
    if tauri_path.exists():
        write_text(tauri_path, update_tauri_config(read_text(tauri_path), new))

    readme_path = root / "README.md"
    if readme_path.exists():
        write_text(readme_path, update_readme_badge(read_text(readme_path), new))

    print(format_tag(new))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        help="repository root (defaults to the parent of this script)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail when a version-bearing file disagrees with Cargo.toml",
    )
    sub = parser.add_subparsers(dest="command")

    current = sub.add_parser("current", help="print the workspace version")
    current.set_defaults(func=command_current)

    following = sub.add_parser("next", help="print the next product tag")
    following.add_argument(
        "--bump", choices=("patch", "minor", "major"), default="patch"
    )
    following.set_defaults(func=command_next)

    sync = sub.add_parser("sync", help="write a version into every version file")
    sync.add_argument("version")
    sync.set_defaults(func=command_sync)

    check = sub.add_parser("check", help="verify every version file agrees")
    check.set_defaults(func=command_check)

    selftest = sub.add_parser("selftest", help="simulate a tag in a temporary tree")
    selftest.set_defaults(func=command_selftest)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.check:
        return command_check(args)
    func = getattr(args, "func", None)
    if func is None:
        parser.error("choose a command: current, next, sync, check, selftest")
    return func(args)


if __name__ == "__main__":
    sys.exit(main())
