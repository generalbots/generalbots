#!/usr/bin/env python3
"""Feature matrix and policy for the ``officestress`` synthetic corpus (#2531).

This module is pure data + pure functions. It has no I/O and no third-party
imports, so it can be unit-tested and compiled in isolation. Every decision
that affects the bytes of a generated file is made here: the target-size
policy, the set of selectable features per format, and the deterministic
derivation of per-feature instance counts.

The matrix is documented in ``README.md`` next to this file. Keep the two in
sync: a feature added here must be described there.
"""

from __future__ import annotations

import math
from dataclasses import dataclass

ISSUE = 2531
MIN_SIZE_MB = 5
MAX_SIZE_MB = 100
KINDS = ("docx", "xlsx", "pptx", "csv", "ods")
OOXML_KINDS = ("docx", "xlsx", "pptx")

# (weight, description). ``weight`` is the fraction of ``units`` that become
# instances of the feature; counts are derived deterministically from it.
FEATURE_MATRIX: dict[str, dict[str, tuple[float, str]]] = {
    "docx": {
        "paragraphs": (1.00, "word/document.xml <w:p> body paragraphs"),
        "tables": (0.05, "word/document.xml <w:tbl> tables with seeded cells"),
        "tracking": (0.10, "word/document.xml <w:ins>/<w:del> tracked revisions"),
        "comments": (0.04, "word/comments.xml <w:comment> + document anchors"),
        "images": (0.02, "word/media/imageN.png + <w:drawing> references"),
        "charts": (0.01, "word/charts/chartN.xml + <w:drawing> references"),
    },
    "xlsx": {
        "cells": (1.00, "xl/worksheets/sheet1.xml <c> value cells"),
        "formulas": (0.20, "sheet1.xml <c><f> formula cells"),
        "shared_strings": (0.30, "xl/sharedStrings.xml + t=\"s\" cells"),
        "tables": (0.02, "xl/tables/tableN.xml <table> definitions"),
        "comments": (0.04, "xl/commentsN.xml <comment> notes"),
        "images": (0.02, "xl/media/imageN.png + drawing references"),
        "charts": (0.01, "xl/charts/chartN.xml + drawing references"),
    },
    "pptx": {
        "slides": (1.00, "ppt/slides/slideN.xml shape trees"),
        "tables": (0.05, "slide <p:graphicFrame> <a:tbl> tables"),
        "comments": (0.04, "ppt/commentAuthors.xml + slide comment parts"),
        "images": (0.04, "ppt/media/imageN.png + <p:pic> references"),
        "charts": (0.02, "ppt/charts/chartN.xml + graphicFrame references"),
    },
    "csv": {
        "rows": (1.00, "plain UTF-8 data rows with fixed columns"),
    },
    "ods": {
        "cells": (1.00, "content.xml <table:table-cell> values"),
        "formulas": (0.20, "content.xml table:formula attributes"),
    },
}

ALL_FEATURES = tuple(sorted({name for specs in FEATURE_MATRIX.values() for name in specs}))


@dataclass(frozen=True)
class Config:
    """A fully-resolved, deterministic generation request."""

    kind: str
    size_mb: int
    seed: int
    units: int
    counts: tuple[tuple[str, int], ...]
    target_bytes: int

    def count(self, feature: str) -> int:
        """Return the instance count for ``feature`` (0 when not selected)."""
        return dict(self.counts).get(feature, 0)

    @property
    def features(self) -> tuple[str, ...]:
        return tuple(name for name, _ in self.counts)


def resolve_size_mb(value: object) -> int:
    """Validate the 5-100 MB corpus policy and return the whole-MB size."""
    try:
        size = float(value)  # type: ignore[arg-type]
    except (TypeError, ValueError) as error:
        raise ValueError(f"size {value!r} is not a number") from error
    if not math.isfinite(size):
        raise ValueError(f"size {value!r} must be finite")
    if size != int(size):
        raise ValueError(f"size {size} MB must be a whole number")
    whole = int(size)
    if not MIN_SIZE_MB <= whole <= MAX_SIZE_MB:
        raise ValueError(
            f"size {whole} MB is outside the #{ISSUE} corpus policy "
            f"[{MIN_SIZE_MB}, {MAX_SIZE_MB}] MB"
        )
    return whole


def resolve_seed(value: object) -> int:
    """Validate a non-negative integer seed."""
    if isinstance(value, bool):
        raise ValueError("seed must be an integer, not a boolean")
    try:
        seed = int(value)  # type: ignore[arg-type]
    except (TypeError, ValueError) as error:
        raise ValueError(f"seed {value!r} is not an integer") from error
    if seed < 0:
        raise ValueError("seed must be non-negative")
    return seed


def resolve_features(kind: str, requested: object = None) -> tuple[str, ...]:
    """Resolve an optional feature subset to an ordered tuple of valid names."""
    if kind not in FEATURE_MATRIX:
        raise ValueError(f"unsupported kind {kind!r}; expected one of {tuple(FEATURE_MATRIX)}")
    available = FEATURE_MATRIX[kind]
    if requested is None:
        return tuple(available)
    if isinstance(requested, str):
        requested = [part for part in requested.split(",") if part]
    if isinstance(requested, (list, tuple)) is False:
        raise ValueError("features must be a comma-separated string or a sequence")
    names = []
    for name in requested:  # type: ignore[union-attr]
        if name not in available:
            raise ValueError(f"feature {name!r} is not available for {kind}; choose from {tuple(available)}")
        if name not in names:
            names.append(name)
    if not names:
        raise ValueError("at least one feature must be selected")
    return tuple(names)


def derive_counts(kind: str, units: int, features: tuple[str, ...]) -> tuple[tuple[str, int], ...]:
    """Derive deterministic per-feature instance counts from ``units``."""
    if units < 1:
        raise ValueError("units must be positive")
    counts: list[tuple[str, int]] = []
    for name in features:
        weight = FEATURE_MATRIX[kind][name][0]
        counts.append((name, max(1, int(units * weight))))
    return tuple(counts)


def resolve_config(
    kind: str,
    size_mb: object,
    seed: object,
    features: object = None,
    units: object = None,
) -> Config:
    """Build a fully-resolved :class:`Config` from raw CLI values."""
    resolved_kind = str(kind).lower()
    whole = resolve_size_mb(size_mb)
    resolved_seed = resolve_seed(seed)
    selected = resolve_features(resolved_kind, features)
    target_bytes = whole * 1024 * 1024
    if units is None:
        # Deterministic density: scales with target size, bounded for sanity.
        resolved_units = max(8, min(4096, target_bytes // 16384))
    else:
        if isinstance(units, bool) or not isinstance(units, int):
            raise ValueError("units must be a positive integer")
        resolved_units = units
        if resolved_units < 1:
            raise ValueError("units must be positive")
    return Config(
        kind=resolved_kind,
        size_mb=whole,
        seed=resolved_seed,
        units=resolved_units,
        counts=derive_counts(resolved_kind, resolved_units, selected),
        target_bytes=target_bytes,
    )
