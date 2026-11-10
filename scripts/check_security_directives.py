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

Coverage model
--------------
A router merged into the main app inherits the layers `apply_middleware`
installs, so its state-changing routes are covered by (a). A router served by
its *own* listener bypasses them: every `axum::serve(...)` call site is
resolved to the file that builds the served router, and that file must carry a
rate limit and either a CSRF layer or an exemption. The scanning lives in
`scripts/security_directives_surface.py`.

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
from pathlib import Path

from security_directives_surface import (
    EXEMPTIONS,
    load_exemptions,
    route_surface,
    standalone_router_files,
    state_changing_routes,
)

MAIN_LAYERS = Path("botserver/src/main_module/server/server_layers.rs")
MAIN_BOOT = Path("botserver/src/main_module/server/run_axum_server.rs")
WS_SESSION = Path("botserver/src/core/bot/ws/session.rs")
WS_LIMITER = Path("botserver/src/core/bot/ws/session/message_limit.rs")

DIRECTIVES = {"csrf", "rate-limit"}


def read(root: Path, relative: Path) -> str:
    return (root / relative).read_text(encoding="utf-8", errors="replace")


def check_main_layers(root: Path, errors: list[str]) -> None:
    """The layers every router merged into the main app depends on."""
    layers = read(root, MAIN_LAYERS)
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
    boot = read(root, MAIN_BOOT)
    if "apply_middleware(" not in boot:
        errors.append(f"{MAIN_BOOT}: the served router is not passed through apply_middleware")
    if "create_rate_limit_layer(" not in boot:
        errors.append(f"{MAIN_BOOT}: the rate limiter is never constructed (IMP-07)")
    if "HttpRateLimitConfig::api()" not in boot:
        errors.append(
            f"{MAIN_BOOT}: the API limiter preset is not used; the general and "
            "per-client quotas then come from an undocumented config (IMP-07)"
        )


def check_ws_limit(root: Path, errors: list[str]) -> None:
    """The WebSocket path needs its own limiter: HTTP middleware stops at the upgrade."""
    if not (root / WS_LIMITER).is_file():
        errors.append(f"{WS_LIMITER}: the WebSocket message limiter module is missing (IMP-07)")
        return
    module = read(root, WS_LIMITER)
    if not re.search(r"MESSAGES_PER_SECOND:\s*u32\s*=\s*10\b", module):
        errors.append(f"{WS_LIMITER}: the documented 10 messages/second quota is not declared")
    session = read(root, WS_SESSION)
    for marker in ("MessageLimiter::new()", ".allow()"):
        if marker not in session:
            errors.append(
                f"{WS_SESSION}: the WebSocket message rate limit is not wired "
                f"({marker}); one socket can drive unbounded LLM turns (IMP-07)"
            )


def check_route_coverage(root: Path, errors: list[str]) -> int:
    """Every state-changing router served by its own listener needs a guard."""
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
    """An exemption must stay live, justified and inside the shrink-only ceiling."""
    exemptions, max_entries = load_exemptions(root, errors)
    if len(exemptions) > max_entries:
        errors.append(
            f"{EXEMPTIONS}: {len(exemptions)} entries exceed max_entries={max_entries}; "
            "the list is shrink-only -- remove an exemption or lower the ceiling"
        )
    seen: set[str] = set()
    for entry in exemptions:
        if entry.directive not in DIRECTIVES:
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

PLANTED = "botserver/crates/fake/src/gateway_server/configure_gateway_routes.rs"

FIXTURE = {
    "botserver/src/main_module/listen_impl.rs": """
pub(crate) async fn listen(app: Router, port: u16) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app.into_make_service()).await
}
""",
    "botserver/src/main_module/server/run_axum_server.rs": """
pub async fn run_axum_server(app_state: Arc<AppState>) -> std::io::Result<()> {
    let (rate_limit_extension, _rate_limiter) =
        create_rate_limit_layer(HttpRateLimitConfig::api(), limits);
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
    PLANTED: """
pub fn configure_gateway_routes(state: Arc<GatewayState>) -> Router {
    Router::new().route("/deploy", post(gateway_deploy))
}
""",
    "scripts/security_directives_exemptions.toml": "max_entries = 1\n",
}

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
        original_layers = layers.read_text(encoding="utf-8")
        layers.write_text(
            original_layers.replace("csrf_middleware(", "csrf_removed("), encoding="utf-8"
        )
        if gate() == 0:
            problems.append("a removed CSRF layer was not detected")
        layers.write_text(original_layers, encoding="utf-8")

        session = temp / WS_SESSION
        original_session = session.read_text(encoding="utf-8")
        session.write_text(
            original_session.replace("MessageLimiter::new()", "None"), encoding="utf-8"
        )
        if gate() == 0:
            problems.append("an unwired WebSocket message limit was not detected")
        session.write_text(original_session, encoding="utf-8")

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
    parser.add_argument("--check", action="store_true", help="run the gate (default)")
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
