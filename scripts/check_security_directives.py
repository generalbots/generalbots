#!/usr/bin/env python3
"""Verify the two AGENTS.md security directives that nothing else checks.

IMP-08 (CSRF) and IMP-07 (rate limiting) are written as mandatory rules but no
gate proved them, so a router that forgot the layer shipped silently. This
script greps the route registration surface and fails when a state-changing
handler is reachable without either

  (a) the CSRF middleware layer from
      `botserver/src/main_module/server/server_layers.rs`, or
  (b) an entry in `scripts/security_directives_exemptions.toml` -- a
      shrink-only list that must name the compensating control it relies on.

It also proves the rate limiters are installed: the general/auth/api limiters
in the main stack and the per-socket WebSocket message limiter.

Surface
-------
* `botserver/src/main_module/**/*.rs` -- the main server, its routers and the
  side listeners it spawns.
* `botserver/crates/*/src/**/*routes*.rs` -- every crate router (this is a
  superset of the literal `routes*.rs` spelling, so `configure_*_routes.rs`
  files are covered too; the deploy gateway lives in one of those).

Coverage model
--------------
A router merged into the main app inherits the layers `apply_middleware`
installs, so its state-changing routes are covered by (a). A router served by
its *own* listener bypasses them: the script resolves every `axum::serve(...)`
call site to the file that builds the served router and requires that file to
carry a rate limit and either a CSRF layer or an exemption.

Usage
-----
    python3 scripts/check_security_directives.py
    python3 scripts/check_security_directives.py --self-test
    python3 scripts/check_security_directives.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import re
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11
    print("error: python3.11+ is required (tomllib)")
    raise SystemExit(1)

EXEMPTIONS = Path("scripts/security_directives_exemptions.toml")

MAIN_LAYERS = Path("botserver/src/main_module/server/server_layers.rs")
MAIN_SERVE = Path("botserver/src/main_module/listen_impl.rs")
MAIN_BOOT = Path("botserver/src/main_module/server/run_axum_server.rs")
WS_SESSION = Path("botserver/src/core/bot/ws/session.rs")
WS_LIMITER = Path("botserver/src/core/bot/ws/session/message_limit.rs")

STATE_CHANGING = ("post", "put", "patch", "delete")
ROUTE_CALL = re.compile(r"\.route\s*\(")
ROUTE_PATH = re.compile(r'\s*"((?:[^"\\]|\\.)*)"')
METHOD_IN_BLOCK = re.compile(r"\b(?:%s)\s*\(" % "|".join(STATE_CHANGING))
SERVE_CALL = re.compile(r"axum::serve\s*\(\s*[^,]+,\s*([^,)]+)")
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
CALLED = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(")
# axum combinators are not router builders; the first remaining call in a
# `let` right-hand side is the builder we want to resolve.
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
    # axum method routers: `get(handler)` is never a router builder.
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
    """Drops `#[cfg(test)]` modules: test routers are not served."""
    marker = text.find("#[cfg(test)]")
    return text if marker == -1 else text[:marker]


def line_of(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def balanced_block(text: str, start: int) -> str:
    """Returns the text from `start` to the `)` matching the open paren before it."""
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
    """Every `path` mounted with POST/PUT/PATCH/DELETE in this file."""
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
    """`fn name` -> files defining it (built once per run)."""
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
        # `listen(app, port)` receives the finished router; the main listener
        # is validated by the explicit layer checks instead.
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
        # The router is built here and only wrapped by another call
        # (`with_security_headers(router().with_state(state), ..)`).
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
    for path in rust_files(root):
        text = code_before_tests(path.read_text(encoding="utf-8", errors="replace"))
        if "axum::serve(" not in text:
            continue
        for call in SERVE_CALL.finditer(text):
            served = IDENT.match(call.group(1).strip())
            if not served:
                continue
            for builder in resolve_builder(root, path, served.group(0)):
                if builder.resolve() != (root / MAIN_SERVE).resolve():
                    standalone.add(builder)
    return standalone


def load_exemptions(root: Path, errors: list[str]) -> tuple[list[Exemption], int]:
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


def check_main_layers(root: Path, errors: list[str]) -> None:
    layers = (root / MAIN_LAYERS).read_text(encoding="utf-8", errors="replace")
    if "csrf_middleware(" not in layers:
        errors.append(
            f"{MAIN_LAYERS}: the CSRF middleware is not installed; every route "
            "merged into the main app is then reachable without a token (IMP-08)"
        )
    for marker, why in (
        ("auth_rate_limit_middleware", "the general/auth HTTP limiter"),
        ("rate_limit_extension", "the shared limiter state"),
    ):
        if marker not in layers:
            errors.append(f"{MAIN_LAYERS}: {why} is not installed ({marker}, IMP-07)")
    boot = (root / MAIN_BOOT).read_text(encoding="utf-8", errors="replace")
    if "apply_middleware(" not in boot:
        errors.append(f"{MAIN_BOOT}: the served router is not passed through apply_middleware")
    if "create_rate_limit_layer(" not in boot:
        errors.append(f"{MAIN_BOOT}: the rate limiter is never constructed (IMP-07)")


def check_ws_limit(root: Path, errors: list[str]) -> None:
    session = root / WS_SESSION
    limiter = root / WS_LIMITER
    if not limiter.is_file():
        errors.append(f"{WS_LIMITER}: the WebSocket message limiter module is missing (IMP-07)")
        return
    module = limiter.read_text(encoding="utf-8", errors="replace")
    if not re.search(r"MESSAGES_PER_SECOND:\s*u32\s*=\s*10\b", module):
        errors.append(f"{WS_LIMITER}: the documented 10 messages/second quota is not declared")
    text = session.read_text(encoding="utf-8", errors="replace")
    for marker in ("MessageLimiter::new()", ".allow()"):
        if marker not in text:
            errors.append(
                f"{WS_SESSION}: the WebSocket message rate limit is not wired "
                f"({marker}); one socket can drive unbounded LLM turns (IMP-07)"
            )


def check_route_coverage(root: Path, errors: list[str]) -> int:
    exemptions, _ = load_exemptions(root, errors)
    exempted = {entry.file for entry in exemptions if entry.directive == "csrf"}
    standalone = {path.resolve() for path in standalone_router_files(root)}
    checked = 0
    for path in route_surface(root):
        text = path.read_text(encoding="utf-8", errors="replace")
        routes = state_changing_routes(text)
        if not routes:
            continue
        checked += 1
        relative = path.relative_to(root).as_posix()
        if path.resolve() not in standalone:
            continue
        has_csrf = "csrf_middleware(" in text
        has_limit = "rate_limit" in text and ".layer(" in text
        if not has_csrf and relative not in exempted:
            where = ", ".join(f"{route} (line {line})" for route, line in routes)
            errors.append(
                f"{relative}: state-changing routes {where} are served by a "
                "listener that bypasses the main CSRF layer, and there is no "
                f"exemption entry in {EXEMPTIONS}"
            )
        if not has_limit:
            errors.append(
                f"{relative}: served by its own listener without a rate limit "
                "(IMP-07); add the limiter layer or extend the exemption"
            )
    return checked


def check_exemptions(root: Path, errors: list[str]) -> None:
    exemptions, max_entries = load_exemptions(root, errors)
    if len(exemptions) > max_entries:
        errors.append(
            f"{EXEMPTIONS}: {len(exemptions)} entries exceed max_entries={max_entries}; "
            "the list is shrink-only -- remove an exemption or lower the ceiling"
        )
    seen: set[str] = set()
    for entry in exemptions:
        if entry.directive not in {"csrf", "rate-limit"}:
            errors.append(f"{EXEMPTIONS}: {entry.file}: unknown directive '{entry.directive}'")
        if not entry.reason.strip():
            errors.append(f"{EXEMPTIONS}: {entry.file}: an exemption needs a reason")
        path = root / entry.file
        if not path.is_file():
            errors.append(f"{EXEMPTIONS}: {entry.file}: file no longer exists (stale exemption)")
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        mounted = {route for route, _ in state_changing_routes(text)}
        for route in entry.routes:
            if route not in mounted:
                errors.append(
                    f"{EXEMPTIONS}: {entry.file}: route '{route}' is no longer mounted "
                    "(stale exemption -- the list may only shrink)"
                )
        if entry.evidence and entry.evidence not in text:
            errors.append(
                f"{EXEMPTIONS}: {entry.file}: evidence '{entry.evidence}' not found; "
                "the compensating control the exemption relies on is gone"
            )
        key = f"{entry.file}:{entry.directive}"
        if key in seen:
            errors.append(f"{EXEMPTIONS}: {entry.file}: duplicate {entry.directive} exemption")
        seen.add(key)


def run(root: Path) -> int:
    errors: list[str] = []
    check_main_layers(root, errors)
    check_ws_limit(root, errors)
    checked = check_route_coverage(root, errors)
    check_exemptions(root, errors)
    for error in errors:
        print(f"error: {error}")
    if errors:
        print(f"\n{len(errors)} security-directive violation(s)")
        return 1
    print(
        "ok: CSRF layer + general/auth/api + WebSocket limiters installed; "
        f"{checked} state-changing route file(s) covered"
    )
    return 0


# --------------------------------------------------------------------------
# self-test: a fixture tree where a POST router is served on its own listener
# --------------------------------------------------------------------------

FIXTURE = {
    "botserver/src/main_module/listen_impl.rs": """
pub(crate) async fn listen(app: Router, port: u16) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app.into_make_service()).await
}
""",
    "botserver/src/main_module/server/run_axum_server.rs": """
pub async fn run_axum_server(app_state: Arc<AppState>) -> std::io::Result<()> {
    let (rate_limit_extension, _rate_limiter) = create_rate_limit_layer(config, limits);
    let app = apply_middleware(base_router, app_state, layers);
    tokio::spawn(async move {
        let gateway_router = crate::deployment::configure_gateway_routes(gateway_state);
        let service = gateway_router.into_make_service_with_connect_info::<SocketAddr>();
        axum::serve(listener, service).await
    });
    listen(app, port).await
}
""",
    "botserver/src/main_module/server/server_layers.rs": """
pub(super) fn apply_middleware(app: Router, layers: ServerLayers) -> Router {
    app.layer(axum::middleware::from_fn(auth_rate_limit_middleware))
        .layer(rate_limit_extension)
        .layer(axum::middleware::from_fn(move |req, next| {
            let mgr = csrf_manager.clone();
            async move { csrf_middleware(mgr, req, next).await }
        }))
}
""",
    "botserver/src/core/bot/ws/session.rs": """
pub async fn handle_ws(socket: WebSocket) {
    let message_limiter = message_limit::MessageLimiter::new();
    loop {
        if !message_limiter.allow() { continue; }
    }
}
""",
    "botserver/src/core/bot/ws/session/message_limit.rs": """
const MESSAGES_PER_SECOND: u32 = 10;
""",
    "botserver/crates/fake/src/gateway_server/configure_gateway_routes.rs": """
pub fn configure_gateway_routes(state: Arc<GatewayState>) -> Router {
    Router::new().route("/deploy", post(gateway_deploy))
}
""",
    "scripts/security_directives_exemptions.toml": "max_entries = 1\n",
}


PLANTED = "botserver/crates/fake/src/gateway_server/configure_gateway_routes.rs"

PLANTED_ROUTER_FIXED = """
pub fn configure_gateway_routes(state: Arc<GatewayState>) -> Router {
    Router::new()
        .route("/deploy", post(gateway_deploy))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
}
"""

PLANTED_EXEMPTION = (
    "max_entries = 1\n\n"
    "[[exemption]]\n"
    f'file = "{PLANTED}"\n'
    'directive = "csrf"\n'
    'reason = "machine-to-machine: X-Deploy-Key header, no cookie session"\n'
    'evidence = "fn configure_gateway_routes"\n'
    'routes = ["/deploy"]\n'
)


def write_fixture(root: Path) -> None:
    for relative, body in FIXTURE.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body.lstrip(), encoding="utf-8")


def self_test() -> int:
    """Each scenario plants one violation and expects the gate to name it."""
    problems: list[str] = []
    temp = Path(tempfile.mkdtemp(prefix="security-directives-"))
    noise = io.StringIO()

    def gate() -> int:
        with contextlib.redirect_stdout(noise):
            return run(temp)

    try:
        write_fixture(temp)
        if gate() == 0:
            problems.append("a listener bypassing CSRF was not detected")

        (temp / PLANTED).write_text(PLANTED_ROUTER_FIXED.lstrip(), encoding="utf-8")
        (temp / EXEMPTIONS).write_text(PLANTED_EXEMPTION, encoding="utf-8")
        if gate() != 0:
            problems.append("a documented exemption was not accepted")

        layers = temp / MAIN_LAYERS
        original = layers.read_text(encoding="utf-8")
        layers.write_text(original.replace("csrf_middleware(", "csrf_removed("), encoding="utf-8")
        if gate() == 0:
            problems.append("a removed CSRF layer was not detected")
        layers.write_text(original, encoding="utf-8")

        session = temp / WS_SESSION
        original = session.read_text(encoding="utf-8")
        session.write_text(original.replace("MessageLimiter::new()", "None"), encoding="utf-8")
        if gate() == 0:
            problems.append("an unwired WebSocket message limit was not detected")
        session.write_text(original, encoding="utf-8")

        (temp / EXEMPTIONS).write_text("max_entries = 0\n", encoding="utf-8")
        if gate() == 0:
            problems.append("a stale exemption (entry removed, route unlisted) was not detected")
    finally:
        shutil.rmtree(temp, ignore_errors=True)

    if problems:
        for problem in problems:
            print(f"self-test FAILED: {problem}")
        return 1
    print(
        "self-test ok: planted router/layer/WebSocket violations fail, "
        "the documented exemption passes"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=str(Path(__file__).resolve().parent.parent))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    root = Path(args.root).resolve()
    if not (root / "botserver").is_dir():
        print(f"error: {root} does not look like the repository root")
        return 1
    return run(root)


if __name__ == "__main__":
    sys.exit(main())
