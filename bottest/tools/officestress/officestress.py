#!/usr/bin/env python3
"""``officestress`` — deterministic synthetic Office corpus generator (#2531).

Test-only tool. Builds DOCX/XLSX/PPTX (plus CSV and ODS) files of an exact
requested size from a fixed seed, with a selectable, documented feature matrix
(formulas, images, tables, charts, shared strings, comments, tracking, ...).
Every run emits a manifest with byte size, per-feature instance counts and
SHA-256, so a corpus can be reproduced and audited without the binary files.

    python3 bottest/tools/officestress/officestress.py --self-test
    python3 bottest/tools/officestress/officestress.py --all \
        --out /tmp/opencode/officestress --manifest
    python3 bottest/tools/officestress/officestress.py --kind xlsx --size-mb 25 \
        --features cells,formulas,shared_strings,tables --manifest
    python3 bottest/tools/officestress/officestress.py --verify \
        --out /tmp/opencode/officestress

Exit codes: 0 success, 1 self-test/verification failure, 2 bad args or I/O.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
import zipfile
from pathlib import Path
from typing import Any

from os_builders import build_csv, build_parts, verify_feature_counts
from os_matrix import ISSUE, KINDS, MIN_SIZE_MB, MAX_SIZE_MB, Config, resolve_config
from os_zip import FILLER_MEMBER, build_archive, sha256_file

GENERATOR = "bottest/tools/officestress/officestress.py"
ACCEPTANCE_SIZES = (5, 10, 25, 50, 100)
ACCEPTANCE_KINDS = ("docx", "xlsx", "pptx")
DEFAULT_SEED = 2531


def _case_name(config: Config) -> str:
    features = "-".join(config.features)
    return f"{config.kind}-{config.size_mb}mb-seed{config.seed}-{features}.{config.kind}"


def _artifact_payloads(path: Path, kind: str) -> tuple[bytes, ...]:
    """Return all decoded part bytes (excluding filler) for marker counting."""
    if kind == "csv":
        return (path.read_bytes(),)
    with zipfile.ZipFile(path) as archive:
        return tuple(archive.read(name) for name in archive.namelist() if name != FILLER_MEMBER)


def generate(config: Config, out: Path) -> dict[str, Any]:
    """Generate one corpus file and return its manifest entry."""
    target = out / _case_name(config)
    target.parent.mkdir(parents=True, exist_ok=True)
    if config.kind == "csv":
        target.write_bytes(build_csv(config))
        digest = sha256_file(target)
        size = target.stat().st_size
    else:
        metadata = build_archive(target, build_parts(config), config.target_bytes, config.seed)
        digest, size = str(metadata["sha256"]), int(metadata["size_bytes"])
    if size != config.target_bytes:
        raise ValueError(f"{target}: size {size} != target {config.target_bytes}")
    counts = verify_feature_counts(_artifact_payloads(target, config.kind), config)
    return {
        "file": target.name,
        "kind": config.kind,
        "size_mb": config.size_mb,
        "size_bytes": size,
        "seed": config.seed,
        "units": config.units,
        "features": list(config.features),
        "feature_counts": counts,
        "sha256": digest,
    }


def build_manifest(cases: list[dict[str, Any]]) -> str:
    """Serialize a deterministic, sorted manifest."""
    document = {
        "generated_by": GENERATOR,
        "issue": ISSUE,
        "synthetic_only": True,
        "compression": "ZIP_STORED",
        "min_size_mb": MIN_SIZE_MB,
        "max_size_mb": MAX_SIZE_MB,
        "acceptance_sizes_mb": list(ACCEPTANCE_SIZES),
        "cases": sorted(cases, key=lambda case: case["file"]),
    }
    return json.dumps(document, indent=2, sort_keys=True) + "\n"


def _load_manifest(path: Path) -> dict[str, Any]:
    document = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(document, dict) or not isinstance(document.get("cases"), list):
        raise ValueError(f"{path}: expected an object with a 'cases' list")
    return document


def verify_manifest(out: Path) -> int:
    """Regenerate every manifest case in a temp dir and compare bytes/SHA/counts."""
    manifest = _load_manifest(out / "manifest.json")
    failures: list[str] = []
    checked = 0
    with tempfile.TemporaryDirectory(prefix="officestress-verify-") as folder:
        temp = Path(folder)
        for case in manifest["cases"]:
            try:
                config = resolve_config(
                    case["kind"], case["size_mb"], case["seed"],
                    case["features"], case["units"],
                )
                rebuilt = generate(config, temp)
                checked += 1
                if rebuilt["sha256"] != case["sha256"]:
                    failures.append(f"{case['file']}: SHA-256 mismatch")
                if rebuilt["size_bytes"] != case["size_bytes"]:
                    failures.append(f"{case['file']}: size mismatch")
                if rebuilt["feature_counts"] != case["feature_counts"]:
                    failures.append(f"{case['file']}: feature-count mismatch")
            except (OSError, ValueError, KeyError, zipfile.BadZipFile) as error:
                failures.append(f"{case.get('file', '?')}: {error}")
    print(f"verify: {checked} case(s) regenerated")
    for failure in failures:
        print(f"  FAIL {failure}", file=sys.stderr)
    return 1 if failures or not checked else 0


def self_test() -> int:
    """Offline checks: determinism, exact size, policy, feature counting."""
    checks = 0
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(label)

    try:
        with tempfile.TemporaryDirectory(prefix="officestress-selftest-") as folder:
            root = Path(folder)
            for kind in KINDS:
                config = resolve_config(kind, MIN_SIZE_MB, DEFAULT_SEED, units=64)
                first = generate(config, root / "a")
                second = generate(config, root / "b")
                check(first["sha256"] == second["sha256"], f"{kind}: same seed -> identical SHA-256")
                check((root / "a" / first["file"]).read_bytes() == (root / "b" / second["file"]).read_bytes(),
                      f"{kind}: same seed -> identical bytes")
                check(first["size_bytes"] == config.target_bytes, f"{kind}: target size exact")
                check(all(value > 0 for value in first["feature_counts"].values()),
                      f"{kind}: every selected feature counted")
                other = generate(resolve_config(kind, MIN_SIZE_MB, DEFAULT_SEED + 1, units=64), root / "c")
                check(other["sha256"] != first["sha256"], f"{kind}: different seed -> different SHA-256")

            subset = resolve_config("xlsx", 5, DEFAULT_SEED, "cells,formulas,shared_strings", units=64)
            check(subset.features == ("cells", "formulas", "shared_strings"), "xlsx: feature subset honoured")
            check(subset.count("charts") == 0, "xlsx: unselected feature count is zero")
            entry = generate(subset, root / "subset")
            check(set(entry["feature_counts"]) == {"cells", "formulas", "shared_strings"}, "xlsx: subset manifest counts")

            for bad in (MIN_SIZE_MB - 1, MAX_SIZE_MB + 1, 0, -1, float("nan"), float("inf"), "25.5"):
                try:
                    resolve_config("docx", bad, DEFAULT_SEED)
                except ValueError:
                    check(True, f"policy rejects size {bad!r}")
                else:
                    check(False, f"policy rejects size {bad!r}")
            for good in (MIN_SIZE_MB, MAX_SIZE_MB, 25):
                check(resolve_config("docx", good, DEFAULT_SEED).size_mb == good, f"policy accepts {good} MB")
            try:
                resolve_config("xlsx", 5, DEFAULT_SEED, "not_a_feature")
            except ValueError:
                check(True, "unknown feature rejected")
            else:
                check(False, "unknown feature rejected")

            manifest_a = build_manifest([{"file": "z.xlsx", "kind": "xlsx"}])
            manifest_b = build_manifest([{"file": "z.xlsx", "kind": "xlsx"}])
            check(manifest_a == manifest_b, "manifest is deterministic")
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        print(f"self-test: unexpected error: {error}", file=sys.stderr)
        return 1

    print(f"self-test: {checks}/{checks} checks passed")
    if failures:
        for failure in failures:
            print(f"  FAIL {failure}", file=sys.stderr)
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", type=Path, default=Path("/tmp/opencode/officestress"))
    parser.add_argument("--kind", choices=KINDS, default="xlsx")
    parser.add_argument("--size-mb", default=25)
    parser.add_argument("--seed", default=DEFAULT_SEED)
    parser.add_argument("--features", default=None, help="comma-separated feature subset")
    parser.add_argument("--units", type=int, default=None, help="override structural element count")
    parser.add_argument("--all", action="store_true", help="generate the full 5/10/25/50/100 MB DOCX/XLSX/PPTX matrix")
    parser.add_argument("--manifest", action="store_true", help="write manifest.json")
    parser.add_argument("--verify", action="store_true", help="regenerate manifest cases and compare bytes/SHA/counts")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()
    if args.verify:
        try:
            return verify_manifest(args.out)
        except (OSError, ValueError, json.JSONDecodeError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 2
    try:
        if args.all:
            cases = [
                resolve_config(kind, size, args.seed)
                for kind in ACCEPTANCE_KINDS
                for size in ACCEPTANCE_SIZES
            ]
        else:
            cases = [resolve_config(args.kind, args.size_mb, args.seed, args.features, args.units)]
        results = [generate(config, args.out) for config in cases]
        for entry in results:
            print(f"generated {entry['file']} ({entry['size_bytes']} bytes, sha256={entry['sha256'][:12]}...)")
        if args.manifest:
            target = args.out / "manifest.json"
            target.write_text(build_manifest(results), encoding="utf-8")
            print(f"wrote {target}")
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
