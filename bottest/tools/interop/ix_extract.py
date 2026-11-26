#!/usr/bin/env python3
"""Structural feature extraction from OOXML DOCX/PPTX/XLSX (#2463).

Test-only and stdlib-only. A package is opened as a ZIP; a fixed set of XML
parts is parsed with :mod:`xml.etree.ElementTree` and reduced to a deterministic
feature vector:

    headings, tables, images, formulas   -> non-negative integer counts
    fonts                                -> sorted, de-duplicated list of names

Matching is by XML *local name* (the part after ``}``), so the extractor is
namespace-prefix agnostic. Counts are a property of the bytes in each part, so
the result is stable across runs and machines. Only a fixed allow-list of parts
is read; themes, rels and media bytes are intentionally ignored.

The five features are exactly the "structural losses" that #2463's conversion
report must surface: headings, tables, images, formulas and fonts.
"""

from __future__ import annotations

import fnmatch
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path

KINDS = ("docx", "pptx", "xlsx")
FEATURES = ("headings", "tables", "images", "formulas", "fonts")
COUNT_FEATURES = ("headings", "tables", "images", "formulas")

PART_PATTERNS: dict[str, tuple[str, ...]] = {
    "docx": (
        "word/document.xml", "word/styles.xml", "word/fontTable.xml",
    ),
    "pptx": (
        "ppt/slides/slide*.xml", "ppt/slideMasters/*.xml",
        "ppt/theme/*.xml",
    ),
    "xlsx": (
        "xl/workbook.xml", "xl/worksheets/sheet*.xml", "xl/styles.xml",
        "xl/tables/*.xml", "xl/drawings/*.xml",
    ),
}


class ExtractError(Exception):
    """A deterministic, short failure code for an unreadable package."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def kind_for(path: Path | str) -> str:
    """Return the supported OOXML kind from a path suffix, or ''."""
    suffix = Path(path).suffix.lower().lstrip(".")
    return suffix if suffix in KINDS else ""


def local_name(tag: str) -> str:
    """Strip an optional ``{namespace}`` from an ElementTree tag/attribute."""
    return tag.rsplit("}", 1)[-1]


def _attr(element: ET.Element, local: str) -> str:
    for key, value in element.attrib.items():
        if local_name(key) == local:
            return value or ""
    return ""


def iter_local(root: ET.Element | None, name: str):
    """Yield descendants whose local name equals ``name``."""
    if root is None:
        return
    for element in root.iter():
        if local_name(element.tag) == name:
            yield element


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


def _add_font(fonts: set[str], value: str) -> None:
    """Add a concrete font name; ignore empty/theme-reference placeholders."""
    name = value.strip() if isinstance(value, str) else ""
    if name and name[0] not in "+@*":
        fonts.add(name)


def _docx_features(parts: dict[str, ET.Element]) -> dict[str, object]:
    document = parts.get("word/document.xml")
    styles = parts.get("word/styles.xml")
    font_table = parts.get("word/fontTable.xml")
    fonts: set[str] = set()
    for root in (document, styles, font_table):
        for element in root.iter() if root is not None else ():
            tag = local_name(element.tag)
            if tag == "rFonts":
                for key in ("ascii", "hAnsi", "cs", "eastAsia"):
                    _add_font(fonts, _attr(element, key))
            elif tag == "font":
                _add_font(fonts, _attr(element, "name"))
    headings = 0
    for paragraph in iter_local(document, "p"):
        for child in paragraph.iter():
            tag = local_name(child.tag)
            if tag == "pStyle" and _attr(child, "val").lower().startswith("heading"):
                headings += 1
                break
            if tag == "outlineLvl":
                headings += 1
                break
    return {
        "headings": headings,
        "tables": sum(1 for _ in iter_local(document, "tbl")),
        "images": (sum(1 for _ in iter_local(document, "drawing"))
                   + sum(1 for _ in iter_local(document, "pict"))),
        "formulas": (sum(1 for _ in iter_local(document, "oMath"))
                     + sum(1 for _ in iter_local(document, "oleObject"))
                     + sum(1 for _ in iter_local(document, "object"))),
        "fonts": sorted(fonts),
    }


def _pptx_features(parts: dict[str, ET.Element]) -> dict[str, object]:
    roots = [root for name, root in parts.items()
             if "/slides/" in name or "/slideMasters/" in name]
    fonts: set[str] = set()
    headings = 0
    for root in roots:
        for element in root.iter():
            tag = local_name(element.tag)
            if tag in ("latin", "ea", "cs"):
                _add_font(fonts, _attr(element, "typeface"))
            elif tag == "ph" and _attr(element, "type").lower() in (
                    "title", "ctrtitle"):
                headings += 1
    return {
        "headings": headings,
        "tables": sum(1 for _ in iter_local_from(roots, "tbl")),
        "images": sum(1 for _ in iter_local_from(roots, "pic")),
        "formulas": (sum(1 for _ in iter_local_from(roots, "oleObj"))
                     + sum(1 for _ in iter_local_from(roots, "oleObject"))),
        "fonts": sorted(fonts),
    }


def _xlsx_features(parts: dict[str, ET.Element]) -> dict[str, object]:
    worksheets = [root for name, root in parts.items()
                  if "/worksheets/" in name]
    tables = [root for name, root in parts.items() if "/tables/" in name]
    drawings = [root for name, root in parts.items() if "/drawings/" in name]
    styles = parts.get("xl/styles.xml")
    fonts: set[str] = set()
    for font in iter_local(styles, "font"):
        for name_element in iter_local(font, "name"):
            _add_font(fonts, _attr(name_element, "val"))
    return {
        "headings": 0,
        "tables": sum(1 for _ in iter_local_from(tables, "table")),
        "images": sum(1 for _ in iter_local_from(drawings, "pic")),
        "formulas": sum(1 for _ in iter_local_from(worksheets, "f")),
        "fonts": sorted(fonts),
    }


def iter_local_from(roots: list[ET.Element], name: str):
    """Yield local-name matches from several parsed roots."""
    for root in roots:
        yield from iter_local(root, name)


def extract(path: Path | str) -> dict[str, object]:
    """Return the feature vector for one OOXML package at ``path``.

    Result keys are :data:`FEATURES`. Raises :class:`ExtractError` on a bad
    package; callers decide how to surface that failure.
    """
    file = Path(path)
    kind = kind_for(file)
    if not kind:
        raise ExtractError("unknown-kind")
    parts = _load_parts(file, kind)
    if kind == "docx":
        features = _docx_features(parts)
    elif kind == "pptx":
        features = _pptx_features(parts)
    else:
        features = _xlsx_features(parts)
    return {"kind": kind, "features": features}
