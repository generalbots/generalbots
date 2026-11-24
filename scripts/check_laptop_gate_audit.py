#!/usr/bin/env python3
"""Static audit of the #2471 laptop acceptance gate — WHAT IS ACTUALLY CHECKED.

This script deliberately distinguishes two different things that are easy to
conflate:

  * hard guarantees that really exist in the tree today (QEMU boot smoke with a
    kiosk-shell marker, the appliance verifier wired into CI, its fail-closed
    guards, the PR/release gate) — each is asserted and a missing one exits 1;
  * the laptop criteria demanded by issue #2471 — each is printed as
    [AUTOMATED] only when a script/test in the tree really exercises it, and as
    [GAP] when the only trace is a comment, a SKIP reason or a data placeholder.

A [GAP] never fails this audit. That is the point: exit 0 means "the guarantees
that exist are present", NOT "laptop acceptance is satisfied". The real laptop
UAT (booted ISO, QEMU device profile, mouse+keyboard scenarios, screenshots and
five physical machines) is hardware-blocked and stays OPEN.

Pure stdlib, read-only, no network, no cargo, no browser, no commits.
Usage: python3 scripts/check_laptop_gate_audit.py [--root DIR]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

P = {
    "workflow": ".github/workflows/desktop-os-iso.yml",
    "ci": ".github/workflows/ci.yml",
    "build_iso": "scripts/build_iso.sh",
    "verifier": "desktop/os/release/verify-image.sh",
    "release": "desktop/os/release/build-release-image.sh",
    "manifest": "desktop/os/release/appliance-manifest.tsv",
    "build_image": "desktop/os/build-image.sh",
    "acceptance": "desktop/os/tests/laptop-acceptance.sh",
    "matrix": "desktop/os/tests/hardware-matrix.psv",
}

SCREENSHOT_RX = r"screenshot|scrot|xwd|grim|screencopy|import\s+-window"
INPUT_RX = r"xdotool|ydotool|mouse(?:move|click)|key(?:board)?\s+(?:type|press)"
DATA_ROW_RX = re.compile(r"^[A-Z][A-Z0-9_.-]*\|")


def read_rel(root: Path, rel: str) -> str:
    path = root / rel
    if not path.is_file():
        return ""
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"[warn] unreadable {rel}: {exc}", file=sys.stderr)
        return ""


def read(root: Path, key: str) -> str:
    return read_rel(root, P[key])


def is_code(line: str) -> bool:
    """Comments, SKIPs and PSV/heredoc data rows are not executable checks."""
    stripped = line.strip()
    if not stripped or stripped.startswith(("#", "//")):
        return False
    if "SKIP" in stripped.upper() or DATA_ROW_RX.match(stripped):
        return False
    return True


def code_text(text: str) -> str:
    return "\n".join(line for line in text.splitlines() if is_code(line))


def find(text: str, pattern: str, flags: int = re.IGNORECASE):
    """First code line matching pattern, as (line_number, trimmed_line)."""
    rx = re.compile(pattern, flags)
    for number, line in enumerate(text.splitlines(), start=1):
        if is_code(line) and rx.search(line):
            return number, line.strip()[:110]
    return None


def find_any(root: Path, paths: list[str], pattern: str):
    for rel in paths:
        hit = find(read_rel(root, rel), pattern)
        if hit:
            return rel, hit[0], hit[1]
    return None


def source_paths(root: Path) -> list[str]:
    """Executable-looking sources where a scripted laptop check could live."""
    paths = [P[key] for key in ("workflow", "build_image", "build_iso", "acceptance")]
    for pattern in ("desktop/os/**/*.sh", "desktop/os/**/*.conf"):
        paths.extend(str(p.relative_to(root)) for p in sorted(root.glob(pattern)))
    return list(dict.fromkeys(paths))


def evidence(hit) -> str:
    if not hit:
        return "none"
    return f"{hit[0]}:{hit[1]}"


# --- hard guarantees: absence is a real failure --------------------------------

def g_smoke(root: Path):
    workflow = read(root, "workflow")
    required = (
        ("mkosi vm invocation", r"mkosi\s+--output-dir\s+/out\s+vm"),
        ("timeout", r"timeout\s+480"),
        ("serial log artifact", r"vm-smoke\.log"),
        ("required (not best-effort) gate", r"REQUIRED boot smoke"),
    )
    missing = [label for label, rx in required if not find(workflow, rx)]
    if find(code_text(workflow), r"continue-on-error"):
        missing.append("no continue-on-error")
    if missing:
        return False, "missing: " + ", ".join(missing)
    where = find(workflow, required[0][1])
    return True, f"{P['workflow']}:{where[0]} runs a timed, logged, release-blocking mkosi vm"


def g_boot_marker(root: Path):
    workflow = read(root, "workflow")
    image = read(root, "build_image")
    marker = r"Started General Bots kiosk shell"
    hits = [find(workflow, marker), find(image, marker)]
    if any(hit is None for hit in hits):
        return False, "kiosk-shell boot marker missing in workflow or build-image.sh"
    return True, f"{P['workflow']}:{hits[0][0]} and {P['build_image']}:{hits[1][0]} grep the boot marker"


def g_verifier_in_ci(root: Path):
    release = read(root, "release")
    workflow = read(root, "workflow")
    call = find(release, r"bash\s+\"\$VERIFIER\"")
    ci_call = find(workflow, r"build-release-image\.sh")
    if not call or not ci_call:
        return False, "verifier call in build-release-image.sh or its CI invocation is missing"
    return True, f"{P['release']}:{call[0]} -> {P['workflow']}:{ci_call[0]} invokes the verifier"


def g_schema(root: Path):
    text = read(root, "verifier")
    parts = (
        r"EXPECTED_SCHEMA=\"gb-os-appliance/1\"",
        r"actual_schema=\$\(manifest_get schema\)",
        r"die\s+\"manifest schema mismatch",
    )
    missing = [rx for rx in parts if not find(text, rx)]
    if missing:
        return False, "missing schema guard fragments: " + ", ".join(missing)
    return True, f"{P['verifier']}:{find(text, parts[0])[0]} rejects an unknown manifest schema"


def g_min_markers(root: Path):
    text = read(root, "verifier")
    parts = (
        r"MIN_MARKERS=10",
        r"marker_count=\$\(awk",
        r"\[ \"\$marker_count\" -ge \"\$MIN_MARKERS\" \]",
        r"die\s+\"manifest has too few marker/path rows",
    )
    missing = [rx for rx in parts if not find(text, rx)]
    if missing:
        return False, "missing minimum-marker fragments: " + ", ".join(missing)
    return True, f"{P['verifier']}:{find(text, parts[0])[0]} refuses a vacuously empty manifest"


def g_root_mount_fail(root: Path):
    text = read(root, "verifier")
    parts = (
        r"fail\(\)\s*\{\s*record FAIL",
        r"fail root\.mount",
        r"\[ \"\$failed\" -eq 0 \]",
    )
    missing = [rx for rx in parts if not find(text, rx)]
    if missing:
        return False, "missing root.mount fail-closed fragments: " + ", ".join(missing)
    return True, f"{P['verifier']}:{find(text, parts[1])[0]} records root.mount FAIL and aggregates it"


def g_gate_wiring(root: Path):
    workflow = read(root, "workflow")
    parts = (
        r"pull_request:",
        r"desktop/os/\*\*",
        r"botserver/crates/botsettings/\*\*",
        r"needs: laptop-acceptance-offline",
        r"bash desktop/os/tests/laptop-acceptance\.sh",
    )
    missing = [rx for rx in parts if not find(workflow, rx)]
    if missing:
        return False, "missing gate wiring fragments: " + ", ".join(missing)
    return True, f"{P['workflow']}:{find(workflow, parts[3])[0]} gates the image job on the offline harness"


GUARANTEES = (
    ("QEMU_SMOKE_GATE", g_smoke),
    ("SMOKE_BOOT_TO_SHELL_MARKER", g_boot_marker),
    ("VERIFIER_INVOKED_IN_CI", g_verifier_in_ci),
    ("VERIFIER_SCHEMA_GUARD", g_schema),
    ("VERIFIER_MIN_MARKER_ROWS", g_min_markers),
    ("VERIFIER_ROOT_MOUNT_FAIL_CLOSED", g_root_mount_fail),
    ("PR_AND_RELEASE_GATE_WIRING", g_gate_wiring),
)


# --- issue criteria: [AUTOMATED] only when a real script/test exists -------------

def criterion(root: Path, sources: list[str], rx: str, gap_detail: str,
              auto_detail: str = "scripted check found"):
    hit = find_any(root, sources, rx)
    if hit:
        return "AUTOMATED", evidence(hit) + " — " + auto_detail
    return "GAP", "no script/test found — " + gap_detail


def matrix_rows(root: Path):
    rows = []
    for line in read(root, "matrix").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or stripped.startswith("case_id"):
            continue
        rows.append([field.strip() for field in stripped.split("|")])
    return rows


def criteria(root: Path):
    sources = source_paths(root)
    workflow = [P["workflow"]]
    rows = []

    hit = find_any(root, workflow, r"Started General Bots kiosk shell")
    rows.append(("SMOKE-BOOT", "QEMU smoke asserts boot-to-kiosk-shell marker",
                 "AUTOMATED" if hit else "GAP",
                 evidence(hit) + " — text marker only; no visual desktop proof"))

    status, detail = criterion(
        root, sources,
        r"(?:launch|open)\s+(?:an?\s+)?(?:app|chat|shell)|app.{0,20}launch|TOOL_EXEC",
        "no smoke/test launches an app and asserts it opened",
        "app launch assertion found in code")
    rows.append(("SMOKE-APP", "at least one app launch assertion in the smoke", status, detail))

    status = "GAP"
    detail = "release packaging does not invoke the verifier"
    verifier_call = find_any(root, [P["release"]], r"bash\s+\"\$VERIFIER\"")
    if verifier_call:
        status, detail = "AUTOMATED", evidence(verifier_call) + " — release packaging invokes the verifier in CI"
    rows.append(("VERIFIER-CI", "appliance verifier is invoked in CI before blessing", status, detail))

    device_checks = (
        ("C1.1", "QEMU profile: fake battery", r"(?:-device|fake)[^|]{0,40}battery"),
        ("C1.2", "QEMU profile: mac80211_hwsim Wi-Fi", r"mac80211_hwsim"),
        ("C1.3", "QEMU profile: audio device", r"-device[^|]{0,40}(?:ich9|ac97|hda|sound)"),
        ("C1.4", "QEMU profile: second head", r"second head|two heads|dual.?head"),
    )
    for cid, title, rx in device_checks:
        status, detail = criterion(root, sources, rx,
                                   "mentioned only in a SKIP reason/comment, if at all",
                                   "QEMU device argument found in executable code")
        rows.append((cid, title, status, detail))

    scripted = (
        ("C1.5", "scripted: shell visible in booted image", SCREENSHOT_RX),
        ("C1.6", "scripted: chat message round-trip",
         r"chat.{0,40}round.?trip|send.{0,20}(?:message|chat).{0,40}(?:assert|expect|grep)"),
        ("C1.7", "scripted: tray indicators present/live", r"tray.{0,40}(?:visible|present|assert)"),
        ("C1.8", "scripted: power menu opens", r"power.menu.{0,40}(?:open|assert|click)"),
        ("C1.9", "scripted: lock requires the credential",
         r"lock.{0,40}(?:credential|password).{0,40}(?:assert|expect|test)"),
    )
    for cid, title, rx in scripted:
        status, detail = criterion(root, sources, rx,
                                   "static markers/SKIP rows exist; no booted-shell assertion",
                                   "assertion found in code (inspect whether it is booted-shell evidence)")
        rows.append((cid, title, status, detail))

    serial = find_any(root, workflow, r"vm-smoke\.log")
    screenshot = find_any(root, sources, SCREENSHOT_RX)
    rows.append(("C2", "screenshots + serial log on every OS-shell closure", "GAP",
                 f"serial log artifact {evidence(serial)}; screenshots {evidence(screenshot)}; "
                 "no issue-closure attachment automation"))

    matrix = matrix_rows(root)
    required_classes = ("intel laptop", "amd laptop", "wi-fi-only", "external-display")
    blob = " ".join(" ".join(row) for row in matrix).lower()
    covered = [name for name in required_classes if name in blob]
    if matrix and len(covered) == len(required_classes):
        rows.append(("C3a", "hardware matrix data declares the required classes", "AUTOMATED",
                     f"{P['matrix']}: {len(matrix)} cases; " + ", ".join(covered)))
    else:
        rows.append(("C3a", "hardware matrix data declares the required classes", "GAP",
                     f"missing classes: {', '.join(n for n in required_classes if n not in covered)}"))

    pending = sum(1 for row in matrix if row and row[-1].upper() == "PENDING")
    if matrix and pending == 0:
        rows.append(("C3b", "physical runs with model/firmware evidence recorded", "AUTOMATED",
                     f"{P['matrix']}: no PENDING cases"))
    else:
        rows.append(("C3b", "physical runs with model/firmware evidence recorded", "GAP",
                     f"{pending}/{len(matrix)} cases PENDING; masked models/firmware, no screenshots"))

    status, detail = criterion(root, workflow, r"pull_request:", "PR trigger absent",
                               "PR trigger declared")
    if status == "AUTOMATED":
        status, detail = criterion(root, workflow, r"desktop/os/\*\*", "desktop/os/** path filter absent",
                                   "desktop/os/** path filter present")
    if status == "AUTOMATED":
        status, detail = criterion(root, workflow, r"botsettings/\*\*", "botsettings path filter absent",
                                   "botsettings path filter present")
    rows.append(("C4a", "gate runs on PRs touching OS/shell/botsettings", status, detail))

    status, detail = criterion(root, workflow, r"needs: laptop-acceptance-offline",
                               "image job does not depend on the gate",
                               "image job depends on the offline gate")
    rows.append(("C4b", "gate blocks publishing a release ISO on failure", status, detail))

    status, detail = criterion(root, sources, INPUT_RX,
                               "only static no-getty/no-rescue markers; no input injection",
                               "input injection script found")
    rows.append(("C5", "no terminal: every scenario uses mouse+keyboard", status, detail))

    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT,
                        help="repository root to audit (default: parent of this script)")
    args = parser.parse_args()
    root = args.root.resolve()

    print(f"Laptop gate static audit (#2471)\nroot: {root}")
    ci = read(root, "ci")
    ci_jobs = [name for name, rx in (("ISO/appliance", r"generalbots-os|mkosi vm|verify-image"),
                                     ("ISO job id", r"^\s{2}\S*(?:iso|appliance)\S*:\s*$"))
               if re.search(rx, ci, re.MULTILINE)]
    print(f"note: ci.yml OS/ISO jobs found: {', '.join(ci_jobs) if ci_jobs else 'none'} "
          "(gate lives in desktop-os-iso.yml)\n")

    print("Hard guarantees (asserted; a missing one exits 1):")
    failures = []
    for name, check in GUARANTEES:
        ok, detail = check(root)
        print(f"  [{'OK  ' if ok else 'MISS'}] {name}: {detail}")
        if not ok:
            failures.append(name)

    print("\nIssue criteria ([AUTOMATED] = scripted check exists, [GAP] = honest omission):")
    rows = criteria(root)
    automated = 0
    gaps = 0
    for cid, title, status, detail in rows:
        if status == "AUTOMATED":
            automated += 1
        else:
            gaps += 1
        print(f"  [{status:9}] {cid:10} {title} — {detail}")

    print(f"\nSummary: guarantees={len(GUARANTEES) - len(failures)}/{len(GUARANTEES)} "
          f"criteria AUTOMATED={automated} GAP={gaps} (total={len(rows)})")
    if failures:
        print("verdict: FAIL — missing guarantee(s): " + ", ".join(failures))
        return 1
    print("verdict: PASS (static) — GAP lines are NOT failures; real laptop UAT is hardware-blocked, "
          "issue stays OPEN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
