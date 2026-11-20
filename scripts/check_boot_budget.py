#!/usr/bin/env python3
"""Count-only boot-graph budget guard for the suite (issue #2495).

The suite shell ships dozens of parser-blocking ``<script src>`` tags and
render-blocking stylesheets. This guard makes the *direction* of that debt
executable, offline, without a browser: it measures the boot graph of three
surfaces and compares every debt metric against a checked-in shrink-only
ceiling. Counts may stay equal or fall, never rise.

Surfaces measured: desktop (botui/ui/suite/desktop.html), chat
  (botui/ui/suite/partials/chat.html), all_pages (every botui/ui/suite/*.html,
  including desktop.html, summed)

Per surface, the guard counts (see BASELINE_METRICS for gated fields):
  script_src          <script src=...> opening tags
  stylesheet          <link rel=stylesheet href=...> tags
  defer_async         script tags carrying >=1 defer/async attribute (adoption)
  inline_script       <script> tags without a src attribute
  js_bytes            sum of stat().st_size for local script srcs (per reference)
  css_bytes           sum of stat().st_size for local stylesheet hrefs
  blocking_script_src script_src - defer_async, floored at 0 (gated debt metric)

Polarity (important, so the ratchet cannot punish an improvement):
  The six *debt* metrics (script_src, stylesheet, inline_script, js_bytes,
  css_bytes, blocking_script_src) are gated ceilings: lower is better.
  defer_async is reported but NOT gated: adding defer/async is an improvement,
  and it already lowers blocking_script_src. Removing it raises that gated
  metric, so the debt cannot grow silently.

Honesty limits:
  * Count-and-stat only: no Lighthouse, browser, trace or compressor.
    js_bytes/css_bytes are uncompressed on-disk bytes, not transferred KB.
  * The issue's 350 KB compressed JS / 120 KB CSS @390px budget is a Lighthouse
    target to be recorded in #2421; it is printed as a stated budget line but
    is deliberately NOT asserted here.
  * Local refs (``/suite/...`` and relative paths) resolve under
    ``<root>/botui/ui``; remote/data refs count but add 0 bytes. Query strings
    (``?v=1``) and fragments are stripped before resolution. Regex tag parsing
    is lexical: well-formed HTML here counts exactly; generated DOM is invisible.

Usage:
    python3 scripts/check_boot_budget.py               # check (default)
    python3 scripts/check_boot_budget.py --update      # lower ceilings only
    python3 scripts/check_boot_budget.py --self-test   # synthetic checks

Environment:
    GB_BOOT_ROOT   repository root override (like sibling guards)

Exit codes: 0 = at/below baseline, 1 = growth or self-test failure,
            2 = baseline/usage error.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile

ROOT = os.path.abspath(
    os.environ.get("GB_BOOT_ROOT") or os.path.join(os.path.dirname(__file__), "..")
)
DEFAULT_BASELINE = os.path.join(ROOT, "scripts", "boot_budget_baseline.json")

SURFACES: dict[str, tuple[str, ...]] = {
    "desktop": ("botui/ui/suite/desktop.html",),
    "chat": ("botui/ui/suite/partials/chat.html",),
    "all_pages": (
        "botui/ui/suite/designer.html",
        "botui/ui/suite/desktop.html",
        "botui/ui/suite/editor.html",
        "botui/ui/suite/login.html",
    ),
}

DEBT_METRICS = (
    "script_src",
    "stylesheet",
    "inline_script",
    "js_bytes",
    "css_bytes",
    "blocking_script_src",
)
REPORT_METRICS = DEBT_METRICS + ("defer_async",)

METRIC_NOTES = {
    "script_src": "parser-blocking <script src> tags",
    "stylesheet": "render-blocking <link rel=stylesheet> tags",
    "inline_script": "<script> blocks with no src",
    "js_bytes": "uncompressed on-disk bytes of local script srcs",
    "css_bytes": "uncompressed on-disk bytes of local stylesheet hrefs",
    "blocking_script_src": "script_src tags with no defer/async (gated budget)",
    "defer_async": "script tags with defer/async (adoption; higher is better)",
}

# Stated target from issue #2495; recorded here for the budget line only. This
# guard measures raw bytes, so it does not enforce these compressed figures.
STATED_BUDGET = {"compressed_js_kb": 350, "compressed_css_kb": 120, "viewport_px": 390}

SCRIPT_TAG_RE = re.compile(r"<script\b([^>]*)>", re.IGNORECASE | re.DOTALL)
LINK_TAG_RE = re.compile(r"<link\b([^>]*)>", re.IGNORECASE | re.DOTALL)
ATTR_RE = re.compile(
    r"([A-Za-z_:][-A-Za-z0-9_:.]*)\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s>]+))"
)
BOOL_ATTR_RE = re.compile(r"(?:^|\s)(defer|async)(?=\s|=|$)", re.IGNORECASE)
REMOTE_RE = re.compile(r"^(?:[A-Za-z][A-Za-z0-9+.-]*:|//)")


def parse_attrs(tag_text: str) -> dict[str, str]:
    """Lower-cased attribute map; boolean attributes map to an empty string."""
    attrs: dict[str, str] = {}
    for match in ATTR_RE.finditer(tag_text):
        attrs[match.group(1).lower()] = match.group(2) or match.group(3) or match.group(4) or ""
    for match in BOOL_ATTR_RE.finditer(tag_text):
        attrs.setdefault(match.group(1).lower(), "")
    return attrs


def is_stylesheet(attrs: dict[str, str]) -> bool:
    return "stylesheet" in attrs.get("rel", "").lower().split()


def local_file(ref: str, html_path: str, root: str) -> str | None:
    """Resolve a local URL reference to an existing file under root, or None."""
    clean = ref.split("?", 1)[0].split("#", 1)[0].strip()
    if not clean or REMOTE_RE.match(clean):
        return None
    if clean.startswith("/"):
        candidate = os.path.join(root, "botui", "ui", clean.lstrip("/"))
    else:
        candidate = os.path.join(os.path.dirname(html_path), clean)
    try:
        real = os.path.realpath(candidate)
        if os.path.commonpath([real, os.path.realpath(root)]) != os.path.realpath(root):
            return None
    except (ValueError, OSError):
        return None
    return real if os.path.isfile(real) else None


def file_bytes(path: str | None) -> int:
    if path is None:
        return 0
    try:
        return os.stat(path).st_size
    except OSError:
        return 0


def measure_html(path: str, root: str) -> dict[str, int]:
    """Count one HTML file's boot graph. Missing/unreadable files measure zero."""
    counts = dict.fromkeys(REPORT_METRICS, 0)
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return counts
    for attrs in (parse_attrs(tag) for tag in SCRIPT_TAG_RE.findall(text)):
        src = attrs.get("src")
        if src is None:
            counts["inline_script"] += 1
            continue
        counts["script_src"] += 1
        if "defer" in attrs or "async" in attrs:
            counts["defer_async"] += 1
        else:
            counts["blocking_script_src"] += 1
        counts["js_bytes"] += file_bytes(local_file(src, path, root))
    for tag in LINK_TAG_RE.findall(text):
        attrs = parse_attrs(tag)
        if not is_stylesheet(attrs):
            continue
        counts["stylesheet"] += 1
        href = attrs.get("href")
        if href is not None:
            counts["css_bytes"] += file_bytes(local_file(href, path, root))
    return counts


def measure_surface(rel_paths: tuple[str, ...], root: str) -> dict[str, int]:
    """Sum HTML files that make up one surface."""
    totals = dict.fromkeys(REPORT_METRICS, 0)
    for rel in rel_paths:
        counts = measure_html(os.path.join(root, rel), root)
        for metric in REPORT_METRICS:
            totals[metric] += counts[metric]
    return totals


def measure_all(root: str) -> dict[str, dict[str, int]]:
    return {name: measure_surface(paths, root) for name, paths in SURFACES.items()}


def load_baseline(path: str) -> dict[str, dict[str, int]] | None:
    """Return validated ceilings, or None after printing a reason."""
    try:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, ValueError) as error:
        print(f"check_boot_budget: cannot load baseline {path}: {error}", file=sys.stderr)
        return None
    if not isinstance(data, dict):
        print(f"check_boot_budget: baseline {path} is not an object", file=sys.stderr)
        return None
    raw = data.get("surfaces")
    if not isinstance(raw, dict):
        print(f"check_boot_budget: baseline {path} has no 'surfaces' object", file=sys.stderr)
        return None
    ceilings: dict[str, dict[str, int]] = {}
    for name in SURFACES:
        entry = raw.get(name)
        values = entry.get("counts") if isinstance(entry, dict) else None
        if not isinstance(values, dict):
            print(f"check_boot_budget: baseline {path} lacks surfaces.{name}.counts", file=sys.stderr)
            return None
        row: dict[str, int] = {}
        for metric in DEBT_METRICS:
            value = values.get(metric)
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                print(
                    f"check_boot_budget: baseline {path} needs non-negative integer "
                    f"surfaces.{name}.counts.{metric}",
                    file=sys.stderr,
                )
                return None
            row[metric] = value
        ceilings[name] = row
    return ceilings


def write_baseline(path: str, measured: dict[str, dict[str, int]]) -> None:
    payload = {
        "issue": 2495,
        "note": (
            "Shrink-only ceilings for the six debt metrics. Lower is always accepted; "
            "raising one fails the guard. defer_async is informational adoption, not gated."
        ),
        "stated_budget": STATED_BUDGET,
        "stated_budget_note": (
            "Target from issue #2495 (Lighthouse, compressed, 390px). This guard counts "
            "uncompressed local bytes only and does not enforce the compressed figures."
        ),
        "metric_notes": METRIC_NOTES,
        "surfaces": {
            name: {
                "files": list(paths),
                "counts": {metric: measured[name][metric] for metric in DEBT_METRICS},
            }
            for name, paths in SURFACES.items()
        },
    }
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, ensure_ascii=False)
        handle.write("\n")


def growths(
    measured: dict[str, dict[str, int]], ceilings: dict[str, dict[str, int]]
) -> list[tuple[str, str, int, int]]:
    """(surface, metric, current, ceiling) for every gated metric that grew."""
    return [
        (name, metric, measured[name][metric], ceilings[name][metric])
        for name in SURFACES
        for metric in DEBT_METRICS
        if measured[name][metric] > ceilings[name][metric]
    ]


def update_allowed(
    measured: dict[str, dict[str, int]], ceilings: dict[str, dict[str, int]]
) -> bool:
    return not growths(measured, ceilings)


def print_table(
    measured: dict[str, dict[str, int]], ceilings: dict[str, dict[str, int]]
) -> None:
    print(f"{'surface':<10} {'metric':<21} {'current':>10} {'ceiling':>10}  status")
    for name in SURFACES:
        for metric in REPORT_METRICS:
            current = measured[name][metric]
            if metric not in DEBT_METRICS:
                print(f"{name:<10} {metric:<21} {current:>10} {'-':>10}  INFO")
                continue
            ceiling = ceilings[name][metric]
            status = "PASS" if current <= ceiling else "FAIL"
            print(f"{name:<10} {metric:<21} {current:>10} {ceiling:>10}  {status}")
    budgets = "; ".join(
        f"<={measured[name]['blocking_script_src']} {name}"
        for name in SURFACES
    )
    print(f"budget: {budgets} blocking scripts before first interaction (current ceilings)")
    print(
        f"budget: STATED TARGET {STATED_BUDGET['compressed_js_kb']} KB compressed JS / "
        f"{STATED_BUDGET['compressed_css_kb']} KB CSS @ "
        f"{STATED_BUDGET['viewport_px']}px — Lighthouse measurement tracked in #2421, "
        "not asserted by this count-only guard"
    )
    print(
        "budget: six debt metrics are gated shrink-only; defer_async is adoption "
        "(higher is better) and lowers blocking_script_src"
    )


def self_test() -> int:
    """Prove counting and shrink-only logic on synthetic input (no repo scan)."""
    failures: list[str] = []

    def check(name: str, condition: bool) -> None:
        if condition:
            print(f"  ok: {name}")
        else:
            print(f"  FAIL: {name}")
            failures.append(name)

    attrs = parse_attrs('src="/a.js?v=1" defer type="module" data-x=\'y\'')
    check(
        "parse_attrs handles double/single quotes and boolean attrs",
        attrs.get("src") == "/a.js?v=1" and "defer" in attrs and attrs.get("data-x") == "y",
    )
    html = (
        '<script src="/suite/js/a.js?v=1"></script>\n'
        '<script defer src="/suite/js/b.js"></script>\n'
        '<script>window.x = 1;</script>\n'
        '<link rel="stylesheet" href="/suite/css/a.css?v=2" media="(min-width: 768px)">\n'
        '<link rel="preload" href="/suite/js/a.js">\n'
    )
    with tempfile.TemporaryDirectory() as tmp:
        page = os.path.join(tmp, "page.html")
        with open(page, "w", encoding="utf-8") as handle:
            handle.write(html)
        os.makedirs(os.path.join(tmp, "botui", "ui", "suite", "js"), exist_ok=True)
        os.makedirs(os.path.join(tmp, "botui", "ui", "suite", "css"), exist_ok=True)
        js_a = os.path.join(tmp, "botui", "ui", "suite", "js", "a.js")
        js_b = os.path.join(tmp, "botui", "ui", "suite", "js", "b.js")
        css_a = os.path.join(tmp, "botui", "ui", "suite", "css", "a.css")
        for path, payload in ((js_a, "x" * 10), (js_b, "y" * 20), (css_a, "z" * 30)):
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(payload)
        counts = measure_html(page, tmp)
        check(
            "script/inline/stylesheet/defer counts on synthetic HTML",
            counts["script_src"] == 2
            and counts["inline_script"] == 1
            and counts["stylesheet"] == 1
            and counts["defer_async"] == 1
            and counts["blocking_script_src"] == 1,
        )
        check(
            "bytes are stat sizes with query strings stripped",
            counts["js_bytes"] == 30
            and counts["css_bytes"] == 30
            and local_file("https://example.com/x.js", page, tmp) is None,
        )
        check(
            "path traversal outside root is rejected",
            local_file("/../../etc/passwd", page, tmp) is None,
        )
        baseline = {name: dict.fromkeys(DEBT_METRICS, 10) for name in SURFACES}
        even = {name: dict.fromkeys(REPORT_METRICS, 10) for name in SURFACES}
        check("equal counts are not growth", growths(even, baseline) == [])
        lower = {name: dict.fromkeys(REPORT_METRICS, 9) for name in SURFACES}
        check("lower counts are not growth", growths(lower, baseline) == [])
        raised = {name: dict.fromkeys(REPORT_METRICS, 10) for name in SURFACES}
        raised["chat"]["css_bytes"] = 11
        check("a raised gated metric is growth", growths(raised, baseline) == [
            ("chat", "css_bytes", 11, 10)
        ])
        check("update refuses growth", not update_allowed(raised, baseline))
        check("update accepts shrinkage", update_allowed(lower, baseline))
        adoption = {name: dict.fromkeys(REPORT_METRICS, 10) for name in SURFACES}
        adoption["desktop"]["defer_async"] = 20
        check("defer_async adoption is not gated", growths(adoption, baseline) == [])
        bad_path = os.path.join(tmp, "bad.json")
        with open(bad_path, "w", encoding="utf-8") as handle:
            handle.write(json.dumps({"surfaces": {"desktop": {"counts": {}}}}))
        check("incomplete baseline is rejected", load_baseline(bad_path) is None)
    if failures:
        print(f"check_boot_budget: self-test FAIL — {len(failures)} check(s): {', '.join(failures)}")
        return 1
    print("check_boot_budget: self-test OK — 11 checks passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Boot-graph count budget guard (#2495)")
    parser.add_argument("--baseline", default=DEFAULT_BASELINE, help="baseline JSON path")
    parser.add_argument(
        "--update", action="store_true",
        help="lower the ceilings to current counts (refuses raises)",
    )
    parser.add_argument(
        "--self-test", action="store_true",
        help="check counting and shrink-only logic on synthetic strings",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    try:
        measured = measure_all(ROOT)
    except OSError as error:
        print(f"check_boot_budget: scan failed: {error}", file=sys.stderr)
        return 2

    existing = os.path.isfile(args.baseline)
    ceilings = load_baseline(args.baseline) if existing else {
        name: dict.fromkeys(DEBT_METRICS, 0) for name in SURFACES
    }
    if ceilings is None:
        return 2
    rel = os.path.relpath(args.baseline, ROOT)

    if args.update:
        if existing and not update_allowed(measured, ceilings):
            for name, metric, current, ceiling in growths(measured, ceilings):
                print(
                    f"check_boot_budget: refusing to raise {name}.{metric} "
                    f"({current} > baseline {ceiling}); shrink the graph first"
                )
            return 1
        write_baseline(args.baseline, measured)
        summary = ", ".join(
            f"{name}[{','.join(f'{m}:{ceilings[name][m]}->{measured[name][m]}' for m in DEBT_METRICS)}]"
            for name in SURFACES
        )
        print(f"check_boot_budget: baseline written to {rel} ({summary})")
        return 0

    print_table(measured, ceilings)
    grown = growths(measured, ceilings)
    if grown:
        print(f"check_boot_budget: FAIL — {len(grown)} gated metric(s) grew against {rel}:")
        for name, metric, current, ceiling in grown:
            print(f"  {name}.{metric}: {current} > {ceiling} (+{current - ceiling})")
        print("  remediation: remove/merge references or add defer/async, then run --update")
        return 1
    note = "" if existing else " (no baseline file: run --update to record ceilings)"
    print(f"check_boot_budget: OK — all gated metrics at/below {rel}{note}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
