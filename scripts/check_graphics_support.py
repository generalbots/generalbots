#!/usr/bin/env python3
"""Graphics-support manifest gate for the OS appliance image (#2481).

Validates that an OS image package manifest (``desktop/os/mkosi.conf``)
carries the Mesa, VA-API, vendor-driver, firmware and graphics-diagnostics
packages that let WebKitGTK/media use hardware acceleration on supported
laptops. This is a *manifest* check only: it never boots an image, probes a
GPU, or runs a benchmark.

QEMU / software fallback (documented, not a defect):
    A virtual machine or a host without a usable DRM driver falls back to
    Mesa's llvmpipe software rasterizer. The support bundle then reports
    ``graphics_support.compositing = "software"`` and
    ``graphics_support.software_rendering = true``. That is the expected
    QEMU path. VA-API decode profiles, WebKit hardware compositing, H.264 CPU
    thresholds and battery drain can only be confirmed on real hardware;
    those acceptance steps stay CI/hardware-gated and are outside this gate.

Usage:
    python3 scripts/check_graphics_support.py --check [manifest]
    python3 scripts/check_graphics_support.py --self-test
    python3 scripts/check_graphics_support.py --explain

Exit codes: 0 = manifest satisfies the graphics package set (or self-test
passed), 1 = missing packages, 2 = usage or unreadable manifest.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "desktop" / "os" / "mkosi.conf"

# Groups mirror the #2481 slice in desktop/os/mkosi.conf. An ANY_OF group is
# satisfied by any one member; every other group requires all of its members.
REQUIRED: dict[str, tuple[str, ...]] = {
    "mesa-userspace": ("libgl1-mesa-dri", "libegl-mesa0", "libgbm1"),
    "vaapi-runtime": ("libva2", "mesa-va-drivers", "vainfo"),
    "vaapi-vendor": ("intel-media-va-driver", "i965-va-driver"),
    "firmware": ("firmware-amd-graphics", "firmware-misc-nonfree"),
    "graphics-diagnostics": ("mesa-utils", "glmark2-es2-wayland"),
}
ANY_OF = {"vaapi-vendor"}

EXPLAIN = """QEMU / software fallback (#2481)
Mesa's llvmpipe (also softpipe/swrast) is the expected rasterizer in a VM or
on a host without a DRM driver. A diagnostics bundle from such a host reports
graphics_support.compositing = "software" and software_rendering = true; that
is not a failed image. Hardware compositing, VA-API decode profiles, the 1080p
H.264 CPU threshold and idle battery drain require a real laptop and are not
asserted by this manifest gate."""


def parse_packages(text: str) -> set[str]:
    """Collect package names from mkosi ``Packages=`` blocks only."""
    packages: set[str] = set()
    in_packages = False
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("["):
            in_packages = False
            continue
        if line.startswith("Packages="):
            in_packages = True
            inline = line.split("=", 1)[1].split("#", 1)[0].strip()
            packages.update(inline.split())
            continue
        if in_packages:
            # A new key at column zero closes the continued Packages= block;
            # continuation lines are indented in mkosi.conf.
            if not raw[:1].isspace() and "=" in line:
                in_packages = False
                continue
            token = line.split("#", 1)[0].strip()
            if token:
                packages.update(token.split())
    return packages


def missing_packages(packages: set[str]) -> list[str]:
    """Return human-readable reasons the package set is incomplete."""
    missing: list[str] = []
    for group, members in REQUIRED.items():
        present = [member for member in members if member in packages]
        if group in ANY_OF:
            if not present:
                missing.append(f"{group}: need one of {', '.join(members)}")
        elif len(present) != len(members):
            for member in members:
                if member not in packages:
                    missing.append(f"{group}: {member}")
    return missing


def check(path: Path) -> int:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"graphics gate: cannot read {path}: {exc}", file=sys.stderr)
        return 2
    packages = parse_packages(text)
    missing = missing_packages(packages)
    if missing:
        for reason in missing:
            print(f"  MISSING {reason}", file=sys.stderr)
        print(
            f"graphics gate: FAIL — {len(missing)} required package(s) "
            f"missing from {path}",
            file=sys.stderr,
        )
        print(EXPLAIN, file=sys.stderr)
        return 1
    print(
        f"graphics gate: OK — {len(packages)} package(s) parsed; "
        "Mesa, VA-API, firmware and graphics diagnostics present"
    )
    print("QEMU/software fallback is documented; no hardware benchmark is run.")
    return 0


def self_test() -> int:
    full = """[Content]
Packages=
    libgl1-mesa-dri
    libegl-mesa0
    libgbm1
    libva2
    mesa-va-drivers
    vainfo
    i965-va-driver
    firmware-amd-graphics
    firmware-misc-nonfree
    mesa-utils
    glmark2-es2-wayland
Repositories=main
"""
    cases = [
        ("full manifest", full, []),
        (
            "missing firmware",
            full.replace("    firmware-amd-graphics\n", ""),
            ["firmware: firmware-amd-graphics"],
        ),
        (
            "missing VA-API driver",
            full.replace("    mesa-va-drivers\n", ""),
            ["vaapi-runtime: mesa-va-drivers"],
        ),
        (
            "alternative vendor driver",
            full.replace("    i965-va-driver\n", "    intel-media-va-driver\n"),
            [],
        ),
    ]
    failures = 0
    for name, text, expected in cases:
        got = missing_packages(parse_packages(text))
        if got != expected:
            print(
                f"self-test FAIL {name}: expected {expected}, got {got}",
                file=sys.stderr,
            )
            failures += 1
    parsed = parse_packages(full)
    if "Repositories" in parsed or "main" in parsed:
        print("self-test FAIL: non-package key leaked into the package set", file=sys.stderr)
        failures += 1
    if failures:
        return 1
    print(f"graphics gate: self-test passed ({len(cases) + 1} cases)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="validate the manifest (default)")
    parser.add_argument("--self-test", action="store_true", help="run built-in parser fixtures")
    parser.add_argument("--explain", action="store_true", help="print the QEMU/software fallback note")
    parser.add_argument("manifest", nargs="?", type=Path, default=DEFAULT_MANIFEST,
                        help="package manifest (default: desktop/os/mkosi.conf)")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.explain:
        print(EXPLAIN)
        return 0
    return check(args.manifest)


if __name__ == "__main__":
    sys.exit(main())
