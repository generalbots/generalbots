# Production Security Verification (2026-11-06)

This page records the production verification performed on 2026-11-06 with the
owner's authorization. It is the user-facing summary of the detailed evidence
in the repository's `SECURITY.md` and the security report.

## Scope

- `chat.pragmatismo.com.br` public edge (DNS, TLS, headers, auth gates,
  WebSocket handshake, external ports).
- `srv1` / `srv2` hosts and every Incus container (listeners, firewall,
  service authentication, configuration).
- Supply chain: Rust/npm/Python dependencies, secrets, CI gates.
- Container OS packages and the static Go service binaries.

## Verified controls

| Control | Result |
|---------|--------|
| Security headers | CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy on the public origin |
| Protected API | `/api/auth`, `/api/bots`, `/api/users`, `/api/admin`, `/api/drive/files`, `/api/crm/contacts` return `401 missing_token` without a credential |
| Login | Wrong credentials return `401`; traversal/XSS/SQLi markers are not reflected; no CORS wildcard |
| External ports | 443 open; 5432/8443/5858/5859 blocked from three independent external nodes |
| VNC ports | 6080/5901/8443 on srv2 blocked externally; 6080 allowed only from the proxy egress |
| Secrets store | Vault unsealed, unseal keys mode 0600, token required |
| Cache | Valkey requires a password (`NOAUTH` without it) |
| Database | PostgreSQL no longer trusts `0.0.0.0/0`; external 5432 blocked |
| Containers | No `security.privileged`; Incus socket and Chromium CDP proxies restricted to their client containers |
| OS packages | All security updates applied; only held Incus packages (bot) and the kernel package (desktops, reboot pending) remain |
| Service binaries | Caddy **2.11.9**, Forgejo **17.0.1**, Zitadel **4.20.1**, Vault **2.2.1**, LiveKit **1.13.11** and MinIO **2025-10-15** deployed and verified (MinIO built from the official source tag) |

## Fixes applied during the verification

- Unsealed Vault (botserver had been crash-looping because it could not read
  `secret/gbo/drive`).
- Added the six IMP-09 headers to the Caddy vhosts and synced the serving
  config path so they survive a restart.
- Removed the public INPUT rules for 5432/8443/5858/5859 and set the srv2
  firewall to DROP.
- Restricted the container-to-host Incus socket (8591) and Chromium CDP
  (9223) proxies.
- Narrowed PostgreSQL `pg_hba` to the Incus subnets and enabled Valkey
  `requirepass`.
- Applied the pending OS security updates to every container.
- Upgraded the edge and service binaries with a backup at every step:
  Caddy 2.11.9 (and synced the serving Caddyfile), Forgejo 17.0.1,
  Zitadel 4.20.1 (running `zitadel setup` for the schema migration),
  Vault 2.2.1 (file-storage migration, unsealed), LiveKit 1.13.11 and
  MinIO 2025-10-15 (built from the official source tag with
  `scripts/build-minio-fixed.sh`).
- Ran authenticated cross-tenant probes with two disposable DB-only tenants
  and tokens minted from the persisted SaaS secret. Drive isolation, bucket
  discovery, RBAC and path traversal all held. Found and fixed a medium
  cross-tenant disclosure: `/api/bots/list` and `/api/bots/tree` returned
  every tenant's bots (including private ones) to any authenticated user.
  Both endpoints now scope to the caller's organizations (public bots stay
  visible); verified post-deploy that the five private bots are no longer
  listed.
- Rotated the Vault unseal keys (5 shares, threshold 3, seal/unseal verified),
  verified the historical Zitadel and GCP credentials are dead at the
  providers, and revoked the leaked WhatsApp system-user token. Only the
  Azure Search/LUIS item remains, with no active consumer and no Azure
  console access.

## Open items

| Item | Tracking |
|------|----------|
| Rotate the Azure Search/LUIS keys (no active consumer; needs Azure console access) | issue #1761 |

| Apply the parent dependency upgrades (quick-xml, git2, calamine, rsa, zip) | issues #1757, #1762–#1765 |

The rate-limiter fix (per-client 10 req/s on `/api/auth/*`) is merged and
takes effect with the next binary deployment.

## Reproducing the checks

```bash
# Headers
curl -sS -D - -o /dev/null https://chat.pragmatismo.com.br/ \
  | grep -Ei 'content-security-policy|strict-transport-security|x-frame-options|x-content-type-options|referrer-policy|permissions-policy'

# Unauthenticated API
curl -sS -o /dev/null -w '%{http_code}\n' https://chat.pragmatismo.com.br/api/bots

# Health
curl -sS -o /dev/null -w '%{http_code}\n' https://chat.pragmatismo.com.br/api/health
```

Full evidence, per-finding detail and the residual-risk register are in
`SECURITY.md`.
