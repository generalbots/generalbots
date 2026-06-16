# Issue #006: CRM — Inconsistent API prefix by pipeline stage

**Severity:** LOW
**Components:** `botui/ui/suite/crm/crm.html`
**Type:** Inconsistency (design, not functional)

## Description

The CRM pipeline uses **different API prefixes** depending on the stage:

- Stage "new" (leads): uses `hx-get="/api/ui/crm/count?stage=new"` and `hx-get="/api/ui/crm/pipeline?stage=new"`
- Stages "qualified", "proposal", "negotiation", "won", "lost": use `hx-get="/api/crm/count?stage=..."` and `hx-get="/api/crm/pipeline?stage=..."`

## Crate Audit Result

O crate `botcontacts` registra **ambos os prefixos** no servidor:
- `configure_crm_api_routes()` → `/api/crm/contacts`, `/api/crm/accounts`, `/api/crm/leads`, `/api/crm/opportunities`, `/api/crm/deals`, `/api/crm/stats`, etc. (31 rotas)
- `configure_crm_ui_routes()` → `/api/ui/crm/count`, `/api/ui/crm/pipeline`, `/api/ui/crm/contacts`, `/api/ui/crm/accounts`, `/api/ui/crm/deals` (5 rotas)

Ambas as funções são chamadas. Portanto, **ambos os prefixos funcionam** para todos os estágios. A diferença não quebra nada.

**Mas:** A integração de `botcontacts` em server.rs está atualmente **desabilitada** (linhas 951-952 comentadas):
```rust
// Sources/contacts disabled — requires ConfigManagerOps/crate state construction
// #[cfg(feature = "sources")] ... 
// #[cfg(feature = "contacts")] ...
```

**Isso significa que as rotas de `botcontacts` NÃO estão ativas no servidor!** As rotas de CRM que funcionam vêm do `botapps` crate, que tem `/api/crm/` routes via `configure_crm_routes()` — mas `botcontacts` tem rotas MAIS COMPLETAS que não estão sendo usadas.

## Impact

- Funcionalmente, as rotas de CRM em `botapps` estão ativas e funcionam para ambos os prefixos.
- A inconsistência de prefixo é apenas um problema de legibilidade/manutenção.
- O crate `botcontacts` (com 50+ rotas CRM mais completas) está desabilitado — CRM funciona com um subconjunto menor via `botapps`.

## Suggested Fix

1. **Ativar** `botcontacts` no server.rs para obter as rotas CRM completas.
2. **Padronizar** o prefixo do CRM: decidir entre `/api/crm/` (REST) e `/api/ui/crm/` (HTMX) e usar consistentemente.
3. **Atualizar** `crm.html` para usar o mesmo prefixo em todos os estágios.
