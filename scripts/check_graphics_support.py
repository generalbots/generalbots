#!/usr/bin/env python3
"""Graphics-support manifest gate for the OS appliance image (#2481).

Validates that an OS image package manifest (``desktop/os/mkosi.conf``)
carries the Mesa, VA-API, vendor-driver, firmware and graphics-diagnostics
packages that let WebKitGTK/media use hardware acceleration on supported
laptops. This is a *manifest* check only: it never boots an image, probes a
GPU, or runs a benchmark.

Only ``Packages=`` blocks of a mkosi config are parsed. A missing block, a
malformed line, an unreadable/non-text file or a missing required package all
fail closed (non-zero exit), so a broken manifest can never be mistaken for a
good one.

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
passed), 1 = manifest fails the gate (missing package or malformed content),
2 = usage or unreadable manifest.
"""

from __future__ import annotations

import argparse
import re
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

# Debian binary-package names: lowercase letters/digits, then letters, digits,
# '+', '-' or '.'. Anything else inside a Packages= block is malformed input.
PACKAGE_RE = re.compile(r"^[a-z0-9][a-z0-9+.-]*$")

EXPLAIN = """QEMU / software fallback (#2481)
Mesa's llvmpipe (also softpipe/swrast) is the expected rasterizer in a VM or
on a host without a DRM driver. A diagnostics bundle from such a host reports
graphics_support.compositing = "software" and software_rendering = true; that
is not a failed image. Hardware compositing, VA-API decode profiles, the 1080p
H.264 CPU threshold and idle battery drain require a real laptop and are not
asserted by this manifest gate."""


def parse_packages(text: str) -> tuple[set[str], list[str]]:
    """Collect package names from mkosi ``Packages=`` blocks only.

    Returns ``(packages, problems)``. ``problems`` is empty for a well-formed
    manifest that contains at least one ``Packages=`` block; any structural
    problem (non-text input, a malformed line, an invalid package token or a
    missing ``Packages=`` block) is recorded so callers fail closed.
    """
    if not isinstance(text, str):
        return set(), ["manifest content is not text"]

    packages: set[str] = set()
    problems: list[str] = []
    in_packages = False
    found_block = False

    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("["):
            # A new section always terminates a continued Packages= block.
            in_packages = False
            continue
        if raw[:1].isspace():
            # Indented continuation of the previous key's value.
            if in_packages:
                token = stripped.split("#", 1)[0].strip().rstrip("\\").strip()
                for word in token.split():
                    _add_package(word, packages, problems)
            continue
        # A column-zero line must be a "Key=value" pair.
        if "=" not in stripped:
            problems.append(f"unexpected line: {stripped!r}")
            in_packages = False
            continue
        key, _, value = stripped.partition("=")
        if key.strip() == "Packages":
            in_packages = True
            found_block = True
            inline = value.split("#", 1)[0].strip().rstrip("\\").strip()
            for word in inline.split():
                _add_package(word, packages, problems)
        else:
            in_packages = False

    if not found_block:
        problems.append("no Packages= block found")
    return packages, problems


def _add_package(word: str, packages: set[str], problems: list[str]) -> None:
    """Add one validated package token, recording malformed ones."""
    if PACKAGE_RE.match(word):
        packages.add(word)
    else:
        problems.append(f"invalid package token: {word!r}")


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


def evaluate(text: str) -> tuple[set[str], list[str]]:
    """Parse a manifest and return ``(packages, problems)``, failing closed."""
    packages, problems = parse_packages(text)
    problems.extend(f"MISSING {reason}" for reason in missing_packages(packages))
    return packages, problems


def check(path: Path) -> int:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        print(f"graphics gate: cannot read {path}: {exc}", file=sys.stderr)
        return 2
    packages, problems = evaluate(text)
    if problems:
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        print(
            f"graphics gate: FAIL — {len(problems)} issue(s) in {path}",
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
    # (name, manifest text, expected to pass, substring expected in problems)
    cases: list[tuple[str, object, bool, str | None]] = [
        ("good manifest", full, True, None),
        (
            "missing package (firmware)",
            full.replace("    firmware-amd-graphics\n", ""),
            False,
            "firmware: firmware-amd-graphics",
        ),
        (
            "missing package (VA-API runtime)",
            full.replace("    mesa-va-drivers\n", ""),
            False,
            "vaapi-runtime: mesa-va-drivers",
        ),
        (
            "alternative vendor driver",
            full.replace("    i965-va-driver\n", "    intel-media-va-driver\n"),
            True,
            None,
        ),
        (
            "missing Packages= section",
            "[Content]\nRepositories=main\n",
            False,
            "no Packages= block found",
        ),
        (
            "malformed input (non-text)",
            b"[Content]\nPackages=libgl1-mesa-dri\n",
            False,
            "manifest content is not text",
        ),
        (
            "malformed input (stray line)",
            full.replace("Repositories=main", "this is not a key"),
            False,
            "unexpected line",
        ),
        (
            "malformed input (invalid token)",
            full.replace("    libgbm1", "    BAD TOKEN!"),
            False,
            "invalid package token",
        ),
        (
            "backslash continuation",
            full.replace(
                "    libgl1-mesa-dri\n    libegl-mesa0",
                "    libgl1-mesa-dri \\\n    libegl-mesa0",
            ),
            True,
            None,
        ),
    ]

    failures = 0
    for name, text, expect_ok, must_mention in cases:
        _, problems = evaluate(text)  # type: ignore[arg-type]
        passed = not problems
        if passed != expect_ok:
            print(
                f"self-test FAIL {name}: expected pass={expect_ok}, "
                f"got pass={passed} (problems={problems})",
                file=sys.stderr,
            )
            failures += 1
            continue
        if must_mention and not any(must_mention in p for p in problems):
            print(
                f"self-test FAIL {name}: expected problem containing "
                f"{must_mention!r}, got {problems}",
                file=sys.stderr,
            )
            failures += 1

    parsed, _ = parse_packages(full)
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
