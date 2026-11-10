#!/usr/bin/env python3
"""Unit tests for ``scripts/next_version.py`` (#2118).

Run directly (``python3 scripts/tests/test_next_version.py``), through the
script (``python3 scripts/next_version.py --self-test``) or with
``python3 -m unittest discover -s scripts/tests``.

Every test builds a throw-away git repository in a temporary directory, so the
suite is offline and never touches the working tree.
"""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import next_version as nv  # noqa: E402

TAURI = '{\n  "productName": "General Bots",\n  "version": "7.0.0"\n}\n'
README = (
    "[![Version](https://img.shields.io/badge/version-7.0.0-informational.svg)]"
    "(https://github.com/generalbots/generalbots/releases)\n"
)
BOTBOOK_README = (
    "# General Bots Documentation\n\n"
    "This documentation describes General Bots version 7.0.0.\n\n"
    "The current version is 7.0.0 released under the MIT license.\n"
)


def git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        text=True,
        capture_output=True,
    ).stdout


def write_artifacts(root: Path, version: str = "7.0.0") -> None:
    (root / "botapp").mkdir(parents=True, exist_ok=True)
    (root / "botbook" / "src").mkdir(parents=True, exist_ok=True)
    (root / "Cargo.toml").write_text(
        '[workspace]\nmembers = ["botapp"]\n\n'
        f'[workspace.package]\nversion = "{version}"\n',
        encoding="utf-8",
    )
    (root / "Cargo.lock").write_text(
        f'[[package]]\nname = "botapp"\nversion = "{version}"\n\n'
        '[[package]]\nname = "serde"\nversion = "1.0.0"\n',
        encoding="utf-8",
    )
    (root / "botapp" / "Cargo.toml").write_text(
        '[package]\nname = "botapp"\nversion.workspace = true\n', encoding="utf-8"
    )
    (root / "botapp" / "tauri.conf.json").write_text(
        TAURI.replace("7.0.0", version), encoding="utf-8"
    )
    (root / "README.md").write_text(README.replace("7.0.0", version), encoding="utf-8")
    (root / "botbook" / "src" / "README.md").write_text(
        BOTBOOK_README.replace("7.0.0", version), encoding="utf-8"
    )
    (root / "botbook" / "src" / "introduction.md").write_text(
        f"# Introduction\n\n- **Version**: {version}\n", encoding="utf-8"
    )
    (root / "botbook" / "src" / "features.md").write_text(
        f"# Features\n\n**Version:** {version}\n", encoding="utf-8"
    )


def make_repo(root: Path, version: str = "7.0.0") -> Path:
    write_artifacts(root, version)
    git(root, "init", "-q", "-b", "main")
    git(root, "config", "user.email", "release-bot@generalbots.org")
    git(root, "config", "user.name", "Release Bot")
    git(root, "add", "-A")
    git(root, "commit", "-q", "-m", "chore: initial")
    return root


def commit(root: Path, subject: str, body: str = "") -> None:
    message = subject if not body else f"{subject}\n\n{body}"
    git(root, "commit", "-q", "--allow-empty", "-m", message)


def compute(root: Path, bump: str = "auto") -> tuple[int, int, int]:
    tags = nv.git_product_tags(root)
    latest = nv.latest_tag(tags)
    subjects = nv.git_subjects(root, latest)
    bodies = nv.git_bodies(root, latest)
    workspace = nv.rv.workspace_version(nv.rv.read_text(root / "Cargo.toml"))
    return nv.compute_next(workspace, latest, subjects, bodies, bump)[0]


class BumpRulesTest(unittest.TestCase):
    """feat -> minor, everything else -> patch, breaking -> major."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = make_repo(Path(self._tmp.name))
        git(self.root, "tag", "v7.0.0")

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_feat_bumps_minor(self) -> None:
        commit(self.root, "feat(chat): add the widget launcher")
        self.assertEqual(compute(self.root), (7, 1, 0))

    def test_fix_bumps_patch(self) -> None:
        commit(self.root, "fix(build): bind the version before splitting it")
        self.assertEqual(compute(self.root), (7, 0, 1))

    def test_non_conventional_bumps_patch(self) -> None:
        commit(self.root, "Merge pull request #2118 from generalbots/main")
        self.assertEqual(compute(self.root), (7, 0, 1))

    def test_feat_bang_bumps_major(self) -> None:
        commit(self.root, "feat(api)!: drop the v1 product endpoint")
        self.assertEqual(compute(self.root), (8, 0, 0))

    def test_breaking_change_footer_bumps_major(self) -> None:
        commit(
            self.root,
            "fix(config): rename the product key",
            "The old key stops loading.\n\nBREAKING CHANGE: .product renamed.",
        )
        self.assertEqual(compute(self.root), (8, 0, 0))

    def test_highest_bump_wins(self) -> None:
        commit(self.root, "fix(a): one")
        commit(self.root, "feat(b): two")
        commit(self.root, "chore(c): three")
        self.assertEqual(compute(self.root), (7, 1, 0))

    def test_explicit_bump_overrides_history(self) -> None:
        commit(self.root, "feat(chat): add the widget launcher")
        self.assertEqual(compute(self.root, bump="patch"), (7, 0, 1))
        self.assertEqual(compute(self.root, bump="major"), (8, 0, 0))

    def test_only_commits_after_the_tag_count(self) -> None:
        commit(self.root, "feat(a): before the tag")
        git(self.root, "tag", "v7.1.0")
        commit(self.root, "fix(b): after the tag")
        self.assertEqual(compute(self.root), (7, 1, 1))


class TagHandlingTest(unittest.TestCase):
    """Pre-release tags, malformed tags and a repository with no tag."""

    def test_no_tag_keeps_the_workspace_version(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            commit(root, "feat(chat): add the widget launcher")
            self.assertEqual(compute(root), (7, 0, 0))

    def test_prerelease_tag_is_promoted_when_nothing_follows(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            git(root, "tag", "v7.1.0-rc.1")
            self.assertEqual(compute(root), (7, 1, 0))

    def test_prerelease_tag_bumps_from_its_base(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            git(root, "tag", "v7.1.0-rc.1")
            commit(root, "feat(chat): late widget")
            self.assertEqual(compute(root), (7, 2, 0))

    def test_final_release_sorts_above_its_prerelease(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            git(root, "tag", "v7.1.0-rc.1")
            git(root, "tag", "v7.1.0")
            latest = nv.latest_tag(nv.git_product_tags(root))
            self.assertIsNotNone(latest)
            self.assertEqual(latest.name, "v7.1.0")

    def test_non_version_tags_are_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            git(root, "tag", "verify-splits-219")
            git(root, "tag", "v7.0.34")
            names = [tag.name for tag in nv.git_product_tags(root)]
            self.assertEqual(names, ["v7.0.34"])
            commit(root, "fix(a): one")
            self.assertEqual(compute(root), (7, 0, 35))


class ArtifactSyncTest(unittest.TestCase):
    """``--write`` reaches every artifact and ``--check`` catches drift."""

    def test_sync_writes_every_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            changed = nv.sync(root, (7, 4, 2))
            self.assertIn("Cargo.toml", changed)
            self.assertIn("Cargo.lock", changed)
            self.assertIn("botbook/src/README.md", changed)
            self.assertEqual(nv.check(root), [])
            for artifact in nv.artifacts(root):
                self.assertEqual(
                    nv.artifact_versions(root, artifact),
                    ["7.4.2"] * artifact.expected,
                    artifact.path,
                )
            # A non-workspace dependency keeps its own version.
            self.assertIn('name = "serde"\nversion = "1.0.0"', (root / "Cargo.lock").read_text())

    def test_check_detects_a_mismatched_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            nv.sync(root, (7, 4, 2))
            (root / "botapp" / "tauri.conf.json").write_text(TAURI, encoding="utf-8")
            problems = nv.check(root)
            self.assertEqual(len(problems), 1)
            self.assertIn("botapp/tauri.conf.json claims 7.0.0", problems[0])

    def test_check_detects_a_missing_claim(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            (root / "botbook" / "src" / "README.md").write_text(
                "# General Bots Documentation\n", encoding="utf-8"
            )
            problems = nv.check(root)
            self.assertEqual(len(problems), 1)
            self.assertIn("expected 2 version claim(s), found 0", problems[0])

    def test_package_json_is_discovered(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            (root / "package.json").write_text('{\n  "version": "7.0.0"\n}\n', encoding="utf-8")
            self.assertIn("package.json", [artifact.path for artifact in nv.artifacts(root)])
            nv.sync(root, (7, 4, 2))
            self.assertEqual(nv.check(root), [])

    def test_internal_npm_package_is_not_a_product_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            (root / "botui").mkdir()
            (root / "botui" / "package.json").write_text(
                '{\n  "name": "botui",\n  "version": "1.0.0"\n}\n', encoding="utf-8"
            )
            self.assertNotIn(
                "botui/package.json", [artifact.path for artifact in nv.artifacts(root)]
            )
            self.assertEqual(nv.check(root), [])
            self.assertIn(
                "product version", nv.NON_PRODUCT_VERSION_FILES["botui/package.json"]
            )


class CommandLineTest(unittest.TestCase):
    """The exit codes the workflows depend on."""

    def test_write_then_check_passes_and_drift_fails(self) -> None:
        script = Path(__file__).resolve().parents[1] / "next_version.py"
        with tempfile.TemporaryDirectory() as tmp:
            root = make_repo(Path(tmp))
            git(root, "tag", "v7.0.0")
            commit(root, "feat(chat): add the widget launcher")

            written = subprocess.run(
                [sys.executable, str(script), "--root", str(root), "--write"],
                text=True,
                capture_output=True,
            )
            self.assertEqual(written.returncode, 0, written.stderr)
            self.assertEqual(written.stdout.strip(), "v7.1.0")

            clean = subprocess.run(
                [sys.executable, str(script), "--root", str(root), "--check"],
                text=True,
                capture_output=True,
            )
            self.assertEqual(clean.returncode, 0, clean.stderr)

            (root / "README.md").write_text(README, encoding="utf-8")
            drifted = subprocess.run(
                [sys.executable, str(script), "--root", str(root), "--check"],
                text=True,
                capture_output=True,
            )
            self.assertEqual(drifted.returncode, 1)
            self.assertIn("README.md claims 7.0.0", drifted.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
