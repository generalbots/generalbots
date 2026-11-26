#!/usr/bin/env python3
"""Deterministic synthetic OOXML fixtures for the interop self-test (#2463).

Test-only and stdlib-only. Builds minimal DOCX/PPTX/XLSX packages in memory
with a controlled feature mix (headings, tables, images, formulas and an
explicit font list). The same arguments always produce byte-identical ZIPs:
members are `ZIP_STORED`, sorted by name, with a fixed DOS timestamp and fixed
creator fields. No real document text, tenant name or client data is embedded.

These fixtures are intentionally structural: they exercise the extractor and
the conversion-report gate, not Word/Excel rendering.
"""

from __future__ import annotations

import io
import zipfile
from typing import Iterable


def _zip(members: dict[str, bytes | str]) -> bytes:
    """Write a deterministic, uncompressed ZIP from a name -> payload map."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
        for name in sorted(members):
            payload = members[name]
            data = payload.encode("utf-8") if isinstance(payload, str) else payload
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 0
            info.external_attr = 0
            archive.writestr(info, data)
    return buffer.getvalue()


def _declare(namespaces: Iterable[str]) -> str:
    return " ".join(f'xmlns:{prefix}="{uri}"' for prefix, uri in namespaces)


def build_docx(headings: int, tables: int, images: int, formulas: int,
               fonts: Iterable[str]) -> bytes:
    """Return a DOCX with the requested structural feature counts."""
    font_list = list(fonts)
    primary = font_list[0] if font_list else "Calibri"
    ns = _declare((
        ("w", "http://schemas.openxmlformats.org/wordprocessingml/2006/main"),
        ("m", "http://schemas.openxmlformats.org/officeDocument/2006/math"),
    ))
    body: list[str] = []
    for index in range(headings):
        body.append(
            f'<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr>'
            f'<w:r><w:rPr><w:rFonts w:ascii="{primary}" '
            f'w:hAnsi="{primary}"/></w:rPr><w:t>H{index}</w:t></w:r></w:p>'
        )
    for _ in range(tables):
        body.append("<w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>")
    for _ in range(images):
        body.append("<w:p><w:r><w:drawing/></w:r></w:p>")
    for _ in range(formulas):
        body.append("<w:p><m:oMath><m:r><m:t>x</m:t></m:r></m:oMath></w:p>")
    font_table = "".join(
        f'<w:font w:name="{name}"/>' for name in font_list
    )
    members: dict[str, bytes | str] = {
        "word/document.xml": f'<?xml version="1.0"?><w:document {ns}>'
                             f'<w:body>{"".join(body)}</w:body></w:document>',
        "word/styles.xml": f'<w:styles {ns}/>',
        "word/fontTable.xml": f'<w:fonts {ns}>{font_table}</w:fonts>',
    }
    return _zip(members)


def build_pptx(headings: int, tables: int, images: int, formulas: int,
               fonts: Iterable[str]) -> bytes:
    """Return a PPTX with one slide and the requested feature counts."""
    font_list = list(fonts)
    ns = _declare((
        ("p", "http://schemas.openxmlformats.org/presentationml/2006/main"),
        ("a", "http://schemas.openxmlformats.org/drawingml/2006/main"),
    ))
    shapes: list[str] = []
    for _ in range(headings):
        shapes.append(
            '<p:sp><p:nvSpPr><p:nvPr><p:ph type="title"/>'
            '</p:nvPr></p:nvSpPr></p:sp>'
        )
    for _ in range(tables):
        shapes.append("<p:graphicFrame><a:tbl><a:tr><a:tc/></a:tr>"
                      "</a:tbl></p:graphicFrame>")
    for _ in range(images):
        shapes.append("<p:pic/>")
    for _ in range(formulas):
        shapes.append("<p:graphicFrame><p:oleObj/></p:graphicFrame>")
    for name in font_list:
        shapes.append(f'<p:sp><a:rPr><a:latin typeface="{name}"/>'
                      "</a:rPr></p:sp>")
    slide = (f'<?xml version="1.0"?><p:sld {ns}><p:cSld><p:spTree>'
             f'{"".join(shapes)}</p:spTree></p:cSld></p:sld>')
    members: dict[str, bytes | str] = {
        "ppt/slides/slide1.xml": slide,
    }
    return _zip(members)


def build_xlsx(formulas: int, tables: int, images: int,
               fonts: Iterable[str]) -> bytes:
    """Return an XLSX with the requested feature counts (headings = 0)."""
    font_list = list(fonts)
    ns_main = _declare((
        ("m", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
    ))
    ns_draw = _declare((
        ("xdr", "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"),
    ))
    cells = "".join("<row><c><f>1+1</f><v>2</v></c></row>"
                    for _ in range(formulas))
    font_nodes = "".join(
        f'<font><name val="{name}"/></font>' for name in font_list
    )
    members: dict[str, bytes | str] = {
        "xl/workbook.xml": f'<workbook {ns_main}><sheets>'
                           f'<sheet name="Sheet1"/></sheets></workbook>',
        "xl/worksheets/sheet1.xml": f'<worksheet {ns_main}><sheetData>'
                                    f'{cells}</sheetData></worksheet>',
        "xl/styles.xml": f'<styleSheet {ns_main}><fonts>{font_nodes}'
                         "</fonts></styleSheet>",
        "xl/drawings/drawing1.xml": f'<xdr:wsDr {ns_draw}>'
                                    + "<xdr:pic/>" * images
                                    + "</xdr:wsDr>",
    }
    for index in range(tables):
        members[f"xl/tables/table{index + 1}.xml"] = (
            f'<table {ns_main} id="{index + 1}" name="T{index + 1}"/>'
        )
    return _zip(members)


def expected(kind: str, headings: int = 0, tables: int = 0, images: int = 0,
             formulas: int = 0, fonts: Iterable[str] = ()) -> dict[str, object]:
    """Return the feature vector a builder above is expected to extract."""
    return {
        "kind": kind,
        "features": {
            "headings": headings,
            "tables": tables,
            "images": images,
            "formulas": formulas,
            "fonts": sorted(set(fonts)),
        },
    }
