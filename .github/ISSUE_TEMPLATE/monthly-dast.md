---
name: Monthly DAST reminder
about: Manual, monthly active scan of the public origin (not run in CI by design)
title: "sec/dast: monthly active scan of chat.pragmatismo.com.br"
labels: security, type:test
assignees: ""
---

## Why this is manual

Active scanning of production is noisy and can page customers. It is
deliberately **not** a CI job. The automated gates cover supply chain only
(`.github/workflows/security.yml`); this checklist covers the running estate.

Run it in a low-traffic window, at ≤10 req/s per host, and stop on the first
sign of user-visible degradation (`/api/health` or Caddy 5xx).

## Checklist

- [ ] Confirm the current rules of engagement in #1718 are still valid.
- [ ] Passive pass: DNS/CT/TLS/headers (`curl -sSI`, `openssl s_client`) — no
      authentication or payloads.
- [ ] Unauthenticated surface: `/`, `/api/product`, `/api/health`, `/ws`
      upgrade without a token, `.well-known/security.txt`.
- [ ] Authentication: login error/timing enumeration, token audience/expiry,
      logout invalidation. Password spray is **not** part of this run.
- [ ] Authorization smoke: one non-admin account hitting an admin-only
      endpoint and one `sec-test-*` tenant boundary probe.
- [ ] Injection smoke: one benign `'` and one `<img onerror>` in a search
      field; verify the response is escaped and no 5xx.
- [ ] Rate limits: `/api/auth`, `/api/signup`, WebSocket message rate.
- [ ] Headers: all six IMP-09 headers present on the origin.
- [ ] Evidence: `/tmp/opencode/pentest/dast/<YYYY-MM>.md` (hostnames only, no
      IPs, no tokens, no customer PII).
- [ ] File one issue per confirmed finding with a curl/python repro and a
      `file:line` root cause.
- [ ] Delete every `sec-test-*` org/user/bot/drive object created for the run.

## Out of scope

- Destructive payloads (`DROP`, `DELETE`, `rm`, disk fill).
- Customer tenants and `*.generalbots.org` bots.
- Third-party SaaS providers.
- DoS/volumetric load.

## Done when

- [ ] Every section has a pass/fail verdict, even where nothing was found.
- [ ] Findings are filed and cross-linked to #1717.
- [ ] The next month's reminder is scheduled.
