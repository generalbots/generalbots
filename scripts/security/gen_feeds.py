#!/usr/bin/env python3
"""Generate validating Atom/RSS advisory feeds from the canonical bundle (#2719).

One advisory source (`botlib/.../advisories.json`) produces every channel:
  * security/feeds/advisories.atom / .rss                  (all advisories)
  * security/feeds/product-<slug>.atom / .rss              (per product)
  * security/feeds/head-links.html                         (<head> <link> tags)
  * security/feeds/manifest.json                           (channel inventory)

The bulletin page (a separate static site) includes `head-links.html` verbatim;
`--check-head FILE` proves a rendered <head> actually links every generated
feed. No feed text is hand-maintained: a new advisory only changes the bundle.

Usage:
    python3 scripts/security/gen_feeds.py --out security/feeds
    python3 scripts/security/gen_feeds.py --check-head website/security/index.html
    python3 scripts/security/gen_feeds.py --self-test
Exit codes: 0 clean, 1 failure, 2 usage error.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from email.utils import format_datetime
from pathlib import Path

from advisory_source import (
    advisories_for_product,
    load_bundle,
    product_slugs,
    public_base,
    slugify,
)

ATOM_NS = "http://www.w3.org/2005/Atom"
RSS_VERSION = "2.0"
FEED_LIMIT = 100
TITLE_MAX = 256
SUMMARY_MAX = 4096


class FeedError(ValueError):
    """A feed cannot be generated or validated."""


def _date(value: str) -> datetime:
    try:
        return datetime.strptime(value, "%Y-%m-%d").replace(tzinfo=timezone.utc)
    except (TypeError, ValueError) as error:
        raise FeedError(f"invalid advisory date: {value!r}") from error


def _rfc822(value: datetime) -> str:
    return format_datetime(value)


def _clamp(value: str, limit: int) -> str:
    text = value.strip()
    if not text:
        raise FeedError("empty feed text")
    return text if len(text) <= limit else text[: limit - 1] + "…"


def _advisory_url(base: str, advisory_id: str) -> str:
    # IDs are stable GBSA-YYYY-NNNN tokens from the canonical bundle.
    if not advisory_id.replace("-", "").isalnum():
        raise FeedError(f"unsafe advisory id: {advisory_id!r}")
    return f"{base}/advisories/{advisory_id}"


def _feed_updated(records: list[dict]) -> datetime:
    dates = []
    for record in records:
        dates.append(_date(record["published"]))
        if record.get("updated"):
            dates.append(_date(record["updated"]))
    return max(dates) if dates else datetime(2026, 1, 1, tzinfo=timezone.utc)


def build_atom(records: list[dict], base: str, product_slug: str | None) -> bytes:
    """Return a namespace-correct Atom 1.0 document for the given records."""
    name = "All advisories" if product_slug is None else f"Advisories for {product_slug}"
    feed_id = f"tag:generalbots.org,2026:security/feeds/{product_slug or 'all'}.atom"
    self_url = (
        f"{base}/feeds/product-{product_slug}.atom"
        if product_slug is not None
        else f"{base}/feeds/advisories.atom"
    )
    updated = _feed_updated(records)

    feed = ET.Element(f"{{{ATOM_NS}}}feed")
    ET.SubElement(feed, f"{{{ATOM_NS}}}id").text = feed_id
    ET.SubElement(feed, f"{{{ATOM_NS}}}title").text = _clamp(name, TITLE_MAX)
    ET.SubElement(feed, f"{{{ATOM_NS}}}updated").text = updated.isoformat()
    author = ET.SubElement(feed, f"{{{ATOM_NS}}}author")
    ET.SubElement(author, f"{{{ATOM_NS}}}name").text = "General Bots Security"
    ET.SubElement(feed, f"{{{ATOM_NS}}}link", {"rel": "self", "href": self_url})
    ET.SubElement(feed, f"{{{ATOM_NS}}}link", {"rel": "alternate", "href": base})

    for record in records[:FEED_LIMIT]:
        entry = ET.SubElement(feed, f"{{{ATOM_NS}}}entry")
        url = _advisory_url(base, record["id"])
        ET.SubElement(entry, f"{{{ATOM_NS}}}id").text = (
            f"tag:generalbots.org,2026:advisory/{record['id']}"
        )
        ET.SubElement(entry, f"{{{ATOM_NS}}}title").text = _clamp(record["title"], TITLE_MAX)
        published = _date(record["published"])
        ET.SubElement(entry, f"{{{ATOM_NS}}}published").text = published.isoformat()
        ET.SubElement(entry, f"{{{ATOM_NS}}}updated").text = (
            _date(record["updated"]).isoformat() if record.get("updated") else published.isoformat()
        )
        ET.SubElement(entry, f"{{{ATOM_NS}}}link", {"rel": "alternate", "href": url})
        summary = record.get("summary") or record["title"]
        ET.SubElement(entry, f"{{{ATOM_NS}}}summary").text = _clamp(summary, SUMMARY_MAX)
        ET.SubElement(entry, f"{{{ATOM_NS}}}category", {"term": record["severity"]})
        ET.SubElement(entry, f"{{{ATOM_NS}}}category", {"term": record["product"]})

    ET.register_namespace("", ATOM_NS)
    return ET.tostring(feed, encoding="utf-8", xml_declaration=True)


def build_rss(records: list[dict], base: str, product_slug: str | None) -> bytes:
    """Return a well-formed RSS 2.0 document for the given records."""
    name = "General Bots security advisories" if product_slug is None else (
        f"General Bots security advisories: {product_slug}"
    )
    channel_url = base if product_slug is None else f"{base}/feeds/product-{product_slug}.rss"
    rss = ET.Element("rss", {"version": RSS_VERSION})
    channel = ET.SubElement(rss, "channel")
    ET.SubElement(channel, "title").text = _clamp(name, TITLE_MAX)
    ET.SubElement(channel, "link").text = channel_url
    ET.SubElement(channel, "description").text = _clamp(
        f"Security advisories generated from the canonical advisory source ({name}).", SUMMARY_MAX
    )
    ET.SubElement(channel, "lastBuildDate").text = _rfc822(_feed_updated(records))
    ET.SubElement(channel, "generator").text = "generalbots/gen_feeds 1.0"

    for record in records[:FEED_LIMIT]:
        item = ET.SubElement(channel, "item")
        ET.SubElement(item, "title").text = _clamp(record["title"], TITLE_MAX)
        ET.SubElement(item, "link").text = _advisory_url(base, record["id"])
        ET.SubElement(item, "guid", {"isPermaLink": "false"}).text = record["id"]
        ET.SubElement(item, "pubDate").text = _rfc822(_date(record["published"]))
        summary = record.get("summary") or record["title"]
        ET.SubElement(item, "description").text = _clamp(summary, SUMMARY_MAX)
        ET.SubElement(item, "category").text = record["severity"]
        ET.SubElement(item, "category").text = record["product"]

    return ET.tostring(rss, encoding="utf-8", xml_declaration=True)


def feed_files(base: str, bundle: dict) -> dict[str, bytes]:
    """Every generated feed artifact, keyed by relative file name."""
    files: dict[str, bytes] = {}
    all_records = advisories_for_product(bundle, None)
    files["advisories.atom"] = build_atom(all_records, base, None)
    files["advisories.rss"] = build_rss(all_records, base, None)
    for slug in product_slugs(bundle):
        records = advisories_for_product(bundle, slug)
        files[f"product-{slug}.atom"] = build_atom(records, base, slug)
        files[f"product-{slug}.rss"] = build_rss(records, base, slug)
    return files


def head_links(files: dict[str, bytes], base: str) -> str:
    """The exact <link> tags the bulletin <head> must contain."""
    lines = ["<!-- #2719: include these tags verbatim in the bulletin page <head> -->"]
    for name in sorted(files):
        mime = "application/atom+xml" if name.endswith(".atom") else "application/rss+xml"
        lines.append(
            f'<link rel="alternate" type="{mime}" title="General Bots advisories ({name})" '
            f'href="{base}/feeds/{name}" />'
        )
    return "\n".join(lines) + "\n"


def check_head(head_html: str, files: dict[str, bytes], base: str) -> None:
    """Fail unless the rendered <head> links every generated feed."""
    for name in files:
        needle = f'href="{base}/feeds/{name}"'
        if needle not in head_html:
            raise FeedError(f"bulletin <head> does not link {name}")


def generate(out: Path, bundle: dict, base: str) -> list[str]:
    """Write all feed artifacts atomically and return their relative names."""
    files = feed_files(base, bundle)
    # Validate before writing anything (fail closed: never publish half a feed).
    for name, body in files.items():
        ET.fromstring(body)
    out.mkdir(parents=True, exist_ok=True)
    for name, body in files.items():
        target = out / name
        temp = target.with_suffix(target.suffix + ".tmp")
        temp.write_bytes(body)
        temp.replace(target)
    links = head_links(files, base)
    (out / "head-links.html").write_text(links, encoding="utf-8")
    manifest = {
        "schema": 1,
        "source": "botlib/src/security/advisory/data/advisories.json",
        "feeds": sorted(files),
        "counts": {name: len(advisories_for_product(bundle, _feed_slug(name))) for name in files},
    }
    (out / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return sorted([*files, "head-links.html", "manifest.json"])


def _feed_slug(name: str) -> str | None:
    if name.startswith("product-"):
        return name[len("product-"):].rsplit(".", 1)[0]
    return None


def self_test() -> int:
    """Offline proof: validity, per-product isolation, ordering and head links."""
    try:
        with tempfile.TemporaryDirectory(prefix="gb-feeds-") as temp:
            out = Path(temp) / "feeds"
            base = "https://generalbots.org/security"
            bundle = load_bundle()
            names = generate(out, bundle, base)
            for name in names:
                if name.endswith((".atom", ".rss")):
                    root = ET.parse(out / name).getroot()
                    if root.tag not in ("{http://www.w3.org/2005/Atom}feed", "rss"):
                        raise FeedError(f"{name} has the wrong root element")
            atom = ET.parse(out / "advisories.atom").getroot()
            author = atom.find(f"{{{ATOM_NS}}}author/{{{ATOM_NS}}}name")
            if author is None or not author.text:
                raise FeedError("Atom feed has no author name")
            entries = atom.findall("{http://www.w3.org/2005/Atom}entry")
            if len(entries) != len(bundle["advisories"]):
                raise FeedError("all-feed entry count differs from the canonical bundle")
            ids = [entry.findtext("{http://www.w3.org/2005/Atom}id") for entry in entries]
            if len(set(ids)) != len(ids):
                raise FeedError("duplicate entry ids")
            for slug in product_slugs(bundle):
                product_atom = ET.parse(out / f"product-{slug}.atom").getroot()
                for entry in product_atom.findall("{http://www.w3.org/2005/Atom}entry"):
                    terms = [item.get("term") for item in entry.findall(
                        f"{{{ATOM_NS}}}category")]
                    # The last category is always the product of the record.
                    if not terms or not terms[-1] or slugify(terms[-1]) != slug:
                        raise FeedError(f"product feed {slug} leaked an unrelated advisory")
            links = (out / "head-links.html").read_text(encoding="utf-8")
            feed_names = set(feed_files(base, bundle))
            check_head(links, {name: b"" for name in feed_names}, base)
            try:
                check_head("<head></head>", {name: b"" for name in feed_names}, base)
            except FeedError:
                pass
            else:
                raise FeedError("empty head was accepted")
            again = feed_files(base, bundle)
            for name, body in again.items():
                if body != (out / name).read_bytes():
                    raise FeedError(f"{name} is not deterministic")
    except (FeedError, ValueError, OSError) as error:
        print(f"gen_feeds self-test FAILED: {error}", file=sys.stderr)
        return 1
    print(f"gen_feeds self-test: ok ({len(names)} artifacts)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, help="directory for generated feed artifacts")
    parser.add_argument("--check-head", type=Path, help="rendered bulletin page to validate")
    parser.add_argument("--base", default=public_base(), help="public bulletin origin")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.self_test:
            return self_test()
        bundle = load_bundle()
        files = feed_files(args.base, bundle)
        if args.check_head:
            check_head(args.check_head.read_text(encoding="utf-8"), files, args.base)
            print(f"head links: ok ({len(files)} feeds referenced)")
            return 0
        if args.out is None:
            parser.error("--out or --check-head or --self-test is required")
        written = generate(args.out, bundle, args.base)
        print(f"generated {len(written)} feed artifacts in {args.out}")
        return 0
    except (FeedError, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
