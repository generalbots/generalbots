#!/usr/bin/env python3
"""BotBook guards (#2082): the checks that keep the book traceable.

Three families, all derived from the repository, never from prose:

``SUMMARY``
    Every chapter registered in ``botbook/src/SUMMARY.md`` exists and has a body.
    A page that exists but is absent from ``SUMMARY.md`` is never rendered by
    mdBook, so links to it are reported as ``LINK_UNPUBLISHED``.

``LINK``
    Every relative Markdown link resolves: the target file exists
    (``LINK_MISSING``), it is published (``LINK_UNPUBLISHED``), and the anchor
    names a real heading (``BAD_ANCHOR``). Absolute site routes (``/terms``) and
    URLs are not book pages and are ignored.

``VERSION`` / ``PORT``
    Product-version claims must match the root ``Cargo.toml``; port claims must
    match the canonical table in ``AGENTS.md`` (``botserver`` 8080, suite 3000,
    cloud 4000, login 5000, Forgejo 4747). Component versions (PostgreSQL,
    Valkey, …) come from ``botserver/3rdparty.toml`` and are a separate guard.

Usage
-----
    python3 scripts/botbook_guards.py            # report only (exit 0)
    python3 scripts/botbook_guards.py --check    # exit 1 on a new violation
    python3 scripts/botbook_guards.py --prune    # drop satisfied allowlist rows
    python3 scripts/botbook_guards.py --self-test  # inject defects, assert exit 1

Pre-existing debt is recorded in ``botbook/guards-allowlist.txt``
(``kind|path|detail|reason``); the file may only shrink. A violation that is not
in the allowlist fails ``--check`` and prints ``path:line: KIND detail``.
"""

from __future__ import annotations

import argparse
import re
import shutil
import sys
import tempfile
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BOOK = ROOT / "botbook" / "src"
SUMMARY = BOOK / "SUMMARY.md"
ALLOWLIST = ROOT / "botbook" / "guards-allowlist.txt"
CARGO = ROOT / "Cargo.toml"
AGENTS = ROOT / "AGENTS.md"

# Service → ports it may be documented on. AGENTS.md "Ports (quick reference)"
# is the authority; `check_port_table` fails when this map drifts from it.
SERVICES: dict[str, set[int]] = {
    "botserver": {8080, 5858},
    "suite": {3000},
    "cloud": {4000},
    "login": {5000},
    "forgejo": {4747},
    "alm": {4747},
    "coredns": {53},
    "postgres": {5432},
    "postgresql": {5432},
    "valkey": {6379},
    "redis": {6379},
    "minio": {9000, 9100},
    "vault": {8200},
    "qdrant": {6333},
    "livekit": {7880},
    "caddy": {80, 443},
    "stalwart": {25, 465, 587, 993},
}
ANCHORED_SERVICES = {"botserver", "suite", "cloud", "login", "forgejo", "alm"}
SERVICE_RE = re.compile(r"\b(" + "|".join(sorted(SERVICES, key=len, reverse=True)) + r")\b")
LINK_RE = re.compile(r"\]\(([^)\s]+)\)")
HEADING_RE = re.compile(r"^(#{1,6})\s+(.*)$")
VERSION_RE = re.compile(
    r"(?:General\s*Bots|BotBook)[^\n]{0,40}?\bv?(\d+\.\d+\.\d+)\b|\bv(\d+\.\d+\.\d+)\b",
    re.I,
)
PORT_RE = re.compile(r"\bport\s+(\d{2,5})\b|:(\d{4,5})\b")
PRODUCT_MAJORS = {"6", "7"}
ALLOWED_ORPHANS = {"README.md", "SUMMARY.md"}


@dataclass(frozen=True)
class Violation:
    kind: str
    path: str
    line: int
    detail: str
    message: str

    def key(self) -> tuple[str, str, str]:
        return (self.kind, self.path, self.detail)

    def render(self) -> str:
        return f"{self.path}:{self.line}: {self.kind} {self.message}"


@lru_cache(maxsize=None)
def read(path: Path) -> str:
    """Page text, cached for one run (cleared by `run_all`)."""
    return path.read_text(encoding="utf-8", errors="replace")


def slugify(heading: str) -> str:
    text = heading.strip().lower().replace("`", "")
    text = re.sub(r"[^\w\s-]", "", text)
    return re.sub(r"\s+", "-", text)


def pages(book: Path) -> list[Path]:
    return sorted(book.rglob("*.md"))


def published(book: Path) -> set[str]:
    text = read(book / "SUMMARY.md") if (book / "SUMMARY.md").is_file() else ""
    return {match.group(1)[2:] for match in re.finditer(r"\]\((\./[^)#]+)\)", text)}


def check_summary(book: Path) -> list[Violation]:
    summary = book / "SUMMARY.md"
    if not summary.is_file():
        return [Violation("SUMMARY_MISSING", "botbook/src/SUMMARY.md", 1, "SUMMARY.md", "file is absent")]
    out: list[Violation] = []
    for number, line in enumerate(read(summary).splitlines(), 1):
        for match in LINK_RE.finditer(line):
            target = match.group(1)
            if target.startswith(("http://", "https://", "mailto:", "#", "/")):
                continue
            path = (book / target[2:]) if target.startswith("./") else (summary.parent / target)
            rel = f"botbook/src/{path.relative_to(book).as_posix()}" if book in path.parents else target
            if not path.is_file():
                out.append(Violation("SUMMARY_MISSING", rel, number, target,
                                     f"chapter target {target} does not exist"))
                continue
            body = re.sub(r"\s+", "", read(path))
            if len(body) < 40:
                out.append(Violation("SUMMARY_EMPTY", rel, number, target,
                                     f"chapter {target} has no body text"))
    return out


def check_links(book: Path) -> list[Violation]:
    known = published(book)
    headings = {
        path.relative_to(book).as_posix(): {
            slugify(match.group(2)) for match in
            (HEADING_RE.match(line) for line in read(path).splitlines()) if match
        }
        for path in pages(book)
    }
    out: list[Violation] = []
    for path in pages(book):
        rel = path.relative_to(book).as_posix()
        for number, line in enumerate(read(path).splitlines(), 1):
            for match in LINK_RE.finditer(line):
                target = match.group(1)
                if target.startswith(("http://", "https://", "mailto:", "#", "/")):
                    continue
                file_part, _, anchor = target.partition("#")
                if not file_part:
                    continue
                resolved = (path.parent / file_part).resolve()
                try:
                    key = resolved.relative_to(book.resolve()).as_posix()
                except ValueError:
                    key = None
                if not resolved.exists():
                    out.append(Violation("LINK_MISSING", f"botbook/src/{rel}", number, target,
                                         f"link target {target} does not exist"))
                    continue
                if key and key.endswith(".md") and key not in known and key not in ALLOWED_ORPHANS:
                    out.append(Violation("LINK_UNPUBLISHED", f"botbook/src/{rel}", number, target,
                                         f"link target {target} is not in SUMMARY.md"))
                    continue
                if anchor and key in headings and anchor not in headings[key]:
                    out.append(Violation("BAD_ANCHOR", f"botbook/src/{rel}", number, target,
                                         f"anchor #{anchor} not found in {file_part}"))
    return out


def product_version() -> str:
    match = re.search(r'^version\s*=\s*"([^"]+)"', read(CARGO), re.M)
    return match.group(1) if match else ""


def check_versions(book: Path) -> list[Violation]:
    current = product_version()
    if not current:
        return [Violation("VERSION_DRIFT", "Cargo.toml", 1, "version", "no version in Cargo.toml")]
    out: list[Violation] = []
    for path in pages(book):
        rel = path.relative_to(book).as_posix()
        for number, line in enumerate(read(path).splitlines(), 1):
            for match in VERSION_RE.finditer(line):
                claimed = match.group(1) or match.group(2)
                if match.group(1) is None and claimed.split(".")[0] not in PRODUCT_MAJORS:
                    continue
                if claimed != current:
                    out.append(Violation("VERSION_DRIFT", f"botbook/src/{rel}", number, claimed,
                                         f"claims version {claimed}, Cargo.toml has {current}"))
    return out


def check_ports(book: Path) -> list[Violation]:
    out: list[Violation] = []
    for path in pages(book):
        rel = path.relative_to(book).as_posix()
        for number, line in enumerate(read(path).splitlines(), 1):
            lowered = line.lower()
            named = set(SERVICE_RE.findall(lowered))
            if len(named) != 1:
                continue
            name = named.pop()
            near = [m.start() for m in re.finditer(rf"\b{name}\b", lowered)]
            for match in PORT_RE.finditer(line):
                # The service name must be next to the number: `botserver:9000`,
                # `botserver (port 9000)`. A bare number 30 characters away is a
                # different subject on the same line.
                if min(abs(match.start() - start) for start in near) > 30:
                    continue
                port = int(match.group(1) or match.group(2))
                if match.group(1) is None and name not in ANCHORED_SERVICES:
                    continue
                if port not in SERVICES[name]:
                    out.append(Violation(
                        "PORT_DRIFT", f"botbook/src/{rel}", number, f"{name}:{port}",
                        f"documents {name} on port {port}, AGENTS.md says "
                        f"{sorted(SERVICES[name])}",
                    ))
    return out


def check_port_table() -> list[Violation]:
    """AGENTS.md quick reference and this script must agree."""
    out: list[Violation] = []
    text = read(AGENTS) if AGENTS.is_file() else ""
    table = {name.strip(): int(port) for port, name in
             re.findall(r"(\d{2,5})\s*=\s*([a-z][a-z ]*)", text)}
    for name in ("botserver", "suite", "cloud", "login"):
        documented = table.get(name)
        if documented is not None and documented not in SERVICES[name]:
            out.append(Violation("PORT_TABLE", "AGENTS.md", 1, f"{name}:{documented}",
                                 f"AGENTS.md lists {name} on {documented}, guard expects "
                                 f"{sorted(SERVICES[name])}"))
    return out


def run_all(book: Path) -> list[Violation]:
    read.cache_clear()
    violations = check_summary(book) + check_links(book) + check_versions(book) + check_ports(book)
    violations += check_port_table()
    return sorted(violations, key=lambda v: (v.path, v.line, v.kind))


# ─── allowlist ───────────────────────────────────────────────────────────────

def load_allowlist() -> dict[tuple[str, str, str], str]:
    entries: dict[tuple[str, str, str], str] = {}
    if not ALLOWLIST.is_file():
        return entries
    for line in read(ALLOWLIST).splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("|")
        if len(parts) < 4:
            continue
        entries[(parts[0].strip(), parts[1].strip(), parts[2].strip())] = parts[3].strip()
    return entries


def write_allowlist(entries: dict[tuple[str, str, str], str]) -> None:
    lines = [
        "# BotBook guard allowlist (#2082). Reviewed debt, not a mute button.",
        "# Format: kind|path|detail|reason — the file may only shrink.",
        "",
    ]
    for (kind, path, detail), reason in sorted(entries.items()):
        lines.append(f"{kind}|{path}|{detail}|{reason}")
    ALLOWLIST.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ─── self-test ───────────────────────────────────────────────────────────────

SELF_TEST_CASES = {
    "LINK_MISSING": ("extra/bad-link.md", "# Bad link\n\nSee [missing](./does-not-exist.md).\n"),
    "BAD_ANCHOR": ("extra/bad-anchor.md", "# Bad anchor\n\nSee [wrong](./good.md#nope).\n"),
    "VERSION_DRIFT": ("extra/bad-version.md", "# Bad version\n\nGeneral Bots v6.4.0 shipped this.\n"),
    "PORT_DRIFT": ("extra/bad-port.md", "# Bad port\n\nThe botserver listens on port 9999.\n"),
}


def self_test() -> int:
    failures: list[str] = []
    with tempfile.TemporaryDirectory(prefix="botbook-guards-") as temp:
        book = Path(temp) / "src"
        shutil.copytree(BOOK, book)
        (book / "extra").mkdir(exist_ok=True)
        (book / "extra" / "good.md").write_text("# Good\n\nBody text.\n", encoding="utf-8")
        (book / "extra" / "empty.md").write_text("# Empty\n\nFiller body text long enough to pass the length gate.\n", encoding="utf-8")
        cases = dict(SELF_TEST_CASES)
        cases["SUMMARY_EMPTY"] = ("extra/empty.md", "# Empty\n")
        # Register every injected page so the anchor/version/port cases are not
        # masked by LINK_UNPUBLISHED.
        summary = read(book / "SUMMARY.md")
        summary += "\n- [Good](./extra/good.md)\n- [Empty](./extra/empty.md)\n"
        for rel, _ in cases.values():
            summary += f"- [Injected](./{rel})\n"
        (book / "SUMMARY.md").write_text(summary, encoding="utf-8")
        baseline = {v.key() for v in run_all(book)}
        for kind, (rel, body) in cases.items():
            target = book / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(body, encoding="utf-8")
            fresh = {v.key() for v in run_all(book)} - baseline
            if kind not in {key[0] for key in fresh}:
                failures.append(f"{kind}: injected defect not reported")
            if kind != "SUMMARY_EMPTY":
                target.unlink()
        # A clean tree must not report anything the baseline did not.
        leftover = {v.key() for v in run_all(book)} - baseline
        leftover = {key for key in leftover if key[0] != "SUMMARY_EMPTY"}
        if leftover:
            failures.append(f"clean tree reports {sorted(leftover)}")
    for failure in failures:
        print(f"self-test FAIL: {failure}")
    print(f"self-test: {len(SELF_TEST_CASES) + 1} cases, {len(failures)} failure(s)")
    return 1 if failures else 0


# ─── entry point ─────────────────────────────────────────────────────────────

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="exit 1 on a new violation")
    parser.add_argument("--prune", action="store_true", help="drop satisfied allowlist rows")
    parser.add_argument("--self-test", action="store_true", help="inject defects and assert exit 1")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    violations = run_all(BOOK)
    allow = load_allowlist()
    new = [v for v in violations if v.key() not in allow]
    stale = [key for key in allow if key not in {v.key() for v in violations}]
    if args.prune:
        write_allowlist({key: allow[key] for key in allow if key not in stale})
        print(f"pruned {len(stale)} allowlist row(s)")
        return 0

    for violation in new:
        print(violation.render())
    kinds: dict[str, int] = {}
    for violation in violations:
        kinds[violation.kind] = kinds.get(violation.kind, 0) + 1
    summary = ", ".join(f"{kind}={count}" for kind, count in sorted(kinds.items())) or "none"
    print(f"botbook_guards: {len(violations)} known, {len(new)} new, "
          f"{len(stale)} stale allowlist row(s) — {summary}")
    if stale:
        for kind, path, detail in stale[:10]:
            print(f"  stale: {kind}|{path}|{detail} (run --prune)")
    if args.check and new:
        print("botbook_guards: FAIL (fix the pages or add a reviewed allowlist row)")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
