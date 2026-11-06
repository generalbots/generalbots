#!/usr/bin/env python3
"""Reject mutable GitHub Actions references in .github/workflows (#1726).

A workflow that runs `uses: some/action@v4` executes whatever that tag points
at *today*. Tags and branches are mutable, so a compromised or retargeted tag
turns into remote code execution with our repository token. This check
therefore requires every remote `uses:` to carry a full 40-character commit
SHA (plus a version comment for humans), and rejects triggers that hand a
fork's code our secrets.

Rules enforced
--------------
1. `owner/repo@ref` -- `ref` MUST be a 40-character lowercase hex commit SHA.
2. `docker://image` -- MUST pin the image by digest (`@sha256:...`). A tag is
   mutable in exactly the same way as an action tag.
3. Local actions (`./path`) are fine: they are part of this commit.
4. `pull_request_target` is rejected outright. The only legitimate use is
   labelling/commenting without checking out the fork, and this repository has
   none, so the safe policy is "do not use it".
5. Checking out the pull-request head inside a privileged workflow
   (`github.event.pull_request.head.sha` / `head_ref`) is rejected.

Allowlist
---------
`dtolnay/rust-toolchain` selects the toolchain *from the ref itself*: it has no
tags to speak of, so pinning it to a SHA would defeat its purpose. The
reference is a release channel (`stable`, `1.83.0`, ...), not code that runs
arbitrarily -- the action body is fetched from the default branch. The
exception is explicit here, and each entry needs a reason.

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

WORKFLOWS_DIR = Path(".github/workflows")

# owner/repo -> reason. Keep this list as short as possible.
REF_ALLOWLIST = {
    "dtolnay/rust-toolchain": (
        "the toolchain name IS the ref; the action body is fetched from the "
        "action's default branch and the ref never names a commit"
    ),
}

USES_RE = re.compile(r"^(?P<indent>\s*)(?:-\s+)?uses:\s*(?P<ref>\S+)\s*(?P<comment>#.*)?$")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
DIGEST_RE = re.compile(r"@sha256:[0-9a-f]{64}$")


def short(text: str, width: int = 60) -> str:
    return text if len(text) <= width else text[: width - 1] + "…"


def check_uses(path: Path, lineno: int, ref: str, comment: str, errors: list[str], warnings: list[str]) -> None:
    where = f"{path}:{lineno}"

    if ref.startswith("./") or ref.startswith(".\\"):
        return  # local composite action, versioned by this commit

    if ref.startswith("docker://"):
        if not DIGEST_RE.search(ref):
            errors.append(
                f"{where}: `uses: {short(ref)}` must pin the image by digest "
                "(docker://repo/image@sha256:<64 hex>), not by a mutable tag"
            )
        return

    if "@" not in ref:
        errors.append(f"{where}: `uses: {short(ref)}` has no `@<commit sha>` ref")
        return

    action, _, version = ref.rpartition("@")
    if action in REF_ALLOWLIST:
        return

    if SHA_RE.match(version):
        if not comment:
            warnings.append(
                f"{where}: `uses: {short(ref)}` is SHA-pinned but has no "
                "trailing `# vX.Y.Z` comment; humans cannot tell what it is"
            )
        return

    errors.append(
        f"{where}: `uses: {short(ref)}` is pinned to a mutable ref `{version}`; "
        "use the 40-character commit SHA of the release "
        "(e.g. `uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.2.2`)"
    )


def check_privileged_triggers(path: Path, lines: list[str], errors: list[str]) -> None:
    for idx, raw in enumerate(lines, start=1):
        stripped = raw.strip()
        if stripped == "pull_request_target:" or stripped.startswith("pull_request_target: "):
            errors.append(
                f"{path}:{idx}: `pull_request_target` runs with the base repo's "
                "secrets and token; use `pull_request` and split privileged work "
                "into a separate `workflow_run` workflow"
            )
        if "github.event.pull_request.head.sha" in stripped or "github.event.pull_request.head.ref" in stripped:
            errors.append(
                f"{path}:{idx}: checking out the pull-request head is only safe "
                "in an unprivileged `pull_request` workflow"
            )


def main() -> int:
    if not WORKFLOWS_DIR.is_dir():
        print(f"no {WORKFLOWS_DIR} directory; nothing to check")
        return 0

    files = sorted(p for p in WORKFLOWS_DIR.iterdir() if p.suffix in {".yml", ".yaml"})
    if not files:
        print(f"no workflow files under {WORKFLOWS_DIR}")
        return 0

    errors: list[str] = []
    warnings: list[str] = []
    pinned = 0

    for path in files:
        lines = path.read_text(encoding="utf-8").splitlines()
        check_privileged_triggers(path, lines, errors)
        for lineno, raw in enumerate(lines, start=1):
            match = USES_RE.match(raw)
            if not match:
                continue
            ref = match.group("ref").strip().strip("'\"")
            comment = (match.group("comment") or "").strip()
            before = len(errors)
            check_uses(path, lineno, ref, comment, errors, warnings)
            if len(errors) == before:
                pinned += 1

    for warning in warnings:
        print(f"warning: {warning}")
    for error in errors:
        print(f"error: {error}")

    if errors:
        print(f"\n{len(errors)} mutable reference(s) found in {len(files)} workflow file(s)")
        return 1

    print(f"ok: {pinned} action reference(s) across {len(files)} workflow file(s) are pinned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
