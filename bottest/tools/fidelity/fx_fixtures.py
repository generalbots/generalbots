#!/usr/bin/env python3
"""Synthetic OOXML fixtures for the fidelity self-test (#2457).

Builds structurally OOXML-like DOCX/PPTX/XLSX ZIP packages from integer specs.
They are *not* valid for Word/Excel: elements are placed directly under the
root for deterministic counting. Every ZIP member is ``ZIP_STORED`` with the
DOS epoch and ``create_system=0``, so the bytes are reproducible on any Python
3 and OS. No real text, tenant name or path enters the fixtures.
"""

from __future__ import annotations

import io
import zipfile
from pathlib import Path

W_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
P_NS = "http://schemas.openxmlformats.org/presentationml/2006/main"
A_NS = "http://schemas.openxmlformats.org/drawingml/2006/main"
C_NS = "http://schemas.openxmlformats.org/drawingml/2006/chart"
S_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
ZIP_EPOCH = (1980, 1, 1, 0, 0, 0)

# (relative path, kind, spec). Shared by all three corpora.
ORIGINAL_SPECS: dict[str, tuple[str, dict[str, int]]] = {
    "doc_a.docx": ("docx", {
        "paragraphs": 6, "tables": 1, "text_tokens": 12, "bold": 3,
        "italic": 1, "underline": 0, "hyperlinks": 2, "images": 1,
        "charts": 1, "footnotes": 1, "tracked_changes": 2,
        "comments": 1, "numbering": 2,
    }),
    "slides_b.pptx": ("pptx", {
        "slides": 3, "shapes": 9, "tables": 1, "text_tokens": 15,
        "bold": 4, "italic": 2, "underline": 1, "images": 2,
        "charts": 1, "masters": 1,
    }),
    "nested/sheet_c.xlsx": ("xlsx", {
        "sheets": 2, "rows": 10, "cells": 60, "formulas": 5,
        "shared_strings": 8, "tables": 1, "text_tokens": 20,
        "bold": 2, "italic": 0, "underline": 0, "comments": 1,
    }),
}

# Degraded round-trip: charts flattened/dropped, formulas lost, one text token
# changed. This is exactly the silent degradation #2457 wants surfaced.
DEGRADED_SPECS: dict[str, tuple[str, dict[str, int]]] = {
    "doc_a.docx": ("docx", {**ORIGINAL_SPECS["doc_a.docx"][1], "charts": 0}),
    "slides_b.pptx": ("pptx", {**ORIGINAL_SPECS["slides_b.pptx"][1], "charts": 0}),
    "nested/sheet_c.xlsx": ("xlsx", {
        **ORIGINAL_SPECS["nested/sheet_c.xlsx"][1],
        "formulas": 0, "text_tokens": 19,
    }),
}
EXTRA_SPECS: dict[str, tuple[str, dict[str, int]]] = {
    "orphan.xlsx": ("xlsx", {
        "sheets": 1, "rows": 2, "cells": 4, "formulas": 0,
        "shared_strings": 0, "tables": 0, "text_tokens": 2,
        "bold": 0, "italic": 0, "underline": 0, "comments": 0,
    }),
}


def _rep(tag: str, count: int) -> str:
    return f"<{tag}/>" * count


def _zip(parts: list[tuple[str, str]]) -> bytes:
    """Pack XML parts into a deterministic stored ZIP held in memory."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, xml in sorted(parts):
            info = zipfile.ZipInfo(name, date_time=ZIP_EPOCH)
            info.compress_type = zipfile.ZIP_STORED
            info.external_attr = 0o644 << 16
            info.create_system = 0
            info.create_version = 20
            info.extract_version = 20
            archive.writestr(info, xml.encode("utf-8"))
    return buffer.getvalue()


def _docx(spec: dict[str, int]) -> bytes:
    body: list[str] = [f'<w:document xmlns:w="{W_NS}"><w:body>']
    body.append(_rep("w:p", spec["paragraphs"]))
    body.extend(f"<w:t>tok{i:04d}</w:t>" for i in range(spec["text_tokens"]))
    body.append(_rep("w:tbl", spec["tables"]))
    body.append(_rep("w:b", spec["bold"]))
    body.append(_rep("w:i", spec["italic"]))
    body.append(_rep("w:u", spec["underline"]))
    body.append(_rep("w:hyperlink", spec["hyperlinks"]))
    body.append(_rep("w:drawing", spec["images"]))
    body.append(_rep("w:footnoteReference", spec["footnotes"]))
    ins = spec["tracked_changes"] // 2 + spec["tracked_changes"] % 2
    body.append(_rep("w:ins", ins))
    body.append(_rep("w:del", spec["tracked_changes"] - ins))
    body.append(_rep("w:commentReference", spec["comments"]))
    body.append(_rep("w:numPr", spec["numbering"]))
    body.append("</w:body></w:document>")
    parts = [("word/document.xml", "".join(body)),
             ("word/footnotes.xml", f'<w:footnotes xmlns:w="{W_NS}"/>'),
             ("word/comments.xml", f'<w:comments xmlns:w="{W_NS}"/>')]
    for i in range(spec["charts"]):
        parts.append((f"word/charts/chart{i + 1}.xml",
                      f'<c:chartSpace xmlns:c="{C_NS}"><c:chart/></c:chartSpace>'))
    return _zip(parts)


def _pptx(spec: dict[str, int]) -> bytes:
    parts = [("ppt/presentation.xml", f'<p:presentation xmlns:p="{P_NS}"/>')]
    for i in range(spec["slides"]):
        slide = [f'<p:sld xmlns:p="{P_NS}" xmlns:a="{A_NS}">',
                 "<p:cSld><p:spTree>"]
        if i == 0:
            slide.append(_rep("p:sp", spec["shapes"]))
            slide.append(_rep("a:tbl", spec["tables"]))
            slide.extend(f'<a:rPr b="1"/>' for _ in range(spec["bold"]))
            slide.extend(f'<a:rPr i="1"/>' for _ in range(spec["italic"]))
            slide.extend(f'<a:rPr u="sng"/>' for _ in range(spec["underline"]))
            slide.append(_rep("p:pic", spec["images"]))
            slide.extend(f"<a:t>ptok{j:04d}</a:t>"
                         for j in range(spec["text_tokens"]))
        slide.append("</p:spTree></p:cSld></p:sld>")
        parts.append((f"ppt/slides/slide{i + 1}.xml", "".join(slide)))
    for i in range(spec["masters"]):
        parts.append((f"ppt/slideMasters/slideMaster{i + 1}.xml",
                      f'<p:sldMaster xmlns:p="{P_NS}"/>'))
    for i in range(spec["charts"]):
        parts.append((f"ppt/charts/chart{i + 1}.xml",
                      f'<c:chartSpace xmlns:c="{C_NS}"><c:chart/></c:chartSpace>'))
    return _zip(parts)


def _xlsx(spec: dict[str, int]) -> bytes:
    sheets = _rep("sheet", spec["sheets"])
    parts = [("xl/workbook.xml",
              f'<workbook xmlns="{S_NS}"><sheets>{sheets}</sheets></workbook>')]
    cells: list[str] = []
    for i in range(spec["cells"]):
        if i < spec["formulas"]:
            cells.append("<c><f>1+1</f></c>")
        elif i < spec["formulas"] + spec["text_tokens"]:
            cells.append(f"<c><v>xtok{i:04d}</v></c>")
        else:
            cells.append("<c/>")
    worksheet = (f'<worksheet xmlns="{S_NS}"><sheetData>'
                 + _rep("row", spec["rows"]) + "".join(cells)
                 + "</sheetData></worksheet>")
    parts.append(("xl/worksheets/sheet1.xml", worksheet))
    shared = "".join("<si><t>sstok</t></si>"
                     for _ in range(spec["shared_strings"]))
    parts.append(("xl/sharedStrings.xml",
                  f'<sst xmlns="{S_NS}">{shared}</sst>'))
    font = (f'<font><b/></font>' * spec["bold"]
            + '<font><i/></font>' * spec["italic"]
            + '<font><u/></font>' * spec["underline"])
    parts.append(("xl/styles.xml",
                  f'<styleSheet xmlns="{S_NS}"><fonts>{font}</fonts></styleSheet>'))
    for i in range(spec["tables"]):
        parts.append((f"xl/tables/table{i + 1}.xml", f'<table xmlns="{S_NS}"/>'))
    for i in range(spec["comments"]):
        parts.append((f"xl/comments{i + 1}.xml",
                      f'<comments xmlns="{S_NS}"><comment>note</comment></comments>'))
    return _zip(parts)


BUILDERS = {"docx": _docx, "pptx": _pptx, "xlsx": _xlsx}


def build(kind: str, spec: dict[str, int]) -> bytes:
    """Build one synthetic package for ``kind`` from an integer spec."""
    return BUILDERS[kind](spec)


def write_corpus(root: Path,
                 specs: dict[str, tuple[str, dict[str, int]]]) -> Path:
    """Write a corpus directory tree from a relative-path -> spec mapping."""
    for relpath, (kind, spec) in specs.items():
        target = root / relpath
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(build(kind, spec))
    return root
