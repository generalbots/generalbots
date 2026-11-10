#!/usr/bin/env python3
"""Route-surface scanner for `scripts/check_security_directives.py`.

The gate needs to answer one question per state-changing handler: *is this
router served by the main app (which installs the CSRF and rate-limit layers)
or by a listener of its own?* This module holds that scanning so the gate file
stays a list of rules.

Surface
-------
* `botserver/src/main_module/**/*.rs` -- the main server and the side
  listeners it spawns.
* `botserver/crates/*/src/**/*routes*.rs` -- every crate router. This is a
  superset of the literal `routes*.rs` spelling, so `configure_*_routes.rs`
  files are covered too (the deploy gateway lives in one of those).

Everything here is grep-level analysis on purpose: it runs offline in CI
without a Rust toolchain and fails loudly instead of parsing a grammar it
cannot fully understand.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11
    import sys

    sys.exit("error: python3.11+ is required (tomllib)")

EXEMPTIONS = Path("scripts/security_directives_exemptions.toml")
MAIN_SERVE = Path("botserver/src/main_module/listen_impl.rs")

STATE_CHANGING = ("post", "put", "patch", "delete")
ROUTE_CALL = re.compile(r"\.route\s*\(")
ROUTE_PATH = re.compile(r'\s*"((?:[^"\\]|\\.)*)"')
METHOD_IN_BLOCK = re.compile(r"\b(?:%s)\s*\(" % "|".join(STATE_CHANGING))
SERVE_CALL = re.compile(r"axum::serve\s*\(\s*[^,]+,\s*([^,)]+)")
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
CALLED = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(")

# axum combinators and method routers are never the function that *builds* the
# router, so they are skipped when resolving `let x = ...`.
COMBINATORS = {
    "new",
    "with_state",
    "layer",
    "route",
    "nest",
    "nest_service",
    "merge",
    "fallback",
    "into_make_service",
    "into_make_service_with_connect_info",
    "await",
    "get",
    "post",
    "put",
    "patch",
    "delete",
    "head",
    "options",
    "any",
    "on",
}


@dataclass(frozen=True)
class Exemption:
    file: str
    directive: str
    reason: str
    evidence: str
    routes: tuple[str, ...]


# `fn name` -> defining files, filled by `definition_index` once per run.
_INDEX_CACHE: dict[str, dict[str, list[Path]]] = {}


def route_surface(root: Path) -> list[Path]:
    """Every file that may register a route in the served surface."""
    files: list[Path] = []
    main = root / "botserver/src/main_module"
    if main.is_dir():
        files.extend(sorted(main.rglob("*.rs")))
    crates = root / "botserver/crates"
    if crates.is_dir():
        for crate in sorted(crates.iterdir()):
            src = crate / "src"
            if src.is_dir():
                files.extend(sorted(src.rglob("*routes*.rs")))
    return files


def rust_files(root: Path) -> list[Path]:
    """Source files searched when resolving a served router (tests excluded)."""
    base = root / "botserver"
    if not base.is_dir():
        return []
    files: list[Path] = []
    for path in sorted(base.rglob("*.rs")):
        if "target" in path.parts:
            continue
        relative = path.relative_to(root).as_posix()
        if "/tests/" in relative or path.name == "tests.rs" or path.name.endswith("_tests.rs"):
            continue
        files.append(path)
    return files


def code_before_tests(text: str) -> str:
    """Drops `#[cfg(test)]` modules: a test router is never served."""
    marker = text.find("#[cfg(test)]")
    return text if marker == -1 else text[:marker]


def line_of(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def balanced_block(text: str, start: int) -> str:
    """Text from `start` to the `)` matching the open paren before it."""
    depth = 1
    index = start
    in_string = False
    while index < len(text) and depth:
        char = text[index]
        if in_string:
            if char == "\\":
                index += 2
                continue
            if char == '"':
                in_string = False
        elif char == '"':
            in_string = True
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        index += 1
    return text[start:index]


def state_changing_routes(text: str) -> list[tuple[str, int]]:
    """Every path mounted with POST/PUT/PATCH/DELETE in this file."""
    found: list[tuple[str, int]] = []
    for call in ROUTE_CALL.finditer(text):
        path_match = ROUTE_PATH.match(text, call.end())
        if not path_match:
            continue
        block = balanced_block(text, path_match.end())
        if METHOD_IN_BLOCK.search(block):
            found.append((path_match.group(1), line_of(text, call.start())))
    return found


def definition_index(root: Path) -> dict[str, list[Path]]:
    """`fn name` -> files defining it, built once per process."""
    cached = _INDEX_CACHE.get(str(root))
    if cached is not None:
        return cached
    index: dict[str, list[Path]] = {}
    for path in rust_files(root):
        text = code_before_tests(path.read_text(encoding="utf-8", errors="replace"))
        for name in re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", text):
            index.setdefault(name, []).append(path)
    _INDEX_CACHE[str(root)] = index
    return index


def resolve_builder(root: Path, serving: Path, served: str, depth: int = 0) -> list[Path]:
    """Files that build the router passed to `axum::serve` in `serving`."""
    if depth > 4:
        return []
    text = code_before_tests(serving.read_text(encoding="utf-8", errors="replace"))
    assign = re.compile(r"let\s+(?:mut\s+)?%s\s*=\s*(.+?);" % re.escape(served), re.S)
    match = assign.search(text)
    if not match:
        # `listen(app, port)` receives the finished router; the main listener is
        # validated by the explicit layer checks instead.
        return []
    rhs = match.group(1)
    if "apply_middleware(" in rhs:
        return []
    calls = [name for name in CALLED.findall(rhs) if name not in COMBINATORS]
    if not calls:
        # `let service = gateway_router.into_make_service...()` -- follow the
        # variable back to the `let` that built the router.
        lead = IDENT.match(rhs.strip())
        if lead and lead.group(0) != served:
            return resolve_builder(root, serving, lead.group(0), depth + 1)
        return [serving]
    index = definition_index(root)
    candidates: list[Path] = []
    for name in calls:
        for path in index.get(name, []):
            if path not in candidates:
                candidates.append(path)
    if serving in candidates:
        # Built here and only wrapped by another call, e.g.
        # `with_security_headers(router().with_state(state), ..)`.
        return [serving]
    if not candidates:
        return [serving]
    if "::" in rhs:
        segments = [
            segment
            for segment in rhs.split("(")[0].split("::")
            if segment not in {"crate", "super", "self"}
        ]
        hint = segments[-2] if len(segments) >= 2 else ""
        narrowed = [path for path in candidates if hint and hint in path.as_posix()]
        if narrowed:
            return narrowed
    return candidates


def standalone_router_files(root: Path) -> set[Path]:
    """Router-builder files served by a listener other than the main one."""
    standalone: set[Path] = set()
    main = (root / MAIN_SERVE).resolve()
    for path in rust_files(root):
        text = code_before_tests(path.read_text(encoding="utf-8", errors="replace"))
        if "axum::serve(" not in text:
            continue
        for call in SERVE_CALL.finditer(text):
            served = IDENT.match(call.group(1).strip())
            if not served:
                continue
            for builder in resolve_builder(root, path, served.group(0)):
                if builder.resolve() != main:
                    standalone.add(builder)
    return standalone


def load_exemptions(root: Path, errors: list[str]) -> tuple[list[Exemption], int]:
    """Parses the shrink-only exemption registry; malformed input is an error."""
    path = root / EXEMPTIONS
    if not path.is_file():
        return [], 0
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        errors.append(f"{EXEMPTIONS}: not valid TOML: {error}")
        return [], 0
    entries = data.get("exemption", [])
    exemptions = [
        Exemption(
            file=entry.get("file", ""),
            directive=entry.get("directive", ""),
            reason=entry.get("reason", ""),
            evidence=entry.get("evidence", ""),
            routes=tuple(entry.get("routes", [])),
        )
        for entry in entries
    ]
    return exemptions, int(data.get("max_entries", len(exemptions)))
