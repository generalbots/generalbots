#!/usr/bin/env python3
"""Static font inventory and metric-compatible fallback decisions (#2463).

Test-only and stdlib-only. This module never reads a file, opens a socket or
starts a server: it answers one question from a font name alone -- "what will
GB do when it renders this font if it is not installed?"

The decision is deliberately machine-readable and total: every input font maps
to exactly one record with a stable ``status``, ``strategy`` and ``reason``.
The three metric-compatible pairs below are the well-known Croscore substitutions
(Calibri/Carlito, Cambria/Caladea, Arial/Liberation Sans, ...). A substitution
is only called ``metric-compatible`` when a same-metrics replacement exists in
:data:`AVAILABLE_FONTS`; a visually-similar but metrically different mapping is
reported as ``approximate`` (still warned, but not claimed as metric-compatible).
Symbol fonts have no text-compatible replacement and are reported ``unresolved``.

No WOPI/Office Online or Google import is implemented here: those need a live
server and are explicitly out of scope for this offline slice.
"""

from __future__ import annotations

# Fonts GB ships / can render without substitution (case-insensitive lookup).
AVAILABLE_FONTS: tuple[str, ...] = (
    "Carlito",
    "Caladea",
    "Liberation Sans",
    "Liberation Serif",
    "Liberation Mono",
    "Liberation Sans Narrow",
    "DejaVu Sans",
    "DejaVu Serif",
    "DejaVu Sans Mono",
    "Comic Neue",
    "Noto Sans",
    "Noto Serif",
    "Noto Sans Mono",
)

# Proprietary font -> metric-compatible replacement (same advance widths).
METRIC_COMPATIBLE: dict[str, str] = {
    "calibri": "Carlito",
    "cambria": "Caladea",
    "arial": "Liberation Sans",
    "helvetica": "Liberation Sans",
    "times new roman": "Liberation Serif",
    "times": "Liberation Serif",
    "courier new": "Liberation Mono",
    "courier": "Liberation Mono",
    "arial narrow": "Liberation Sans Narrow",
}

# Visually similar but NOT metric-compatible (warn; do not claim compatibility).
APPROXIMATE: dict[str, str] = {
    "verdana": "DejaVu Sans",
    "tahoma": "DejaVu Sans",
    "trebuchet ms": "DejaVu Sans",
    "georgia": "Liberation Serif",
    "comic sans ms": "Comic Neue",
}

# Symbol/pictograph fonts: no metric-compatible text replacement exists.
NO_METRIC_EQUIVALENT: tuple[str, ...] = (
    "symbol",
    "wingdings",
    "webdings",
    "wingdings 2",
    "wingdings 3",
    "ms gothic",
)

_AVAILABLE_CF = frozenset(name.casefold() for name in AVAILABLE_FONTS)
_NO_EQUIVALENT_CF = frozenset(name.casefold() for name in NO_METRIC_EQUIVALENT)

# A fallback is only usable if GB actually ships it.
_FALLBACK_VALID = {
    key: value for key, value in METRIC_COMPATIBLE.items()
    if value.casefold() in _AVAILABLE_CF
}
_APPROX_VALID = {
    key: value for key, value in APPROXIMATE.items()
    if value.casefold() in _AVAILABLE_CF
}


def normalize(name: str) -> str:
    """Return a trimmed font name, or '' for non/theme-reference names."""
    if not isinstance(name, str):
        return ""
    value = name.strip()
    if not value or value[0] in "+@*":
        return ""
    return value


def decide(name: str) -> dict[str, object]:
    """Return the deterministic render decision for one font name.

    Keys (all present in every record):

    ``font``       original, trimmed name
    ``status``     available | substituted | unresolved
    ``fallback``   concrete replacement name, or None
    ``strategy``   none | metric-compatible | approximate
    ``warning``    True when the user must be warned
    ``compatible`` True only for a genuine metric-compatible substitution
    ``reason``     stable short code for the decision
    """
    font = normalize(name)
    key = font.casefold()
    if not font:
        return {
            "font": "", "status": "unresolved", "fallback": None,
            "strategy": "none", "warning": False, "compatible": False,
            "reason": "empty-font-name",
        }
    if key in _AVAILABLE_CF:
        return {
            "font": font, "status": "available", "fallback": None,
            "strategy": "none", "warning": False, "compatible": False,
            "reason": "in-inventory",
        }
    if key in _NO_EQUIVALENT_CF:
        return {
            "font": font, "status": "unresolved", "fallback": None,
            "strategy": "none", "warning": True, "compatible": False,
            "reason": "no-metric-equivalent",
        }
    if key in _FALLBACK_VALID:
        return {
            "font": font, "status": "substituted",
            "fallback": _FALLBACK_VALID[key],
            "strategy": "metric-compatible", "warning": True,
            "compatible": True, "reason": "metric-compatible-mapping",
        }
    if key in _APPROX_VALID:
        return {
            "font": font, "status": "substituted",
            "fallback": _APPROX_VALID[key],
            "strategy": "approximate", "warning": True,
            "compatible": False, "reason": "approximate-mapping",
        }
    return {
        "font": font, "status": "unresolved", "fallback": None,
        "strategy": "none", "warning": True, "compatible": False,
        "reason": "no-mapping",
    }


def decide_all(fonts: list[str] | tuple[str, ...]) -> list[dict[str, object]]:
    """Return one :func:`decide` record per unique font, sorted by font name."""
    seen = sorted({normalize(font) for font in fonts} - {""})
    return [decide(font) for font in seen]
