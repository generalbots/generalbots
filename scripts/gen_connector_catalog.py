#!/usr/bin/env python3
"""Connector catalog generator (#2080): `botbook/src/06-channels/connectors/`.

Sources, all read directly:

* ``botserver/src/apps/integration_catalog/providers/*.rs`` — the seeded
  providers (``provider("slug", "Name", ProviderSpec { .. })``).
* ``botserver/src/apps/integration_catalog/auth.rs`` — ``AuthProfile``
  constants: auth method and credential field keys (never values).
* ``botserver/src/apps/integration_catalog/actions/*.rs`` — action template
  sets: key, verb, risk.
* ``botserver/crates/botintegrations/src/providers/**`` — the live adapter
  registry. A provider is ``built`` only when an adapter exists and every
  advertised action is implemented, mirroring
  ``integration_catalog::derived_status`` (#2048).

Usage
-----
    python3 scripts/gen_connector_catalog.py --report   # inventory
    python3 scripts/gen_connector_catalog.py --check    # exit 1 when stale
"""

from __future__ import annotations

import argparse
import re
import sys

from catalog_common import (
    ADAPTERS, CATALOG, CONN_BEGIN, CONN_DIR, CONN_END, MAX_PAGE_LINES, anchor, cell, read,
)

CATEGORY_ORDER = [
    "featured", "llm", "productivity", "developer", "startups",
    "small_business", "finance", "social_messaging", "lifestyle",
]
CATEGORY_TITLES = {
    "featured": "Featured", "llm": "LLM Providers", "productivity": "Productivity",
    "developer": "Developer", "startups": "Startups", "small_business": "Small Business",
    "finance": "Finance", "social_messaging": "Social & Messaging", "lifestyle": "Lifestyle",
}
STRATEGY_LABEL = {
    "integrate": "Integrate", "improve": "Improve", "new_app": "New app", "verify": "Verify",
}
AUTH_LABEL = {
    "oauth2": "OAuth 2.0", "api_key": "API key", "basic": "Basic", "token": "Token",
    "protocol": "Protocol", "access_key": "Access key", "unknown": "Unknown",
    "unsupported": "Unsupported",
}
AUTH_VARIANTS = {
    "OAuth2": "oauth2", "ApiKey": "api_key", "Basic": "basic", "Token": "token",
    "Protocol": "protocol", "AccessKey": "access_key", "Unknown": "unknown",
    "Unsupported": "unsupported",
}
PROVIDER_CALL = re.compile(r'provider\(\s*"')


def snake(variant: str) -> str:
    """`ApiKey` → `api_key`; keeps the label table readable."""
    return AUTH_VARIANTS.get(variant, variant.lower())


def parse_providers() -> list[dict]:
    providers: list[dict] = []
    for path in sorted((CATALOG / "providers").glob("*.rs")):
        text = read(path)
        starts = [m.start() for m in PROVIDER_CALL.finditer(text)]
        for index, start in enumerate(starts):
            end = starts[index + 1] if index + 1 < len(starts) else len(text)
            chunk = text[start:end]
            head = re.match(r'provider\(\s*"([a-z0-9_-]+)",\s*"([^"]+)"', chunk)
            if not head:
                continue
            category = re.search(r"category: Category::(\w+)", chunk)
            strategy = re.search(r"strategy: Strategy::(\w+)", chunk)
            priority = re.search(r"priority: Priority::(\w+)", chunk)
            module = re.search(r'module: Some\("([^"]+)"\)', chunk)
            docs = re.search(r'official_docs: Some\("([^"]+)"\)', chunk)
            auth = re.search(r"auth: &(\w+)", chunk)
            actions = re.search(r"actions: (\w+)", chunk)
            providers.append({
                "slug": head.group(1),
                "name": head.group(2),
                "category": (category.group(1) if category else "Unknown").lower()
                .replace("smallbusiness", "small_business").replace("socialmessaging", "social_messaging"),
                "strategy": (strategy.group(1) if strategy else "Unknown")
                .replace("NewApp", "new_app").lower(),
                "planned": bool(re.search(r"planned: true", chunk)),
                "priority": (priority.group(1) if priority else "?").lower(),
                "module": module.group(1) if module else "",
                "docs": docs.group(1) if docs else "",
                "auth": auth.group(1) if auth else "UNKNOWN",
                "actions": actions.group(1) if actions else "",
                "llm_available": "llm_available: true" in chunk,
                "source": f"botserver/src/apps/integration_catalog/providers/{path.name}",
            })
    return providers


def parse_auth() -> dict[str, dict]:
    text = read(CATALOG / "auth.rs")
    field_sets: dict[str, list[tuple[str, bool]]] = {}
    for match in re.finditer(r"(?:const|static) (\w+): &\[AuthField\] = &\[(.*?)\n\];", text, re.S):
        field_sets[match.group(1)] = [
            (f.group(1), f.group(3) == "true")
            for f in re.finditer(
                r'field\(\s*"([^"]+)",\s*"[^"]*",\s*InputType::\w+,\s*(true|false),\s*(true|false)',
                match.group(2),
            )
        ]
    profiles: dict[str, dict] = {}
    for match in re.finditer(r"static (\w+): AuthProfile = AuthProfile \{(.*?)\n\};", text, re.S):
        body = match.group(2)
        method = re.search(r"method: AuthMethod::(\w+)", body)
        fields = re.search(r"fields: (\w+)", body)
        least = re.search(r'least_privilege: "((?:[^"\\]|\\.)*)"', body)
        profiles[match.group(1)] = {
            "method": snake(method.group(1)) if method else "unknown",
            "fields": field_sets.get(fields.group(1), []) if fields else [],
            "least_privilege": least.group(1) if least else "",
        }
    return profiles


def parse_actions() -> dict[str, list[tuple[str, str, str]]]:
    """Action-set constant name → [(key, verb, risk)]."""
    sets: dict[str, list[tuple[str, str, str]]] = {}
    for path in sorted((CATALOG / "actions").glob("*.rs")):
        text = read(path)
        for match in re.finditer(r"const (\w+): &\[ActionTemplate\] = &\[(.*?)\n\];", text, re.S):
            risk = {"read": "low", "write": "medium", "destructive": "high"}
            sets[match.group(1)] = [
                (a.group(2), a.group(3), risk.get(a.group(1), "low"))
                for a in re.finditer(r'\b(read|write|destructive)\(\s*"([^"]+)",\s*"([^"]+)"', match.group(2))
            ]
    return sets


def _string_list(text: str, name: str) -> list[str]:
    match = re.search(rf"const {name}: &\[&str\] = &\[(.*?)\];", text, re.S)
    return re.findall(r'"([^"]+)"', match.group(1)) if match else []


def parse_adapters() -> dict[str, set[str]]:
    """Provider slug → implemented action keys, from the live adapter registry."""
    registry: dict[str, set[str]] = {}
    for path in sorted(ADAPTERS.rglob("*.rs")):
        text = read(path)
        for match in re.finditer(r"pub const (\w+_SPEC): ProviderSpec = ProviderSpec \{(.*?)\n\};", text, re.S):
            body = match.group(2)
            slug = re.search(r'slug:\s*"([a-z0-9_-]+)"', body)
            keys = re.search(r"action_keys:\s*(\w+)", body)
            if slug:
                registry.setdefault(slug.group(1), set()).update(
                    _string_list(text, keys.group(1)) if keys else []
                )
        provider = re.search(r'fn provider\(&self\) -> &\'static str \{\s*"([a-z0-9_-]+)"', text)
        if provider:
            names = re.search(
                r"fn implemented_actions\(&self\) -> &'static \[&'static str\] \{\s*(?:&\[(.*?)\]|(\w+))",
                text, re.S,
            )
            implemented: list[str] = []
            if names and names.group(1) is not None:
                implemented = re.findall(r'"([^"]+)"', names.group(1))
            elif names:
                implemented = _string_list(text, names.group(2))
            registry.setdefault(provider.group(1), set()).update(implemented)
    return registry


def derived_status(provider: dict, actions: list[tuple[str, str, str]],
                   registry: dict[str, set[str]]) -> tuple[str, int]:
    """Mirror of integration_catalog::derived_status (#2048)."""
    implemented_names = registry.get(provider["slug"], set())
    implemented = sum(
        1 for key, _, _ in actions
        if key in implemented_names or any(n.endswith("." + key) for n in implemented_names)
    )
    if provider["planned"]:
        return "planned", implemented
    present = provider["slug"] in registry
    adapted = present or provider["llm_available"] or bool(provider["module"])
    if present and actions and implemented == len(actions):
        return "built", implemented
    if adapted:
        return "partial", implemented
    return "unsupported", implemented


def build_connectors() -> tuple[dict[str, dict], dict[str, str]]:
    rows: dict[str, dict] = {}
    auth = parse_auth()
    action_sets = parse_actions()
    registry = parse_adapters()
    for provider in parse_providers():
        actions = action_sets.get(provider["actions"], [])
        status, implemented = derived_status(provider, actions, registry)
        provider["action_list"] = actions
        provider["implemented"] = implemented
        provider["status"] = status
        provider["auth_profile"] = auth.get(provider["auth"], {})
        rows[provider["slug"]] = provider
    pages: dict[str, str] = {}
    for category in CATEGORY_ORDER:
        selected = sorted(
            (p for p in rows.values() if p["category"] == category),
            key=lambda p: p["name"].lower(),
        )
        if selected:
            pages.update(paginate_category(category, selected))
    pages["index.md"] = render_index(rows, pages)
    return rows, pages


def paginate_category(category: str, providers: list[dict]) -> dict[str, str]:
    """Split a provider family across pages so no page exceeds the size limit."""
    groups: list[list[dict]] = [[]]
    used = 0
    for provider in providers:
        section = render_provider_section(provider)
        if groups[-1] and used + len(section) > MAX_PAGE_LINES:
            groups.append([])
            used = 0
        groups[-1].append(provider)
        used += len(section)
    pages: dict[str, str] = {}
    for index, group in enumerate(groups, 1):
        name = f"{category}.md" if index == 1 else f"{category}-{index}.md"
        for provider in group:
            provider["page"] = name
        pages[name] = render_category_page(category, group, index, len(groups))
    return pages


def render_index(rows: dict[str, dict], pages: dict[str, str]) -> str:
    lines = [
        "# Connector Catalog",
        "",
        CONN_BEGIN,
        "Generated by `scripts/gen_catalogs.py` from the integration catalog seeds",
        "and the live adapter registry. Do not edit by hand.",
        CONN_END,
        "",
        "The connectors listed here are the providers the Integrations app offers.",
        "`Status` is derived, never declared: **built** means an adapter exists and",
        "every advertised action is implemented, **partial** means the adapter (or a",
        "module/LLM binding) covers only part of the surface, **planned** is an",
        "explicit product decision, and **unsupported** has no adapter at all.",
        "",
        "Credential field names are listed without values; secrets live in the",
        "secrets vault and never enter a prompt, a log or this page.",
        "",
        "## Totals by category",
        "",
        "| Category | Providers | Built | Partial | Planned | Unsupported |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for category in CATEGORY_ORDER:
        selected = [p for p in rows.values() if p["category"] == category]
        if not selected:
            continue
        counts = {s: sum(1 for p in selected if p["status"] == s)
                  for s in ("built", "partial", "planned", "unsupported")}
        lines.append(
            f"| [{CATEGORY_TITLES[category]}](./{selected[0]['page']}) | {len(selected)} | "
            f"{counts['built']} | {counts['partial']} | {counts['planned']} | {counts['unsupported']} |"
        )
    counts = {s: sum(1 for p in rows.values() if p["status"] == s)
              for s in ("built", "partial", "planned", "unsupported")}
    lines.append(
        f"| **Total** | **{len(rows)}** | **{counts['built']}** | **{counts['partial']}** | "
        f"**{counts['planned']}** | **{counts['unsupported']}** |"
    )
    lines += [
        "",
        "## All providers",
        "",
        "| Provider | Slug | Category | Auth | Actions | Status | Docs |",
        "|---|---|---|---|---:|---|---|",
    ]
    for provider in sorted(rows.values(), key=lambda p: (p["category"], p["name"].lower())):
        docs = f"[docs]({provider['docs']})" if provider["docs"] else "—"
        lines.append(
            f"| [{cell(provider['name'])}](./{provider['page']}#{anchor(provider['name'])}-{provider['slug']}) "
            f"| `{provider['slug']}` | {CATEGORY_TITLES.get(provider['category'], provider['category'])} "
            f"| {AUTH_LABEL.get(provider['auth_profile'].get('method', 'unknown'), 'Unknown')} "
            f"| {len(provider['action_list'])} | {provider['status']} | {docs} |"
        )
    lines.append("")
    return "\n".join(lines)


def render_category_page(category: str, providers: list[dict], part: int, parts: int) -> str:
    suffix = f" (part {part}/{parts})" if parts > 1 else ""
    lines = [
        f"# Connectors — {CATEGORY_TITLES[category]}{suffix}",
        "",
        CONN_BEGIN,
        "Generated by `scripts/gen_catalogs.py`; do not edit by hand.",
        CONN_END,
        "",
        f"{len(providers)} provider(s) on this page. Back to the [connector index](./index.md).",
        "",
    ]
    for provider in providers:
        lines += render_provider_section(provider)
    return "\n".join(lines)


def render_provider_section(provider: dict) -> list[str]:
    profile = provider["auth_profile"]
    lines = [
        f"## {provider['name']} (`{provider['slug']}`)",
        "",
        f"- Status: **{provider['status']}** — {provider['implemented']}"
        f"/{len(provider['action_list'])} advertised actions implemented",
        f"- Strategy: {STRATEGY_LABEL.get(provider['strategy'], provider['strategy'])}"
        f" · Priority: {provider['priority'].capitalize()}",
        f"- Auth: {AUTH_LABEL.get(profile.get('method', 'unknown'), 'Unknown')}",
    ]
    if profile.get("fields"):
        fields = ", ".join(
            f"`{key}`{' (secret)' if secret else ''}" for key, secret in profile["fields"]
        )
        lines.append(f"- Credential fields: {fields}")
    if profile.get("least_privilege"):
        lines.append(f"- Least privilege: {cell(profile['least_privilege'])}")
    if provider["module"]:
        lines.append(f"- Module: `{provider['module']}`")
    if provider["docs"]:
        lines.append(f"- Official docs: <{provider['docs']}>")
    lines.append(f"- Source: `{provider['source']}`")
    if provider["action_list"]:
        lines += ["", "| Action key | Verb | Risk |", "|---|---|---|"]
        lines += [f"| `{key}` | {verb} | {risk} |" for key, verb, risk in provider["action_list"]]
    else:
        lines.append("- Actions: none declared")
    lines.append("")
    return lines


def summary_block(pages: dict[str, str]) -> str:
    lines = [CONN_BEGIN + "\n", "   - [Connector Catalog](./06-channels/connectors/index.md)\n"]
    for category in CATEGORY_ORDER:
        names = [name for name in pages if name.split("-")[0].removesuffix(".md") == category]
        names.sort(key=lambda name: int(name.removesuffix(".md").split("-")[1]) if "-" in name else 1)
        for name in names:
            title = f"Connectors — {CATEGORY_TITLES[category]}"
            if name != f"{category}.md":
                title += " (continued)"
            lines.append(f"   - [{title}](./06-channels/connectors/{name})\n")
    lines.append(CONN_END + "\n")
    return "".join(lines)


def report() -> int:
    rows, _ = build_connectors()
    statuses: dict[str, int] = {}
    for provider in rows.values():
        statuses[provider["status"]] = statuses.get(provider["status"], 0) + 1
    print(f"providers: {len(rows)}")
    for category in CATEGORY_ORDER:
        selected = [p for p in rows.values() if p["category"] == category]
        if selected:
            print(f"  {category:<18} {len(selected):>3}")
    print("status: " + ", ".join(f"{k}={v}" for k, v in sorted(statuses.items())))
    empty = sorted(p["slug"] for p in rows.values() if not p["action_list"])
    print(f"providers with zero actions: {len(empty)}"
          + (f" ({', '.join(empty[:12])}{'…' if len(empty) > 12 else ''})" if empty else ""))
    total = sum(len(p["action_list"]) for p in rows.values())
    unimplemented = sum(len(p["action_list"]) - p["implemented"] for p in rows.values())
    print(f"advertised actions: {total} ({unimplemented} without a live adapter)")
    return 0

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", action="store_true", help="print the inventory")
    parser.add_argument("--check", action="store_true", help="fail when pages are stale")
    args = parser.parse_args()
    if args.report:
        return report()
    _, pages = build_connectors()
    files = {CONN_DIR / name: body for name, body in pages.items()}
    stale = [path for path, body in files.items() if not path.is_file() or read(path) != body]
    if args.check:
        for path in stale:
            print(f"stale: {path}")
        if stale:
            print("gen_connector_catalog: run python3 scripts/gen_catalogs.py")
            return 1
        print(f"gen_connector_catalog: {len(files)} pages up to date")
        return 0
    print("gen_connector_catalog: use scripts/gen_catalogs.py to write the pages")
    return 0


if __name__ == "__main__":
    sys.exit(main())
