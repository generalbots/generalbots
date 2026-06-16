# Issue #002: MONITORING — 80% endpoints are ghost functionality

**Severity:** CRITICAL
**Components:** `botui/ui/suite/monitoring/` (10+ HTML files), `botserver/crates/botmonitoring/src/lib.rs`
**Type:** Ghost functionality

## Description

The Monitoring app has the **most endpoints** in the entire suite (~50+), but only ~15 are implemented in the backend. Most HTMX calls across the 10+ HTML pages result in silent 404 errors.

---

## Backend-Implemented Endpoints (Existing)

In `botserver/crates/botmonitoring/src/lib.rs`, the `configure()` function registers 18 endpoints:

| Endpoint | Implementation |
|----------|---------------|
| `/api/ui/monitoring/dashboard` | ✅ Dashboard (CPU, RAM, sessions, uptime) |
| `/api/ui/monitoring/services` | ✅ Service list (basic health checks) |
| `/api/ui/monitoring/resources` | ✅ System resources (disk, network) |
| `/api/ui/monitoring/logs` | ✅ Log page (static, no streaming) |
| `/api/ui/monitoring/llm` | ✅ LLM page (static) |
| `/api/ui/monitoring/health` | ✅ Simple health check (db status) |
| `/api/ui/monitoring/timestamp` | ✅ Last update timestamp |
| `/api/ui/monitoring/bots` | ✅ Active bot count |
| `/api/ui/monitoring/services/status` | ✅ Service status |
| `/api/ui/monitoring/resources/bars` | ✅ CPU/MEM bars (SVG) |
| `/api/ui/monitoring/activity/latest` | ✅ Static "System monitoring active..." |
| `/api/ui/monitoring/metric/sessions` | ✅ Active session count |
| `/api/ui/monitoring/metric/messages` | ✅ Returns "--" (placeholder) |
| `/api/ui/monitoring/metric/response_time` | ✅ Returns "--" (placeholder) |
| `/api/ui/monitoring/trend/sessions` | ✅ Returns "↑ 0%" (placeholder) |
| `/api/ui/monitoring/rate/messages` | ✅ Returns "0/hr" (placeholder) |
| `/api/ui/monitoring/sessions` | ✅ Session panel (empty) |
| `/api/ui/monitoring/messages` | ✅ Message panel (empty) |

**Note:** Many handlers are placeholders returning fixed values ("--", "0/hr", etc).

---

## Frontend-Called Endpoints NOT Implemented (GHOST)

### Subpage: resources.html
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/ui/monitoring/resources/cpu` | ❌ |
| `GET /api/ui/monitoring/resources/memory` | ❌ |
| `GET /api/ui/monitoring/resources/disk` | ❌ |
| `GET /api/ui/monitoring/resources/network` | ❌ |
| `GET /api/ui/monitoring/charts/cpu` | ❌ |
| `GET /api/ui/monitoring/charts/memory` | ❌ |
| `GET /api/ui/monitoring/resources/disk/partitions` | ❌ |
| `GET /api/ui/monitoring/resources/processes` | ❌ |
| `GET /api/ui/monitoring/resources/network/interfaces` | ❌ |
| `GET /api/ui/monitoring/resources/system` | ❌ |

### Subpage: health.html
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/ui/monitoring/health/overview` | ❌ |
| `GET /api/ui/monitoring/health/uptime` | ❌ |
| `GET /api/ui/monitoring/health/uptime-percent` | ❌ |
| `GET /api/ui/monitoring/health/last-incident` | ❌ |
| `GET /api/ui/monitoring/health/response-time` | ❌ |
| `GET /api/ui/monitoring/health/checks` | ❌ |
| `GET /api/ui/monitoring/health/dependencies` | ❌ |
| `GET /api/ui/monitoring/health/uptime-history` | ❌ |
| `GET /api/ui/monitoring/health/incidents` | ❌ |

### Subpage: alerts.html
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/ui/monitoring/alerts/summary` | ❌ |
| `GET /api/ui/monitoring/alerts/active` | ❌ |
| `GET /api/ui/monitoring/alerts/rules` | ❌ |
| `POST /api/ui/monitoring/alerts/rules` | ❌ |
| `GET /api/ui/monitoring/alerts/history` | ❌ |

### Subpage: services.html
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/services/summary` | ❌ |
| `GET /api/services/status` | ❌ |
| `GET /api/services/{id}/details` | ❌ |
| `POST /api/services/{id}/restart` | ❌ |
| `POST /api/services/{id}/stop` | ❌ |
| `POST /api/services/{id}/start` | ❌ |
| `POST /api/services/restart-all` | ❌ |

### Subpage: metrics.html
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/ui/monitoring/metrics/last-sync` | ❌ |
| `GET /api/ui/analytics/metric?name=requests` | ❌ |
| `GET /api/ui/analytics/metric?name=latency` | ❌ |
| `GET /api/ui/analytics/metric?name=errors` | ❌ |
| `GET /api/ui/analytics/metric?name=throughput` | ❌ |
| `GET /api/ui/analytics/metrics/list` | ❌ |

### Subpage: index.html (main dashboard)
| Endpoint called | Implemented? | Note |
|----------------|--------------|------|
| `GET /api/monitoring/dashboard` | ❌ | Route is `/api/ui/monitoring/dashboard` |
| `GET /api/ui/monitoring/quick/cpu` | ❌ | |
| `GET /api/ui/monitoring/quick/memory` | ❌ | |
| `GET /api/ui/monitoring/quick/disk` | ❌ | |
| `GET /api/ui/monitoring/quick/network` | ❌ | |
| `GET /api/ui/monitoring/quick/requests` | ❌ | |

### monitoring.js
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/monitoring/export?range=...` | ❌ |

### LLM internals (monitoring.html via HTMX)
| Endpoint called | Implemented? |
|----------------|--------------|
| `GET /api/monitoring/llm/total` | ❌ |
| `GET /api/monitoring/llm/cache-rate` | ❌ |
| `GET /api/monitoring/llm/latency` | ❌ |
| `GET /api/monitoring/llm/tokens` | ❌ |
| `GET /api/monitoring/logs/stream` | ❌ |
| `GET /api/ui/analytics/dashboard` | ❌ |

---

## Additional Prefix Inconsistency

`index.html` (main dashboard) calls `/api/monitoring/dashboard` (without `/ui/`), but the backend registers the route as `/api/ui/monitoring/dashboard` (with `/ui/`). This causes the main dashboard to return 404.

---

## Impact

- Most monitoring tabs load no real data — they show empty states or placeholders.
- "Restart", "Stop", "Start" service buttons are non-functional.
- The main dashboard (`index.html`) likely returns 404 due to prefix mismatch.
- Aggressive polling (every 5s) on broken endpoints generates unnecessary 404 traffic.
- The `/api/services/*` endpoints present a security risk if exposed (see Issue #011).

## Suggested Fix

1. **Implement** the most critical endpoints: `resources/cpu|memory|disk|network`, `health/overview`, `alerts/summary`.
2. **Remove** or comment out restart/stop/start service buttons (security risk).
3. **Fix** the prefix in `index.html`: `/api/monitoring/dashboard` → `/api/ui/monitoring/dashboard`.
4. **Remove** entirely ghost subpages (alerts.html, detailed health.html) or implement their endpoints.
5. **Add** visual fallback for unimplemented endpoints (show "Coming soon").
6. **Reduce** polling intervals on unimplemented endpoints (or don't poll at all).
