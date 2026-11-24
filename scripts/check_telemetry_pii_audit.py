#!/usr/bin/env python3
"""Static audit for #2459: browser/office perf telemetry must not leak PII.

Pure Python, no network, no cargo, no browser. The audit is deliberately
non-vacuous: every assertion names a concrete pattern that must exist in the
current tree, so removing or weakening the guard fails the gate.

Checks (each reported with a stable code and file:line evidence):
  A1..A4  the JS perf module owns a CLOSED allowlist of label keys and ignores
          unknown keys; no dynamic key is forwarded from an arbitrary object.
  B1..B5  allowlist tokens are fixed metric-name keys (no file/title/URL/user
          id/email); dynamic ids are bounded (`sanitize_label`) or hashed
          (`opaque_token`) before they reach a backend label.
  C1..C3  a failed request / unavailable endpoint is logged or surfaced, not
          silently swallowed (JS error branches + Rust log::error!).
  D1..D3  the canonical /metrics scrape router and both RBAC GET permissions
          still exist (same logic as scripts/check_metrics_wiring.py).

Usage:
    python3 scripts/check_telemetry_pii_audit.py

Offline probing only: TELEMETRY_AUDIT_ROOT may point the scan at a scratch
copy of the tree; every assertion still runs unchanged against that copy.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT_ENV = "TELEMETRY_AUDIT_ROOT"

PERF_REL = "botui/ui/suite/js/office-perf-marks.js"
TRANSLATE_REL = "botui/ui/suite/docs/modules/17_translate.js"
COLLECT_REL = "botserver/crates/botcore/src/shared/analytics/collect.rs"
SUB_ROUTES_REL = "botserver/src/main_module/routes/feature_merge_office/sub_routes.rs"
RBAC_REL = "botserver/crates/botsecurity-auth/src/rbac_middleware/routes_user/app_routes.rs"

FORBIDDEN_TOKENS = frozenset({
    "user", "userid", "user_id", "email", "filename", "file_name",
    "title", "url", "uri", "path", "file", "file_id", "doc_id",
    "document_id", "tenant_id", "branch_id", "workbook_id", "session_id",
})


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def locate(text: str, needle: str) -> str:
    """Return ':<line>' evidence for the first literal occurrence of needle."""
    idx = text.find(needle)
    return f":{line_of(text, idx)}" if idx >= 0 else ""


def block_after(text: str, pattern: str):
    """Return (body, start_line) of the first {...} block after a regex match."""
    match = re.search(pattern, text)
    if match is None:
        return None, None
    open_at = text.find("{", match.end())
    if open_at < 0:
        return None, None
    depth = 0
    for i in range(open_at, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[open_at + 1:i], line_of(text, match.start())
    return None, None


def split_list(text: str, name: str):
    """Read a `var NAME = ("..." + "...").split(" ")` allowlist into tokens."""
    match = re.search(
        r"var\s+" + re.escape(name) + r"\s*=\s*\(([^;]*?)\)\.split\(\" \"\)",
        text,
        re.DOTALL,
    )
    if match is None:
        return None
    return " ".join(re.findall(r'"([^"]*)"', match.group(1))).split()


def main() -> int:
    root_arg = os.environ.get(ROOT_ENV)
    root = Path(root_arg).resolve() if root_arg else Path(__file__).resolve().parents[1]

    errors: list[str] = []
    evidence: list[str] = []

    def read(rel: str):
        path = root / rel
        if not path.is_file():
            errors.append(f"[IO] MISSING file {rel}")
            return None
        try:
            return path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            errors.append(f"[IO] UNREADABLE file {rel} ({exc})")
            return None

    def require(ok: bool, code: str, message: str, where: str = "") -> None:
        if ok:
            evidence.append(f"[{code}] {message}{(' ' + where) if where else ''}")
        else:
            errors.append(f"[{code}] {message}{(' ' + where) if where else ''}")

    perf = read(PERF_REL)
    translate = read(TRANSLATE_REL)
    collect = read(COLLECT_REL)
    sub_routes = read(SUB_ROUTES_REL)
    rbac = read(RBAC_REL)

    # --- A. closed allowlist in the JS perf module -----------------------------
    if perf is not None:
        apps = split_list(perf, "KNOWN_APPS")
        formats = split_list(perf, "KNOWN_FORMATS")
        require(apps is not None and len(apps) >= 20,
                "A1", "KNOWN_APPS is an explicit closed allowlist (>=20 keys)",
                PERF_REL + locate(perf, "KNOWN_APPS"))
        require(formats is not None and "pdf" in formats and "docx" in formats,
                "A1", "KNOWN_FORMATS is an explicit closed allowlist (pdf/docx)",
                PERF_REL + locate(perf, "KNOWN_FORMATS"))

        label_body, label_line = block_after(perf, r"function\s+label\s*\(")
        guarded = re.search(
            r"KNOWN_APPS\.indexOf\(lower\)\s*!==\s*-1\)\s*"
            r"return lower\.slice\(0,\s*maxLength \|\| 32\);\s*return NO_LABEL;",
            label_body or "",
        )
        require(guarded is not None,
                "A2", "label(): only KNOWN_APPS keys pass; unknown -> NO_LABEL",
                f"{PERF_REL}:{label_line}" if label_line else "")
        fmt_body, fmt_line = block_after(perf, r"function\s+formatLabel\s*\(")
        fmt_guarded = re.search(
            r"KNOWN_FORMATS\.indexOf\(lower\)\s*!==\s*-1\s*\?\s*lower\s*:\s*NO_LABEL",
            fmt_body or "",
        )
        require(fmt_guarded is not None,
                "A2", "formatLabel(): only KNOWN_FORMATS keys pass; unknown -> NO_LABEL",
                f"{PERF_REL}:{fmt_line}" if fmt_line else "")

        cycle_key = re.search(r"var\s+key\s*=\s*label\(appId,\s*\d+\)\s*;", perf)
        mark_name = re.search(r'"gb\.office\."\s*\+\s*entry\.key\s*\+\s*"\."\s*\+\s*stage', perf)
        require(cycle_key is not None and mark_name is not None,
                "A3", "cycle keys and mark names derive from the labeled app only",
                PERF_REL + locate(perf, "var key = label("))
        for_in_targets = set(re.findall(r"for \(var \w+ in (\w+)\)", perf))
        dynamic_forwarding = (
            "Object.keys(" in perf or "Object.entries(" in perf
            or bool(re.search(r"perf\.mark\([^;]*appId", perf))
            or for_in_targets != {"cycles"}
        )
        require(not dynamic_forwarding,
                "A4", "no dynamic key forwarding from arbitrary objects into marks",
                f"{PERF_REL} (for-in targets: {sorted(for_in_targets)})")

    # --- B. no PII can reach a label ------------------------------------------
    if perf is not None:
        apps = split_list(perf, "KNOWN_APPS") or []
        formats = split_list(perf, "KNOWN_FORMATS") or []
        bad_tokens = [
            token for token in apps + formats
            if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,31}", token)
            or token.lower() in FORBIDDEN_TOKENS
        ]
        require(not bad_tokens,
                "B1", f"allowlist holds only fixed [a-z0-9-] keys, no PII field names"
                      + (f" (bad: {bad_tokens})" if bad_tokens else ""),
                PERF_REL + locate(perf, "KNOWN_APPS"))
        require("sheet" in apps and "docs" in apps,
                "B1", "office app keys sheet/docs are present as fixed labels",
                PERF_REL + locate(perf, "KNOWN_APPS"))
        require(bool(re.search(r'typeof value !== "string" && typeof value !== "number"', perf))
                and "maxLength || 32" in perf and 'NO_LABEL = "unknown"' in perf,
                "B2", "label() rejects non-scalars and bounds length to 32",
                PERF_REL + locate(perf, "function label("))

    if collect is not None:
        sanitize_body, sanitize_line = block_after(collect, r"fn\s+sanitize_label\s*\(")
        sanitized = all(needle in (sanitize_body or "") for needle in (
            "'a'..='z' | '0'..='9' => c",
            "'A'..='Z' => c.to_ascii_lowercase()",
            "_ => '_'",
            "trim_matches('_')",
            "take(32)",
            '"unknown".to_string()',
        ))
        require(sanitized,
                "B3", "sanitize_label maps to bounded [a-z0-9_], trims, caps at 32",
                f"{COLLECT_REL}:{sanitize_line}" if sanitize_line else "")

        opaque_body, opaque_line = block_after(collect, r"fn\s+opaque_token\s*\(")
        hashed = all(needle in (opaque_body or "") for needle in (
            "DefaultHasher", ".hash(&mut hasher)", "hasher.finish()",
            'format!("{prefix}{:016x}"',
        ))
        require(hashed,
                "B4", "opaque_token one-way hashes ids (DefaultHasher -> hex prefix)",
                f"{COLLECT_REL}:{opaque_line}" if opaque_line else "")

        wb_body, wb_line = block_after(
            collect, r"pub async fn track_office_workbook_memory\s*\(")
        require(wb_body is not None
                and 'opaque_token("wb", workbook_id)' in wb_body
                and wb_body.count("workbook_id") == 1,
                "B5", "workbook id reaches its label only through opaque_token('wb')",
                f"{COLLECT_REL}:{wb_line}" if wb_line else "")

        recalc_body, recalc_line = block_after(
            collect, r"pub async fn track_office_recalc\s*\(")
        require(recalc_body is not None
                and 'sanitize_label(app)' in recalc_body
                and bool(re.search(
                    r'scope\.insert\(\s*"branch"[^;]*?opaque_token\("br", branch\)',
                    recalc_body, re.DOTALL)),
                "B5", "recalc app is sanitized and branch is opaque_token('br')",
                f"{COLLECT_REL}:{recalc_line}" if recalc_line else "")

        for fn, needle in (
            ("track_office_export", "sanitize_label(format)"),
            ("track_route_latency", "sanitize_label(route)"),
        ):
            body, at = block_after(collect, r"pub async fn " + fn + r"\s*\(")
            require(body is not None and needle in body,
                    "B5", f"{fn}() labels are sanitized via {needle}",
                    f"{COLLECT_REL}:{at}" if at else "")

    # B6: the legacy track_message used to insert a raw user_id label.
    msg_body, msg_at = block_after(collect, r"pub async fn track_message\s*\(")
    require(
        msg_body is not None
        and 'opaque_token("u"' in msg_body
        and 'labels.insert("user_id"'.replace('"','') not in msg_body.replace('"',''),
        "B6",
        "track_message() hashes the user id and never emits a raw user_id label",
        f"{COLLECT_REL}:{msg_at}" if msg_at else "",
    )

    # --- C. failures are logged or surfaced, never silently swallowed ---------
    if perf is not None:
        mark_catch = re.search(r"perf\.mark\(name\);[\s\S]{0,60}?catch", perf)
        measure_catch = re.search(r"perf\.measure\(measureName[\s\S]{0,400}?catch", perf)
        require(mark_catch is not None and measure_catch is not None,
                "C1", "perf.mark()/perf.measure() have explicit error branches",
                PERF_REL + locate(perf, "try {"))

    if translate is not None:
        surfaced = all(needle in translate for needle in (
            'throw new Error("http " + res.status)',
            'result === "unavailable"',
            "j.error",
            "gbLogError",
        ))
        require(surfaced,
                "C2", "unavailable HTTP result is detected and logged",
                TRANSLATE_REL + locate(translate, 'result === "unavailable"'))
        require(bool(re.search(
                    r'setSaveStatus\(\s*"Translation unavailable"\s*,\s*true\s*\)',
                    translate)),
                "C2", "unavailable endpoint is surfaced to the user, not swallowed",
                TRANSLATE_REL + locate(translate, "Translation unavailable"))

    if collect is not None:
        require("log::error!" in collect,
                "C3", "backend telemetry path logs failures via log::error!",
                COLLECT_REL + locate(collect, "log::error!"))

    # --- D. /metrics wiring (mirrors scripts/check_metrics_wiring.py) ----------
    if sub_routes is not None:
        canonical = re.search(
            r"\.merge\([^;]{0,400}?configure_canonical\s*\(\s*\)",
            sub_routes, re.DOTALL)
        require(canonical is not None,
                "D1", "configure_canonical() merged into the scrape router",
                SUB_ROUTES_REL + locate(sub_routes, "configure_canonical"))

    if rbac is not None:
        for route in ("/metrics", "/api/metrics"):
            perm = re.search(
                r'RoutePermission::new\(\s*"' + re.escape(route)
                + r'"\s*,\s*"GET"', rbac)
            require(perm is not None,
                    "D2", f"RBAC GET permission for {route} present",
                    RBAC_REL + locate(rbac, '"' + route + '"'))

    if errors:
        print("check_telemetry_pii_audit: FAIL")
        for message in errors:
            print(f"  - {message}")
        return 1

    for message in evidence:
        print(f"  OK {message}")
    print(
        f"check_telemetry_pii_audit: OK - {len(evidence)} assertions passed "
        "(closed label allowlist, no PII in labels, failures surfaced, "
        "/metrics + RBAC GET intact)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
