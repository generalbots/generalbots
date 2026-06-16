# Issue #007: SOCIAL — Duplicate analytics endpoints for same feature

**Severity:** LOW
**Components:** `botui/ui/suite/social/social.html`
**Type:** Duplication / Inconsistency

## Description

The Social app calls **two different URLs** for the same analytics feature:

1. **HTMX tab click:** `GET /api/ui/social/analytics` (in the HTML `hx-get` attribute)
2. **JavaScript fetch:** `GET /api/social/analytics` (in the inline JS `fetch()`)

## Crate Audit Result

O crate `botsocial` (mounted sob feature `social`) registra:
- `configure_social_routes()` → `/api/social/analytics` ✅
- `configure_social_ui_routes()` → `/suite/social`, `/suite/social/compose`, `/suite/social/:id` (NÃO `/api/ui/social/analytics`)

**Portanto:**
- `GET /api/social/analytics` (JS fetch) → ✅ **Funciona**
- `GET /api/ui/social/analytics` (HTMX) → ❌ **Não existe** — retorna 404

O HTMX tab click está quebrado. A JS fetch funciona.

## Impact

- A tab "Analytics" no Social, quando clicada via HTMX, retorna 404.
- O refresh via JavaScript `fetch()` funciona.
- Usuários veem erro na primeira navegação para a tab.

## Suggested Fix

1. **Corrigir** o `hx-get` no HTML de `/api/ui/social/analytics` para `/api/social/analytics`.
2. **Ou**: adicionar rota `/api/ui/social/analytics` em `botsocial` como alias.
3. **Padronizar**: decidir se Social usa `/api/social/` ou `/api/ui/social/` e manter consistência.
