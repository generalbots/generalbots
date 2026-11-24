#!/usr/bin/env python3
"""Optional full validation of a CVE JSON 5 record against the official schema.

The CVE programme publishes the record format as a draft-07 JSON Schema with
local ``#/definitions`` references and relative ``file:imports/...`` references
to the FIRST CVSS schemas. This module rewrites only those relative references
to absolute local file URIs, so validation runs against an unmodified official
schema tree with no network access.

Usage (normally called by cve_json5.py, not directly):
    from cve_json5_schema import validate_against_schema
    ok, errors, skipped = validate_against_schema(record, Path("CVE_Record_Format.json"))

``skipped`` is True when the optional ``jsonschema`` dependency is unavailable.
The caller must then report the skip explicitly; it is never a silent pass.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def load_schema(schema_path: Path) -> dict[str, Any]:
    """Load the official schema, resolving its relative CVSS imports locally."""
    text = schema_path.read_text(encoding="utf-8")
    imports_uri = (schema_path.parent / "imports").resolve().as_uri()
    text = text.replace('"file:imports/', f'"{imports_uri}/')
    schema = json.loads(text)
    if not isinstance(schema, dict):
        raise ValueError(f"{schema_path}: CVE schema must be a JSON object")
    return schema


def validate_against_schema(record: dict[str, Any],
                            schema_path: Path) -> tuple[bool, list[str], bool]:
    """Return (passed, errors, skipped). Never raises for bad records."""
    try:
        from jsonschema import Draft7Validator
    except ImportError:
        return False, [], True

    try:
        schema = load_schema(schema_path)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        return False, [f"{schema_path}: cannot load official schema: {error}"], False

    validator = Draft7Validator(schema)
    errors: list[str] = []
    for error in validator.iter_errors(record):
        location = "/".join(str(part) for part in error.absolute_path) or "<root>"
        errors.append(f"{location}: {error.message}")
    return not errors, errors, False
