#!/usr/bin/env python3
"""Route-contract guard for the endpoint catalog (issue #2546).
Checks documented routes against mounted routers; known gaps carry owners.
Usage: python3 scripts/check_route_contract.py [--self-test]
"""
from __future__ import annotations
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from route_contract_status import (
    check_status_contract, parse_not_implemented_handlers, parse_retired_handlers,
    self_test as status_self_test,
)
REPO_ROOT = Path(__file__).resolve().parents[1]
BOTSERVER = REPO_ROOT / "botserver"
CATALOG_MOD = BOTSERVER / "src/core/bot/api_catalog/mod.rs"
INVENTORY_ROOT = BOTSERVER / "src/core/bot"
SOURCE_ROOTS = (BOTSERVER / "src", BOTSERVER / "crates")
METHODS = {"get", "post", "put", "patch", "delete", "options", "head"}
METHOD_ALIASES = {"axum_" + m: m for m in METHODS}
METHOD_ALIASES.update({"any": "any"})
PARAM_PATTERN = re.compile(
    r":([A-Za-z_][A-Za-z0-9_]*)|\{([A-Za-z_][A-Za-z0-9_]*)\}|\*([A-Za-z_][A-Za-z0-9_]*)"
)
def canonical(path: str) -> str:
    """Normalise route params (``:id`` / ``{id}`` / ``*id``) to ``{id}``."""
    return PARAM_PATTERN.sub(lambda found: "{" + next(g for g in found.groups() if g) + "}", path)
def param_style(path: str) -> str:
    """Describe the param markers used in ``path`` for mismatch reporting."""
    markers = []
    for found in PARAM_PATTERN.finditer(path):
        markers.append(":" if found.group(1) else ("{}" if found.group(2) else "*"))
    return ",".join(markers)
@dataclass(frozen=True)
class Endpoint:
    method: str
    path: str
    source: str
@dataclass(frozen=True)
class Registration:
    method: str
    path: str
    handler: str
    source: str
def _iter_rs_files():
    for directory in SOURCE_ROOTS:
        if directory.exists():
            yield from directory.rglob("*.rs")
def _read_intro(text: str) -> dict[str, str]:
    """Collect simple ``const NAME: &str = "..."`` / ``concat!(...)`` values."""
    values: dict[str, str] = {}
    pattern = re.compile(
        r"\b(?:pub(?:\([^)]*\))?\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*&\s*'?static\s+str\s*=\s*(.*?);",
        re.S,
    )
    for match in pattern.finditer(text):
        expr = match.group(2).strip()
        literal = re.fullmatch(r'"([^"]*)"', expr)
        if literal:
            values[match.group(1)] = literal.group(1)
            continue
        concat = re.fullmatch(r"concat!\((.*)\)", expr, re.S)
        if concat:
            parts = [p.strip() for p in concat.group(1).split(",")]
            if all(re.fullmatch(r'"[^"]*"', p) for p in parts):
                values[match.group(1)] = "".join(p[1:-1] for p in parts)
    return values
def _paren_block(text: str, start: int) -> tuple[str, int]:
    """Return the contents of the balanced parentheses starting at ``start``."""
    depth = 0
    in_string = False
    escaped = False
    inner_start = start + 1
    for index in range(start, len(text)):
        char = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                return text[inner_start:index], index + 1
    return text[inner_start:], len(text)
def _top_level_comma(text: str) -> int:
    depth = 0
    in_string = False
    escaped = False
    for index, char in enumerate(text):
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == "," and depth == 0:
            return index
    return -1
def _resolve_path(expr: str, consts: dict[str, str]) -> str | None:
    expr = expr.strip().lstrip("&")
    literal = re.fullmatch(r'"([^"]*)"', expr)
    if literal:
        return literal.group(1)
    name = re.fullmatch(r"(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Z][A-Z0-9_]*)", expr)
    if name and name.group(1) in consts:
        return consts[name.group(1)]
    formatted = re.fullmatch(r'format!\("([^"]*)"\)', expr)
    if formatted:
        def substitute(found: re.Match[str]) -> str:
            return consts.get(found.group(1), found.group(0))
        return re.sub(r"\{([A-Z][A-Z0-9_]*)\}", substitute, formatted.group(1))
    return None
def parse_routes() -> list[Registration]:
    """Parse ``.route(...)`` calls across the whole botserver source tree."""
    registrations: list[Registration] = []
    files = list(_iter_rs_files())
    consts: dict[str, str] = {}
    for file in files:
        consts.update(_read_intro(file.read_text(errors="ignore")))
    for file in files:
        text = file.read_text(errors="ignore")
        position = 0
        while True:
            index = text.find(".route(", position)
            if index < 0:
                break
            body, position = _paren_block(text, index + len(".route"))
            comma = _top_level_comma(body)
            if comma < 0:
                continue
            path = _resolve_path(body[:comma], consts)
            if path is None:
                continue
            tail = body[comma + 1 :]
            for found in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(", tail):
                fn = found.group(1)
                if fn not in METHODS and fn not in METHOD_ALIASES:
                    continue
                inner, _ = _paren_block(tail, found.end() - 1)
                handler_match = re.match(r"\s*([A-Za-z_][A-Za-z0-9_:]+)", inner)
                handler = handler_match.group(1) if handler_match else "?"
                method = "any" if fn == "any" else fn.replace("axum_", "")
                methods = ("get", "post", "put", "patch", "delete") if method == "any" else (method,)
                for verb in methods:
                    registrations.append(
                        Registration(verb.upper(), path, handler, str(file.relative_to(REPO_ROOT)))
                    )
    return registrations
ENTRY_PATTERN = re.compile(r"\bApiEndpoint\s*\{")
FIELD_PATTERN = re.compile(r'\b(method|path|summary)\s*:\s*"([^"]*)"')
def parse_inventory() -> list[Endpoint]:
    """Parse every ``ApiEndpoint`` literal from the inventory tree."""
    endpoints: list[Endpoint] = []
    files = sorted(INVENTORY_ROOT.glob("endpoint_inventory*.rs"))
    for directory in INVENTORY_ROOT.glob("endpoint_inventory_*"):
        if directory.is_dir():
            files.extend(sorted(directory.rglob("*.rs")))
    for file in files:
        text = file.read_text()
        position = 0
        while True:
            index = text.find("ApiEndpoint", position)
            if index < 0:
                break
            brace = text.find("{", index)
            if brace < 0:
                break
            depth = 0
            in_string = False
            escaped = False
            end = brace
            for cursor in range(brace, len(text)):
                char = text[cursor]
                if in_string:
                    if escaped:
                        escaped = False
                    elif char == "\\":
                        escaped = True
                    elif char == '"':
                        in_string = False
                    continue
                if char == '"':
                    in_string = True
                elif char == "{":
                    depth += 1
                elif char == "}":
                    depth -= 1
                    if depth == 0:
                        end = cursor + 1
                        break
            fields = dict(FIELD_PATTERN.findall(text[brace:end]))
            position = end
            if "method" in fields and "path" in fields:
                endpoints.append(
                    Endpoint(fields["method"], fields["path"], str(file.relative_to(REPO_ROOT)))
                )
    return endpoints
def parse_not_implemented() -> list[tuple[str, str, str]]:
    """Read ``NOT_IMPLEMENTED_ENDPOINTS`` from the Rust catalog (single truth)."""
    text = CATALOG_MOD.read_text()
    marker = "NOT_IMPLEMENTED_ENDPOINTS"
    index = text.find(marker)
    if index < 0:
        raise AssertionError(f"{marker} missing from {CATALOG_MOD}")
    assignment = text.find("= &[", index)
    bracket = text.find("[", assignment) if assignment >= 0 else -1
    if bracket < 0:
        return []
    depth = 0
    in_string = False
    escaped = False
    end = bracket
    for cursor in range(bracket, len(text)):
        char = text[cursor]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "[":
            depth += 1
        elif char == "]":
            depth -= 1
            if depth == 0:
                end = cursor + 1
                break
    values = re.findall(r'"([^"]*)"', text[bracket:end])
    if len(values) % 3:
        raise AssertionError(f"malformed {marker}: expected triples, got {len(values)} strings")
    return [(values[i], values[i + 1], values[i + 2]) for i in range(0, len(values), 3)]
def _parse_allowlist(raw: str) -> dict[tuple[str, str], str]:
    entries: dict[tuple[str, str], str] = {}
    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        left, _, owner = line.partition("|")
        method, _, path = left.strip().partition(" ")
        entries[(method.strip().upper(), canonical(path.strip()))] = owner.strip() or "unspecified"
    return entries
# Baseline gaps. Each line is ``METHOD PATH | owner/reason``. New drift fails.
KNOWN_MISSING = _parse_allowlist(
    """
# Nested routers: the prefix is applied at mount time and not visible to a
# static .route() scan; these need a mount-aware guard before they can be removed.
POST /api/ui/skills/install | botsources (nested router under /api/ui/skills)
POST /api/integrations/connectors/:id/connect | botsources (nested router)
POST /api/integrations/connectors/:id/disconnect | botsources (nested router)
GET /api/integrations/etl | botsources (nested router)
POST /api/oauth/github/connect | botsecurity-auth (provider router mounted by prefix)
POST /api/oauth/google/connect | botsecurity-auth (provider router mounted by prefix)
POST /api/oauth/microsoft/connect | botsecurity-auth (provider router mounted by prefix)
# Genuinely stale catalog entries: no mounted handler.
POST /api/vibe/run/:run_id/approve | botvibe (route removed, catalog entry stale)
POST /api/canvas/ai/design | botcanvas (route removed, catalog entry stale)
"""
)
KNOWN_STYLE = _parse_allowlist(
    """
# The catalog uses {} templates for api.exec; axum mounts use :param/*param.
# api_exec::fill_path accepts both, so these are intentional, not broken.
GET /api/jukebox/jobs/{job_id} | endpoint-inventory
POST /api/video/clips/{from_id}/transition/{to_id} | endpoint-inventory
PUT /api/video/layers/{id} | endpoint-inventory
GET /api/video/layers/{id}/keyframes | endpoint-inventory
GET /api/video/projects/{id} | endpoint-inventory
GET /api/video/projects/{id}/analytics | endpoint-inventory
GET /api/video/projects/{id}/audio | endpoint-inventory
POST /api/video/projects/{id}/beat-sync | endpoint-inventory
POST /api/video/projects/{id}/captions | endpoint-inventory
GET /api/video/projects/{id}/clips | endpoint-inventory
GET /api/video/projects/{id}/layers | endpoint-inventory
POST /api/video/projects/{id}/remove-background | endpoint-inventory
POST /api/video/projects/{id}/template | endpoint-inventory
POST /api/video/projects/{id}/transcribe | endpoint-inventory
POST /api/video/projects/{id}/waveform | endpoint-inventory
GET /api/workspaces/{workspace_id} | endpoint-inventory
DELETE /api/workspaces/{workspace_id}/members/{user_id} | endpoint-inventory
GET /api/workspaces/{workspace_id}/pages | endpoint-inventory
POST /api/git/branch/:name | endpoint-inventory (:name vs axum *name)
GET /api/calendar/calendars/{id} | endpoint-inventory
GET /api/canvas/{canvas_id} | endpoint-inventory
GET /api/canvas/{canvas_id}/collaborate | endpoint-inventory
GET /api/canvas/{canvas_id}/collaborators | endpoint-inventory
DELETE /api/canvas/{canvas_id}/collaborators/{user_id} | endpoint-inventory
GET /api/canvas/{canvas_id}/comments | endpoint-inventory
PUT /api/canvas/{canvas_id}/comments/{comment_id}/resolve | endpoint-inventory
GET /api/canvas/{canvas_id}/elements | endpoint-inventory
PUT /api/canvas/{canvas_id}/elements/{element_id} | endpoint-inventory
GET /api/canvas/{canvas_id}/versions | endpoint-inventory
POST /api/security/protection/clamav/quarantine/{id} | endpoint-inventory
POST /api/security/protection/lmd/quarantine/{id} | endpoint-inventory
POST /api/security/protection/{tool}/disable | endpoint-inventory
POST /api/security/protection/{tool}/enable | endpoint-inventory
GET /api/security/protection/{tool}/status | endpoint-inventory
POST /api/security/protection/{tool}/uninstall | endpoint-inventory
POST /api/security/protection/{tool}/update | endpoint-inventory
DELETE /api/settings/security/users/{user_id}/sessions | endpoint-inventory
GET /api/pages/{page_id} | endpoint-inventory
GET /api/plan/{plan_id}/collaborators | endpoint-inventory
GET /api/plan/{plan_id}/presence | endpoint-inventory
GET /api/slides/mentions/{user_id} | endpoint-inventory
GET /api/slides/{id}/collaborators | endpoint-inventory
GET /api/slides/{presentation_id}/presence | endpoint-inventory
GET /api/slides/{presentation_id}/selections | endpoint-inventory
GET /api/slides/{presentation_id}/typing | endpoint-inventory
GET /api/rbac/groups/{group_id} | endpoint-inventory
POST /api/rbac/groups/{group_id}/roles/{role_id} | endpoint-inventory
GET /api/rbac/roles/{role_id} | endpoint-inventory
GET /api/rbac/roles/{role_id}/permissions | endpoint-inventory
POST /api/rbac/users/{user_id}/groups/{group_id} | endpoint-inventory
POST /api/rbac/users/{user_id}/roles/{role_id} | endpoint-inventory
"""
)
KNOWN_DUPLICATES = _parse_allowlist(
    """
# Two routers mount the same method+path with different handlers.
GET /api/product | product-handlers (legacy + current router both mounted)
GET /api/retail/branches | botretail (retail/mod.rs + botretail/lib.rs)
GET /api/retail/promotions | botretail (retail/mod.rs + botretail/lib.rs)
GET /api/retail/suppliers | botretail (retail/mod.rs + botretail/lib.rs)
GET /api/retail/stock | botretail (#2955 GET added to botretail/lib.rs; unmounted src/retail/mod.rs stub still declares it)
GET /api/retail/top-products | botretail (#2955 GET added to botretail/lib.rs; unmounted src/retail/mod.rs stub still declares it)
POST /api/dns/register | misc-handlers (duplicate legacy router)
POST /api/dns/remove | misc-handlers (duplicate legacy router)
"""
)
def check_contract(
    endpoints: list[Endpoint],
    registrations: list[Registration],
    not_implemented: list[tuple[str, str, str]],
    missing: dict[tuple[str, str], str],
    style: dict[tuple[str, str], str],
    duplicates: dict[tuple[str, str], str],
) -> list[str]:
    """Return a list of contract violations (empty means the guard passes)."""
    problems: list[str] = []
    for method, path, owner in not_implemented:
        if not owner or owner.startswith("unspecified"):
            problems.append(f"NOT_IMPLEMENTED {method} {path} has no owner")
    by_route: dict[tuple[str, str], list[Registration]] = {}
    for registration in registrations:
        by_route.setdefault((registration.method, canonical(registration.path)), []).append(registration)
    documented: set[tuple[str, str]] = set()
    for endpoint in endpoints:
        key = (endpoint.method.upper(), canonical(endpoint.path))
        if key in documented:
            problems.append(f"duplicate catalog entry {endpoint.method} {endpoint.path}")
            continue
        documented.add(key)
        matches = by_route.get(key, [])
        if not matches:
            if key in missing:
                continue
            problems.append(
                f"documented {endpoint.method} {endpoint.path} ({endpoint.source}) "
                "has no matching router registration"
            )
            continue
        if param_style(endpoint.path) != param_style(matches[0].path) and key not in style:
            problems.append(
                f"param-style mismatch: catalog {endpoint.path} "
                f"({param_style(endpoint.path)}) vs router {matches[0].path} "
                f"({param_style(matches[0].path)})"
            )
        handlers = {entry.handler for entry in matches if entry.handler != "?"}
        if len(handlers) > 1 and key not in duplicates:
            problems.append(
                f"duplicate handler for {endpoint.method} {endpoint.path}: {sorted(handlers)}"
            )
    return problems
def _self_test() -> int:
    good = [Endpoint("GET", "/api/ok", "inv.rs")]
    bad_missing = [Endpoint("POST", "/api/missing", "inv.rs")]
    bad_style = [Endpoint("GET", "/api/item/{id}", "inv.rs")]
    bad_duplicate = [Endpoint("GET", "/api/retail/branches", "inv.rs")]
    routes = [
        Registration("GET", "/api/ok", "handler_a", "route.rs"),
        Registration("GET", "/api/item/:id", "handler_b", "route.rs"),
        Registration("GET", "/api/retail/branches", "list_branches", "route.rs"),
        Registration("GET", "/api/retail/branches", "mutations::list_branches", "route.rs"),
    ]
    check = lambda ep, ni=(), miss=None, sty=None, dup=None: check_contract(
        ep, routes, list(ni), miss or {}, sty or {}, dup or {}
    )
    assert check(good) == [], "clean contract must pass"
    assert any("no matching router" in p for p in check(bad_missing)), "missing route not detected"
    assert any("param-style mismatch" in p for p in check(bad_style)), "style drift not detected"
    assert any("duplicate handler" in p for p in check(bad_duplicate)), "duplicate not detected"
    assert check(bad_style, sty={("GET", "/api/item/{id}"): "owner"}) == [], "style allowlist ignored"
    assert check(
        bad_duplicate, dup={("GET", "/api/retail/branches"): "owner"}
    ) == [], "duplicate allowlist ignored"
    assert any(
        "has no owner" in p
        for p in check(good, ni=[("GET", "/api/ok", "")])
    ), "missing not-implemented owner not detected"
    assert check(good, ni=[("GET", "/api/ok", "crate.rs:1")]) == [], "owned gap must pass"
    assert status_self_test() == 0, "status guard self-test failed"
    print("self-test: all 7 checks passed")
    return 0
def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return _self_test()
    endpoints = parse_inventory()
    registrations = parse_routes()
    not_implemented = parse_not_implemented()
    problems = check_contract(
        endpoints, registrations, not_implemented, KNOWN_MISSING, KNOWN_STYLE, KNOWN_DUPLICATES
    )
    files = list(_iter_rs_files())
    problems.extend(check_status_contract(
        endpoints, registrations, not_implemented,
        parse_retired_handlers(files), parse_not_implemented_handlers(files), canonical,
    ))
    print(
        f"route contract: {len(endpoints)} catalog entries, "
        f"{len(registrations)} router registrations, "
        f"{len(not_implemented)} not-implemented gaps"
    )
    for problem in problems:
        print(f"FAIL: {problem}", file=sys.stderr)
    if problems:
        print(f"route contract FAILED ({len(problems)} violations)", file=sys.stderr)
        return 1
    print("route contract OK")
    return 0
if __name__ == "__main__":
    sys.exit(main())
