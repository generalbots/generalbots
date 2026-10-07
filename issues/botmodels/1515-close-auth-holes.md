# [BOTMODELS] 1515 — Close auth holes

**Priority:** P0
**Kind:** security
**Depends on:** — · **Blocks:** —

## Problem

Auth is inconsistent across four behaviours, and one is a genuine hole. The
README states *"All endpoints require the `X-API-Key` header"* without exemption.

| Surface | Current behaviour |
|---|---|
| `verify_api_key` | raises 401 correctly |
| `get_api_key` (`src/api/dependencies.py:11-16`) | returns `None` for a missing key **and for a wrong key**, never raises |
| `/api/detect` | uses `get_api_key` → **runs unauthenticated** |
| `GET /api/scoring/health` | no auth dependency at all |
| `WS /v1/audio/stt/stream`, `WS /v1/audio/tts/stream` | **no auth whatsoever** |

Any `X-API-Key` value — or none — grants access to `/api/detect`.

Comparison is `!=`, not constant-time. `hmac.compare_digest` is available and
is already used in the dead `app.py`.

`src/main.py:41-48` sets `allow_origins=["*"]` together with
`allow_credentials=True`. Browsers reject that combination, so credentialed
cross-origin requests silently fail while non-credentialed ones are wide open.

## Goal

- Every HTTP endpoint and WebSocket requires a valid key.
- Constant-time comparison everywhere.
- CORS config browsers actually accept.

## Notes on the WebSocket case

`verify_api_key` is a `Header` dependency and cannot run on a WS upgrade in this
codebase. Options: key in the query string, or
`Sec-WebSocket-Protocol` (keeps the key out of proxy logs). Recommend the latter,
falling back to a query param.

## Code anchors

| What | Where |
|---|---|
| `get_api_key` — returns `None` instead of raising | `src/api/dependencies.py:11-16` |
| `/api/detect` consumer | `src/api/v1/endpoints/anomaly.py` |
| Unauthenticated health route | `src/api/v1/endpoints/scoring.py` |
| Open WebSockets | `src/api/v1/endpoints/voice.py` |
| Invalid CORS combination | `src/main.py:41-48` |

## Acceptance criteria

- [ ] `/api/detect` returns 401 with a missing **and** with a wrong key.
- [ ] `/api/scoring/health` requires auth.
- [ ] Both `/v1/audio/*` WebSockets reject an unauthenticated upgrade.
- [ ] Key comparison uses `hmac.compare_digest`.
- [ ] CORS allows either specific origins or credentials — never both `*` and
      `allow_credentials=True`.
- [ ] README no longer overstates the auth guarantee.

## Non-goals

- Per-bot API keys (single shared key stays).