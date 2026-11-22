#!/usr/bin/env python3
"""Deterministic synthetic large-file corpus for the Office memory gate (#2530).

DOCX/XLSX/PPTX files are OOXML ZIP archives. This generator builds valid,
ZIP_STORED archives whose members are either tiny synthetic XML parts or one
large seeded byte blob that inflates the archive to an exact requested size.
No document text, tenant name, path or real client data ever enters the
archive: the only strings are fixed synthetic markers and fixed-vocabulary
placeholders.

ZIP_STORED is deliberate: deflate output is zlib-version dependent, so a
deflated corpus could change SHA-256 between CI images. Stored members plus a
fixed ZipInfo timestamp (the ZIP epoch, 1980-01-01) make the bytes reproducible
on any Python 3 and any OS. The archives are structurally OOXML-like, which is
what the import/memory soak needs; they are not meant to open in Word.

Usage::

    python3 scripts/office_largefile_corpus.py --self-test
    python3 scripts/office_largefile_corpus.py --kind xlsx --size-mb 25 \
        --out /tmp/opencode/office-corpus --manifest
    python3 scripts/office_largefile_corpus.py --all \
        --out /tmp/opencode/office-corpus --manifest

Exit codes: 0 success, 1 self-test/check failure, 2 bad arguments or I/O error.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import sys
import zipfile
from pathlib import Path
from typing import Any

ISSUE = 2530
MIN_SIZE_MB = 5
MAX_SIZE_MB = 100
CHUNK = 1 << 20
ZIP_EPOCH = (1980, 1, 1, 0, 0, 0)
MARKER = "SYNTHETIC-CORPUS-GENERATEDBOTS-NOT-REAL-DATA"
KINDS = ("docx", "xlsx", "pptx")

# Acceptance corpus: the two sizes named in #2530 plus a DOCX control case.
ACCEPTANCE_CASES = (
    ("docx", 25, 2530),
    ("xlsx", 25, 2530),
    ("pptx", 50, 2530),
)

# Fixed ordering: the last member is always the seeded filler blob.
MEMBERS = {
    "docx": (
        "[Content_Types].xml",
        "_rels/.rels",
        "docProps/core.xml",
        "word/document.xml",
        "word/media/corpus.bin",
    ),
    "xlsx": (
        "[Content_Types].xml",
        "_rels/.rels",
        "xl/workbook.xml",
        "xl/worksheets/sheet1.xml",
        "xl/media/corpus.bin",
    ),
    "pptx": (
        "[Content_Types].xml",
        "_rels/.rels",
        "ppt/presentation.xml",
        "ppt/slides/slide1.xml",
        "ppt/media/corpus.bin",
    ),
}

VOCABULARY = ("alpha", "bravo", "delta", "echo", "foxtrot", "gamma", "lambda")


def resolve_size_mb(value: float) -> float:
    """Reject sizes outside the declared 5-100 MB policy before writing."""
    size = float(value)
    if not MIN_SIZE_MB <= size <= MAX_SIZE_MB:
        raise ValueError(
            f"size {size} MB is outside the #2530 corpus policy "
            f"[{MIN_SIZE_MB}, {MAX_SIZE_MB}] MB"
        )
    return size


def zip_overhead(names: tuple[str, ...]) -> int:
    """Byte cost of local headers + central directory + EOCD for stored members.

    Each member costs 30 bytes of local header + 46 bytes of central header +
    twice its ASCII filename length; the end-of-central-directory record is 22
    bytes. No extra fields, comments or data descriptors are written because
    the archive is opened on a seekable file with fixed ZipInfo objects.
    """
    return sum(76 + 2 * len(name.encode("ascii")) for name in names) + 22


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(CHUNK), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _xml_part(kind: str, name: str, rng: random.Random, rows: int) -> bytes:
    """Return one small, synthetic OOXML-ish XML part as UTF-8 bytes."""
    if name == "[Content_Types].xml":
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="bin" ContentType="application/octet-stream"/>'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Default Extension="xml" ContentType="application/xml"/>'
            f'<Override PartName="/{MEMBERS[kind][2]}" ContentType="application/{MARKER.lower()}.document"/>'
            "</Types>"
        )
    elif name == "_rels/.rels":
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            f'<Relationship Id="rId1" Type="{MARKER}/root" Target="/{MEMBERS[kind][2]}"/>'
            "</Relationships>"
        )
    elif name == "docProps/core.xml":
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">'
            f"<dc:creator xmlns:dc=\"http://purl.org/dc/elements/1.1/\">{MARKER}</dc:creator>"
            f"<dc:title xmlns:dc=\"http://purl.org/dc/elements/1.1/\">{MARKER}</dc:title>"
            "<dcterms:created xmlns:dcterms=\"http://purl.org/dc/terms/\" "
            "xsi:type=\"dcterms:W3CDTF\" "
            "xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">1980-01-01T00:00:00Z</dcterms:created>"
            "</cp:coreProperties>"
        )
    elif name == "word/document.xml":
        paragraphs = "".join(
            f"<w:p><w:r><w:t>{MARKER} {row} {VOCABULARY[rng.randrange(len(VOCABULARY))]}</w:t></w:r></w:p>"
            for row in range(rows)
        )
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            f'<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
            f"<w:body>{paragraphs}</w:body></w:document>"
        )
    elif name == "xl/workbook.xml":
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
            f'<sheets><sheet name="{MARKER}" sheetId="1" r:id="rId1" '
            'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/></sheets>'
            "</workbook>"
        )
    elif name == "xl/worksheets/sheet1.xml":
        cells = "".join(
            f'<row r="{row}"><c r="A{row}"><v>{rng.randrange(1_000_000)}</v></c>'
            f'<c r="B{row}" t="inlineStr"><is><t>{MARKER} {row} '
            f"{VOCABULARY[rng.randrange(len(VOCABULARY))]}</t></is></c></row>"
            for row in range(1, rows + 1)
        )
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
            f"<sheetData>{cells}</sheetData></worksheet>"
        )
    elif name == "ppt/presentation.xml":
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">'
            f'<p:sldIdLst><p:sldId id="256" r:id="rId1" '
            'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/></p:sldIdLst>'
            "</p:presentation>"
        )
    elif name == "ppt/slides/slide1.xml":
        shapes = "".join(
            f'<p:sp><p:nvSpPr><p:cNvPr id="{index}" name="{MARKER} {index}"/></p:nvSpPr>'
            f"<p:txBody><a:p xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">"
            f"<a:r><a:t>{MARKER} {index} "
            f"{VOCABULARY[rng.randrange(len(VOCABULARY))]}</a:t></a:r></a:p></p:txBody></p:sp>"
            for index in range(1, rows + 1)
        )
        body = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">'
            f"<p:cSld><p:spTree>{shapes}</p:spTree></p:cSld></p:sld>"
        )
    else:
        raise ValueError(f"unexpected synthetic member: {name}")
    return body.encode("utf-8")


def _write_archive(path: Path, kind: str, target: int, seed: int, rows: int, blob: int) -> None:
    """Write one archive; `blob` is the tuned filler length in bytes."""
    rng = random.Random(seed)
    names = MEMBERS[kind]
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in names[:-1]:
            info = zipfile.ZipInfo(name, date_time=ZIP_EPOCH)
            info.compress_type = zipfile.ZIP_STORED
            info.external_attr = 0o644 << 16
            archive.writestr(info, _xml_part(kind, name, rng, rows))
        info = zipfile.ZipInfo(names[-1], date_time=ZIP_EPOCH)
        info.compress_type = zipfile.ZIP_STORED
        info.external_attr = 0o644 << 16
        remaining = blob
        with archive.open(info, "w") as sink:
            while remaining > 0:
                count = min(remaining, CHUNK)
                sink.write(rng.randbytes(count))
                remaining -= count


def build_payload(kind: str, target_bytes: int, seed: int, path: Path) -> dict[str, Any]:
    """Build a synthetic archive of exactly `target_bytes` bytes at `path`."""
    if kind not in KINDS:
        raise ValueError(f"unsupported kind {kind!r}; expected one of {KINDS}")
    if target_bytes < 4096:
        raise ValueError(f"target_bytes {target_bytes} is too small for an OOXML skeleton")
    if seed < 0:
        raise ValueError("seed must be non-negative")

    names = MEMBERS[kind]
    rows = max(4, min(400, target_bytes // 8192))
    rng = random.Random(seed)
    fixed = sum(len(_xml_part(kind, name, rng, rows)) for name in names[:-1]) + zip_overhead(names)
    blob = target_bytes - fixed
    if blob <= 0:
        raise ValueError(f"target_bytes {target_bytes} cannot hold the {kind} skeleton")

    path.parent.mkdir(parents=True, exist_ok=True)
    _write_archive(path, kind, target_bytes, seed, rows, blob)
    actual = path.stat().st_size
    if actual != target_bytes:
        # Correct once: stored archives are linear in blob length, so the delta
        # is exact. Fail closed if a Python/ZIP implementation still disagrees.
        corrected = blob - (actual - target_bytes)
        if corrected <= 0:
            raise ValueError(f"invalid corrected blob size {corrected} for {kind}")
        _write_archive(path, kind, target_bytes, seed, rows, corrected)
        actual = path.stat().st_size
    if actual != target_bytes:
        raise ValueError(f"{path}: wrote {actual} bytes, expected {target_bytes}")

    with zipfile.ZipFile(path) as archive:
        if archive.testzip() is not None or archive.namelist() != list(names):
            raise ValueError(f"{path}: archive failed structural validation")
    return {
        "file": path.name,
        "kind": kind,
        "size_bytes": actual,
        "seed": seed,
        "sha256": sha256_file(path),
    }


def build_manifest(cases: list[dict[str, Any]]) -> str:
    document = {
        "generated_by": "scripts/office_largefile_corpus.py",
        "issue": ISSUE,
        "synthetic_only": True,
        "compression": "ZIP_STORED",
        "min_size_mb": MIN_SIZE_MB,
        "max_size_mb": MAX_SIZE_MB,
        "cases": sorted(cases, key=lambda item: item["file"]),
    }
    return json.dumps(document, indent=2, sort_keys=True) + "\n"


def _run_cases(out: Path, cases: list[tuple[str, float, int]]) -> list[dict[str, Any]]:
    results = []
    for kind, size_mb, seed in cases:
        size = resolve_size_mb(size_mb)
        name = f"{kind}-{int(size)}mb-seed{seed}.{kind}"
        print(f"generating {name} ({size} MB, seed {seed})")
        results.append(build_payload(kind, int(size * 1024 * 1024), seed, out / name))
    return results


def self_test() -> int:
    import tempfile

    checks = 0
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(label)

    try:
        with tempfile.TemporaryDirectory(prefix="office-corpus-selftest-") as folder:
            root = Path(folder)
            for kind in KINDS:
                first = build_payload(kind, 65536, 2530, root / f"{kind}-a.{kind}")
                second = build_payload(kind, 65536, 2530, root / f"{kind}-b.{kind}")
                other = build_payload(kind, 65536, 2531, root / f"{kind}-c.{kind}")
                check(first["size_bytes"] == 65536, f"{kind}: exact target size")
                check(first["sha256"] == second["sha256"], f"{kind}: deterministic bytes")
                check(first["sha256"] != other["sha256"], f"{kind}: seed changes bytes")
                with zipfile.ZipFile(root / first["file"]) as archive:
                    check(archive.namelist() == list(MEMBERS[kind]), f"{kind}: member list")
                    check(archive.testzip() is None, f"{kind}: CRCs valid")
                    parts = b"".join(archive.read(name) for name in MEMBERS[kind][:-1])
                    check(MARKER.encode() in parts, f"{kind}: synthetic marker present")
                    check(b"corpus.bin" not in parts, f"{kind}: no real-data field names in XML")
            for bad in (1, 101):
                try:
                    resolve_size_mb(bad)
                except ValueError:
                    check(True, f"policy rejects {bad} MB")
                else:
                    check(False, f"policy rejects {bad} MB")
            check(resolve_size_mb(25) == 25.0, "policy accepts 25 MB")
            manifest_a = build_manifest([{
                "file": "xlsx-25mb-seed2530.xlsx", "kind": "xlsx",
                "size_bytes": 1024, "seed": 2530, "sha256": "abc",
            }])
            manifest_b = build_manifest([{
                "file": "xlsx-25mb-seed2530.xlsx", "kind": "xlsx",
                "size_bytes": 1024, "seed": 2530, "sha256": "abc",
            }])
            check(manifest_a == manifest_b, "manifest is deterministic")
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        print(f"self-test: unexpected error: {error}", file=sys.stderr)
        return 1

    print(f"self-test: {checks}/{checks} checks passed")
    if failures:
        for label in failures:
            print(f"  FAIL {label}", file=sys.stderr)
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", type=Path, default=Path("/tmp/opencode/office-corpus"))
    parser.add_argument("--kind", choices=KINDS, default="xlsx")
    parser.add_argument("--size-mb", type=float, default=25)
    parser.add_argument("--seed", type=int, default=2530)
    parser.add_argument("--all", action="store_true", help="generate the acceptance corpus")
    parser.add_argument("--manifest", action="store_true", help="write manifest.json")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()
    try:
        cases = list(ACCEPTANCE_CASES) if args.all else [(args.kind, args.size_mb, args.seed)]
        results = _run_cases(args.out, cases)
        if args.manifest:
            target = args.out / "manifest.json"
            target.write_text(build_manifest(results), encoding="utf-8")
            print(f"wrote {target}")
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
