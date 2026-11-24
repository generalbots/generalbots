#!/usr/bin/env python3
"""Load the one canonical advisory bundle for every subscription channel (#2719).

Feeds, the email digest and outbound webhook payloads all call `load_bundle()`.
Nothing here re-authors advisory text: if a field is missing it is because the
canonical record does not carry it, never because a channel invented a copy.
"""

from __future__ import annotations

import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUNDLE_PATH = ROOT / "botlib/src/security/advisory/data/advisories.json"

# RFC 4288/5013-ish token for feed file names and <link> hrefs: lowercase,
# ASCII and no path separators, so a product name can never escape the feed dir.
_SLUG = re.compile(r"[^a-z0-9]+")
_PRODUCT_MAX = 128


class SourceError(ValueError):
    """The canonical bundle cannot be used as a subscription source."""


def slugify(value: str) -> str:
    """Stable, filesystem-safe identifier for a product name."""
    if not isinstance(value, str) or not value.strip():
        raise SourceError("product name is empty")
    slug = _SLUG.sub("-", value.strip().lower()).strip("-")
    if not slug or len(slug) > _PRODUCT_MAX:
        raise SourceError("product name cannot be represented as a slug")
    return slug


def load_bundle(path: Path = BUNDLE_PATH) -> dict:
    """Load and validate the canonical advisory bundle.

    Raises SourceError (never a bare KeyError) so callers fail closed with a
    stable message and no caller text is echoed into an HTTP response.
    """
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as error:
        raise SourceError(f"cannot read canonical bundle: {error}") from error
    try:
        bundle = json.loads(raw)
    except ValueError as error:
        raise SourceError(f"canonical bundle is not JSON: {error}") from error
    if not isinstance(bundle, dict) or not isinstance(bundle.get("advisories"), list):
        raise SourceError("canonical bundle has no advisories array")
    for advisory in bundle["advisories"]:
        if not isinstance(advisory, dict):
            raise SourceError("advisory record is not an object")
        for field in ("id", "title", "severity", "product", "published"):
            if not isinstance(advisory.get(field), str) or not advisory[field]:
                raise SourceError(f"advisory is missing {field}")
        try:
            slugify(advisory["product"])
        except SourceError as error:
            raise SourceError(f"advisory {advisory['id']}: {error}") from error
    return bundle


def advisories_for_product(bundle: dict, product_slug: str | None) -> list[dict]:
    """Return canonical advisories, optionally filtered to one product.

    Sorted newest first by `published` then `id` so every channel presents the
    same order without maintaining a second list.
    """
    records = bundle["advisories"]
    if product_slug is not None:
        records = [record for record in records if slugify(record["product"]) == product_slug]
    return sorted(records, key=lambda record: (record["published"], record["id"]), reverse=True)


def product_slugs(bundle: dict) -> list[str]:
    """Every distinct product present in the canonical bundle, sorted."""
    return sorted({slugify(record["product"]) for record in bundle["advisories"]})


def public_base() -> str:
    """Authoritative bulletin origin from the environment, no hard-coded IP.

    Overridable for staging/mirrors; never read from a request header.
    """
    return os.environ.get("GB_BULLETIN_URL", "https://generalbots.org/security").rstrip("/")
