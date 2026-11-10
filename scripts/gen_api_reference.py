#!/usr/bin/env python3
"""Generate the BotBook REST reference and the WebSocket protocol page.

Everything written here is derived from the repository source. No network, no
running server, no timestamps: two consecutive runs produce byte-identical
files.

Sources
-------
REST
  * botserver/src/core/bot/endpoint_inventory*.rs and
    botserver/src/core/bot/api_catalog/*.rs — ``ApiEndpoint { method, path,
    summary }``, the authoritative catalog of what is intentionally exposed.
  * botserver/src/main_module/routes/**/*.rs and
    botserver/crates/*/src/**/*.rs — ``.route("...", get(handler))`` literals,
    the routes the server actually mounts.

WebSocket (rendered by the sibling module ``scripts/gen_api_ws.py``)
  * botserver/src/core/bot/ws/handler.rs — handshake query parameters.
  * botlib/src/message_types.rs — ``MessageType`` 0-9.
  * botlib/src/models/messaging.rs — the ``BotResponse`` frame.
  * botserver/src/core/bot/ws/** — inbound control frames and outbound frames.
  * botui/ui/suite/chat/** — control frames the client sends.

Usage
-----
    python3 scripts/gen_api_reference.py            # write the pages
    python3 scripts/gen_api_reference.py --check    # exit 1 if they are stale
"""

from __future__ import annotations

import argparse
import glob
import os
import re
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from gen_api_ws import build_ws  # noqa: E402  (local module, path set above)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
API_DIR = "botbook/src/08-rest-api-tools"
GEN_DIR = os.path.join(API_DIR, "generated")
WS_PAGE = "botbook/src/07-user-interface/ws-protocol.md"
SUMMARY = "botbook/src/SUMMARY.md"

SUMMARY_BEGIN = "<!-- BEGIN GENERATED API REFERENCE -->"
SUMMARY_END = "<!-- END GENERATED API REFERENCE -->"

MAX_ROWS = 380  # keeps every page well under the 450-line limit
MIN_FAMILY_ROWS = 8  # smaller families are grouped into api-misc*.md


def read(path: str) -> str:
    with open(os.path.join(ROOT, path), encoding="utf-8", errors="ignore") as handle:
        return handle.read()


def repo_path(path: str) -> str:
    return os.path.relpath(path, ROOT).replace(os.sep, "/")


def files(*patterns: str) -> list[str]:
    found: list[str] = []
    for pattern in patterns:
        found.extend(glob.glob(os.path.join(ROOT, pattern), recursive=True))
    return sorted(set(found))


# ─── REST inventory ──────────────────────────────────────────────────────────

ENDPOINT_RE = re.compile(
    r'ApiEndpoint\s*\{\s*method:\s*"([A-Z]+)",\s*path:\s*"([^"]+)",'
    r'\s*summary:\s*"((?:[^"\\]|\\.)*)"',
    re.S,
)


def scan_inventory() -> dict[tuple[str, str], dict]:
    entries: dict[tuple[str, str], dict] = {}
    paths = files(
        "botserver/src/core/bot/endpoint_inventory*.rs",
        "botserver/src/core/bot/api_catalog/*.rs",
    )
    for path in paths:
        text = read(path)
        for method, route, summary in ENDPOINT_RE.findall(text):
            key = (method, normalize(route))
            entries.setdefault(
                key,
                {"summary": summary.replace('\\"', '"'), "source": repo_path(path)},
            )
    return entries


ROUTE_RE = re.compile(r'\.route\(\s*"([^"]+)"\s*,')
METHOD_RE = re.compile(r'\b(get|post|put|delete|patch|head|options|any)\s*\(')


def scan_routes() -> tuple[dict[tuple[str, str], list[str]], int]:
    """Return {(method, path): [source files]} plus the number of nests."""
    routes: dict[tuple[str, str], list[str]] = defaultdict(list)
    nests = 0
    for path in files(
        "botserver/src/main_module/routes/**/*.rs",
        "botserver/crates/*/src/**/*.rs",
    ):
        text = read(path)
        nests += len(re.findall(r"\.nest\(", text))
        for match in ROUTE_RE.finditer(text):
            route = match.group(1)
            tail = text[match.end(): match.end() + 400]
            methods = sorted({m.upper() for m in METHOD_RE.findall(tail)})
            for method in methods or ["ANY"]:
                source = repo_path(path)
                if source not in routes[(method, normalize(route))]:
                    routes[(method, normalize(route))].append(source)
    return routes, nests


PLACEHOLDER_RE = re.compile(r"(:[A-Za-z_][A-Za-z0-9_]*|\{[^}]+\}|\*[A-Za-z0-9_]*)")


def normalize(path: str) -> str:
    """Collapse every path parameter to `:param` so inventory and routes merge."""
    cleaned = PLACEHOLDER_RE.sub(":param", path)
    cleaned = re.sub(r"/+$", "", cleaned)
    return cleaned or "/"


def family(path: str) -> str:
    parts = [p for p in path.split("/") if p]
    if not parts:
        return "root"
    name = parts[1] if parts[0] == "api" and len(parts) > 1 else parts[0]
    return sanitize_family(name)


def sanitize_family(name: str) -> str:
    """File-name safe family label: parameters become `param`."""
    cleaned = PLACEHOLDER_RE.sub("param", name)
    cleaned = re.sub(r"[^A-Za-z0-9_-]", "-", cleaned).strip("-").lower()
    return cleaned or "root"


# ─── rendering ───────────────────────────────────────────────────────────────

HEADER = """<!-- GENERATED by scripts/gen_api_reference.py — do not edit by hand. -->

# {title}

{intro}

Base URL: `http://localhost:8080` — see
[Network Contract](../../02-architecture-packages/network-contract.md).

| Method | Path | Summary | Inventory | Registered in |
|---|---|---|---|---|
"""


def render_family_page(name: str, rows: list[dict], part: int, total_parts: int) -> str:
    suffix = "" if part == 1 else f" (part {part} of {total_parts})"
    intro = (
        f"{len(rows)} routes in the `{name}` family. `Inventory` says whether the "
        "route is also listed in the API command catalog; `Registered in` is the "
        "file that mounts it."
    )
    out = [HEADER.format(title=f"API / {name}{suffix}", intro=intro)]
    for row in rows:
        summary = row["summary"] or "schema not modeled"
        summary = summary.replace("|", "\\|")
        out.append(
            f"| {row['method']} | `{row['path']}` | {summary} | "
            f"{'yes' if row['inventory'] else 'no'} | `{row['source'] or '—'}` |\n"
        )
    out.append("\n[All families](./README.md)\n")
    return "".join(out)


def render_misc_page(rows: list[dict], part: int, total_parts: int) -> str:
    suffix = "" if part == 1 else f" (part {part} of {total_parts})"
    out = [
        "<!-- GENERATED by scripts/gen_api_reference.py — do not edit by hand. -->\n\n",
        f"# API / grouped small families{suffix}\n\n",
        f"Families with fewer than {MIN_FAMILY_ROWS} routes, grouped so the book "
        "stays navigable. `Inventory` and `Registered in` mean the same as on the "
        "family pages.\n",
    ]
    current = None
    for row in rows:
        if row["family"] != current:
            current = row["family"]
            out.append(
                f"\n## `{current}`\n\n"
                "| Method | Path | Summary | Inventory | Registered in |\n"
                "|---|---|---|---|---|\n"
            )
        summary = (row["summary"] or "schema not modeled").replace("|", "\\|")
        out.append(
            f"| {row['method']} | `{row['path']}` | {summary} | "
            f"{'yes' if row['inventory'] else 'no'} | `{row['source'] or '—'}` |\n"
        )
    out.append("\n[All families](./README.md)\n")
    return "".join(out)


def chunk_by_lines(rows: list[dict], budget: int = 400) -> list[list[dict]]:
    """Split rows so a rendered page stays under `budget` lines."""
    chunks: list[list[dict]] = []
    current: list[dict] = []
    lines = 8  # page header + table header
    family = None
    for row in rows:
        cost = 1
        if row["family"] != family:
            cost += 3  # section heading + table header
            family = row["family"]
        if current and lines + cost > budget:
            chunks.append(current)
            current, lines, family = [], 8, None
            cost = 1 + 3
        current.append(row)
        lines += cost
    if current:
        chunks.append(current)
    return chunks


def fit_chunks(rows: list[dict], render) -> list[list[dict]]:
    """Split rows until every rendered page is under the 450-line limit."""
    chunks = chunk_by_lines(rows)
    out: list[list[dict]] = []
    for chunk in chunks:
        out.extend(_fit(chunk, render))
    return out


def _fit(chunk: list[dict], render) -> list[list[dict]]:
    if len(chunk) <= 1:
        return [chunk]
    if len(render(chunk, 1, 1).splitlines()) <= 440:
        return [chunk]
    middle = len(chunk) // 2
    return _fit(chunk[:middle], render) + _fit(chunk[middle:], render)


def build_rest() -> tuple[dict[str, str], dict]:
    inventory = scan_inventory()
    routes, nests = scan_routes()
    merged: dict[tuple[str, str], dict] = {}
    for key in set(inventory) | set(routes):
        method, path = key
        entry = inventory.get(key)
        sources = routes.get(key, [])
        merged[key] = {
            "method": method,
            "path": path,
            "summary": entry["summary"] if entry else "",
            "inventory": entry is not None,
            "source": sources[0] if sources else (entry["source"] if entry else ""),
            "family": family(path),
        }

    by_family: dict[str, list[dict]] = defaultdict(list)
    for row in merged.values():
        by_family[row["family"]].append(row)

    pages: dict[str, str] = {}
    index_rows = []
    misc: list[dict] = []
    for name in sorted(by_family):
        rows = sorted(by_family[name], key=lambda r: (r["path"], r["method"]))
        if len(rows) < MIN_FAMILY_ROWS:
            misc.extend(rows)
            continue
        chunks = fit_chunks(rows, lambda chunk, i, n: render_family_page(name, chunk, i, n))
        links = []
        for number, chunk in enumerate(chunks, start=1):
            filename = f"api-{name}.md" if number == 1 else f"api-{name}-{number}.md"
            pages[os.path.join(GEN_DIR, filename)] = render_family_page(
                name, chunk, number, len(chunks)
            )
            links.append(f"[{filename[:-3]}](./{filename})")
        index_rows.append((name, len(rows), links))

    if misc:
        misc.sort(key=lambda r: (r["family"], r["path"], r["method"]))
        chunks = fit_chunks(misc, lambda chunk, i, n: render_misc_page(chunk, i, n))
        links = []
        for number, chunk in enumerate(chunks, start=1):
            filename = "api-misc.md" if number == 1 else f"api-misc-{number}.md"
            pages[os.path.join(GEN_DIR, filename)] = render_misc_page(chunk, number, len(chunks))
            links.append(f"[{filename[:-3]}](./{filename})")
        index_rows.append((f"misc ({MIN_FAMILY_ROWS - 1} routes or fewer, grouped)", len(misc), links))

    only_inventory = sum(1 for r in merged.values() if r["inventory"] and not routes.get((r["method"], r["path"])))
    only_route = sum(1 for r in merged.values() if not r["inventory"])
    stats = {
        "routes": len(routes),
        "inventory": len(inventory),
        "merged": len(merged),
        "families": len(by_family),
        "nests": nests,
        "only_inventory": only_inventory,
        "only_route": only_route,
    }

    readme = [
        "<!-- GENERATED by scripts/gen_api_reference.py — do not edit by hand. -->\n\n",
        "# Generated REST reference 🟡 BETA\n\n",
        "Every route this server can mount, derived from the code. Prose belongs\n",
        "in the hand-written pages of this chapter; this directory is a table.\n\n",
        "## Regenerate\n\n",
        "```bash\n",
        "python3 scripts/gen_api_reference.py           # rewrite the pages\n",
        "python3 scripts/gen_api_reference.py --check   # fail when stale\n",
        "```\n\n",
        "The generator reads `botserver/src/core/bot/endpoint_inventory*.rs` and\n",
        "`botserver/src/core/bot/api_catalog/*.rs` (the intentional catalog) and the\n",
        "`.route(\"...\")` literals under `botserver/src/main_module/routes/` and\n",
        "`botserver/crates/*/src/`. Path parameters are normalized to `:param`\n",
        "before the two sources are merged.\n\n",
        "## What the columns mean\n\n",
        "| Column | Meaning |\n|---|---|\n",
        "| Method | HTTP verb taken from the method router (`get(`, `post(`, …) |\n",
        "| Path | normalized route, `:param` for any parameter |\n",
        "| Summary | catalog summary, or `schema not modeled` when there is none |\n",
        "| Inventory | `yes` when the API command catalog also lists the route |\n",
        "| Registered in | the file that mounts the route |\n\n",
        "`schema not modeled` is literal: the handler has no typed schema, so no\n",
        "request/response fields are invented here.\n\n",
        f"## Counts\n\n",
        f"- method/path pairs parsed from route literals: **{stats['routes']}**\n",
        f"- catalog entries: **{stats['inventory']}**\n",
        f"- merged routes: **{stats['merged']}**\n",
        f"- catalog-only (no route literal found): **{stats['only_inventory']}**\n",
        f"- route-only (no catalog summary): **{stats['only_route']}**\n",
        f"- families: **{stats['families']}**\n",
        f"- `.nest(` calls (mount prefixes, unresolved): **{stats['nests']}**\n\n",
        "A catalog-only entry is usually a route mounted inside a nested router;\n",
        "`prefix: unresolved` is reported instead of guessing the prefix.\n\n",
        "## Families\n\n",
        "| Family | Routes | Pages |\n|---|---:|---|\n",
    ]
    for name, count, links in index_rows:
        readme.append(f"| `{name}` | {count} | {', '.join(links)} |\n")
    pages[os.path.join(GEN_DIR, "README.md")] = "".join(readme)
    return pages, stats


# ─── SUMMARY block ───────────────────────────────────────────────────────────

def summary_block(pages: dict[str, str]) -> str:
    names = sorted(
        os.path.basename(p)[:-3] for p in pages if p.startswith(GEN_DIR) and p.endswith(".md")
    )
    lines = [SUMMARY_BEGIN + "\n", "   - [Generated REST reference](./08-rest-api-tools/generated/README.md)\n"]
    for name in names:
        if name == "README":
            continue
        lines.append(f"   - [{name}](./08-rest-api-tools/generated/{name}.md)\n")
    lines.append("   - [WebSocket Protocol](./07-user-interface/ws-protocol.md)\n")
    lines.append(SUMMARY_END + "\n")
    return "".join(lines)


def update_summary(current: str, block: str) -> str:
    if SUMMARY_BEGIN in current and SUMMARY_END in current:
        start = current.index(SUMMARY_BEGIN)
        end = current.index(SUMMARY_END) + len(SUMMARY_END) + 1
        return current[:start] + block + current[end:]
    anchor = "- [Chapter 8: REST API & Tools](./08-rest-api-tools/README.md)\n"
    return current.replace(anchor, anchor + block, 1)


# ─── main ────────────────────────────────────────────────────────────────────

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail when committed pages are stale")
    args = parser.parse_args()

    pages, stats = build_rest()
    pages[WS_PAGE] = build_ws()
    pages[SUMMARY] = update_summary(read(SUMMARY), summary_block(pages))

    stale = []
    for path, content in sorted(pages.items()):
        full = os.path.join(ROOT, path)
        existing = None
        if os.path.exists(full):
            with open(full, encoding="utf-8") as handle:
                existing = handle.read()
        if existing == content:
            continue
        stale.append(path)
        if not args.check:
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, "w", encoding="utf-8") as handle:
                handle.write(content)

    if args.check:
        if stale:
            print("gen_api_reference: stale files (run without --check):")
            for path in stale:
                print(f"  {path}")
            return 1
        print("gen_api_reference: up to date")
        return 0

    print(
        "gen_api_reference: wrote {} files "
        "(routes={routes} catalog={inventory} merged={merged} families={families} nests={nests})".format(
            len(stale), **stats
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
