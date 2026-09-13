#!/usr/bin/env python3
"""Generate botbook's Suite Apps status page from the repository itself.

Sources of truth:
  - botserver/src/apps/registry.rs   app catalog (id, title, category, description)
  - botbook/src/07-user-interface/apps/*.md   page coverage
  - botbook/src/assets/suite/*.svg            screen coverage

Run from the repository root:
    python3 scripts/docs_suite_apps_status.py
"""
import os
import re
import sys

REGISTRY = "botserver/src/apps/registry.rs"
DOCS_DIR = "botbook/src/07-user-interface/apps"
SVG_DIR = "botbook/src/assets/suite"
OUT = os.path.join(DOCS_DIR, "suite-apps-status.md")

CATEGORY_LABELS = {
    "ai": "AI & Assistants",
    "business": "Business",
    "office": "Office & Productivity",
    "dev": "Development",
    "system": "System & Tools",
}

# Stability, verified with the maintainer for September 2026.
STABLE = {"chat", "drive", "vibe"}
ADVANCED = {"mail", "sheet"}
IN_TEST = {"docs", "slides"}


def esc(text):
    """Escape text for XML text nodes and attribute values."""
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def parse_catalog():
    src = open(REGISTRY, encoding="utf-8").read()
    body = src[src.index("pub fn all_apps()"):]
    apps = []
    # app("id", "Title", "category", "#color",\n "/url",\n "description",\n "kw",\n "icon")
    for m in re.finditer(
        r'\b(?:widget_)?app\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*'
        r'"([^"]+)"\s*,\s*"([^"]+)"',
        body,
    ):
        apps.append(
            {
                "id": m.group(1),
                "title": m.group(2),
                "category": m.group(3),
                "url": m.group(5),
                "description": m.group(6),
                "widget": "widget_app(" in m.group(0),
            }
        )
    return apps


def stability(app_id):
    if app_id in STABLE:
        return "Stable"
    if app_id in ADVANCED:
        return "Preview — advanced"
    if app_id in IN_TEST:
        return "Preview — in test"
    return "Preview"


def build_svg(apps, docs, svgs):
    total = len(apps)
    have_doc = sum(1 for a in apps if a["id"] in docs)
    have_svg = sum(1 for a in apps if f'{a["id"]}-screen' in svgs)
    pct = round(100 * have_doc / total) if total else 0

    by_cat = {}
    for a in apps:
        c = a["category"]
        by_cat.setdefault(c, [0, 0, 0])
        by_cat[c][0] += 1
        if a["id"] in docs:
            by_cat[c][1] += 1
        if f'{a["id"]}-screen' in svgs:
            by_cat[c][2] += 1

    rows = sorted(by_cat.items(), key=lambda kv: CATEGORY_LABELS.get(kv[0], kv[0]))
    height = 150 + 46 * len(rows)
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 660 {height}" '
        f'font-family="system-ui, -apple-system, sans-serif" role="img" '
        f'aria-label="Suite documentation coverage: {have_doc} of {total} catalog apps have a page">',
        f'  <rect width="660" height="{height}" rx="12" fill="#0d1117"/>',
        '  <text x="330" y="38" text-anchor="middle" fill="#e6edf3" font-size="17" font-weight="600">'
        "Suite Documentation Coverage</text>",
        f'  <text x="330" y="62" text-anchor="middle" fill="#8b949e" font-size="12">'
        f"{have_doc} of {total} catalog apps documented \u00b7 {have_svg} with a screen diagram \u00b7 {pct}%</text>",
    ]

    y = 92
    for cat, (n, d, s) in rows:
        label = CATEGORY_LABELS.get(cat, cat)
        frac = (d / n) if n else 0
        color = "#3fb950" if frac == 1 else ("#d29922" if frac >= 0.5 else "#f85149")
        parts.append(
            f'  <text x="20" y="{y + 12}" fill="#8b949e" font-size="12">{esc(label)}</text>'
        )
        parts.append(f'  <rect x="180" y="{y}" width="360" height="16" rx="4" fill="#161b22"/>')
        if d:
            parts.append(
                f'  <rect x="180" y="{y}" width="{round(360 * frac)}" height="16" rx="4" fill="{color}"/>'
            )
        parts.append(
            f'  <text x="556" y="{y + 13}" fill="#e6edf3" font-size="12">{d}/{n} pages \u00b7 {s} screens</text>'
        )
        y += 46

    parts.append(
        f'  <text x="20" y="{y + 14}" fill="#8b949e" font-size="11">'
        f"Generated from registry.rs by scripts/docs_suite_apps_status.py \u2014 do not edit by hand</text>"
    )
    parts.append("</svg>")
    return "\n".join(parts)


def main():
    apps = parse_catalog()
    if not apps:
        print("ERROR: no apps parsed from the catalog", file=sys.stderr)
        return 1

    docs = {f[:-3] for f in os.listdir(DOCS_DIR) if f.endswith(".md")}
    svgs = {f[:-4] for f in os.listdir(SVG_DIR) if f.endswith(".svg")}

    covered = [a for a in apps if a["id"] in docs]
    missing = [a for a in apps if a["id"] not in docs]
    catalog_ids_set = {a["id"] for a in apps}
    non_app = {}
    extra = []
    for d in sorted(docs):
        if d in catalog_ids_set or d in {"README", "suite-apps-status"}:
            continue
        path = os.path.join(DOCS_DIR, f"{d}.md")
        text = open(path, encoding="utf-8").read()
        if "botbook:not-an-app" in text:
            non_app[d] = text
        else:
            extra.append(d)
    orphan_svgs = sorted(
        s
        for s in svgs
        if s.endswith("-screen") and s[:-7] not in {a["id"] for a in apps}
    )

    total = len(apps)
    have_svg = sum(1 for a in apps if f'{a["id"]}-screen' in svgs)

    out = []
    out.append("# Suite Apps Status 🟡 BETA\n")
    out.append(
        "This page is generated from the application catalog in "
        "`botserver/src/apps/registry.rs`, so the counts below match the software. "
        "Do not edit it by hand — run `python3 scripts/docs_suite_apps_status.py`.\n"
    )
    out.append(
        f"**Verified September 2026.** {total} applications in the catalog, "
        f"{len(covered)} documented, {len(missing)} without a page, "
        f"{have_svg} with a screen diagram.\n"
    )

    out.append("## Coverage\n")
    out.append(build_svg(apps, docs, svgs) + "\n")

    out.append("## Stability\n")
    out.append(
        "Chat, Explorer (Drive) and Vibe are the stable surface. Mail and Sheets are "
        "the most advanced preview apps. Docs and Slides are in test. Everything else "
        "is preview — installed and usable, but withheld from the launcher until "
        "Preview mode is on.\n"
    )
    out.append("| Stability | Count |\n|---|---|")
    counts = {}
    for a in apps:
        counts[stability(a["id"])] = counts.get(stability(a["id"]), 0) + 1
    for k in ["Stable", "Preview — advanced", "Preview — in test", "Preview"]:
        out.append(f"| {k} | {counts.get(k, 0)} |")
    out.append("")

    out.append("## Application inventory\n")
    out.append("| App | Title | Category | Stability | Page | Screen |")
    out.append("|-----|-------|----------|-----------|:----:|:------:|")
    for a in sorted(apps, key=lambda x: x["id"]):
        doc = "\u2705" if a["id"] in docs else "\u274c"
        scr = "\u2705" if f'{a["id"]}-screen' in svgs else "\u274c"
        title = a["title"].replace("|", "\\|")
        out.append(
            f"| `{a['id']}` | {title} | {CATEGORY_LABELS.get(a['category'], a['category'])} "
            f"| {stability(a['id'])} | {doc} | {scr} |"
        )
    out.append("")

    if missing:
        out.append("## Applications without a page\n")
        out.append("| App | Title | Description (from the catalog) |")
        out.append("|-----|-------|--------------------------------|")
        for a in sorted(missing, key=lambda x: x["id"]):
            desc = a["description"].replace("|", "\\|")
            out.append(f"| `{a['id']}` | {a['title']} | {desc} |")
        out.append("")

    if non_app:
        out.append("## Documented surfaces that are not launcher apps\n")
        out.append(
            "These pages document part of the suite that cannot be opened from the app menu: "
            "the shells, the authentication surface, API reference material and redirect notes. "
            "Each states what it is at the top of the page.\n"
        )
        out.append("| Page | What it is |")
        out.append("|---|---|")
        for name, text in non_app.items():
            reason = ""
            m = re.search(r'botbook:not-an-app reason="([^"]+)"', text)
            if m:
                reason = m.group(1)
            out.append(f"| `{name}` | {reason} |")
        out.append("")

    if extra:
        out.append("## Pages that are not catalog apps\n")
        out.append(
            "These pages describe something other than a launcher app — a chapter, a "
            "build plan, or a surface that is not in the catalog. Each needs to be "
            "folded into its real owner, marked as non-app documentation, or removed.\n"
        )
        for e in extra:
            out.append(f"- `{e}`")
        out.append("")

    if orphan_svgs:
        out.append("## Screen diagrams with no catalog app\n")
        out.append(
            "Diagrams for applications that no longer exist in the catalog. Delete them "
            "or restore the app.\n"
        )
        for s in orphan_svgs:
            out.append(f"- `{s}.svg`")
        out.append("")

    out.append("## Planned against done\n")
    out.append(
        "Progress is stated as counts, not dates — nothing here promises a release "
        "window. The \"done\" column is measured from the repository every time this "
        "page is regenerated.\n"
    )
    out.append("| Area | Done | Outstanding |")
    out.append("|---|---|---|")
    out.append(
        f"| App pages | {len(covered)} documented | {len(missing)} applications have no page |"
    )
    out.append(
        f"| Screen diagrams | {have_svg} apps have a screen | {len(apps) - have_svg} applications have none |"
    )
    out.append(
        f"| Non-app pages | {len(non_app)} classified | {len(extra)} pages still need a decision |"
    )
    out.append(
        f"| Screen diagrams for removed apps | 0 resolved | {len(orphan_svgs)} diagrams have no catalog app |"
    )
    out.append("")
    out.append(
        "Per-application migration to the documented set is tracked in the issue "
        "tracker rather than on this page, so the two cannot disagree.\n"
    )

    out.append("## See Also\n")
    out.append("- [Apps overview](./README.md) - What the suite contains")
    out.append(
        "- [Product configuration](../../12-ecosystem-reference/README.md#preview_apps) "
        "- The `apps` and `preview_apps` lists"
    )
    out.append("")

    open(OUT, "w", encoding="utf-8").write("\n".join(out))
    print(f"wrote {OUT}")
    print(f"catalog={total} documented={len(covered)} missing={len(missing)} screens={have_svg}")
    print(f"non-app pages={len(extra)} orphan screens={len(orphan_svgs)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
