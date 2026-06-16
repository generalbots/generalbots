# Issue #004: Partially Ghost Apps — real endpoints mixed with non-existent ones

**Severity:** MEDIUM
**Components:** Multiple apps in `botui/ui/suite/`
**Type:** Ghost functionality (partial)

## Description

Several apps have **partially implemented** backends — some endpoints work, others don't. After auditing all 80+ crates, the picture is more nuanced than initially reported. Many apps have backend code distributed across multiple crates (botapps provides baseline; dedicated crates provide deeper functionality but may be orphaned or different).

---

## 1. HR (`hr/hr.html` + JS)

**Working (via botapps, mounted):**
`GET/POST /api/hr/employees` ✅, `PUT /api/hr/employees/{id}` ✅, `GET /api/hr/recruitment` ✅, `GET /api/hr/attendance` ✅

**Ghost (not implemented anywhere):**
| Endpoint | Feature |
|----------|---------|
| `GET/POST /api/hr/performance` | Performance reviews |
| `GET /api/hr/payroll` | Payroll |
| `GET /api/hr/benefits` | Benefits |
| `GET /api/hr/training` | Training |

Não há crate dedicado de RH (ex.: `bothr`) que implemente esses endpoints faltantes.

**Functional coverage:** 50%

---

## 2. SALES (`sales/sales.html` + `sales/sales.js`)

**Working (via botapps + botcontacts, mounted):**
- `GET /api/sales/deals` ✅ (botapps)
- `GET /api/sales/contacts` ✅ (botapps)
- `GET /api/sales/activities` ✅ (botapps)
- `GET /api/sales/forecast` ✅ (botapps)
- `GET /api/sales/funnel`, `POST /api/sales/funnel/move` ✅ (botcontacts/sales)
- `POST /api/sales/forecast/ai-predict` ✅ (botcontacts/sales)

**Ghost:**
| Endpoint | Feature |
|----------|---------|
| `GET/POST /api/sales/quotes` | Quotes |
| `GET /api/sales/orders` | Orders |
| `GET /api/sales/reports` | Reports |

**Functional coverage:** 57% (but funnel + AI prediction work)

---

## 3. BANKING (`banking/banking.html` + `banking/banking.js`)

**Working (via botapps, mounted):**
- `GET/POST /api/banking/transactions` ✅
- `GET /api/banking/platforms` ✅
- `POST /api/banking/reconcile` ✅ (note: sem `/auto`)
- `GET /api/banking/reports` ✅

**Ghost / incompatible:**
| Endpoint | Problem |
|----------|---------|
| `POST /api/banking/transactions/:id/match` | ❌ Não implementado em nenhum crate |
| `POST /api/banking/platforms/:id/sync` | ❌ Não implementado |
| `POST /api/banking/reconcile/auto` | ⚠ Backend tem `/api/banking/reconcile` (sem `/auto`) |

O crate `botbanking` é puramente de modelos de dados — não tem rotas.

**Functional coverage:** 57% (um endpoint com path mismatch)

**Fix:** Mudar frontend de `/api/banking/reconcile/auto` para `/api/banking/reconcile`.

---

## 4. FRAUD (`fraud/fraud.html` + `fraud/fraud.js`)

**DISTRIBUIDO ENTRE DOIS CRATES:**

**botapps** (mounted em server.rs via `crate::apps::register()`):
- `GET/POST /api/fraud/transactions` ✅
- `GET/POST /api/fraud/rules` ✅ (GET lista, POST cria)
- `PUT /api/fraud/rules/{id}` ✅ (atualiza = toggle)
- `GET/POST /api/fraud/blocklist` ✅
- `DELETE /api/fraud/blocklist/{id}` ✅

**botfraud** (ORPHAN — 9 rotas definidas mas NUNCA montadas):
- `POST /api/fraud/assess` ❌ (orfan)
- `GET/POST /api/fraud/rules` ❌ (duplicata, mas orfan — botapps tem as mesmas)
- `PUT /api/fraud/rules/{id}` ❌ (orfan — botapps tem equivalente)
- `GET /api/fraud/events` ❌ (orfan)
- `GET/POST /api/fraud/blocklist` ❌ (orfan — botapps tem)
- `DELETE /api/fraud/blocklist/{id}` ❌ (orfan — botapps tem)
- `GET /api/fraud/stats` ❌ (orfan — NÃO existe em botapps)

**Frontend ghost:**
| Endpoint | Status |
|----------|--------|
| `GET /api/fraud/stats` | ❌ Definido em botfraud (orfan) |
| `GET /api/fraud/reports` | ❌ Não existe em nenhum crate |
| `POST /api/fraud/transactions/{id}/review` | ❌ Não existe |
| `POST /api/fraud/rules/{id}/toggle` | ⚠ Backend tem `PUT ... /rules/{id}` (método diferente) |

**Veredito corrigido:** Dos 9 endpoints, 5 funcionam (via botapps). `stats` existe no crate orfão. `reports` e `review` não existem. Toggle funciona via PUT em vez de POST.

**Functional coverage corrigida:** 56% (5/9 endpoints, mas com diferenças de método)

---

## 5. ITSM (`itsm/itsm.html` + `itsm/itsm.js`)

**Working (via botapps, mounted):**
`GET/POST /api/itsm/incidents` ✅, `PUT /api/itsm/incidents/{id}` ✅, `GET /api/itsm/cmdb` ✅, `GET /api/itsm/kb` ✅

**Ghost:**
| Endpoint | Feature |
|----------|---------|
| `GET/POST /api/itsm/catalog` | Service catalog |
| `POST /api/itsm/cmdb` | Add CI to CMDB (backend só tem GET) |
| `GET /api/itsm/reports` | Reports |

**Functional coverage:** 57%

---

## 6. COMPLIANCE (`compliance/compliance.html` + JS)

**Working (via botcompliance, mounted sob feature `compliance`):**
- `GET/POST /api/compliance/checks` ✅
- `GET /api/compliance/checks/:check_id` ✅
- `GET/POST /api/compliance/issues` ✅
- `PUT /api/compliance/issues/:issue_id` ✅
- `GET/POST /api/compliance/audit` ✅
- `POST /api/compliance/training` ✅
- `GET /api/compliance/report` ✅
- `POST /api/compliance/evidence` ✅

**Ghost:**
| Endpoint | Feature |
|----------|---------|
| `GET /api/compliance/risks` | ❌ Não implementado (backend não tem) |
| `GET /api/compliance/training` | ⚠ Backend só tem POST, não GET |

**Functional coverage corrigida:** 82% (9/11 endpoints funcionando!)

---

## 7. MARKETING LISTS (`lists/lists.html` + JS)

**Working (via botmarketing, mounted sob feature `marketing`):**
- `GET/POST /api/crm/lists` ✅
- `GET /api/crm/lists/{id}` ✅
- `PUT /api/crm/lists/{id}` ✅
- `DELETE /api/crm/lists/{id}` ✅
- `POST /api/crm/lists/{id}/refresh` ✅

**Ghost:**
| Endpoint | Feature |
|----------|---------|
| `GET/POST /api/crm/lists/{id}/members` | ❌ Members sub-resource |
| `DELETE /api/crm/lists/{id}/members/{memberId}` | ❌ |
| `GET /api/crm/lists/{id}/export` | ❌ |
| `POST /api/crm/lists/import` | ❌ |

**Functional coverage:** 56%

---

## 8. MINUTES (`minutes/minutes.html`)

**Working (via botapps ui_fragments, mounted):**
- `GET /suite/minutes/fragments/upcoming` ✅
- `GET /suite/minutes/fragments/live` ✅
- `GET /suite/minutes/fragments/transcripts` ✅
- `GET /suite/minutes/fragments/documents` ✅
- `GET /suite/minutes/fragments/actions` ✅
- `GET /suite/minutes/fragments/templates` ✅
- `GET /suite/minutes/fragments/signatures` ✅
- `GET /suite/minutes/fragments/attendance/{id}` ✅
- `POST /api/minutes/forms/meeting` ✅
- `POST /api/minutes/forms/action` ✅
- `POST /api/minutes/forms/action/{id}/done` ✅
- `POST /api/minutes/forms/document/{id}` ✅
- `POST /api/minutes/forms/sign/{id}` ✅

**Ghost:**
| Endpoint | Feature |
|----------|---------|
| `PATCH /api/minutes/meetings/{id}` | ❌ Não implementado |

**Functional coverage corrigida:** 93% — a maioria dos endpoints funciona!

---

## 9. VIDEO (`video/video.html` + JS)

**DISTRIBUIDO ENTRE DOIS CRATES:**

**botapps** (mounted — rotas de câmeras):
- `GET/POST /api/video/cameras` ✅
- `DELETE /api/video/cameras/{id}` ✅
- `GET /api/video/alerts` ✅
- `GET /api/video/analytics` ✅

**botvideo** (mounted sob feature `video` — 31 rotas de edição de vídeo):
Rotas como `/api/video/projects`, `/api/video/projects/{id}/clips`, `/api/video/templates`, etc. — **não relacionadas a câmeras**. São para edição de vídeo, não monitoramento.

**Frontend ghost:**
| Endpoint | Problem |
|----------|---------|
| `PUT /api/video/cameras/{id}` | ❌ botapps só tem DELETE para câmeras |
| `POST /api/video/alerts/{id}/acknowledge` | ❌ Não existe em nenhum crate |
| `GET /api/video/recordings` | ❌ Não existe |

**Functional coverage:** 57%

---

## Impact Summary (Revised)

| App | Working | Ghost | Coverage | Notas |
|-----|---------|-------|----------|-------|
| HR | 4 | 4 | 50% | Sem crate dedicado |
| Sales | 7 | 3 | 70% | Funnel + AI prediction extra |
| Banking | 4 | 3 | 57% | 1 path mismatch |
| Fraud | 5 | 4 | 56% | botfraud orfan tem stats |
| ITSM | 4 | 3 | 57% | |
| Compliance | 9 | 2 | 82% ↑ | Muito melhor que o inicial |
| Lists | 5 | 4 | 56% | |
| Minutes | 13 | 1 | 93% ↑ | Quase completo! |
| Video | 4 | 3 | 57% | botvideo é para outro propósito |

**Total corrigido:** 55 real + 27 ghost = **67% functional** (vs 55% reportado inicialmente)

## Key Corrections from Initial Report
1. **Compliance**: sobe de 71% → 82% (backend tem 11 rotas, não 7)
2. **Minutes**: sobe de 33% → 93% (13 endpoints funcionam, não 1)
3. **Sales**: sobe de 57% → 70% (funnel + AI prediction adicionados)
4. **Fraud**: botapps fornece 5 endpoints que funcionam (mais do que os 3 reportados)

## Suggested Fix

1. **Implement** missing endpoints OR remove UI features.
2. **Fraud**: montar `botfraud::configure_fraud_routes()` em server.rs para ativar `stats`, `events`, `assess`.
3. **Banking**: corrigir `/reconcile/auto` → `/reconcile` no frontend.
4. **Video**: decidir se câmeras/monitoramento são do escopo de `botvideo` ou permanecem em `botapps`.
5. **Compliance**: adicionar `GET /api/compliance/training` e `GET /api/compliance/risks`.
6. **Minutes**: adicionar `PATCH /api/minutes/meetings/{id}`.
