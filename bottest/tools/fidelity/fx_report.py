#!/usr/bin/env python3
"""Deterministic JSON/Markdown rendering of fidelity reports (#2457).

Both renderers are pure functions of the report dictionary: no clock, no host
name, no locale and no filesystem ordering leaks into the output. JSON uses
sorted keys and two-space indentation; Markdown uses a fixed table layout.
"""

from __future__ import annotations

import json
from typing import Any


def render_json(report: dict[str, Any]) -> str:
    """Return a stable, newline-terminated JSON document."""
    return json.dumps(report, sort_keys=True, indent=2, ensure_ascii=True) + "\n"


def _verdict(passed: bool) -> str:
    return "PASS" if passed else "FAIL"


def render_markdown(report: dict[str, Any]) -> str:
    """Return a human-readable Markdown report."""
    summary = report["summary"]
    lines: list[str] = [
        "# Office round-trip fidelity report (#2457)",
        "",
        f"- Original: `{report['original']}`",
        f"- Round-tripped: `{report['roundtrip']}`",
        f"- Threshold: `{summary_threshold(report)}`",
        f"- Text weight: `{report['text_weight']}`",
        "",
        "## Summary",
        "",
        "| Metric | Value |",
        "| --- | --- |",
        f"| Files (union) | {summary['files_total']} |",
        f"| Matched | {summary['files_matched']} |",
        f"| Missing | {summary['files_missing']} |",
        f"| Extra | {summary['files_extra']} |",
        f"| Unreadable | {summary['files_unreadable']} |",
        f"| Passed files | {summary['files_passed']} |",
        f"| Failed files | {summary['files_failed']} |",
        f"| Overall score | {summary['overall_score']:.6f} |",
        f"| **Result** | **{_verdict(bool(summary['passed']))}** |",
    ]
    if summary.get("note"):
        lines.append("")
        lines.append(f"Note: `{summary['note']}`.")

    lines.extend(["", "## Per-file", "",
                  "| Path | Kind | Status | Score | Result |",
                  "| --- | --- | --- | ---: | --- |"])
    for entry in report["files"]:
        lines.append(
            f"| `{entry['path']}` | {entry['kind']} | {entry['status']} "
            f"| {entry['score']:.6f} | {_verdict(bool(entry['passed']))} |"
        )

    lines.extend(["", "## Feature differences", "",
                  "Only non-matching features and non-matched files are listed.",
                  "", "| Path | Feature | Type | Original | Round-tripped | Score | Status |",
                  "| --- | --- | --- | --- | --- | ---: | --- |"])
    any_diff = False
    for entry in report["files"]:
        if entry["status"] != "matched":
            any_diff = True
            detail = entry.get("error", "-")
            lines.append(f"| `{entry['path']}` | - | - | - | - | - "
                         f"| {entry['status']} ({detail}) |")
            continue
        for feature in entry.get("features", []):
            if feature["status"] == "match":
                continue
            any_diff = True
            if feature["type"] == "text":
                original = f"len={feature['original_len']} {feature['original_sha256'][:12]}"
                roundtrip = f"len={feature['roundtrip_len']} {feature['roundtrip_sha256'][:12]}"
            else:
                original = str(feature["original"])
                roundtrip = str(feature["roundtrip"])
            lines.append(
                f"| `{entry['path']}` | {feature['name']} | {feature['type']} "
                f"| {original} | {roundtrip} | {feature['score']:.6f} "
                f"| {feature['status']} |"
            )
    if not any_diff:
        lines.append("| - | - | - | - | - | - | none |")

    lines.append("")
    return "\n".join(lines)


def summary_threshold(report: dict[str, Any]) -> str:
    """Format the threshold consistently with the score column."""
    return f"{float(report['threshold']):.6f}"
