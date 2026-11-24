#!/usr/bin/env python3
"""Synthetic OOXML / ODS / CSV content builders for ``officestress`` (#2531).

Each instance of a selected feature emits exactly one stable marker:
``<!-- OSF-<FEATURE>-<index> -->`` in XML parts, or an ``OSF-<FEATURE>-<index>``
field for CSV. The manifest records those counts and ``verify_feature_counts``
re-counts the markers from the finished artifact, so a manifest is verifiable
from the file alone rather than merely asserted.

Files are structurally OOXML-like and contain only synthetic text and seeded
bytes. They are built for import/memory stress, not to open in Word/Excel.
"""

from __future__ import annotations

import random
import struct
import zlib

from os_matrix import Config

VOCABULARY = ("alpha", "bravo", "delta", "echo", "foxtrot", "gamma", "lambda", "nova")
MARKER = "SYNTHETIC-OFFICESTRESS-GENERALBOTS-NOT-REAL-DATA"
MARKER_PREFIX = "OSF-"


def marker(feature: str) -> str:
    """Return the per-instance marker prefix for a feature."""
    return f"{MARKER_PREFIX}{feature.upper()}-"


def _rng(config: Config) -> random.Random:
    return random.Random(config.seed)


def _word(rng: random.Random) -> str:
    return VOCABULARY[rng.randrange(len(VOCABULARY))]


def _num(rng: random.Random) -> int:
    return rng.randrange(1, 1_000_000)


def _tag(feature: str, index: int) -> str:
    return f"<!-- {marker(feature)}{index} -->"


def _xml(body: str) -> bytes:
    return ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' + body).encode("utf-8")


def _png(rng: random.Random) -> bytes:
    """Deterministic 1x1 PNG with seeded RGB bytes (a real, parseable image)."""
    raw = b"\x00" + bytes(rng.randrange(256) for _ in range(3))
    ihdr = struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0)

    def chunk(kind: bytes, data: bytes) -> bytes:
        checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def _content_types(overrides: tuple[str, ...]) -> bytes:
    parts = [
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">',
        '<Default Extension="xml" ContentType="application/xml"/>',
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>',
        '<Default Extension="png" ContentType="image/png"/>',
        '<Default Extension="bin" ContentType="application/octet-stream"/>',
    ]
    parts.extend(f'<Override PartName="/{target}" ContentType="application/xml"/>' for target in overrides)
    parts.append("</Types>")
    return _xml("".join(parts))


def _relationships(extra: tuple[tuple[str, str], ...], target: str) -> bytes:
    entries = [
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">',
        '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" '
        f'Target="{target}"/>',
    ]
    for index, (rel_type, rel_target) in enumerate(extra, start=2):
        entries.append(f'<Relationship Id="rId{index}" Type="{rel_type}" Target="{rel_target}"/>')
    entries.append("</Relationships>")
    return "".join(entries).encode("utf-8")


def _rels_type(name: str) -> str:
    return f"http://schemas.openxmlformats.org/officeDocument/2006/relationships/{name}"


def _docx(config: Config) -> tuple[tuple[str, bytes], ...]:
    rng = _rng(config)
    body: list[str] = []
    for index in range(config.count("paragraphs")):
        body.append(f'<w:p>{_tag("paragraphs", index)}<w:r><w:t>{MARKER} {index} {_word(rng)} {_num(rng)}</w:t></w:r></w:p>')
    for index in range(config.count("tables")):
        rows = "".join(
            f'<w:tr><w:tc><w:p><w:r><w:t>{_word(rng)}{index}{row}</w:t></w:r></w:p></w:tc></w:tr>'
            for row in range(3)
        )
        body.append(f'<w:tbl>{_tag("tables", index)}{rows}</w:tbl>')
    for index in range(config.count("tracking")):
        body.append(f'<w:ins w:id="{index}">{_tag("tracking", index)}<w:r><w:t>{_word(rng)}</w:t></w:r></w:ins>')
    for index in range(config.count("comments")):
        body.append(f'<w:commentRangeStart w:id="{index}"/><w:r><w:t>{_word(rng)}</w:t></w:r><w:commentRangeEnd w:id="{index}"/>')
    for index in range(config.count("images")):
        body.append(f'<w:r>{_tag("images", index)}<w:drawing><a:blip r:embed="rIdImg{index}"/></w:drawing></w:r>')
    for index in range(config.count("charts")):
        body.append(f'<w:r>{_tag("charts", index)}<w:drawing><c:chart r:id="rIdChart{index}"/></w:drawing></w:r>')
    document = _xml(
        '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
        'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
        'xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        f"<w:body>{''.join(body)}</w:body></w:document>"
    )

    extra: list[tuple[str, str]] = []
    if config.count("comments"):
        extra.append((_rels_type("comments"), "comments.xml"))
    for index in range(config.count("images")):
        extra.append((_rels_type("image"), f"media/image{index}.png"))
    for index in range(config.count("charts")):
        extra.append((_rels_type("chart"), f"charts/chart{index}.xml"))

    parts: list[tuple[str, bytes]] = [
        ("[Content_Types].xml", _content_types(("word/document.xml",))),
        ("_rels/.rels", _relationships((), "word/document.xml")),
        ("word/document.xml", document),
    ]
    if config.count("comments"):
        comments = "".join(
            f'<w:comment w:id="{index}">{_tag("comments", index)}<w:p><w:r><w:t>{_word(rng)}</w:t></w:r></w:p></w:comment>'
            for index in range(config.count("comments"))
        )
        parts.append(("word/comments.xml", _xml(f'<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{comments}</w:comments>')))
    for index in range(config.count("charts")):
        parts.append((f"word/charts/chart{index}.xml", _xml('<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></c:chartSpace>')))
    for index in range(config.count("images")):
        parts.append((f"word/media/image{index}.png", _png(rng)))
    parts.append(("word/_rels/document.xml.rels", _relationships(tuple(extra), "document.xml")))
    return tuple(parts)


def _xlsx(config: Config) -> tuple[tuple[str, bytes], ...]:
    rng = _rng(config)
    formulas = config.count("formulas")
    shared = config.count("shared_strings")
    rows: list[str] = []
    for row in range(1, config.units + 1):
        cells = [
            f'<c r="A{row}">{_tag("cells", row - 1)}<v>{_num(rng)}</v></c>',
        ]
        if row <= formulas:
            cells.append(f'<c r="B{row}">{_tag("formulas", row - 1)}<f>SUM(A1:A{row})</f><v>0</v></c>')
        if row <= shared:
            cells.append(f'<c r="C{row}" t="s"><v>{row - 1}</v></c>')
        cells.append(f'<c r="D{row}" t="inlineStr"><is><t>{MARKER} {row} {_word(rng)}</t></is></c>')
        rows.append(f'<row r="{row}">{"".join(cells)}</row>')
    sheet = _xml(
        '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
        f'<sheetData>{"".join(rows)}</sheetData></worksheet>'
    )
    workbook = _xml(
        '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        f'<sheets><sheet name="{MARKER}" sheetId="1" r:id="rId1"/></sheets></workbook>'
    )
    parts: list[tuple[str, bytes]] = [
        ("[Content_Types].xml", _content_types(("xl/workbook.xml", "xl/worksheets/sheet1.xml"))),
        ("_rels/.rels", _relationships((), "xl/workbook.xml")),
        ("xl/workbook.xml", workbook),
        ("xl/worksheets/sheet1.xml", sheet),
    ]
    if shared:
        items = "".join(f'<si>{_tag("shared_strings", index)}<t>{MARKER} {index} {_word(rng)}</t></si>' for index in range(shared))
        parts.append(("xl/sharedStrings.xml", _xml(f'<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="{shared}" uniqueCount="{shared}">{items}</sst>')))
    for index in range(config.count("tables")):
        parts.append((f"xl/tables/table{index}.xml", _xml(f'<table id="{index + 1}" name="T{index}" ref="A1:D{config.units}">{_tag("tables", index)}</table>')))
    if config.count("comments"):
        notes = "".join(f'<comment ref="A{index + 1}" authorId="1">{_tag("comments", index)}<text>{_word(rng)}</text></comment>' for index in range(config.count("comments")))
        parts.append(("xl/comments1.xml", _xml(f'<comments xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">{notes}</comments>')))
    for index in range(config.count("charts")):
        parts.append((f"xl/charts/chart{index}.xml", _xml(f'<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">{_tag("charts", index)}<c:chart/></c:chartSpace>')))
    for index in range(config.count("images")):
        parts.append((f"xl/drawings/drawing{index}.xml", _xml(f'<xdr:wsDr xmlns:xdr="http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing">{_tag("images", index)}</xdr:wsDr>')))
        parts.append((f"xl/media/image{index}.png", _png(rng)))
    parts.append(("xl/_rels/workbook.xml.rels", _relationships((), "worksheets/sheet1.xml")))
    return tuple(parts)


def _pptx(config: Config) -> tuple[tuple[str, bytes], ...]:
    rng = _rng(config)
    slides = config.count("slides")
    slide_ids = "".join(f'<p:sldId id="{256 + index}" r:id="rIdS{index}"/>' for index in range(slides))
    presentation = _xml(
        '<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        f'<p:sldIdLst>{slide_ids}</p:sldIdLst></p:presentation>'
    )
    parts: list[tuple[str, bytes]] = [
        ("[Content_Types].xml", _content_types(("ppt/presentation.xml",))),
        ("_rels/.rels", _relationships((), "ppt/presentation.xml")),
        ("ppt/presentation.xml", presentation),
    ]
    pres_rels: list[tuple[str, str]] = []
    for index in range(slides):
        shapes = [f'<p:sp>{_tag("slides", index)}<p:nvSpPr><p:cNvPr id="1" name="{MARKER}"/></p:nvSpPr></p:sp>']
        for feature in ("tables", "images", "charts", "comments"):
            if index < config.count(feature):
                if feature == "images":
                    shapes.append(f'<p:pic>{_tag("images", index)}<p:blipFill><a:blip r:embed="rIdI{index}"/></p:blipFill></p:pic>')
                else:
                    shapes.append(f'<p:graphicFrame>{_tag(feature, index)}<p:nvGraphicFramePr><p:cNvPr id="{index + 2}" name="{MARKER}"/></p:nvGraphicFramePr></p:graphicFrame>')
        slide = _xml(
            '<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
            'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
            'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
            f'<p:cSld><p:spTree>{"".join(shapes)}</p:spTree></p:cSld></p:sld>'
        )
        parts.append((f"ppt/slides/slide{index}.xml", slide))
        pres_rels.append((_rels_type("slide"), f"slides/slide{index}.xml"))
    if config.count("comments"):
        authors = "".join(f'<p:cmAuthor id="{index}" name="{MARKER}"/>' for index in range(config.count("comments")))
        parts.append(("ppt/commentAuthors.xml", _xml(f'<p:cmAuthorLst xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">{authors}</p:cmAuthorLst>')))
    for index in range(config.count("charts")):
        parts.append((f"ppt/charts/chart{index}.xml", _xml('<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></c:chartSpace>')))
    for index in range(config.count("images")):
        parts.append((f"ppt/media/image{index}.png", _png(rng)))
    parts.append(("ppt/_rels/presentation.xml.rels", _relationships(tuple(pres_rels), "presentation.xml")))
    return tuple(parts)


def _ods(config: Config) -> tuple[tuple[str, bytes], ...]:
    rng = _rng(config)
    cells: list[str] = []
    for row in range(1, config.units + 1):
        row_cells = [f'<table:table-cell office:value-type="float" office:value="{_num(rng)}">{_tag("cells", row - 1)}</table:table-cell>']
        if row <= config.count("formulas"):
            row_cells.append(f'<table:table-cell table:formula="of:=SUM([.A1:.A{row}])">{_tag("formulas", row - 1)}</table:table-cell>')
        cells.append(f"<table:table-row>{''.join(row_cells)}</table:table-row>")
    content = _xml(
        '<office:document-content '
        'xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" '
        'xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">'
        f'<office:body><office:spreadsheet><table:table table:name="{MARKER}">{"".join(cells)}</table:table></office:spreadsheet></office:body>'
        "</office:document-content>"
    )
    return (
        ("mimetype", b"application/vnd.oasis.opendocument.spreadsheet"),
        ("content.xml", content),
    )


def build_parts(config: Config) -> tuple[tuple[str, bytes], ...]:
    """Render all in-archive parts for a ZIP-based corpus file."""
    builders = {"docx": _docx, "xlsx": _xlsx, "pptx": _pptx, "ods": _ods}
    builder = builders.get(config.kind)
    if builder is None:
        raise ValueError(f"kind {config.kind!r} has no ZIP-based builder")
    return builder(config)


def build_csv(config: Config) -> bytes:
    """Render a plain CSV of exactly ``config.target_bytes`` bytes."""
    rng = _rng(config)
    lines: list[str] = ["id,name,value,tag"]
    for index in range(config.count("rows")):
        lines.append(f"{index},{MARKER} {index} {_word(rng)},{_num(rng)},{marker('rows')}{index}")
    blob = ("\n".join(lines) + "\n").encode("utf-8")
    if len(blob) > config.target_bytes:
        raise ValueError("CSV rows exceed the target size; lower --units")
    remainder = config.target_bytes - len(blob)
    pieces: list[str] = []
    length = 0
    while length < remainder:
        word = _word(rng)
        pieces.append(word)
        length += len(word)
    filler = "".join(pieces)[:remainder]
    return blob + filler.encode("utf-8")


def verify_feature_counts(payloads: tuple[bytes, ...], config: Config) -> dict[str, int]:
    """Count feature markers across decoded parts and compare with the manifest."""
    haystack = b"".join(payloads)
    found: dict[str, int] = {}
    for feature, expected in config.counts:
        actual = haystack.count(marker(feature).encode("utf-8"))
        if actual != expected:
            raise ValueError(f"feature {feature}: expected {expected} markers, found {actual}")
        found[feature] = actual
    return found
