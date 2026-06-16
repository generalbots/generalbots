# Issue #010: SECURITY — Infrastructure control endpoints exposed in monitoring UI

**Severity:** HIGH
**Components:** `botui/ui/suite/monitoring/services.html`
**Type:** Security concern

## Description

The Monitoring app's services page exposes buttons that call infrastructure control endpoints:

| Endpoint | Action |
|----------|--------|
| `POST /api/services/{id}/restart` | Restart a service |
| `POST /api/services/{id}/stop` | Stop a service |
| `POST /api/services/{id}/start` | Start a service |
| `POST /api/services/restart-all` | Restart ALL services |

These endpoints, if implemented and exposed without strong authentication, would allow:
- Denial of service (stop all services)
- Service disruption (restart critical services)
- Privilege escalation (unauthorized control of infrastructure)

**Current status:** These endpoints are NOT implemented in the backend (they return 404), which is fortunate. But the UI exposes them as clickable buttons.

## Risk Assessment

| Risk | Likelihood | Impact |
|------|-----------|--------|
| Unauthorized service stop | Low (not implemented) | Critical if ever implemented |
| Unauthorized restart-all | Low (not implemented) | Critical if ever implemented |
| Accidental click by admin | Medium (buttons visible) | Medium |

## Suggested Fix

1. **Remove** these buttons from the UI unless the backend will implement them with proper auth.
2. If implemented in the future, they MUST:
   - Require admin-level RBAC role
   - Have audit logging for every action
   - Require confirmation dialog (not just single click)
   - Rate-limited (max 1 action per 30 seconds)
   - NOT expose `/restart-all` under any circumstances
3. Consider using a separate admin-only interface for these operations, not part of the general monitoring dashboard.
