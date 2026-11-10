#!/usr/bin/env python3
"""Every inbound channel webhook proves its origin, or fails closed (#2142).

An inbound webhook route is anonymous by necessity: the provider posts from its
own infrastructure and holds no user session, so the request itself is the only
evidence that the delivery is genuine. A handler that accepts it without
checking a secret lets a stranger create a bot turn under a victim identity.

This gate discovers every inbound route under `botserver/`, maps it to its
handler, and requires the handler (or the function it delegates to) to call a
known verifier. Failure modes: `unverified` (no verifier call, shrink-only
allowlist), `unclassified` (route missing from `INVENTORY`), `stale-inventory`
(listed route no longer registered), `empty-accept` (verifier accepts an empty
secret) and `stale-allowlist` (entry no longer suppresses anything).

Usage: `python3 scripts/check_webhook_auth.py [--list] [--self-test]`.
Exit codes: 0 clean, 1 violations, 2 usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_DIRS = ("botserver/crates", "botserver/src")
ALLOWLIST = Path("scripts/webhook_auth_allowlist.txt")

# Inbound webhook routes, by path shape. Deliberately narrow: a broad
# "anything under /api/*/messages" pattern would flag analytics endpoints that
# are authenticated REST resources, not provider deliveries.
INBOUND_PATHS = (
    re.compile(r"^/webhook(?:/|$)"),
    re.compile(r"^/api/(?:[a-z0-9_-]+/)*webhook(?:/|$)"),
    re.compile(r"^/api/msteams/messages$"),
)

# Function names that perform the authenticity check. A handler may call one
# directly or delegate to a helper that does.
VERIFIERS = (
    "authorize_delivery",
    "authorize_handshake",
    "verify_webhook_secret_for_bot",
    "verify_webhook_signature",
    "verify_challenge",
    "auth::authorize",
)

# A verifier is a function that decides authenticity; only its body is checked
# for the "empty secret means accept" branch (a plain `data.is_empty()` guard in
# a dispatcher is not an authentication decision).
VERIFIER_FN = re.compile(r"^(?:authorize|verify|secret_matches|signature_gate)")

# A verifier must never turn "no secret configured" into "accept".
EMPTY_ACCEPT = re.compile(
    r"is_empty\(\)\s*\{\s*return\s+(?:true|DeliveryAuth::Verified|WebhookGate::Allowed|Ok\(\(\)\))"
)

ROUTE_METHODS = re.compile(r"\b(?:get|post|put|patch|delete|any)\(\s*([A-Za-z0-9_:]+)")


@dataclass(frozen=True)
class Webhook:
    """One inbound route family owned by a channel."""

    channel: str  # channel name used in the matrix
    crate: str
    routes: tuple[str, ...]


# Every inbound webhook handler in the workspace. `unclassified` fails when a
# route appears that is not listed here, so a new channel cannot ship unchecked.
INVENTORY: tuple[Webhook, ...] = (
    Webhook("telegram", "bottelegram", ("/webhook/telegram", "/webhook/telegram/:bot_name")),
    Webhook("whatsapp", "botwhatsapp", ("/api/whatsapp/webhook",)),
    Webhook("msteams", "botmsteams", ("/api/msteams/messages",)),
    Webhook("instagram", "botinstagram", ("/api/instagram/webhook",)),
    Webhook("stripe-billing", "botcloud", ("/api/billing/webhook",)),
    Webhook("email-tracking", "botcontacts", ("/api/sales/email/webhook",)),
)

# Channels that expose no inbound HTTP handler today. Listed so the enumeration
# is complete: if one of them grows a route, the `unclassified` rule fires and
# forces it into `INVENTORY` with a verifier.
NO_HTTP_HANDLER: tuple[tuple[str, str], ...] = (
    ("slack", "outbound only: provider spec, no inbound route"),
    ("discord", "outbound only: no inbound route"),
    ("facebook", "anonymous path declared, no handler registered"),
    ("email-inbound", "IMAP poller, not an HTTP delivery"),
    ("sms-twilio", "payload parser only, no HTTP route"),
    ("generic", "no generic inbound receiver"),
)


@dataclass
class Violation:
    rule: str
    path: str
    line: int
    message: str

    def render(self) -> str:
        return f"{self.path}:{self.line}: {self.rule}: {self.message}"


@dataclass
class Route:
    path: str
    file: str
    line: int = 1
    handlers: list[str] = field(default_factory=list)


LITERALS = re.compile(r'"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)*\'')


def _strip_literals(text: str) -> str:
    """Blank out string/char literals so paren counting is reliable."""
    return LITERALS.sub(" ", text)


def _route_calls(text: str) -> list[tuple[int, str]]:
    """Every `.route(...)` argument list as `(offset, args)`, parens balanced."""
    calls = []
    for match in re.finditer(r"\.route\s*\(", text):
        start = match.end()
        depth = 1
        index = start
        while index < len(text) and depth:
            char = text[index]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            index += 1
        calls.append((match.start(), text[start : index - 1]))
    return calls


def _rust_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for scan in SCAN_DIRS:
        if (root / scan).is_dir():
            files.extend(sorted((root / scan).rglob("*.rs")))
    return files


def _const_paths(root: Path) -> dict[str, str]:
    """`const NAME: &str = "path"` declarations, for routes registered by name."""
    consts: dict[str, str] = {}
    pattern = re.compile(r"const\s+([A-Z][A-Z0-9_]*)\s*:\s*&str\s*=\s*\"([^\"]*)\"")
    for path in _rust_files(root):
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        for name, value in pattern.findall(text):
            consts[name] = value
    return consts


def discover_routes(root: Path) -> list[Route]:
    """Every inbound webhook route registered in the workspace."""
    consts = _const_paths(root)
    routes: list[Route] = []

    for path in _rust_files(root):
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        if ".route(" not in text:
            continue

        rel = path.relative_to(root).as_posix()
        for offset, call in _route_calls(text):
            head, _, tail = call.partition(",")
            head = head.strip()
            literal = re.findall(r'"((?:[^"\\]|\\.)*)"', head)
            if literal:
                route_path = literal[0]
            elif re.fullmatch(r"[A-Z][A-Z0-9_]*", head):
                route_path = consts.get(head, "")
            else:
                continue

            if not route_path or not any(p.match(route_path) for p in INBOUND_PATHS):
                continue

            routes.append(
                Route(
                    path=route_path,
                    file=rel,
                    line=text[:offset].count("\n") + 1,
                    handlers=ROUTE_METHODS.findall(_strip_literals(tail)),
                )
            )

    return routes


def _function_bodies(path: Path) -> dict[str, str]:
    """`fn name` -> body text (up to the next top-level `fn`)."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return {}

    starts = [m.start() for m in re.finditer(r"^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+", text, re.M)]
    bodies: dict[str, str] = {}
    for index, start in enumerate(starts):
        name_match = re.match(
            r"^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)", text[start:]
        )
        if not name_match:
            continue
        end = starts[index + 1] if index + 1 < len(starts) else len(text)
        bodies[name_match.group(1)] = text[start:end]
    return bodies


def _crate_dir(rel_file: str) -> str:
    parts = rel_file.split("/")
    if "crates" in parts:
        return "/".join(parts[: parts.index("crates") + 2])
    return "/".join(parts[:2])


def _has_verifier(body: str, bodies: dict[str, str], depth: int = 0) -> bool:
    """True when the body calls a verifier, directly or one delegation deep."""
    if any(token in body for token in VERIFIERS):
        return True
    if depth >= 2:
        return False
    for name, called_body in bodies.items():
        call = re.search(rf"\b{re.escape(name)}\s*\(", body)
        if call and _has_verifier(called_body, bodies, depth + 1):
            return True
    return False


def load_allowlist(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    entries: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        key = raw.split("#", 1)[0].strip()
        if key:
            entries[key] = raw.split("#", 1)[1].strip() if "#" in raw else ""
    return entries


def scan(
    root: Path,
    allowlist_path: Path,
    inventory: tuple[Webhook, ...] = INVENTORY,
) -> tuple[list[Violation], list[Route]]:
    routes = discover_routes(root)
    allowlist = load_allowlist(root / allowlist_path)
    violations: list[Violation] = []
    used_allowlist: set[str] = set()
    body_cache: dict[str, dict[str, str]] = {}

    known_routes = {route for webhook in inventory for route in webhook.routes}
    discovered = {route.path for route in routes}

    for route in routes:
        if route.path not in known_routes:
            violations.append(Violation(
                "unclassified", route.file, route.line,
                f"inbound route {route.path} is not in INVENTORY — add it with its verifier",
            ))

        crate_dir = root / _crate_dir(route.file)
        for handler in route.handlers:
            bodies = body_cache.setdefault(crate_dir.as_posix(), {})
            if not bodies:
                for source in sorted(crate_dir.rglob("*.rs")):
                    bodies.update(_function_bodies(source))
            body = bodies.get(handler.split("::")[-1])
            if body is None:
                # A handler the gate cannot read must not silently pass.
                violations.append(Violation(
                    "unknown-handler", route.file, route.line,
                    f"{route.path} -> {handler}() not found in {_crate_dir(route.file)}",
                ))
                continue
            if _has_verifier(body, bodies):
                continue
            if route.path in allowlist:
                used_allowlist.add(route.path)
                continue
            violations.append(Violation(
                "unverified", route.file, route.line,
                f"{route.path} -> {handler}() never calls a verifier ({', '.join(VERIFIERS)})",
            ))

    for webhook in inventory:
        missing = [route for route in webhook.routes if route not in discovered]
        if missing:
            violations.append(Violation(
                "stale-inventory", "scripts/check_webhook_auth.py", 1,
                f"{webhook.channel}: route(s) {missing} no longer registered",
            ))

    for path in _rust_files(root):
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        if not any(token in text for token in VERIFIERS):
            continue
        rel = path.relative_to(root).as_posix()
        for name, body in _function_bodies(path).items():
            if not VERIFIER_FN.match(name):
                continue
            for match in EMPTY_ACCEPT.finditer(body):
                violations.append(Violation(
                    "empty-accept", rel,
                    text[: text.find(body) + match.start()].count("\n") + 1,
                    f"{name}() accepts the request when the secret is empty",
                ))

    for key in allowlist:
        if key not in used_allowlist:
            violations.append(Violation(
                "stale-allowlist", str(allowlist_path), 1,
                f"'{key}' no longer suppresses a violation — remove it (shrink-only)",
            ))

    return violations, routes


def render_matrix() -> str:
    lines = ["| Channel | Inbound route | Gate |", "|---|---|---|"]
    for webhook in INVENTORY:
        lines.append(f"| {webhook.channel} | {', '.join(webhook.routes)} | `{webhook.crate}` |")
    lines.extend(f"| {channel} | — ({note}) | n/a |" for channel, note in NO_HTTP_HANDLER)
    return "\n".join(lines)


def self_test() -> int:
    """Prove the four rules fire, and that the allowlist is shrink-only."""
    failures: list[str] = []

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        crate = root / "botserver/crates/botdemo/src"
        crate.mkdir(parents=True)

        (crate / "webhook.rs").write_text(
            'pub fn configure() {\n'
            '    Router::new()\n'
            '        .route("/webhook/demo", post(handle_demo))\n'
            '        .route("/webhook/open", post(handle_open))\n'
            '        .route("/webhook/new", post(handle_new))\n'
            '}\n'
            'async fn handle_demo() { authorize_delivery("s", None); }\n'
            'async fn handle_open() { /* no gate */ }\n'
            'async fn handle_new() { authorize_delivery("s", None); }\n',
            encoding="utf-8",
        )
        (crate / "bad.rs").write_text(
            'pub fn authorize_delivery(secret: &str) -> bool {\n'
            '    if secret.is_empty() {\n'
            '        return true;\n'
            '    }\n'
            '    false\n'
            '}\n',
            encoding="utf-8",
        )

        allowlist = root / "allowlist.txt"
        allowlist.write_text("/webhook/open # demo: authenticated elsewhere\n", encoding="utf-8")

        # The demo crate has no INVENTORY entry, so every route is unclassified
        # too; the assertions below only look at the rule under test.
        inventory = (
            Webhook("demo", "botdemo", ("/webhook/demo", "/webhook/open", "/webhook/new")),
        )
        violations, _ = scan(root, Path("allowlist.txt"), inventory)

        rules = {violation.rule for violation in violations}
        if "empty-accept" not in rules:
            failures.append("an empty-secret accept branch was not detected")
        if "unverified" in rules:
            failures.append("the allowlisted handler was reported")
        if "unclassified" in rules:
            failures.append("an inventory route was reported as unclassified")

        allowlist.unlink()
        violations, _ = scan(root, Path("allowlist.txt"), inventory)
        if "unverified" not in {violation.rule for violation in violations}:
            failures.append("an unverified handler was not detected")

        allowlist.write_text("/webhook/gone # stale\n", encoding="utf-8")
        violations, _ = scan(root, Path("allowlist.txt"), inventory)
        if "stale-allowlist" not in {violation.rule for violation in violations}:
            failures.append("a stale allowlist entry was not detected")

        partial = (Webhook("demo", "botdemo", ("/webhook/demo", "/webhook/open")),)
        violations, _ = scan(root, Path("allowlist.txt"), partial)
        if "unclassified" not in {violation.rule for violation in violations}:
            failures.append("a route missing from INVENTORY was not detected")
        if "stale-inventory" in {violation.rule for violation in violations}:
            failures.append("a registered route was reported as stale inventory")

    if failures:
        for failure in failures:
            print(f"self-test FAILED: {failure}", file=sys.stderr)
        return 1

    print("self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--list", action="store_true", help="print the channel matrix")
    parser.add_argument("--self-test", action="store_true", help="prove the rules fire")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if args.list:
        print(render_matrix())
        return 0

    violations, routes = scan(ROOT, ALLOWLIST)
    if violations:
        for violation in violations:
            print(violation.render(), file=sys.stderr)
        print(f"RESULT: FAILED ({len(violations)} violation(s))", file=sys.stderr)
        return 1

    print(f"RESULT: ok ({len(routes)} inbound route(s), all verified)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
