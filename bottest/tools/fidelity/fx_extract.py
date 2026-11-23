#!/usr/bin/env python3
"""Structural feature extraction from OOXML DOCX/PPTX/XLSX packages (#2457).

Test-only, stdlib-only. A package is read as a ZIP; a fixed set of XML parts is
parsed with :mod:`xml.etree.ElementTree` and reduced to a deterministic feature
vector: integer counts for structural elements and a normalised text signature.

Matching is by XML *local name* (the part after ``}``), so the extractor is
namespace-prefix agnostic: ``<w:p>``, ``<p:p>`` and an unprefixed ``<p>`` all
count the same. Elements are counted in document order, which is a property of
the bytes in each part, so the result is stable across runs and machines.

Formatting markers that drawingml expresses as attributes (PPTX ``b="1"``,
``i="1"``, ``u="sng"``) are counted by attribute local name/value as well.

This module never writes to disk and never opens a network connection.
"""

from __future__ import annotations

import fnmatch
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path

KINDS = ("docx", "pptx", "xlsx")

# Ordered, fixed feature schema. (name, type) pairs drive both extraction and
# reporting; adding a feature means adding it here and in extract().
FEATURES: dict[str, tuple[tuple[str, str], ...]] = {
    "docx": (
        ("paragraphs", "count"), ("tables", "count"), ("text", "text"),
        ("bold", "count"), ("italic", "count"), ("underline", "count"),
        ("hyperlinks", "count"), ("images", "count"), ("charts", "count"),
        ("footnotes", "count"), ("tracked_changes", "count"),
        ("comments", "count"), ("numbering", "count"),
    ),
    "pptx": (
        ("slides", "count"), ("shapes", "count"), ("tables", "count"),
        ("text", "text"), ("bold", "count"), ("italic", "count"),
        ("underline", "count"), ("images", "count"), ("charts", "count"),
        ("masters", "count"),
    ),
    "xlsx": (
        ("sheets", "count"), ("rows", "count"), ("cells", "count"),
        ("formulas", "count"), ("shared_strings", "count"), ("tables", "count"),
        ("text", "text"), ("bold", "count"), ("italic", "count"),
        ("underline", "count"), ("comments", "count"),
    ),
}

# Only these parts are read. Anything else in the package (themes, layouts,
# media, rels, custom XML) is ignored by design: the harness scores structure,
# not byte-for-byte equality -- that is covered by office_fidelity.rs.
PART_PATTERNS: dict[str, tuple[str, ...]] = {
    "docx": (
        "word/document.xml", "word/footnotes.xml", "word/comments.xml",
        "word/charts/chart*.xml",
    ),
    "pptx": (
        "ppt/slides/slide*.xml", "ppt/slideMasters/*.xml",
        "ppt/notesSlides/*.xml", "ppt/charts/chart*.xml",
    ),
    "xlsx": (
        "xl/workbook.xml", "xl/worksheets/sheet*.xml", "xl/sharedStrings.xml",
        "xl/tables/*.xml", "xl/comments*.xml", "xl/styles.xml",
    ),
}


class ExtractError(Exception):
    """A deterministic, short failure code for an unreadable package."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def kind_for(path: Path | str) -> str:
    """Return the OOXML kind from a path suffix, or an empty string."""
    suffix = Path(path).suffix.lower().lstrip(".")
    return suffix if suffix in KINDS else ""


def local_name(tag: str) -> str:
    """Strip an optional ``{namespace}`` from an ElementTree tag/attribute."""
    return tag.rsplit("}", 1)[-1]


def _load_parts(path: Path, kind: str) -> dict[str, ET.Element]:
    """Parse the relevant XML parts of ``path`` into a name -> root map."""
    if kind not in PART_PATTERNS:
        raise ExtractError("unknown-kind")
    try:
        with zipfile.ZipFile(path) as archive:
            names = sorted(
                name for name in archive.namelist()
                if any(fnmatch.fnmatchcase(name, pattern)
                       for pattern in PART_PATTERNS[kind])
            )
            parts: dict[str, ET.Element] = {}
            for name in names:
                try:
                    parts[name] = ET.fromstring(archive.read(name))
                except ET.ParseError as exc:
                    raise ExtractError(f"bad-xml:{name}") from exc
            return parts
    except zipfile.BadZipFile as exc:
        raise ExtractError("not-a-zip") from exc
    except OSError as exc:
        raise ExtractError("io-error") from exc


def count_local(root: ET.Element | None, name: str) -> int:
    """Count descendant elements whose local name equals ``name``."""
    if root is None:
        return 0
    return sum(1 for element in root.iter()
               if local_name(element.tag) == name)


def count_parts(parts: dict[str, ET.Element], name: str) -> int:
    """Count an element local name across a map of parsed parts."""
    return sum(count_local(root, name) for root in parts.values())


def count_attr_value(parts: dict[str, ET.Element], attr: str, value: str) -> int:
    """Count elements with an attribute (local name ``attr``) equal to value."""
    total = 0
    for root in parts.values():
        for element in root.iter():
            if any(local_name(key) == attr and val == value
                   for key, val in element.attrib.items()):
                total += 1
    return total


def count_attr_truthy(parts: dict[str, ET.Element], attr: str) -> int:
    """Count elements with a truthy attribute of the given local name."""
    total = 0
    falsey = {"", "0", "none", "false", "off"}
    for root in parts.values():
        for element in root.iter():
            if any(local_name(key) == attr and val.strip().lower() not in falsey
                   for key, val in element.attrib.items()):
                total += 1
    return total


def gather_text(parts: dict[str, ET.Element], tags: set[str]) -> str:
    """Collect and normalise text from elements whose local name is in tags."""
    tokens: list[str] = []
    for root in parts.values():
        for element in root.iter():
            if local_name(element.tag) in tags and element.text:
                tokens.append(element.text)
    return " ".join(" ".join(tokens).split())


def _subset(parts: dict[str, ET.Element], pattern: str) -> dict[str, ET.Element]:
    return {name: root for name, root in parts.items()
            if fnmatch.fnmatchcase(name, pattern)}


def extract(path: Path, kind: str) -> dict[str, object]:
    """Return the feature vector for one package.

    Keys are the feature names from :data:`FEATURES` for ``kind``; count
    values are ``int`` and the ``text`` value is a normalised ``str``.
    """
    parts = _load_parts(path, kind)

    if kind == "docx":
        if "word/document.xml" not in parts:
            raise ExtractError("missing-part:word/document.xml")
        document = parts["word/document.xml"]
        charts = {name: root for name, root in parts.items()
                  if "/charts/" in name}
        return {
            "paragraphs": count_local(document, "p"),
            "tables": count_local(document, "tbl"),
            "text": gather_text(parts, {"t"}),
            "bold": count_local(document, "b"),
            "italic": count_local(document, "i"),
            "underline": count_local(document, "u"),
            "hyperlinks": count_local(document, "hyperlink"),
            "images": count_local(document, "drawing"),
            "charts": count_parts(charts, "chart"),
            "footnotes": count_local(document, "footnoteReference"),
            "tracked_changes": (count_local(document, "ins")
                                + count_local(document, "del")),
            "comments": count_local(document, "commentReference"),
            "numbering": count_local(document, "numPr"),
        }

    if kind == "pptx":
        slides = _subset(parts, "ppt/slides/slide*.xml")
        masters = {name: root for name, root in parts.items()
                   if "/slideMasters/" in name}
        charts = {name: root for name, root in parts.items()
                  if "/charts/" in name}
        return {
            "slides": len(slides),
            "shapes": count_parts(slides, "sp"),
            "tables": count_parts(slides, "tbl"),
            "text": gather_text(slides, {"t"}),
            "bold": count_attr_value(slides, "b", "1"),
            "italic": count_attr_value(slides, "i", "1"),
            "underline": count_attr_truthy(slides, "u"),
            "images": count_parts(slides, "pic"),
            "charts": count_parts(charts, "chart"),
            "masters": len(masters),
        }

    if kind == "xlsx":
        workbook = parts.get("xl/workbook.xml")
        worksheets = {name: root for name, root in parts.items()
                      if "/worksheets/" in name}
        tables = {name: root for name, root in parts.items()
                  if "/tables/" in name}
        comments = _subset(parts, "xl/comments*.xml")
        text_parts: dict[str, ET.Element] = {}
        text_parts.update(worksheets)
        if "xl/sharedStrings.xml" in parts:
            text_parts["xl/sharedStrings.xml"] = parts["xl/sharedStrings.xml"]
        text_parts.update(comments)
        return {
            "sheets": count_local(workbook, "sheet"),
            "rows": count_parts(worksheets, "row"),
            "cells": count_parts(worksheets, "c"),
            "formulas": count_parts(worksheets, "f"),
            "shared_strings": count_local(parts.get("xl/sharedStrings.xml"), "si"),
            "tables": count_parts(tables, "table"),
            "text": gather_text(text_parts, {"t", "v"}),
            "bold": count_local(parts.get("xl/styles.xml"), "b"),
            "italic": count_local(parts.get("xl/styles.xml"), "i"),
            "underline": count_local(parts.get("xl/styles.xml"), "u"),
            "comments": count_parts(comments, "comment"),
        }

    raise ExtractError("unknown-kind")
