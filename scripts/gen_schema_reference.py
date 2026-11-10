#!/usr/bin/env python3
"""Render half of the database reference generator (issue #2078).

Turns the inventory produced by `scripts/schema_parse.py` into the BotBook
pages under `botbook/src/12-ecosystem-reference/`.

Usage:
    python3 scripts/gen_schema_reference.py --report   # inventory to stdout
    python3 scripts/gen_schema_reference.py            # write the pages
    python3 scripts/gen_schema_reference.py --check    # fail on drift

Standard library only, no network, deterministic output (two runs are byte
identical because every collection is sorted and no timestamp is emitted).
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from schema_parse import (
    BOOK_REF, GEN_NOTE, OUT_DIR, REPO, build_inventory, read, report,
)


def md_escape(text: str) -> str:
    return text.replace("|", "\\|").replace("<", "&lt;").replace(">", "&gt;")


def table_section(t: dict) -> list[str]:
    lines = [f"### {t['name']}", ""]
    if t["comment"]:
        lines += [t["comment"], ""]
    gate = t["feature"] or "always available"
    mig = t["migration"] or "no migration (legacy / pre-consolidation)"
    lines += [f"- Domain: `{t['module']}` · Feature gate: `{gate}` · Migration: `{mig}`",
              f"- Rust source: `{t['source']}`", ""]
    lines += ["| Column | SQL type | Null | Default | Key | References |", "|---|---|---|---|---|---|"]
    for c in t["columns"]:
        keys = []
        if c["pk"]:
            keys.append("PK")
        if c["unique"]:
            keys.append("UQ")
        ref = ""
        if c["references"]:
            keys.append("FK")
            ref = f"{c['references'][0]}.{c['references'][1]}"
        lines.append("| {} | {} | {} | {} | {} | {} |".format(
            md_escape(c["name"]), md_escape(c["sql_type"]),
            "yes" if c["nullable"] else "no", md_escape(c["default"]) or "—",
            ", ".join(keys) or "—", ref or "—"))
    if t["indexes"]:
        lines += ["", "Indexes: " + ", ".join(f"`{i}`" for i in t["indexes"])]
    drift = []
    if t["missing_in_migrations"]:
        drift.append("not found in migrations: " + ", ".join(f"`{c}`" for c in t["missing_in_migrations"]))
    if t["extra_in_migrations"]:
        drift.append("in migrations only: " + ", ".join(f"`{c}`" for c in t["extra_in_migrations"]))
    if drift:
        lines += ["", "Drift: " + "; ".join(drift)]
    lines.append("")
    return lines


def domain_pages(inv: dict) -> dict[str, str]:
    by_domain: dict[str, list[dict]] = {}
    for t in inv["tables"].values():
        by_domain.setdefault(t["module"], []).append(t)
    pages: dict[str, str] = {}
    for domain in sorted(by_domain):
        sections = [table_section(t) for t in sorted(by_domain[domain], key=lambda x: x["name"])]
        chunks, cur, cur_lines = [], [], 0
        for sec in sections:
            if cur and cur_lines + len(sec) > 380:
                chunks.append(cur)
                cur, cur_lines = [], 0
            cur.append(sec)
            cur_lines += len(sec)
        if cur:
            chunks.append(cur)
        for idx, chunk in enumerate(chunks, start=1):
            fname = f"{domain}.md" if idx == 1 else f"{domain}-{idx}.md"
            head = [GEN_NOTE, "", f"# Schema: {domain}" + ("" if idx == 1 else f" (part {idx})"), "",
                    "Part of the [generated database reference](./index.md).", ""]
            pages[fname] = "\n".join(head + [ln for sec in chunk for ln in sec]).rstrip() + "\n"
    return pages


def collect_edges(inv: dict) -> list[dict]:
    tables = inv["tables"]
    edges = [e for e in inv["edges"] if e["from"] in tables or e["to"] in tables]
    known = {(e["from"], e["column"]) for e in edges}
    for t in tables.values():
        for c in t["columns"]:
            if c["references"] and (t["name"], c["name"]) not in known:
                edges.append({"from": t["name"], "to": c["references"][0],
                              "column": c["name"], "sql": True})
    return sorted(edges, key=lambda e: (e["from"], e["column"], e["to"]))


def relationships_page(inv: dict) -> str:
    tables, all_edges = inv["tables"], collect_edges(inv)
    linked = {e["from"] for e in all_edges} | {e["to"] for e in all_edges}
    by_domain: dict[str, int] = {}
    for e in all_edges:
        src = tables.get(e["from"])
        d = src["module"] if src else "unowned"
        by_domain[d] = by_domain.get(d, 0) + 1
    lines = [GEN_NOTE, "", "# Schema Relationships", "",
             "Every edge below comes from a `joinable!` in `botschema` or an explicit SQL",
             "`REFERENCES` in `botserver/migrations`. A `_id` suffix alone is never a",
             "foreign key. Each domain page carries the Mermaid graph and the edge table.",
             "", f"Total edges: **{len(all_edges)}** · tables with at least one edge: "
             f"**{len(linked)}** · tables without edges: **{len(tables) - len(linked)}**", "",
             "| Domain | Edges | Graph and edges |", "|---|---|---|"]
    for d in sorted(by_domain):
        lines.append(f"| `{d}` | {by_domain[d]} | [`{d}`](./schema/relationships-{d}.md) |")
    orphans = sorted(n for n in tables if n not in linked)
    lines += ["", "## Tables without relationships", "",
              f"{len(orphans)} tables declare no `joinable!` and no SQL `REFERENCES`:", "",
              ", ".join(f"`{n}`" for n in orphans) or "None."]
    return "\n".join(lines).rstrip() + "\n"


def relationship_pages(inv: dict) -> dict[str, str]:
    tables, all_edges = inv["tables"], collect_edges(inv)
    by_domain: dict[str, list[dict]] = {}
    for e in all_edges:
        src = tables.get(e["from"])
        by_domain.setdefault(src["module"] if src else "unowned", []).append(e)
    pages: dict[str, str] = {}
    for d in sorted(by_domain):
        blocks = []
        for e in by_domain[d]:
            nullable, src = False, tables.get(e["from"])
            if src:
                for c in src["columns"]:
                    if c["name"] == e["column"]:
                        nullable = c["nullable"]
            blocks.append((f"  {e['from']} {'-.->' if nullable else '-->'}|{e['column']}| {e['to']}",
                           f"| `{e['from']}` | `{e['column']}` | `{e['to']}` | "
                           f"{'yes' if nullable else 'no'} | "
                           f"{'migrations' if e.get('sql') else 'joinable!'} |"))
        chunks, cur = [], []
        for b in blocks:
            if cur and len(cur) * 3 + 20 > 380:
                chunks.append(cur)
                cur = []
            cur.append(b)
        chunks.append(cur)
        for idx, chunk in enumerate(chunks, start=1):
            name = f"relationships-{d}.md" if idx == 1 else f"relationships-{d}-{idx}.md"
            lines = [GEN_NOTE, "", f"# Relationships: {d}" + ("" if idx == 1 else f" (part {idx})"), "",
                     "Part of the [relationship reference](./../relationships.md). Dashed edges are",
                     "nullable foreign keys.", "", "```mermaid", "graph LR"]
            lines += [b[0] for b in chunk]
            lines += ["```", "", "| From | Column | To | Nullable | Source |", "|---|---|---|---|---|"]
            lines += [b[1] for b in chunk]
            pages[name] = "\n".join(lines).rstrip() + "\n"
    return pages


def index_page(inv: dict, pages: dict[str, str]) -> str:
    tables = inv["tables"]
    by_domain: dict[str, int] = {}
    for t in tables.values():
        by_domain[t["module"]] = by_domain.get(t["module"], 0) + 1
    by_gate: dict[str, int] = {}
    for t in tables.values():
        g = t["feature"] or "always available"
        by_gate[g] = by_gate.get(g, 0) + 1
    lines = [GEN_NOTE, "", "# Generated Database Reference", "",
             "This reference is generated from `botschema/src/**` (`table!`, `joinable!`) and",
             "`botserver/migrations/*/up.sql` (primary keys, nullability, defaults, indexes,",
             "`REFERENCES`). Regenerate with:", "",
             "```bash", "python3 scripts/gen_schema_reference.py", "```", "",
             f"Tables: **{len(tables)}** · columns: **{sum(len(t['columns']) for t in tables.values())}** · "
             f"domains: **{len(by_domain)}** · join edges: **{len(inv['edges'])}** · "
             f"`allow_tables_to_appear_in_same_query!` groups: **{inv['groups']}**", "",
             "## Domains", "", "| Domain | Tables | Pages |", "|---|---|---|"]
    for d in sorted(by_domain):
        names = sorted(p for p in pages if p == f"{d}.md" or p.startswith(f"{d}-"))
        links = " · ".join(f"[{n[:-3]}](./{n})" for n in names)
        lines.append(f"| `{d}` | {by_domain[d]} | {links} |")
    lines += ["", "## Feature gates", "", "| Gate | Tables |", "|---|---|"]
    for g in sorted(by_gate):
        lines.append(f"| `{g}` | {by_gate[g]} |")
    lines += ["", "## See also", "",
              "- [Relationships](./../relationships.md) — every foreign-key edge and one graph per domain",
              "- [Tables index](./../tables.md) — flat list of all tables",
              "- [Schema overview](./../schema.md) — design principles, data types, migrations"]
    return "\n".join(lines).rstrip() + "\n"


def tables_page(inv: dict, pages: dict[str, str]) -> str:
    tables = inv["tables"]
    lines = [GEN_NOTE, "", "# Database Tables", "",
             f"All **{len(tables)}** tables declared in `botschema` are documented in the",
             "[generated schema reference](./schema/index.md); this page is only its index.",
             "Regenerate with `python3 scripts/gen_schema_reference.py`.", "",
             "| Table | Domain | Columns | Migration | Page |", "|---|---|---|---|---|"]
    for t in sorted(tables.values(), key=lambda x: x["name"]):
        names = sorted(p for p in pages if p == f"{t['module']}.md" or p.startswith(f"{t['module']}-"))
        page = f"[{t['module']}](./schema/{names[0]})" if names else "—"
        lines.append(f"| `{t['name']}` | `{t['module']}` | {len(t['columns'])} | "
                     f"{t['migration'] or '—'} | {page} |")
    return "\n".join(lines).rstrip() + "\n"


def render(inv: dict) -> dict[Path, str]:
    pages = domain_pages(inv)
    files = {OUT_DIR / name: content for name, content in pages.items()}
    files[OUT_DIR / "index.md"] = index_page(inv, pages)
    files[BOOK_REF / "tables.md"] = tables_page(inv, pages)
    files[BOOK_REF / "relationships.md"] = relationships_page(inv)
    for name, content in relationship_pages(inv).items():
        files[OUT_DIR / name] = content
    return files


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--report", action="store_true", help="print the inventory")
    ap.add_argument("--check", action="store_true", help="fail when pages are stale")
    args = ap.parse_args()
    inv = build_inventory()
    if args.report:
        report(inv)
        return 0
    files = render(inv)
    if args.check:
        stale = [p for p, body in files.items() if not p.is_file() or read(p) != body]
        summary = REPO / "botbook" / "src" / "SUMMARY.md"
        summary_text = read(summary) if summary.is_file() else ""
        book_src = REPO / "botbook" / "src"
        unregistered = [p for p in files
                        if book_src in p.parents and f"(./{p.relative_to(book_src)})" not in summary_text]
        for p in stale:
            print(f"stale: {p.relative_to(REPO)}")
        for p in unregistered:
            print(f"not in SUMMARY.md: {p.relative_to(book_src)}")
        return 1 if stale or unregistered else 0
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for path, body in sorted(files.items()):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body, encoding="utf-8")
    over = [p for p, b in files.items() if b.count("\n") > 450]
    print(f"wrote {len(files)} files: {len(inv['tables'])} tables, "
          f"{len(inv['edges'])} joinable! edges, {len(inv['migrations'])} migration tables")
    for p in over:
        print(f"WARNING: {p.relative_to(REPO)} exceeds 450 lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
