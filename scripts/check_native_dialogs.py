#!/usr/bin/env python3
"""Native dialog guard (issues #2297/#2295).

Suite UI must use the themed in-app dialogs (GBModal / GBToasts / GBAlerts)
instead of blocking browser dialogs: alert/confirm/prompt are blocked inside
the desktop-shell webview and cannot be themed or cancelled safely. The GB
dialog systems themselves are exempt because they own the isolated fallback.

Usage:
    python3 scripts/check_native_dialogs.py --check
    python3 scripts/check_native_dialogs.py --list
    python3 scripts/check_native_dialogs.py --self-test
    python3 scripts/check_native_dialogs.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botui/ui")
SUFFIXES = (".js", ".html")
SKIP_SEGMENTS = ("vendor", "node_modules", "tests")
SKIP_SUFFIXES = (".orig",)
# The GB dialog systems own the isolated-browser fallback to the native APIs.
ALLOWED_FILES = frozenset({
    Path("botui/ui/suite/js/gb-modal.js"),
    Path("botui/ui/suite/js/gb-shared.js"),
    Path("botui/ui/suite/js/gb-alerts.js"),
    Path("botui/ui/suite/js/gb-toasts.js"),
})
NATIVE_DIALOG = re.compile(
    r"\b(?:window|globalThis)\s*\.\s*(?:alert|confirm|prompt)\s*\("
)


def candidates(root: Path) -> list[Path]:
    base = root / SCAN_ROOT
    if not base.is_dir():
        return []
    files = []
    for path in base.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in SUFFIXES:
            continue
        if any(part in SKIP_SEGMENTS for part in path.parts):
            continue
        if any(str(path).endswith(suffix) for suffix in SKIP_SUFFIXES):
            continue
        if path.relative_to(root) in ALLOWED_FILES:
            continue
        files.append(path)
    return sorted(files)


def scan_text(text: str) -> list[int]:
    """Return 1-based line numbers of native dialog calls in `text`."""
    return [text.count("\n", 0, m.start()) + 1 for m in NATIVE_DIALOG.finditer(text)]


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
            continue
        for line in scan_text(text):
            print(
                f"{path.relative_to(root)}:{line}: native dialog call; "
                "use GBModal / GBToasts / GBAlerts"
            )
            violations += 1
    if violations:
        print(f"\n{violations} native dialog call(s) in first-party UI.")
        return 1
    print("no native alert/confirm/prompt calls in first-party UI (GB dialog systems exempt)")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in scan_text(text):
            print(f"{path.relative_to(root)}:{line}")
            total += 1
    print(f"\n{total} site(s)")
    return 0


def self_test() -> int:
    bad = [
        'window.alert("x")',
        'window.confirm("x")',
        'window.prompt("x", "y")',
        'globalThis.alert("x")',
        "if (ok) window . confirm (msg);",
    ]
    good = [
        'GBModal.confirm(message, onYes)',
        'GBModal.prompt(message, fallback, title, onOk)',
        'GBToasts.show(title, message, "info")',
        "// native dialogs are forbidden in the suite",
        "var fn = alert;",
    ]
    failures = 0
    for line in bad:
        if not scan_text(line):
            print(f"self-test: missed {line}")
            failures += 1
    for line in good:
        if scan_text(line):
            print(f"self-test: false positive on {line}")
            failures += 1

    # End-to-end: a synthetic first-party file must fail --check, and the
    # themed replacement must pass. Runs entirely under a temporary root.
    with tempfile.TemporaryDirectory(prefix="native-dialog-guard-") as tmp:
        sandbox = Path(tmp)
        scan_dir = sandbox / SCAN_ROOT / "suite"
        scan_dir.mkdir(parents=True)
        synthetic = scan_dir / "synthetic.js"
        synthetic.write_text('window.alert("synthetic");\n', encoding="utf-8")
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = check(sandbox)
        if status != 1 or "synthetic.js:1" not in output.getvalue():
            print("self-test: --check did not catch the synthetic violation")
            failures += 1
        synthetic.write_text('GBModal.alert("themed");\n', encoding="utf-8")
        clean_output = io.StringIO()
        with contextlib.redirect_stdout(clean_output):
            status = check(sandbox)
        if status != 0:
            print("self-test: --check reported a false positive on a themed dialog")
            failures += 1

    if failures:
        return 1
    print("self-test ok (synthetic window.alert caught by --check)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", default=True)
    mode.add_argument("--list", action="store_true")
    mode.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    root = args.root.resolve()
    if args.list:
        return list_sites(root)
    return check(root)


if __name__ == "__main__":
    raise SystemExit(main())
