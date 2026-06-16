# Issue #011: TICKETS — SLA panel is static, makes zero API calls

**Severity:** LOW
**Components:** `botui/ui/suite/tickets/partials/sla.html`
**Type:** Ghost functionality

## Description

The SLA panel partial (`sla.html`) displays SLA metrics with nice formatting (targets, percentages, status badges) but **makes zero API calls to load real data**.

## Crate Audit Result

O crate `bottickets` (mounted sob feature `tickets`) TEM um endpoint de SLA:
- `GET /api/tickets/sla` → `list_sla_policies` ✅ (25ª rota em `lib.rs`)

**Mas** a partial `sla.html` não chama nenhum endpoint — é HTML estático sem HTMX ou JavaScript. O endpoint existe no backend mas o frontend não o consome.

**Outros endpoints de tickets que funcionam (36 rotas no total):**
- `GET/POST /api/tickets` ✅
- `GET /api/tickets/stats`, `GET /api/tickets/overdue` ✅
- `GET/PUT/DELETE /api/tickets/:id` ✅
- `PUT /api/tickets/:id/assign|status|resolve|close|reopen` ✅
- `GET /api/ui/tickets/*` (11 rotas UI) ✅

## Impact

- O SLA panel sempre mostra dados estáticos, não reflete a realidade.
- O endpoint `GET /api/tickets/sla` existe mas não é usado pelo frontend.
- Usuários podem tomar decisões baseadas em dados incorretos.

## Suggested Fix

1. **Adicionar** HTMX ao `sla.html`: `hx-get="/api/tickets/sla"` para carregar dados reais.
2. **Ou**: criar endpoint específico para o painel SLA (`GET /api/ui/tickets/sla/stats`) com métricas calculadas.
3. **Ou**: remover o partial SLA se não estiver no roadmap.
