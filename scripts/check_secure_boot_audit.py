#!/usr/bin/env python3
"""Static audit of Secure Boot / signed-UKI readiness (#2478) — WHAT IS REALLY CHECKED.

Two different things are easy to conflate, so they are kept separate:

  * static guarantees that exist in the tree today (a signed UKI recipe and
    signing-key injection, a fail-loud release / bootable dev split, an MOK
    runbook, a fail-closed appliance verifier, a wired OVMF smoke, an optional
    measured-boot profile, a sanitized PCR-capture script and a shippable
    Secure-Boot fallback artifact). Each is asserted with its exact config
    line; a missing one is printed as an explicit [GAP] (never a fake PASS)
    and fails the audit.
  * the issue's real acceptance criteria. Each is printed as [AUTOMATED] only
    when a script/config in the tree really exercises it, and as [GAP] when the
    only remaining trace is hardware, firmware or an unobserved CI run.

A [GAP] in the criteria table never fails this script. Exit 0 means "the declared
static guarantees hold", NOT "Secure Boot acceptance is satisfied". Enrolling
keys in firmware, booting a signed image with Secure Boot ON on real hardware,
and observing an OVMF run are hardware/runtime-blocked and keep the issue OPEN.

Pure stdlib, read-only, no network, no cargo, no browser, no commits, no keys.
Usage: python3 scripts/check_secure_boot_audit.py [--root DIR]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

P = {
    "mkosi": "desktop/os/mkosi.conf",
    "sb_conf": "desktop/os/mkosi.conf.d/20-secure-boot.conf",
    "release_profile": "desktop/os/mkosi.profiles/secure-boot-release.conf",
    "measured_profile": "desktop/os/mkosi.profiles/secure-boot-measured.conf",
    "build_image": "desktop/os/build-image.sh",
    "keygen": "desktop/os/generate-secure-boot-keys.sh",
    "readme": "desktop/os/README.md",
    "runbook": "desktop/os/README-secure-boot.md",
    "pcr_capture": "desktop/os/diagnostics/capture-measured-boot.sh",
    "fallback_artifact": "desktop/os/diagnostics/secure-boot-fallback.txt",
    "verifier": "desktop/os/release/verify-image.sh",
    "manifest": "desktop/os/release/appliance-manifest.tsv",
    "workflow": ".github/workflows/desktop-os-iso.yml",
}


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
    """Comments, SKIPs and TSV/PSV data rows are not executable declarations."""
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


def find_any(root: Path, paths: list[str], pattern: str):
    for rel in paths:
        hit = find(read_rel(root, rel), pattern)
        if hit:
            return rel, hit[0], hit[1]
    return None


def raw_code_lines(text: str) -> list[str]:
    """Non-blank, non-comment lines; unlike is_code() it keeps SKIP sentinels."""
    return [ln.strip() for ln in text.splitlines()
            if ln.strip() and not ln.lstrip().startswith(("#", "//"))]


def evidence(hit) -> str:
    if not hit:
        return "none"
    if len(hit) == 3:
        return f"{hit[0]}:{hit[1]} ({hit[2]})"
    return f":{hit[0]} ({hit[1]})"


def missing_fragments(text: str, parts) -> list[str]:
    return [rx for rx in parts if not find(text, rx)]


# --- hard guarantees: absence is a REAL failure (and an explicit GAP) -----------

def g_uki_and_signing_key_path(root: Path):
    """Image recipe declares a signed UKI and a signing-key injection path.

    mkosi signs internally via --secure-boot-key/--secure-boot-certificate;
    there is no literal sbverify/sbsign call to assert. The key path IS the
    signing step, so it is the branch asserted here.
    """
    profile = read(root, "release_profile")
    image = read(root, "build_image")
    parts = (
        ("signed UKI", r"^UnifiedKernelImages=signed\s*$"),
        ("signed bootloader/shim", r"^ShimBootloader=signed\s*$"),
        ("signing key path", r"--secure-boot-key="),
        ("signing certificate path", r"--secure-boot-certificate="),
    )
    hits = {}
    for name, rx in parts:
        hit = find(profile if name.startswith(("signed",)) else image, rx)
        if hit:
            hits[name] = hit
    if len(hits) != len(parts):
        absent = ", ".join(name for name, _ in parts if name not in hits)
        observed = (
            find(profile, r"UnifiedKernelImages=")
            or find(read(root, "sb_conf"), r"UnifiedKernelImages=")
            or find(read(root, "mkosi"), r"UnifiedKernelImages=")
        )
        return False, (
            f"[GAP] signed UKI/key path not declared (absent: {absent}); "
            f"recipe evidence: {evidence(observed) if observed else 'none'} — no fake PASS"
        )
    return True, (
        f"{P['release_profile']}:{hits['signed UKI'][0]} {hits['signed UKI'][1]}; "
        f":{hits['signed bootloader/shim'][0]} {hits['signed bootloader/shim'][1]}; "
        f"{P['build_image']}:{hits['signing key path'][0]} {hits['signing key path'][1]}; "
        f":{hits['signing certificate path'][0]} {hits['signing certificate path'][1]}"
    )


def g_dev_unsigned_bootable(root: Path):
    conf = read(root, "sb_conf")
    image = read(root, "build_image")
    parts = (
        ("unsigned UKI default", conf, r"^UnifiedKernelImages=unsigned\s*$"),
        ("SecureBoot=no", conf, r"^SecureBoot=no\s*$"),
        ("dev-mode default", image, r'mode="\$\{GB_OS_BUILD_MODE:-dev\}"'),
    )
    hits = {}
    for name, text, rx in parts:
        hit = find(text, rx)
        if hit:
            hits[name] = hit
    if len(hits) != len(parts):
        absent = ", ".join(name for name, _, _ in parts if name not in hits)
        return False, f"[GAP] dev unsigned/bootable default missing (absent: {absent}) — no fake PASS"
    return True, (
        f"{P['sb_conf']}:{hits['unsigned UKI default'][0]} {hits['unsigned UKI default'][1]}; "
        f":{hits['SecureBoot=no'][0]} {hits['SecureBoot=no'][1]}; "
        f"{P['build_image']}:{hits['dev-mode default'][0]} {hits['dev-mode default'][1]}"
    )


def g_release_fail_loud(root: Path):
    image = read(root, "build_image")
    parts = (
        r"RELEASE BUILD REFUSED: signing keys are missing",
        r'if \[ -z "\$key" \] \|\| \[ -z "\$cert" \]',
        r"signing key file not found",
        r"signing key is not a PEM private key",
        r"--smoke requires --mode release",
    )
    missing = missing_fragments(image, parts)
    if missing:
        return False, f"[GAP] release fail-loud guards absent: {', '.join(missing)} — no fake PASS"
    hit = find(image, parts[0])
    return True, f"{P['build_image']}:{hit[0]} refuses missing/malformed keys; smoke requires release"


def g_mok_runbook(root: Path):
    runbook = read(root, "runbook")
    keygen = read(root, "keygen")
    a = find(runbook, r"mokutil --import")
    b = find(keygen, r"mokutil --import")
    if not a or not b:
        return False, "[GAP] documented MOK enrolment path (mokutil --import) is missing — no fake PASS"
    return True, f"{P['runbook']}:{a[0]} and {P['keygen']}:{b[0]} document mokutil --import"


def g_verifier_schema(root: Path):
    text = read(root, "verifier")
    parts = (
        r'EXPECTED_SCHEMA="gb-os-appliance/1"',
        r"manifest_get schema",
        r'die "manifest schema mismatch',
    )
    missing = missing_fragments(text, parts)
    if missing:
        return False, f"[GAP] verifier schema guard absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[0])
    return True, f"{P['verifier']}:{hit[0]} rejects an unknown manifest schema"


def g_verifier_min_markers(root: Path):
    text = read(root, "verifier")
    parts = (
        r"MIN_MARKERS=10",
        r"marker_count=\$\(awk",
        r'\[ "\$marker_count" -ge "\$MIN_MARKERS" \]',
        r'die "manifest has too few marker/path rows',
    )
    missing = missing_fragments(text, parts)
    if missing:
        return False, f"[GAP] verifier empty-manifest guard absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[0])
    return True, f"{P['verifier']}:{hit[0]} refuses a vacuously empty manifest"


def g_verifier_boot_markers(root: Path):
    text = read(root, "verifier")
    parts = (
        r"boot\.partitions_min",
        r"pass boot\.esp|fail boot\.esp",
        r"manifest_all boot\.marker",
        r'grep -qiF -- "\$marker"',
    )
    missing = missing_fragments(text, parts)
    if missing:
        return False, f"[GAP] verifier boot-marker checks absent: {', '.join(missing)} — no fake PASS"
    hit = find(text, parts[2])
    return True, f"{P['verifier']}:{hit[0]} checks GPT/boot.esp and El Torito boot.marker rows"


def g_smoke_ci_wired(root: Path):
    wf = read(root, "workflow")
    image = read(root, "build_image")
    wf_parts = (
        r"build-image\.sh",
        r"--mode release",
        r"--smoke",
        r"generalbots-os-secure-boot",
    )
    img_parts = (
        r"--firmware=uefi-secure-boot",
        r"--firmware-variables=microsoft-mok",
        r"Started General Bots kiosk shell",
    )
    missing = [f"workflow:{rx}" for rx in missing_fragments(wf, wf_parts)]
    missing += [f"build-image:{rx}" for rx in missing_fragments(image, img_parts)]
    if missing:
        return False, f"[GAP] OVMF SB smoke wiring absent: {', '.join(missing)} — no fake PASS"
    hit = find(wf, wf_parts[3])
    return True, f"{P['workflow']}:{hit[0]} uploads SB smoke evidence; wrapper uses uefi-secure-boot/microsoft-mok"


def g_measured_boot_profile(root: Path):
    profile = read(root, "measured_profile")
    parts = (r"SignExpectedPcr=yes", r"SplitArtifacts=pcrs")
    missing = missing_fragments(profile, parts)
    if missing:
        return False, f"[GAP] optional measured-boot profile absent: {', '.join(missing)} — no fake PASS"
    hit = find(profile, parts[0])
    return True, f"{P['measured_profile']}:{hit[0]} declares signed PCRs and a pcrs artifact"


def g_pcr_capture_artifact(root: Path):
    """Observed PCR capture is sanitized, explicit on skip and never dumps raw data."""
    lines = raw_code_lines(read(root, "pcr_capture"))
    if not lines:
        return False, "[GAP] PCR-capture script absent or empty — no fake PASS"
    parts = (
        ("tpm2 source", r"command -v tpm2_pcrread"), ("kernel event-log fallback", r"binary_bios_measurements"),
        ("sha256 sanitisation", r"hashlib\.sha256\(\)"), ("explicit skip status", r'"status": "skip"'),
        ("--out option", r"--out\)"), ("--check option", r"--check\)"),
        ("sanitized JSON schema", r"gb-measured-boot/1"),
    )
    absent = [name for name, rx in parts if not any(re.search(rx, ln) for ln in lines)]
    banned = (
        ("binary log dumped with cat", r"\bcat\s+.*binary_bios_measurements"),
        ("binary log dumped with od/xxd", r"\b(od|xxd)\b"),
        ("python prints raw data", r"\bprint\([^)]*\b(data|event|raw|digest)\b"),
    )
    leaks = [name for name, rx in banned if any(re.search(rx, ln, re.I) for ln in lines)]
    if absent or leaks:
        detail = [f"absent: {', '.join(absent)}"] if absent else []
        detail += [f"unsafe raw-data output: {', '.join(leaks)}"] if leaks else []
        return False, f"[GAP] PCR-capture guarantees failed ({'; '.join(detail)}) — no fake PASS"
    return True, (f"{P['pcr_capture']}: tpm2_pcrread + bounded binary_bios_measurements fallback, "
                  "sha256-only JSON, explicit skip status, --out/--check; raw data never dumped")


def g_fallback_artifact(root: Path):
    """A shippable fallback text: disable-SB guidance, vendor keys, MOK pointer."""
    lines = raw_code_lines(read(root, "fallback_artifact"))
    parts = (
        ("disable-SB guidance", r"disable Secure Boot"), ("Dell setup key", r"Dell.*F2"),
        ("Lenovo setup key", r"Lenovo.*F1"), ("ASUS setup key", r"ASUS.*(F2|Del)"),
        ("MOK enrolment pointer", r"mokutil --import"), ("runbook pointer", r"README-secure-boot\.md"),
    )
    absent = [name for name, rx in parts if not any(re.search(rx, ln, re.I) for ln in lines)]
    if absent:
        return False, f"[GAP] fallback artifact missing ({', '.join(absent)}) — no fake PASS"
    return True, (f"{P['fallback_artifact']}: disable-SB guidance + vendor key table + MOK/runbook pointer")


GUARANTEES = (
    ("UKI_AND_SIGNING_KEY_PATH", g_uki_and_signing_key_path),
    ("DEV_UNSIGNED_BOOTABLE", g_dev_unsigned_bootable),
    ("RELEASE_FAIL_LOUD_MISSING_KEYS", g_release_fail_loud),
    ("MOK_ENROLMENT_RUNBOOK", g_mok_runbook),
    ("VERIFIER_SCHEMA_GUARD", g_verifier_schema),
    ("VERIFIER_MIN_MARKER_ROWS", g_verifier_min_markers),
    ("VERIFIER_BOOT_MARKERS", g_verifier_boot_markers),
    ("OVMF_SB_SMOKE_WIRED_IN_CI", g_smoke_ci_wired),
    ("MEASURED_BOOT_PROFILE", g_measured_boot_profile),
    ("PCR_CAPTURE_SANITIZED_ARTIFACT", g_pcr_capture_artifact),
    ("FALLBACK_RELEASE_ARTIFACT", g_fallback_artifact),
)


# --- issue criteria: [AUTOMATED] only for real script/config evidence ------------

def criteria(root: Path):
    rows = []
    image = read(root, "build_image")
    conf = read(root, "sb_conf")
    readme = read(root, "readme")
    profile = read(root, "measured_profile")
    wf = read(root, "workflow")

    hit = find_any(root, [P["release_profile"], P["build_image"]],
                   r"UnifiedKernelImages=signed|ShimBootloader=signed|--secure-boot-key=")
    rows.append(("AC1a", "signed UKI + bootloader, checked-in policy",
                 "AUTOMATED" if hit else "GAP", evidence(hit) + " — static recipe/policy"))

    hit = find_any(root, [P["runbook"], P["keygen"]], r"mokutil --import")
    rows.append(("AC1b", "documented MOK enrolment path (unsigned-dev case)",
                 "AUTOMATED" if hit else "GAP", evidence(hit) + " — runbook text, not an enrolment"))

    rows.append(("AC1c", "MOK key enrolled in firmware/MokList on target", "GAP",
                 "needs mokutil --import + reboot on hardware; no offline evidence"))

    hit = find(image, r"RELEASE BUILD REFUSED")
    rows.append(("AC2a", "release build fails loudly on missing keys",
                 "AUTOMATED" if hit else "GAP", evidence(hit) + " — static guard"))

    hit = find(conf, r"^UnifiedKernelImages=unsigned\s*$")
    rows.append(("AC2b", "development keeps a bootable unsigned UKI (declared)",
                 "AUTOMATED" if hit else "GAP", evidence(hit) + " — declaration only"))

    rows.append(("AC2c", "development image observed to boot successfully", "GAP",
                 "no offline mkosi vm boot observed; declaration is not runtime proof"))

    doc = (
        find(readme, r"documented fallback is"),
        find(readme, r"set it to \*\*Disabled\*\*"),
        find(readme, r"Dell.*F2|Lenovo.*F1|ASUS.*Del"),
    )
    if all(doc):
        rows.append(("AC3a", "documented fallback + firmware-specific key names",
                     "AUTOMATED", f"{P['readme']}:{doc[0][0]} (+ vendor key table) — static docs"))
    else:
        rows.append(("AC3a", "documented fallback + firmware-specific key names",
                     "GAP", f"{P['readme']}: fallback/vendor table incomplete"))

    fb_ok, fb_detail = g_fallback_artifact(root)
    rows.append(("AC3b", "fallback in release-note artifacts + recovery screen (#2468)",
                 "AUTOMATED" if fb_ok else "GAP",
                 fb_detail + " — shippable static artifact; on-screen rendering needs runtime UAT"))

    hit = find(wf, r"--smoke")
    rows.append(("AC4a", "QEMU/OVMF SB smoke with MS keys wired in CI",
                 "AUTOMATED" if hit else "GAP", evidence(hit) + " — scripted CI step"))

    rows.append(("AC4b", "observed passing OVMF boot, SB ON + MS/MOK keys enrolled", "GAP",
                 "CI step is written but no passing run observed offline"))

    rows.append(("AC4c", "real laptop boots with Secure Boot ON", "GAP",
                 "hardware-blocked; no physical machine in this pass"))

    hit = find(profile, r"SignExpectedPcr=yes")
    rows.append(("AC5a", "optional measured-boot profile declares signed PCRs",
                 "AUTOMATED" if hit else "GAP", evidence(hit) + " — static config"))

    pcr_ok, pcr_detail = g_pcr_capture_artifact(root)
    rows.append(("AC5b", "TPM PCR values recorded into diagnostics bundle",
                 "AUTOMATED" if pcr_ok else "GAP",
                 pcr_detail + " — capture path + measured profile; real values need a TPM"))

    rows.append(("AC5c", "real PCR values observed on a TPM-equipped machine", "GAP",
                 "no TPM/tools in this environment; capture script emits an explicit skip, never fabricated values"))
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT,
                        help="repository root to audit (default: parent of this script)")
    args = parser.parse_args()
    root = args.root.resolve()

    print(f"Secure Boot / signed UKI static audit (#2478)\nroot: {root}")
    print("note: no literal sbverify/sbsign call exists; mkosi signs via "
          "--secure-boot-key/--secure-boot-certificate (the asserted key path).\n")

    print("Static guarantees (asserted; a missing one is an explicit GAP and exits 1):")
    failures = []
    for name, check in GUARANTEES:
        ok, detail = check(root)
        print(f"  [{'OK  ' if ok else 'MISS'}] {name}: {detail}")
        if not ok:
            failures.append(name)

    print("\nIssue criteria ([AUTOMATED] = real script/config, [GAP] = honest omission):")
    rows = criteria(root)
    automated = sum(1 for *_x, status, _d in rows if status == "AUTOMATED")
    gaps = len(rows) - automated
    for cid, title, status, detail in rows:
        print(f"  [{status:9}] {cid:5} {title} — {detail}")

    print(f"\nSummary: guarantees={len(GUARANTEES) - len(failures)}/{len(GUARANTEES)} "
          f"criteria AUTOMATED={automated} GAP={gaps} (total={len(rows)})")
    if failures:
        print("verdict: FAIL — missing declared static guarantee(s): " + ", ".join(failures))
        return 1
    print("verdict: PASS (static) — criteria GAP lines are NOT failures; firmware enrolment, "
          "OVMF run and real-hardware Secure Boot UAT are blocked, issue stays OPEN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
