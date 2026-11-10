# Network Contract 🟡 BETA

One page for every port, base URL and path prefix in the platform. Every other
page links here instead of repeating numbers.

## Ports

| Surface | Port | Authority |
|---|---:|---|
| Suite (botui) | 3000 | `botui/src/main.rs` (`unwrap_or(3000)`) |
| Cloud (botui) | 4000 | `botui/src/main.rs` (`unwrap_or(4000)`) |
| Login (botui) | 5000 | `botui/src/main.rs` (`unwrap_or(5000)`) |
| API + WebSocket (botserver) | 8080 | `botlib/src/http_client.rs` (`DEFAULT_BOTSERVER_URL`), `botserver/src/main_module/routes/misc_handlers/handle_dns_remove.rs` (`"ports": { "api": 8080, "ws": 8080 }`) |
| Production API (container) | 5858 | `.github/ISSUE_TEMPLATE.md` health check; container health endpoint |

The botserver port is `config.server.port` (`botserver/src/main_body.rs`); 8080
is the development default. Nothing in this repository binds 8000 or 9000.

## Base URLs

| Use | URL |
|---|---|
| REST | `http://localhost:8080/api` |
| WebSocket | `ws://localhost:8080/ws` |
| Suite | `http://localhost:3000` |
| Cloud | `http://localhost:4000` |
| Login / signup | `http://localhost:5000` |

## Prefix rules

| Rule | Evidence |
|---|---|
| `/api` is the local REST prefix; the **only** local `/api/v1` route is the legacy `POST /api/v1/sandbox/exec` | `grep -rn '"/api/v1' botserver/src botserver/crates` returns one route literal (`botserver/crates/botagent/src/lib.rs`) plus external Microsoft Graph paths in `botconnectors` |
| Drive file transfer is `/api/files/*`, not `/files/*` | `botserver/src/core/bot/api_catalog/commands_list.rs` (`/api/files/list`, `/api/files/write`, …) |
| `/chat/*` is not a REST prefix | chat is the WebSocket pipeline (`botserver/src/core/bot/ws/`); the only `/api/chat*` routes are listed in the generated reference |
| The deploy gateway listens on 5860 | `botserver/src/main_module/server/run_axum_server.rs` (`0.0.0.0:5860`) |
| BotUI serves static files, botserver never serves a complete HTML page | `botui/src/main.rs`; botserver returns API payloads and HTMX fragments only |

## Who serves what

| Port | Serves | Does not serve |
|---|---|---|
| 3000 | `botui/ui/suite/*` — chat, apps, desktop | `/cloud/*` |
| 4000 | `botui/ui/cloud/*` — store, dashboard, plans | `/login`, `/signup` (redirect to 5000) |
| 5000 | `botui/ui/login/*` — login, signup | application pages |
| 8080 | `/api/*`, `/cloud/partials/*`, `/ws` | complete HTML pages |

## Related pages

- [REST API chapter](../08-rest-api-tools/README.md) — the endpoint catalog
- [WebSocket protocol](../07-user-interface/ws-protocol.md) — frame contract
- [Installation](../01-getting-started/installation.md) — how the ports are bound
