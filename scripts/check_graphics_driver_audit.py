#!/usr/bin/env python3
"""Static audit of the #2481 OS graphics / VA-API driver support.

WHAT THIS IS: a read-only audit of what the tree actually guarantees about
graphics drivers in the OS image. It never boots an image, probes a GPU, runs
`vainfo`, starts a browser or calls cargo. Two kinds of findings are printed:

  * hard guarantees — static facts in the image recipe/detector. Each is
    asserted; if any is missing the audit exits 1 and names it.
  * issue criteria — each is labelled [AUTOMATED] only when a real script/test
    in the tree exercises it, otherwise [GAP] with the reason. A [GAP] never
    fails this audit: exit 0 means "the static guarantees are present", NOT
    "hardware acceptance is satisfied".

The real GPU/VA-API runtime UAT (vainfo profiles, 1080p H.264 CPU threshold,
non-llvmpipe WebKit compositing, booted-image environment, battery drain) is
hardware-blocked and stays OPEN. This audit deliberately does not claim it.

Note on scope: the CI graphics manifest guard (`scripts/check_graphics_support.py`)
exists and is self-testable, but grepping `.github/workflows/ci.yml` (read-only)
shows it is NOT currently wired into CI. That criterion is therefore reported
as an honest [GAP], not assumed to run.

Usage: python3 scripts/check_graphics_driver_audit.py [--root DIR]
Exit: 0 = all hard guarantees present (GAP lines may exist), 1 = guarantee missing.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

P = {
    "manifest": "desktop/os/mkosi.conf",
    "dropin": "desktop/os/mkosi.conf.d/45-graphics.conf",
    "detector": "desktop/os/graphics/extra/usr/local/libexec/gb-graphics/detect-graphics.sh",
    "profiles": "desktop/os/graphics/extra/usr/local/libexec/gb-graphics/gpu-driver-profiles.psv",
    "readme": "desktop/os/graphics/README.md",
    "ci": ".github/workflows/ci.yml",
    "guard": "scripts/check_graphics_support.py",
    "diag_html": "botui/ui/suite/settings/partials/15_os_appliance.html",
    "diag_report": "desktop/os/tests/step_99_report.sh",
}

PKG_RE = re.compile(r"^[a-z0-9][a-z0-9+.-]*$")

# Sources where an executable runtime check could live (comments are dropped).
# The static manifest guard itself is deliberately excluded: its docstring talks
# about vainfo/battery/compositing but it does not execute those checks.
CHECK_SOURCES = (
    ".github/workflows/ci.yml",
    ".github/workflows/desktop-os-iso.yml",
    "scripts/build_iso.sh",
    "scripts/build_vm_image.sh",
)


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


def code_lines(text: str) -> str:
    """Drop comment lines so a mention in a comment is never proof of a check."""
    return "\n".join(line for line in text.splitlines()
                     if line.strip() and not line.lstrip().startswith("#"))


def line_of(text: str, pattern: str) -> int:
    """1-based line number of the first code line matching pattern, or 0."""
    rx = re.compile(pattern)
    for number, line in enumerate(text.splitlines(), start=1):
        stripped = line.strip()
        if stripped and not stripped.startswith("#") and rx.search(line):
            return number
    return 0


def parse_packages(text: str) -> set[str]:
    """Collect package tokens from mkosi ``Packages=`` blocks only."""
    found: set[str] = set()
    in_block = False
    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("["):
            in_block = False
            continue
        if raw[:1].isspace():
            if in_block:
                for word in stripped.split("#", 1)[0].rstrip("\\").split():
                    if PKG_RE.match(word):
                        found.add(word)
            continue
        if "=" not in stripped:
            in_block = False
            continue
        key, _, value = stripped.partition("=")
        in_block = key.strip() == "Packages"
        if in_block:
            for word in value.split("#", 1)[0].rstrip("\\").split():
                if PKG_RE.match(word):
                    found.add(word)
    return found


def source_hits(root: Path, pattern: str, paths: list[str] | None = None) -> list[str]:
    """Paths of executable-looking sources with a real (non-comment) match.

    ``paths=None`` uses the CI workflow / build scripts / OS test scripts.
    An explicit list restricts the search to the named sources only.
    """
    if paths is None:
        paths = list(CHECK_SOURCES) + sorted(
            str(p.relative_to(root)) for p in (root / "desktop" / "os" / "tests").glob("*.sh")
        )
    rx = re.compile(pattern)
    return [rel for rel in paths if rx.search(code_lines(read_rel(root, rel)))]


# --- hard guarantees: a missing one is a real failure -------------------------

def g_image_packages(root: Path) -> tuple[bool, str]:
    pkg = parse_packages(read(root, "manifest"))
    need = {
        "libgl1-mesa-dri", "libegl-mesa0", "libgbm1",  # Mesa GL/EGL/GBM
        "libva2", "mesa-va-drivers", "vainfo",          # VA-API userspace/tools
        "libgbm1", "libwebkit2gtk-4.1-0",               # WebKit GBM client
    }
    missing = sorted(need - pkg)
    if missing:
        return False, "mkosi.conf Packages= missing: " + ", ".join(missing)
    vendor = {"intel-media-va-driver", "i965-va-driver"} & pkg
    if not vendor:
        return False, "mkosi.conf Packages= missing VA vendor driver (intel-media/i965)"
    return True, f"{P['manifest']}: Mesa/GBM, VA-API, VA vendor and WebKitGTK packages present"


def g_swrast_gbm(root: Path) -> tuple[bool, str]:
    """The 47b3768d9 fix: software profiles must not force GBM_BACKEND=drm."""
    text = read(root, "detector")
    sw = line_of(text, r"GBM_BACKEND=swrast")
    drm = line_of(text, r"GBM_BACKEND=drm")
    branch = max((line_of(text, r'P_SW" = "1"'), line_of(text, r"P_SW == 1")))
    if not sw or not drm or not branch:
        return False, "detect-graphics.sh lacks swrast/GBM software branch or drm hardware branch"
    lines = text.splitlines()
    if not (branch < sw < drm):
        return False, "GBM_BACKEND=swrast is not gated by the software-profile branch"
    if not any("else" in line.strip() for line in lines[sw - 1:drm - 1]):
        return False, "GBM_BACKEND=drm is unconditional (swrast fix regressed)"
    if not line_of(text, r'gbm_backend="swrast"'):
        return False, "--audit planned_env no longer reports GBM_BACKEND=swrast"
    return True, f"{P['detector']}:{sw} swrast for software profiles, :{drm} drm for hardware"


def g_webkit_fallback(root: Path) -> tuple[bool, str]:
    detector = code_lines(read(root, "detector"))
    required = (
        (r"WEBKIT_DISABLE_COMPOSITING_MODE=0", "WebKit software-path override"),
        (r"LIBGL_ALWAYS_SOFTWARE=%s", "forced llvmpipe on software profiles"),
        (r"falling back to SOFTWARE rendering", "explicit no-GPU fallback log"),
    )
    missing = [label for rx, label in required if not re.search(rx, detector)]
    if missing:
        return False, "detector missing: " + ", ".join(missing)
    rows = [line for line in code_lines(read(root, "profiles")).splitlines()
            if line.count("|") == 9]
    software = [row for row in rows if row.split("|")[7] == "1"]
    if not software:
        return False, "no LIBGL_SW=1 software profile in gpu-driver-profiles.psv"
    if "graphics/extra" not in read(root, "dropin"):
        return False, "45-graphics.conf no longer ships graphics/extra into the image"
    readme = read(root, "readme")
    if "llvmpipe" not in readme or "QEMU" not in readme:
        return False, "QEMU/llvmpipe software fallback is not documented"
    return True, f"software fallback wired ({len(software)} profiles) and documented"


def g_guard_script(root: Path) -> tuple[bool, str]:
    guard = read(root, "guard")
    if not guard:
        return False, "scripts/check_graphics_support.py missing"
    for token in ("--self-test", "def check(", "MISSING"):
        if token not in guard:
            return False, f"check_graphics_support.py lost {token!r}"
    return True, "check_graphics_support.py present with --check/--self-test"


GUARANTEES = (
    ("image-packages", g_image_packages),
    ("swrast-gbm-fix", g_swrast_gbm),
    ("webkit-software-fallback", g_webkit_fallback),
    ("manifest-guard-script", g_guard_script),
)


# --- #2481 criteria: AUTOMATED only with a real check in the tree -------------

def c_mesa(root: Path) -> tuple[bool, str]:
    ok, why = g_image_packages(root)
    return ok, why


def c_vaapi_image(root: Path) -> tuple[bool, str]:
    pkg = parse_packages(read(root, "manifest"))
    need = {"libva2", "mesa-va-drivers", "vainfo", "gstreamer1.0-vaapi"}
    missing = sorted(need - pkg)
    return (not missing, P["manifest"] + ": VA-API packages present" if not missing
            else "missing: " + ", ".join(missing))


def c_swrast(root: Path) -> tuple[bool, str]:
    ok, why = g_swrast_gbm(root)
    return ok, why


def c_webkit_fallback(root: Path) -> tuple[bool, str]:
    ok, why = g_webkit_fallback(root)
    return ok, why


def c_guard(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"check_graphics_support\.py")
    ci_hit = [h for h in hits if h.endswith("ci.yml")]
    if not ci_hit:
        return False, "no check_graphics_support.py invocation in .github/workflows/ci.yml"
    return True, ", ".join(hits)


def c_vainfo(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"\bvainfo\b\s+(?:--?|\b)")
    return (bool(hits), ", ".join(hits) if hits else
            "vainfo is only a package/comment; no runtime decode-profile check")


def c_h264_cpu(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"(?:1080p|h\.264).{0,80}(?:cpu|threshold)|(?:cpu|threshold).{0,80}(?:1080p|h\.264)")
    return (bool(hits), ", ".join(hits) if hits else
            "no 1080p H.264 CPU-threshold measurement script")


def c_hw_compositing(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"glxinfo|webk(?:it)?[^\n]{0,40}composit|composit[^\n]{0,40}llvmpipe")
    return (bool(hits), ", ".join(hits) if hits else
            "no hardware-compositing probe; --audit is sysfs-only and does not run vainfo")


def c_diag_strings(root: Path) -> tuple[bool, str]:
    paths = [P["diag_html"], P["diag_report"]]
    paths += sorted(str(p.relative_to(root)) for p in
                    (root / "botui" / "ui" / "suite" / "settings").glob("**/*.js"))
    hits = source_hits(root, r"graphics_support|driver_class|gpu.?driver", paths)
    return (bool(hits), ", ".join(hits) if hits else
            "no GPU/driver strings found in support-bundle/report sources (#2468)")


def c_battery(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"(?:idle|discharge).{0,40}(?:30\s*min|drain)|battery.{0,40}drain")
    return (bool(hits), ", ".join(hits) if hits else
            "no 30-minute idle battery-drain comparison script")


def c_image_boot(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"60-gb-graphics\.conf")
    return (bool(hits), ", ".join(hits) if hits else
            "no booted-image check that the graphics env reaches the WebKit session")


def c_multimonitor(root: Path) -> tuple[bool, str]:
    hits = source_hits(root, r"xrandr|wlr-randr|multi.?monitor|external.?display.{0,40}arrange")
    return (bool(hits), ", ".join(hits) if hits else
            "no multi-monitor arrangement check (adjacent #2456, not a #2481 criterion)")


# (label, from-issue?, detector). The adjacent item is listed for completeness.
CRITERIA = (
    ("Mesa userspace drivers in image", True, c_mesa),
    ("VA-API packages + tools in image", True, c_vaapi_image),
    ("swrast/GBM software path configured", True, c_swrast),
    ("WebKit fallback without GPU documented", True, c_webkit_fallback),
    ("vainfo decode profile (hardware)", True, c_vainfo),
    ("1080p H.264 CPU below threshold", True, c_h264_cpu),
    ("WebKit hardware compositing (hardware)", True, c_hw_compositing),
    ("GPU/driver strings in diagnostics (#2468)", True, c_diag_strings),
    ("30-min idle battery-drain comparison", True, c_battery),
    ("Booted image: graphics env reaches WebKit", True, c_image_boot),
    ("CI graphics guard wired in ci.yml", True, c_guard),
    ("Multi-monitor arrangement (adjacent #2456)", False, c_multimonitor),
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT,
                        help="tree to audit (default: repository root)")
    args = parser.parse_args()
    root = args.root.resolve()

    print(f"#2481 graphics/static audit — root: {root}")
    print("\nHard guarantees (a missing one fails the audit):")
    failures: list[str] = []
    for name, fn in GUARANTEES:
        ok, why = fn(root)
        print(f"  [{'OK  ' if ok else 'FAIL'}] {name:26} {why}")
        if not ok:
            failures.append(f"{name}: {why}")

    print("\nIssue criteria (AUTOMATED = real check in tree; GAP = not statically verifiable):")
    print(f"  {'criterion':48} {'verdict':11} evidence")
    automated = gaps = 0
    for label, in_issue, fn in CRITERIA:
        ok, why = fn(root)
        verdict = "[AUTOMATED]" if ok else "[GAP]"
        automated += int(ok)
        gaps += int(not ok)
        evidence = why if in_issue else "(not a #2481 criterion) " + why
        evidence = evidence if len(evidence) <= 74 else evidence[:71] + "..."
        print(f"  {label:48} {verdict:11} {evidence}")

    print(f"\nSummary: automated={automated} gap={gaps} "
          f"hard_guarantee_failures={len(failures)}")
    if failures:
        for failure in failures:
            print(f"FAIL {failure}", file=sys.stderr)
        print("Static guarantees are incomplete; GAP lines are honest and do not fail.",
              file=sys.stderr)
        return 1
    print("Static guarantees present. Real GPU/VA-API runtime UAT is hardware-blocked "
          "and NOT claimed by this audit.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
