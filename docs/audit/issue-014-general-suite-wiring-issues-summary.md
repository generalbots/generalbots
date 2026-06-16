# Issue #014: General Suite Wiring Issues — Summary of all discrepancies (REVISED)

**Severity:** INFO (tracking/reference)
**Components:** Cross-cutting
**Type:** Meta-issue

## Description

This is a reference issue that summarizes all findings across the suite audit, **revised after auditing all 80+ crates**. Individual detailed issues exist for each category.

---

## Audit Scope (Revised)

- **Backend routes analyzed:** `botserver/src/main_module/server.rs` (1532 lines) + **80+ crates** in `botserver/crates/`
- **Frontend HTML files analyzed:** 80+ apps in `botui/ui/suite/`
- **JS files analyzed:** 40+ JavaScript files across key apps
- **Documentation analyzed:** `botbook/src/07-user-interface/apps/` and `botbook/src/08-rest-api-tools/`

## Key Corrections from Crate Audit

The initial analysis only checked `server.rs`. After auditing all crates, **several conclusions changed dramatically**:

| App | Initial Verdict | Revised Verdict | Why |
|-----|----------------|----------------|-----|
| **Compliance** | 71% functional | **82% functional** | botcompliance tem 14 rotas montadas |
| **Minutes** | 33% functional | **93% functional** | botapps ui_fragments tem 13 rotas |
| **Sales** | 57% functional | **70% functional** | funnel + AI prediction extra |
| **M365** | ❌ Ghost | ✅ Existe | botapps tem /api/m365/* |
| **Project** | ❌ Fully ghost | 🟡 Prefix mismatch | 11 rotas em /projects/* sem /api/ |
| **Goals** | ❌ Fully ghost | 🟡 23 rotas orfas | Código existe, wrappers vazios bloqueiam |
| **Learn** | ❌ Fully ghost | 🟡 12 rotas orfas | API existe mas não montada |
| **Fraud** | ❌ Stats ghost | 🟡 botfraud orfan tem stats | stats existe mas não montado |

---

## Consolidated Findings

### CRITICAL Severity (2 issues)
| # | App | Problem | Ghost Count |
|---|-----|---------|-------------|
| #001 | Drive | 3 conflicting API namespaces; HTMX template completely broken | 17+ |
| #002 | Monitoring | Many frontend endpoints don't exist; prefix mismatch on dashboard | 35+ |

### HIGH Severity (1 issue — REVISED)
| # | App | Problem | Revised Status |
|---|-----|---------|----------------|
| #003 | Retail | Fully ghost — no backend at all | ❌ Confirmado |
| #003 | Project | Prefix mismatch: frontend calls `/api/ui/project/*`, backend has `/projects/*` | 🟡 |
| #003 | Goals | 23 routes defined in botanalytics but blocked by empty wrappers | 🟡 |
| #003 | Learn | 12 API routes defined in botlearn but not mounted (3 UI routes work) | 🟡 |

### MEDIUM Severity (5 issues)
| # | App | Problem |
|---|-----|---------|
| #004 | HR, Sales, Banking, Fraud, ITSM, Lists, Video | Partial implementation (avg ~60% functional) |
| #004 | **Compliance, Minutes** | **Revised up: 82% and 93% functional** |
| #005 | Products | Missing pricelist-form.html partial; dual-prefix confusion |
| #006 | CRM | Inconsistent prefix by stage (cosmetic — both work) |
| #010 | Monitoring/Security | Infrastructure control endpoints exposed in UI |
| #013 | Botbook | Documentation drift; hundreds of endpoints undocumented |

### LOW Severity (4 issues)
| # | App | Problem |
|---|-----|---------|
| #007 | Social | HTMX analytics call broken (JS fetch works) |
| #008 | Campaigns | Duplicate POST + coming-soon alerts |
| #009 | People | Enrichment feature unavailable |
| #011 | Tickets | SLA panel is static (endpoint exists but unused) |
| #012 | Drive | Sentient prototype is complete mock |

---

## Quantitative Summary

| Metric | Count |
|--------|-------|
| Total suite apps | ~80 directories |
| Apps analyzed in depth | ~30 |
| **Rotas totais em todos os crates** | **~800+** |
| **Rotas efetivamente montadas** | **~600+** (estimado) |
| **Rotas orfas (código existe, não montado)** | **~60** (botfraud, botlearn API, botanalytics goals/insights, botcoredirectory api.rs) |
| Apps with confirmed ghost endpoints | 9 (partial) |
| Apps fully ghost | 1 (retail) |
| Apps with route prefix mismatches | 3 (project, drive HTMX, monitoring dashboard) |
| Apps with orphan crate routes | 4 (fraud, learn, goals, insights) |
| Botbook documentation issues | 5+ apps with incorrect/outdated API docs |

## Total Route Estimate by Crate

| Crate | Routes | Status |
|-------|--------|--------|
| botapps | 116 | ✅ Montado |
| botcoredirectory | 73 | ✅ Montado |
| botdocs | 95+ | ✅ Montado |
| botsheet | 90+ | ✅ Montado |
| botslides | 82+ | ✅ Montado |
| botemail | 55 | ✅ Montado |
| botvideo | 34 | ✅ Montado |
| botpeople | 32 | ✅ Montado |
| botcalendar | 31 | ✅ Montado |
| botproducts | 31 | ✅ Montado (feature billing) |
| botmarketing | 27 | ✅ Montado |
| botpaper | 24 | ✅ Montado |
| botsocial | 20 | ✅ Montado |
| botmonitoring | 18 | ✅ Montado |
| botcompliance | 14 | ✅ Montado |
| botproject | 11 | ✅ Montado |
| botfraud | 9 | ❌ Orfão |
| botanalytics insights | 9 | ❌ Orfão |
| botanalytics goals | 23 | ❌ Orfão |
| botlearn API | 12 | ❌ Orfão |
| botlearn UI | 3 | ✅ Montado |
| botcoredirectory (api.rs) | 5 | ❌ Orfão |
| botretail | 0 | Nenhum |
| botbanking | 0 | Nenhum |
| botdrive | 16 handlers | ✅ Rotas em server.rs |

## Recommended Actions

1. **Montar crates orfãos**: botfraud, botanalytics goals/insights, botlearn API, botcoredirectory api.rs
2. **Corrigir prefixos**: project, drive HTMX template, monitoring dashboard
3. **Criar API registry** — fonte única da verdade para endpoints, gerada das definições de rota
4. **CI check automatizado** — comparar chamadas frontend vs rotas backend
5. **Atualizar botbook** — documentar centenas de endpoints não documentados
6. **Feature flags** — UI features sem backend devem ficar atrás de flags de desenvolvimento
