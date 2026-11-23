#!/usr/bin/env python3
"""Shared constants and helpers for the CVE JSON 5 generator (#2715).

Keeping the ID shapes and error type in one place makes it impossible for the
generator, the profile validator and their self-tests to disagree about what a
CNA-assigned CVE ID looks like.
"""

from __future__ import annotations

import re
from datetime import datetime
from typing import Any, Iterator

DATA_VERSION = "5.1"
CVE_RE = re.compile(r"^CVE-[0-9]{4}-[0-9]{4,19}$")
GBSA_RE = re.compile(r"^GBSA-[0-9]{4}-[0-9]{4}$")
UUID_RE = re.compile(
    r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"
)
SHORT_NAME_RE = re.compile(r"^[A-Za-z0-9._-]{2,32}$")
CWE_RE = re.compile(r"^CWE-[1-9][0-9]*$")
TIMESTAMP_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?$"
)
CREDIT_TYPES = {
    "reporter": "reporter", "researcher": "finder", "internal": "finder",
    "analyst": "analyst", "coordinator": "coordinator",
}


class GenerationError(Exception):
    """Input cannot produce a valid, CNA-backed CVE record."""


def valid_timestamp(value: Any) -> bool:
    """True for an RFC 3339 timestamp with a real calendar date."""
    if not isinstance(value, str) or not TIMESTAMP_RE.match(value):
        return False
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return False
    return True


def walk(value: Any, path: str = "<root>") -> Iterator[tuple[str, Any]]:
    """Yield (dotted path, scalar) for every scalar in a nested record."""
    if isinstance(value, dict):
        for key, item in value.items():
            yield from walk(item, f"{path}.{key}")
    elif isinstance(value, list):
        for index, item in enumerate(value):
            yield from walk(item, f"{path}[{index}]")
    else:
        yield path, value
