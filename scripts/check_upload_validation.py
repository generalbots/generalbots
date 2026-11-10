#!/usr/bin/env python3
"""Upload-validation gate (#2132).

Every HTTP handler that accepts a file — a `Multipart` extractor or a raw
`body: String`/`Bytes`/`Body` — must call the shared upload policy
(`botsecurity_core::upload_policy::validate_upload`) before it persists
anything. The control exists since #2112; this gate keeps it wired.

    python3 scripts/check_upload_validation.py            # human report
    python3 scripts/check_upload_validation.py --check    # exit 1 on violations
    python3 scripts/check_upload_validation.py --json     # machine output
    python3 scripts/check_upload_validation.py --self-test

A handler is reported as an upload surface when its signature mentions
`Multipart`, or a whole-body extractor (`body: String`, `body: Bytes`,
`body: axum::body::Body`). Text-only JSON bodies (`Json<...>`) are not upload
surfaces and are not scanned.

The allowlist (`scripts/upload_validation_allowlist.txt`) records handlers that
carry text payloads which are parsed into rows/messages instead of being
stored as files. It is **shrink-only**: an entry that no longer matches a
scanned handler fails the gate, so the list cannot grow stale.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

# `GB_UPLOAD_VALIDATION_ROOT` lets the self-test inject a fixture tree (and is
# handy when reviewing a worktree) without touching the real sources.
ROOT = Path(
    os.environ.get("GB_UPLOAD_VALIDATION_ROOT")
    or Path(__file__).resolve().parent.parent
)
ALLOWLIST = Path(
    os.environ.get("GB_UPLOAD_VALIDATION_ALLOWLIST")
    or Path(__file__).resolve().parent / "upload_validation_allowlist.txt"
)

SCAN_ROOTS = ("botserver/src", "botserver/crates")
SKIP_PARTS = ("/target/", "/tests/", "/test/")

FN_START = re.compile(r"^\s*pub async fn\s+([a-z0-9_]+)\s*(?:<[^>]*>)?\s*\(", re.MULTILINE)
BODY_EXTRACTOR = re.compile(r"body:\s*(?:axum::body::)?(Bytes|String|Body)\b")
VALIDATOR = "validate_upload("
SIZE_CHECK = "check_declared_length("


@dataclass
class Handler:
    path: str
    function: str
    kind: str
    validator: bool
    size_check: bool

    @property
    def key(self) -> str:
        return f"{self.path}::{self.function}"


@dataclass
class Report:
    handlers: list[Handler] = field(default_factory=list)
    violations: list[str] = field(default_factory=list)
    stale_allowlist: list[str] = field(default_factory=list)
    unused_allowlist: list[str] = field(default_factory=list)


def iter_sources() -> list[Path]:
    files: list[Path] = []
    for root in SCAN_ROOTS:
        base = ROOT / root
        if not base.exists():
            continue
        for path in base.rglob("*.rs"):
            posix = f"/{path.as_posix()}/"
            if any(part in posix for part in SKIP_PARTS):
                continue
            files.append(path)
    return sorted(files)


def signature(text: str, start: int) -> str:
    """Text from the opening `(` of a signature to its matching `)`."""
    depth = 0
    for index in range(start, len(text)):
        char = text[index]
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                return text[start : index + 1]
    return text[start : start + 400]


def handler_chunks(text: str) -> list[tuple[str, str, str]]:
    """`(function, signature, body)` for every `pub async fn` in `text`."""
    matches = list(FN_START.finditer(text))
    chunks: list[tuple[str, str, str]] = []
    for index, match in enumerate(matches):
        open_paren = text.index("(", match.start())
        sig = signature(text, open_paren)
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        chunks.append((match.group(1), sig, text[match.start() : end]))
    return chunks


def classify(sig: str) -> str | None:
    # Struct methods (`&self`) are never HTTP handlers.
    if "&self" in sig or "&mut self" in sig:
        return None
    if "Multipart" in sig:
        return "multipart"
    body = BODY_EXTRACTOR.search(sig)
    if body:
        return f"body:{body.group(1)}"
    return None


def load_allowlist() -> dict[str, str]:
    if not ALLOWLIST.exists():
        return {}
    entries: dict[str, str] = {}
    for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        key, _, reason = line.partition("|")
        entries[key.strip()] = reason.strip()
    return entries


def scan() -> Report:
    allow = load_allowlist()
    report = Report()
    matched_allow: set[str] = set()

    for path in iter_sources():
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="replace")

        for function, sig, body in handler_chunks(text):
            kind = classify(sig)
            if kind is None:
                continue

            handler = Handler(
                path=rel,
                function=function,
                kind=kind,
                validator=VALIDATOR in body,
                size_check=SIZE_CHECK in body,
            )
            report.handlers.append(handler)

            if handler.validator:
                continue
            if handler.key in allow:
                matched_allow.add(handler.key)
                continue
            report.violations.append(
                f"{rel}:{function}: {kind} handler does not call "
                f"botsecurity_core::upload_policy::validate_upload"
            )

    for key in allow:
        if key not in matched_allow:
            report.unused_allowlist.append(key)

    report.stale_allowlist = list(report.unused_allowlist)
    return report


def render(report: Report) -> str:
    lines = ["Upload surfaces (#2132):", ""]
    width = max((len(h.path) for h in report.handlers), default=10)
    for handler in report.handlers:
        state = "validated" if handler.validator else "NOT VALIDATED"
        size = "size-precheck" if handler.size_check else "-"
        lines.append(
            f"  {handler.path:<{width}}  {handler.function:<40} "
            f"{handler.kind:<12} {state:<14} {size}"
        )

    lines.append("")
    lines.append(f"handlers: {len(report.handlers)}  "
                 f"validated: {sum(1 for h in report.handlers if h.validator)}  "
                 f"allowlisted: {len(load_allowlist())}")

    if report.violations:
        lines.append("")
        lines.append("VIOLATIONS:")
        lines.extend(f"  {v}" for v in report.violations)

    if report.stale_allowlist:
        lines.append("")
        lines.append("STALE ALLOWLIST ENTRIES (remove them):")
        lines.extend(f"  {key}" for key in report.stale_allowlist)

    return "\n".join(lines)


UNVALIDATED = (
    "pub async fn upload(\n"
    "    State(state): State<Arc<AppState>>,\n"
    "    mut multipart: axum::extract::Multipart,\n"
    ") -> impl IntoResponse { persist(); }\n"
)
VALIDATED = UNVALIDATED.replace(
    "persist();", "let _ = validate_upload(1, 2, 3, 4); persist();"
)


def run_gate(root: Path, allowlist: Path) -> int:
    env = dict(os.environ)
    env["GB_UPLOAD_VALIDATION_ROOT"] = str(root)
    env["GB_UPLOAD_VALIDATION_ALLOWLIST"] = str(allowlist)
    return subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--check"],
        env=env,
        capture_output=True,
        text=True,
        timeout=120,
    ).returncode


def self_test() -> int:
    """Fail-closed proof: the gate must reject an unvalidated handler.

    The classifier is exercised directly, then the real CLI runs against a
    fixture tree with and without the validator, and against a stale allowlist.
    """
    chunk = handler_chunks(UNVALIDATED)[0]
    assert classify(chunk[1]) == "multipart", chunk[1]
    raw = "pub async fn put(body: axum::body::Body) -> Response {}\n"
    assert classify(handler_chunks(raw)[0][1]) == "body:Body"
    json = "pub async fn create(Json(body): Json<Payload>) -> Response {}\n"
    assert classify(handler_chunks(json)[0][1]) is None

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        source = root / "botserver" / "src" / "fake" / "upload.rs"
        source.parent.mkdir(parents=True)
        allowlist = root / "allowlist.txt"
        allowlist.write_text("", encoding="utf-8")

        source.write_text(UNVALIDATED, encoding="utf-8")
        if run_gate(root, allowlist) != 1:
            print("self-test FAILED: an unvalidated upload handler passed")
            return 1

        source.write_text(VALIDATED, encoding="utf-8")
        if run_gate(root, allowlist) != 0:
            print("self-test FAILED: a validated upload handler was refused")
            return 1

        allowlist.write_text(
            "botserver/src/fake/upload.rs::gone | stale entry\n", encoding="utf-8"
        )
        if run_gate(root, allowlist) != 1:
            print("self-test FAILED: a stale allowlist entry passed")
            return 1

    print("check_upload_validation self-test: OK")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Upload-validation gate (#2132)")
    parser.add_argument("--check", action="store_true", help="exit 1 on violations")
    parser.add_argument("--json", action="store_true", help="machine-readable report")
    parser.add_argument("--self-test", action="store_true", help="run the gate self-test")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    report = scan()

    if args.json:
        print(json.dumps(
            {
                "handlers": [
                    {
                        "path": h.path,
                        "function": h.function,
                        "kind": h.kind,
                        "validator": h.validator,
                        "size_check": h.size_check,
                    }
                    for h in report.handlers
                ],
                "violations": report.violations,
                "stale_allowlist": report.stale_allowlist,
            },
            indent=2,
        ))
    else:
        print(render(report))

    if args.check and (report.violations or report.stale_allowlist):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
