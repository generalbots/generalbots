#!/usr/bin/env python3
"""Static audit of the #2440 appliance verifier and its CI wiring — WHAT IS REALLY CHECKED.

Issue #2440 is about one thing: the artifact published as ``generalbots-os.iso``
must be the real mkosi kiosk appliance (boots into generalbots-shell.service),
not the GRUB payload carrier. This script audits the *offline* guarantees that
make that claim checkable, and deliberately keeps them separate from the
runtime acceptance criteria that this environment cannot exercise.

Two classes of statement are printed:

  * Static guarantees (schema guard, non-vacuous marker floor, root.mount
    fail-closed routing, verifier invocation in the release builder and in CI,
    evidence surfaced in the CI log). Each is asserted with an exact source
    fragment AND, where relevant, with the manifest data it depends on. A
    missing guarantee is an explicit [GAP] and exits 1 — no fabricated PASS.

  * The issue's real acceptance criteria. A row is [AUTOMATED] only when the
    tree contains a script/config that genuinely exercises it (a CI smoke, a
    guard, a data contract). It is [GAP] when the only remaining trace is a
    real image, a privileged mkosi build, guestfish/xorriso inspection, a boot
    or an observed release run. A [GAP] here NEVER fails this audit.

Exit 0 means "the declared static guarantees hold", NOT "issue #2440 is
satisfied". Building the image with mkosi, mounting it read-only with
guestfish, inspecting El Torito with xorriso, booting it and confirming the
published release asset are hardware/tool-blocked in this offline pass; the
issue stays OPEN.

Pure stdlib, read-only, no network, no cargo, no browser, no commits, no
guestfish/VM operations.
Usage: python3 scripts/check_os_verifier_audit.py [--root DIR]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

P = {
    "verifier": "desktop/os/release/verify-image.sh",
    "manifest": "desktop/os/release/appliance-manifest.tsv",
    "release": "desktop/os/release/build-release-image.sh",
    "workflow": ".github/workflows/desktop-os-iso.yml",
    "release_wf": ".github/workflows/release.yml",
    "build_iso": "scripts/build_iso.sh",
}

EXPECTED_SCHEMA = "gb-os-appliance/1"
MIN_MARKERS = 10
MARKER_KEYS = ("identity.marker", "unit.marker", "boot.marker", "required.path")


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
    """Comments, SKIP reasons and TSV data rows are not executable guarantees."""
    stripped = line.strip()
    if not stripped or stripped.startswith(("#", "//")):
        return False
    if "SKIP" in stripped.upper():
        return False
    if re.match(r"^[a-z][a-z0-9_.-]*\t", stripped):
        return False
    return True


def find(text: str, pattern: str, flags: int = re.IGNORECASE):
    """First non-comment line matching pattern, as (line_number, trimmed_line)."""
    rx = re.compile(pattern, flags)
    for number, line in enumerate(text.splitlines(), start=1):
        if is_code(line) and rx.search(line):
            return number, line.strip()[:115]
    return None


def find_skip(text: str, pattern: str):
    """Find a recorded SKIP call (is_code deliberately hides 'SKIP' lines)."""
    rx = re.compile(pattern, re.IGNORECASE)
    for number, line in enumerate(text.splitlines(), start=1):
        stripped = line.strip()
        if stripped.startswith(("skip ", "skip(")) and rx.search(stripped):
            return number, stripped[:115]
    return None


def find_any(root: Path, paths: list[str], pattern: str):
    for rel in paths:
        hit = find(read_rel(root, rel), pattern)
        if hit:
            return rel, hit[0], hit[1]
    return None


def evidence(hit) -> str:
    if not hit:
        return "none"
    if len(hit) == 3:
        return f"{hit[0]}:{hit[1]} ({hit[2]})"
    return f":{hit[0]} ({hit[1]})"


def evidence_uploaded(workflow: str) -> bool:
    """True when the verifier evidence manifest is a CI artifact upload path."""
    return "upload-artifact" in workflow and "dist/generalbots-os.manifest.txt" in workflow


def missing_fragments(text: str, parts) -> list[str]:
    return [rx for rx in parts if not find(text, rx)]


def manifest_rows(root: Path) -> list[tuple[str, str]]:
    """Parse the two-column TSV contract, skipping comments and blank lines."""
    rows: list[tuple[str, str]] = []
    for line in read(root, "manifest").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        fields = stripped.split("\t")
        if len(fields) == 2:
            rows.append((fields[0].strip(), fields[1].strip()))
    return rows


# --- static guarantees: absence is a REAL failure --------------------------------

def g_schema_guard(root: Path):
    """The verifier rejects an unknown or missing manifest schema before any work."""
    text = read(root, "verifier")
    parts = (
        r'EXPECTED_SCHEMA="gb-os-appliance/1"',
        r"actual_schema=\$\(manifest_get schema\)",
        r'die "manifest schema mismatch',
        r"actual_schema:-<missing>",
    )
    missing = missing_fragments(text, parts)
    declared = [value for key, value in manifest_rows(root) if key == "schema"]
    if declared != [EXPECTED_SCHEMA]:
        missing.append(f"manifest schema={declared!r}, expected ['{EXPECTED_SCHEMA}']")
    if missing:
        return False, f"[GAP] schema guard absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[0])
    return True, (f"{P['verifier']}:{hit[0]} requires schema '{EXPECTED_SCHEMA}'; "
                  f"{P['manifest']} declares it; mismatch/missing dies before image work")


def g_min_markers(root: Path):
    """A truncated/empty manifest must not pass the marker loops vacuously."""
    text = read(root, "verifier")
    parts = (
        r"MIN_MARKERS=10",
        r"marker_count=\$\(awk",
        r'\[ "\$marker_count" -ge "\$MIN_MARKERS" \]',
        r'die "manifest has too few marker/path rows',
    )
    missing = missing_fragments(text, parts)
    count = sum(1 for key, _value in manifest_rows(root) if key in MARKER_KEYS)
    if count < MIN_MARKERS:
        missing.append(f"manifest carries only {count} marker/path rows (< {MIN_MARKERS})")
    if missing:
        return False, f"[GAP] non-vacuity guard absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[0])
    return True, (f"{P['verifier']}:{hit[0]} enforces >= {MIN_MARKERS} rows; "
                  f"{P['manifest']} carries {count} (identity/unit/boot/required.path)")


def g_root_mount_fail(root: Path):
    """A bad root device records root.mount FAIL and reaches the aggregation gate."""
    text = read(root, "verifier")
    parts = (
        r"fail\(\)\s*\{\s*record FAIL",
        r"fail root\.mount",
        r'\[ "\$failed" -eq 0 \]',
        r'die "image is NOT a verified General Bots OS appliance',
    )
    skip_parts = (
        r"skip required\.paths .*root\.mount FAIL",
        r"skip identity\.markers .*root\.mount FAIL",
    )
    missing = missing_fragments(text, parts)
    missing += [rx for rx in skip_parts if not find_skip(text, rx)]
    early_die = find(text, r"die[^\n]*root\.mount")
    if early_die:
        missing.append("root.mount routed through an early die instead of fail()")
    if missing:
        return False, f"[GAP] root.mount fail-closed wiring absent: {', '.join(missing)} — no fake PASS"
    fail_hit = find(text, parts[1])
    gate_hit = find(text, parts[3])
    return True, (f"{P['verifier']}:{fail_hit[0]} records root.mount FAIL, dependents SKIP; "
                  f":{gate_hit[0]} fails the run with failed > 0")


def g_enabled_unit_check(root: Path):
    """The kiosk shell unit must be ENABLED for boot, from manifest data only.

    Presence of a unit file is not enough: systemd starts a unit only when a
    symlink in a target's .wants/ directory enables it. The check must read
    that link from the manifest, assert it is a symlink to the unit, and SKIP
    as a root-dependent group when the root filesystem cannot be mounted.
    """
    text = read(root, "verifier")
    parts = (
        r'enabled_path=\$\(manifest_get unit\.enabled_path\)',
        r'gf_root islink',
        r'gf_root readlink',
        r'pass unit\.enabled',
        r'fail unit\.enabled',
    )
    skip_parts = (r"skip unit\.enabled .*root\.mount FAIL",)
    missing = missing_fragments(text, parts)
    missing += [rx for rx in skip_parts if not find_skip(text, rx)]
    declared = [value for key, value in manifest_rows(root) if key == "unit.enabled_path"]
    if len(declared) != 1 or not declared[0]:
        missing.append(f"manifest unit.enabled_path={declared!r}, expected exactly one non-empty value")
    if missing:
        return False, f"[GAP] enabled-unit check absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[3])
    return True, (f"{P['verifier']}:{hit[0]} asserts the enablement symlink "
                  f"{declared[0]} read from the manifest; SKIP when root is unmountable")


def g_verifier_in_release_build(root: Path):
    """The release builder verifies the image before giving it the final name."""
    text = read(root, "release")
    parts = (
        r'VERIFIER="\$SCRIPT_DIR/verify-image\.sh"',
        r'\[ -f "\$VERIFIER" \] \|\| die "image verifier not found',
        r'bash "\$VERIFIER" --image "\$IMAGE" --manifest "\$MANIFEST"',
        r'--evidence "\$stage_dir/evidence\.tsv"',
        r"check\.%s=%s",
        r"verification=PASS",
    )
    missing = missing_fragments(text, parts)
    if missing:
        return False, f"[GAP] release-builder verifier call absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[2])
    return True, (f"{P['release']}:{hit[0]} runs the verifier with --evidence before "
                  "the final name is created; failures abort publication")


def g_verifier_in_ci(root: Path):
    """The release-image job really invokes that builder with the pinned manifest."""
    wf = read(root, "workflow")
    parts = (
        r"for f in build-release-image\.sh verify-image\.sh appliance-manifest\.tsv; do",
        r"bash /release/build-release-image\.sh --image /dist/generalbots-os\.iso",
        r"--manifest /release/appliance-manifest\.tsv",
        r"--out-dir /dist/verified",
        r"actions/upload-artifact@",
    )
    missing = missing_fragments(wf, parts)
    if missing:
        return False, f"[GAP] CI wiring absent: {', '.join(missing)} — no fake PASS"
    hit = find(wf, parts[1])
    return True, (f"{P['workflow']}:{hit[0]} calls build-release-image.sh with the "
                  "pinned manifest; the image job uploads its artifacts")


def g_evidence_surfaced_in_ci(root: Path):
    """The verifier's recorded checks are emitted into the CI log and dist/.

    The evidence manifest is printed with ``cat`` into the CI log, copied to
    dist/ next to the published image, AND uploaded as a dedicated CI
    artifact (EVID-UP is therefore AUTOMATED).
    """
    wf = read(root, "workflow")
    rel = read(root, "release")
    wf_parts = (
        r"cat /dist/verified/generalbots-os\.manifest\.txt",
        r"for f in generalbots-os\.iso SHA256SUMS-os generalbots-os\.manifest\.txt; do",
        r'cp -f "dist/verified/\$f" "dist/\$f"',
    )
    rel_parts = (
        r"check\.%s=%s",
        r"while IFS=.*read -r name status detail",
    )
    missing = [f"workflow:{rx}" for rx in missing_fragments(wf, wf_parts)]
    missing += [f"release:{rx}" for rx in missing_fragments(rel, rel_parts)]
    if missing:
        return False, f"[GAP] evidence emission absent: {', '.join(missing)} — no fake PASS"
    hit = find(wf, wf_parts[0])
    uploaded = "dist/generalbots-os.manifest.txt" in wf and "upload-artifact" in wf
    return uploaded, (f"{P['workflow']}:{hit[0]} prints the evidence manifest to the CI "
                      "log, copies it to dist/ and uploads it as a CI artifact")


GUARANTEES = (
    ("SCHEMA_GUARD", g_schema_guard),
    ("MIN_MARKER_ROWS", g_min_markers),
    ("ROOT_MOUNT_FAIL_CLOSED", g_root_mount_fail),
    ("ENABLED_UNIT_CHECK", g_enabled_unit_check),
    ("VERIFIER_IN_RELEASE_BUILD", g_verifier_in_release_build),
    ("VERIFIER_IN_CI", g_verifier_in_ci),
    ("EVIDENCE_SURFACED_IN_CI", g_evidence_surfaced_in_ci),
)


# --- issue criteria: [AUTOMATED] only for real script/config evidence -------------

def criteria(root: Path):
    wf = read(root, "workflow")
    rel = read(root, "release")
    verifier = read(root, "verifier")
    build_iso = read(root, "build_iso")
    rows = []

    hit = find(rel, r'cp -- "\$IMAGE" "\$stage_dir/generalbots-os\.iso"')
    tag = find(wf, r"v\[0-9\]\+\\\.\[0-9\]")
    rows.append(("AC1a", "one final-name image, blessed only after verification",
                 "AUTOMATED" if hit else "GAP",
                 evidence(hit) + f" — static release-builder wiring; pin gate at {evidence(tag)}"))

    hit = find(wf, r"Started General Bots kiosk shell")
    rows.append(("AC1b", "QEMU smoke asserts the kiosk-shell boot marker (scripted)",
                 "AUTOMATED" if hit else "GAP",
                 evidence(hit) + " — text marker in vm-smoke.log, not a visual shell proof"))

    rows.append(("AC1c", "published ISO observed to boot into generalbots-shell.service", "GAP",
                 "no real image was built/booted offline; marker assertion is not runtime proof"))

    pin = find(wf, r"a pinned vMAJOR\.MINOR\.PATCH tag is required")
    sums = find(wf, r"sha256sum -c -")
    rows.append(("AC2a", "pinned tag + same-pin checksum gate (static CI wiring)",
                 "AUTOMATED" if pin and sums else "GAP",
                 f"pin at {evidence(pin)}, checksum at {evidence(sums)}"))

    rows.append(("AC2b", "observed passing CI run attaching the OS asset to a release", "GAP",
                 "workflow/release wiring exists; no passing run or release asset observed here"))

    carrier = find(build_iso, r"generalbots-payload\.iso")
    readme = find(build_iso, r"payload carrier, NOT a bootable OS")
    rows.append(("AC3", "carrier renamed and its README stops calling itself an OS",
                 "AUTOMATED" if carrier and readme else "GAP",
                 f"carrier name at {evidence(carrier)}, README note at {evidence(readme)} — static"))

    parts = (
        r"manifest_all identity\.marker",
        r"manifest_all unit\.marker",
        r"manifest_all required\.path",
        r"gf_root cat \"\$identity_file\"",
        r"gf_root exists \"\$path\"",
    )
    missing = missing_fragments(verifier, parts)
    rows.append(("AC4a", "in-image identity / shell unit / path assertions are wired",
                 "GAP" if missing else "AUTOMATED",
                 f"{P['verifier']}: missing {', '.join(missing)}" if missing
                 else f"{P['verifier']}: guestfish reads os-release + unit and probes required paths"))

    rows.append(("AC4b", "guestfish inspection of a REAL mkosi image", "GAP",
                 "guestfish not installed here and no real image exists; stub/fixture runs only"))

    required_smoke = find(wf, r"REQUIRED boot smoke")
    no_coe = find(wf, r"continue-on-error")
    rows.append(("AC5a", "QEMU boot smoke is release-blocking, not best-effort",
                 "AUTOMATED" if required_smoke and not no_coe else "GAP",
                 evidence(required_smoke) + ("; no continue-on-error in workflow"
                                             if not no_coe else f"; continue-on-error at {evidence(no_coe)}")))

    rows.append(("R1", "build the real image with mkosi (privileged container)", "GAP",
                 "no mkosi build executed in this offline slice — hardware/tool-blocked"))
    rows.append(("R2", "xorriso El Torito BIOS/UEFI inspection of the built hybrid ISO", "GAP",
                 "xorriso is optional and absent; real-image boot records never inspected"))
    rows.append(("R3", "boot the built image (QEMU/real laptop) and observe the shell", "GAP",
                 "no booted image, screenshots or device profile in this pass"))
    rows.append(("EVID-UP", "verifier evidence uploaded as a dedicated CI artifact/release asset",
                 "AUTOMATED" if evidence_uploaded(wf) else "GAP",
                 f"{P['workflow']}: Upload image artifacts includes dist/generalbots-os.manifest.txt"))
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT,
                        help="repository root to audit (default: parent of this script)")
    args = parser.parse_args()
    root = args.root.resolve()

    print(f"OS appliance verifier static audit (#2440)\nroot: {root}")
    missing_files = [rel for rel in P.values() if not (root / rel).is_file()]
    if missing_files:
        print("note: absent files: " + ", ".join(missing_files))

    print("\nStatic guarantees (asserted; a missing one is an explicit GAP and exits 1):")
    failures = []
    for name, check in GUARANTEES:
        ok, detail = check(root)
        print(f"  [{'OK  ' if ok else 'MISS'}] {name}: {detail}")
        if not ok:
            failures.append(name)

    print("\nIssue criteria ([AUTOMATED] = real script/config evidence, [GAP] = honest omission):")
    rows = criteria(root)
    automated = sum(1 for row in rows if row[2] == "AUTOMATED")
    gaps = len(rows) - automated
    for cid, title, status, detail in rows:
        print(f"  [{status:9}] {cid:6} {title} — {detail}")

    print(f"\nSummary: guarantees={len(GUARANTEES) - len(failures)}/{len(GUARANTEES)} "
          f"criteria AUTOMATED={automated} GAP={gaps} (total={len(rows)})")
    if failures:
        print("verdict: FAIL — missing static guarantee(s): " + ", ".join(failures))
        return 1
    print("verdict: PASS (static) — criteria GAP lines are NOT failures; mkosi build, "
          "guestfish/xorriso inspection and boot UAT are blocked, issue stays OPEN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
