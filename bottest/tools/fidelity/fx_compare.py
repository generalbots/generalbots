#!/usr/bin/env python3
"""Deterministic round-trip comparison and scoring (#2457).

Pairs two corpus directories by relative path, extracts a structural feature
vector for each Office file and produces a deterministic report dictionary.
No I/O beyond reading the packages; rendering lives in :mod:`fx_report`.

Scoring
-------
* count features score 1.0 when equal, otherwise
  ``1 - |a-b| / max(a,b)`` (0..1); status ``dropped``/``added`` records direction.
* the ``text`` feature scores 1.0 when the normalised text is identical,
  otherwise a SequenceMatcher ratio over the first ``MAX_DIFF_CHARS`` chars;
  full-length SHA-256 digests are kept as evidence.
* a file's score is the weighted mean of its features (text carries
  ``text_weight``). A missing/extra/unreadable file scores 0.
* the corpus score is the unweighted mean of all file scores in the union of
  both directories. PASS when it is >= ``threshold``.
"""

from __future__ import annotations

from difflib import SequenceMatcher
from hashlib import sha256
from pathlib import Path

import fx_extract as fx

MAX_DIFF_CHARS = 4000
DEFAULT_THRESHOLD = 0.95
DEFAULT_TEXT_WEIGHT = 3.0
SCHEMA_VERSION = 1
ISSUE = 2457


def discover(root: Path) -> dict[str, Path]:
    """Map relative POSIX path -> file for every Office file under ``root``."""
    found: dict[str, Path] = {}
    if not root.is_dir():
        return found
    for path in root.rglob("*"):
        if not path.is_file() or fx.kind_for(path) == "":
            continue
        found[path.relative_to(root).as_posix()] = path
    return found


def _count_score(original: int, roundtrip: int) -> tuple[float, str]:
    if original == roundtrip:
        return 1.0, "match"
    score = max(0.0, min(1.0, 1.0 - abs(original - roundtrip)
                         / max(original, roundtrip)))
    status = "dropped" if roundtrip < original else "added"
    return round(score, 6), status


def _text_score(original: str, roundtrip: str) -> tuple[float, str, dict[str, object]]:
    meta: dict[str, object] = {
        "original_len": len(original),
        "roundtrip_len": len(roundtrip),
        "original_sha256": sha256(original.encode("utf-8")).hexdigest(),
        "roundtrip_sha256": sha256(roundtrip.encode("utf-8")).hexdigest(),
    }
    if original == roundtrip:
        return 1.0, "match", meta
    score = SequenceMatcher(
        None, original[:MAX_DIFF_CHARS], roundtrip[:MAX_DIFF_CHARS],
        autojunk=False,
    ).ratio()
    return round(score, 6), "degraded", meta


def _shell(path: str, kind: str, status: str, score: float,
           passed: bool, features: list[dict[str, object]] | None = None,
           error: str = "") -> dict[str, object]:
    entry: dict[str, object] = {
        "path": path, "kind": kind, "status": status,
        "score": score, "passed": passed,
    }
    if features is not None:
        entry["features"] = features
    if error:
        entry["error"] = error
    return entry


def compare_pair(relpath: str, original: Path, roundtrip: Path, kind: str,
                 threshold: float, text_weight: float) -> dict[str, object]:
    """Compare one paired file and return its report entry."""
    try:
        before = fx.extract(original, kind)
    except fx.ExtractError as exc:
        return _shell(relpath, kind, "unreadable-original", 0.0, False,
                      error=exc.code)
    try:
        after = fx.extract(roundtrip, kind)
    except fx.ExtractError as exc:
        return _shell(relpath, kind, "unreadable-roundtrip", 0.0, False,
                      error=exc.code)

    features: list[dict[str, object]] = []
    weighted = 0.0
    total_weight = 0.0
    for name, ftype in fx.FEATURES[kind]:
        weight = text_weight if ftype == "text" else 1.0
        if ftype == "text":
            score, status, meta = _text_score(str(before[name]), str(after[name]))
            feature: dict[str, object] = {
                "name": name, "type": "text", "score": score, "status": status,
            }
            feature.update(meta)
        else:
            score, status = _count_score(int(before[name]), int(after[name]))
            feature = {
                "name": name, "type": "count",
                "original": before[name], "roundtrip": after[name],
                "score": score, "status": status,
            }
        features.append(feature)
        weighted += score * weight
        total_weight += weight

    score = round(weighted / total_weight, 6)
    return _shell(relpath, kind, "matched", score, score >= threshold, features)


def build_report(original: Path, roundtrip: Path,
                 threshold: float = DEFAULT_THRESHOLD,
                 text_weight: float = DEFAULT_TEXT_WEIGHT) -> dict[str, object]:
    """Build the full deterministic report for two corpus directories."""
    before = discover(original)
    after = discover(roundtrip)
    entries: list[dict[str, object]] = []
    for relpath in sorted(set(before) | set(after)):
        kind = fx.kind_for(relpath)
        if relpath in before and relpath in after:
            entries.append(compare_pair(relpath, before[relpath],
                                        after[relpath], kind, threshold,
                                        text_weight))
        elif relpath in before:
            entries.append(_shell(relpath, kind, "missing", 0.0, False))
        else:
            entries.append(_shell(relpath, kind, "extra", 0.0, False))

    total = len(entries)
    if total == 0:
        overall = 0.0
        note = "no-office-files"
    else:
        overall = round(sum(float(e["score"]) for e in entries) / total, 6)
        note = ""
    summary: dict[str, object] = {
        "files_total": total,
        "files_matched": sum(1 for e in entries if e["status"] == "matched"),
        "files_missing": sum(1 for e in entries if e["status"] == "missing"),
        "files_extra": sum(1 for e in entries if e["status"] == "extra"),
        "files_unreadable": sum(1 for e in entries
                                if str(e["status"]).startswith("unreadable")),
        "files_passed": sum(1 for e in entries if e["passed"]),
        "files_failed": sum(1 for e in entries if not e["passed"]),
        "overall_score": overall,
        "passed": total > 0 and overall >= threshold,
    }
    if note:
        summary["note"] = note

    return {
        "tool": "fidelity",
        "issue": ISSUE,
        "schema_version": SCHEMA_VERSION,
        "original": str(original),
        "roundtrip": str(roundtrip),
        "threshold": threshold,
        "text_weight": text_weight,
        "summary": summary,
        "files": entries,
    }
