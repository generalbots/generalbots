# Issue #003: Apps with Non-Functional Frontend — incomplete backend wiring

**Severity:** HIGH
**Components:** `botui/ui/suite/retail/`, `botui/ui/suite/project/`, `botui/ui/suite/goals/`, `botui/ui/suite/learn/`
**Type:** Ghost functionality / Route mismatch / Orphan modules

## Description

The following apps have complete HTML/JS frontend interfaces but the backend is either absent, uses different route prefixes, or has routes defined but not mounted in the server.

---

## 1. RETAIL (`retail/retail.html` + `retail/retail.js`)

**Backend: NENHUM.** O crate `botretail` é puramente de modelos de dados (Product, StockLevel, PriceList, Promotion, PosSession) — não tem `axum::Router`, `.route()`, ou função `configure` alguma. Nenhuma rota `/api/retail/*` existe em lugar nenhum.

**All endpoints called are GHOST:**
| Endpoint | Method | Feature |
|----------|--------|---------|
| `/api/retail/stock` | GET | List stock/inventory |
| `/api/retail/branches` | GET | List branches |
| `/api/retail/promos` | GET | List promotions |
| `/api/retail/suppliers` | GET | List suppliers |
| `/api/retail/top-products` | GET | Top products |

**Veredito:** 100% ghost. Não há backend algum.

---

## 2. PROJECT (`project/project.html`)

**Backend: EXISTE, MAS COM PREFIXO DIFERENTE.** O crate `botproject` (11 rotas) está **montado em server.rs** sob `#[cfg(feature = "project")]`, mas registra rotas **sem o prefixo `/api/`**:

**Rotas que o backend realmente serve:**
| Método | Path real (backend) | Handler |
|--------|--------------------|---------|
| POST | `/projects` | create_project |
| GET | `/projects/:project_id` | get_project |
| DELETE | `/projects/:project_id` | delete_project |
| POST | `/projects/:project_id/tasks` | create_task |
| GET | `/projects/:project_id/tasks` | get_tasks |
| GET | `/projects/:project_id/gantt` | get_gantt_chart |
| GET | `/projects/:project_id/timeline` | get_timeline |
| GET | `/projects/:project_id/critical-path` | get_critical_path |
| PUT | `/tasks/:task_id/progress` | update_task_progress |
| POST | `/tasks/:task_id/dependencies` | add_dependency |
| DELETE | `/tasks/:task_id` | delete_task |

**Rotas que o frontend chama:**
| Método | Path (frontend) | Funcionalidade |
|--------|----------------|----------------|
| GET | `/api/ui/project/new` | New project form |
| GET | `/projects` | List projects (esse pode funcionar!) |
| GET | `/tasks` | List tasks |
| POST | `/api/ui/project/gantt` | Update Gantt chart |

**Problema:** O frontend usa `/api/ui/project/new` e `/api/ui/project/gantt`, mas o backend registra `/projects` e `/projects/:project_id/gantt`. A única chamada que poderia funcionar é `GET /projects` (sem prefixo `/api/`), mas o retorno seria JSON e não HTML. As demais (especialmente as com prefixo `/api/ui/`) **não existem** e retornam 404.

**Veredito:** Backend existe mas os caminhos não batem com o que o frontend chama.

---

## 3. GOALS (`goals/goals.html` + JS)

**Backend: ROTAS DEFINIDAS NO CRATE MAS BLOQUEADAS POR WRAPPERS VAZIOS.**

O crate `botanalytics` define **23 rotas de Goals** (15 API + 8 UI):
- `goals.rs`: `GET/POST /api/goals/objectives`, `PUT/DELETE /api/goals/objectives/:id`, `POST /api/goals/key-results/:id/check-in`, etc.
- `goals_ui.rs`: `GET /api/ui/goals/objectives`, `GET /api/ui/goals/dashboard`, `GET /api/ui/goals/progress`, etc.

Porém, os **wrappers em `botserver/src/analytics/goals.rs` e `goals_ui.rs` retornam `Router::new()` vazio**, sem delegar para `botanalytics::goals::configure_goals_routes()`. As rotas **existem no código compilado mas nunca são chamadas**.

**Funcional:** Nada — 0 rotas servidas de 23 definidas.

**Veredito:** Código backend existe mas é código morto — nunca alcançado.

---

## 4. LEARN / LEARN-APP (`learn/learn.html`, `learn/learn-app.html`)

**Backend: ROTAS DE API DEFINIDAS MAS NÃO MONTADAS; ROTAS DE UI MONTADAS PARCIALMENTE.**

O crate `botlearn` define:
- **12 rotas de API** em `routes_learn.rs` (`/api/learn/courses`, `/api/learn/enroll`, `/api/learn/progress/:enrollment_id`, `/api/learn/certificates/issue`, etc.) — **NÃO MONTADAS** em server.rs
- **3 rotas de UI** em `ui.rs` (`/suite/learn`, `/suite/learn/create`, `/suite/learn/:id`) — **MONTADAS** em server.rs line 622

**Rotas que o frontend chama vs backend definido:**

| Frontend | Backend definido? | Montado? |
|----------|------------------|----------|
| `GET/POST /api/learn/courses` | ✅ POST em `routes_learn.rs` | ❌ Orfan |
| `GET /api/learn/courses/{id}` | ❌ (backen tem `POST /api/learn/courses/:id/publish`) | ❌ |
| `GET /api/learn/certificates` | ✅ `GET /api/learn/certificates/verify` | ❌ Orfan |
| `GET /api/learn/progress` | ✅ `PUT /api/learn/progress/:enrollment_id` | ❌ Orfan |
| `POST /api/learn/courses/{id}/enroll` | ✅ `POST /api/learn/enroll` | ❌ Orfan |
| `POST /api/learn/lessons/{id}/complete` | ✅ `POST /api/learn/complete/:enrollment_id` | ❌ Orfan |
| `POST /api/learn/quizzes/{id}/submit` | ❌ Não definido | ❌ |

**Veredito:** A maioria dos endpoints de API que o frontend chama TEM implementação no crate `botlearn`, mas **nenhuma está montada** no servidor. Se `configure_learn_api_routes()` fosse chamada em server.rs, a maioria dos recursos funcionaria. As 3 rotas de UI (`/suite/learn/*`) estão funcionais.

---

## Impact Summary (Revised)

| App | Backend Code | Routes Mounted | Frontend Calls Match | UX |
|-----|-------------|----------------|---------------------|-----|
| Retail | ❌ Nenhum | 0/0 | ❌ | Broken |
| Project | ✅ 11 rotas em botproject | ✅ 11/11 montadas | ❌ Prefixo diferente | Broken |
| Goals | ✅ 23 rotas em botanalytics | ❌ 0/23 (wrappers vazios) | ❌ | Broken |
| Learn API | ✅ 12 rotas em botlearn | ❌ 0/12 (nunca chamadas) | ✅ Parcial | Broken |
| Learn UI | ✅ 3 rotas em botlearn | ✅ 3/3 montadas | ✅ 3/3 | ✅ Working |

- **Problemas identificados:**
  1. **Retail**: não há backend algum — criar ou remover
  2. **Project**: alinhar prefixo — frontend usa `/api/ui/project/*`, backend tem `/projects/*`
  3. **Goals**: remover wrappers vazios em `botserver/src/analytics/goals.rs` e `goals_ui.rs` para que as 23 rotas do crate `botanalytics` sejam servidas
  4. **Learn**: adicionar `sub_router.merge(crate::learn::creator::configure_learn_api_routes()...)` no bloco `#[cfg(feature = "learn")]` em server.rs

## Suggested Fix

1. **Retail**: decidir: implementar backend OU remover da suite. Se mantido, marcar UI como "Under development".
2. **Project**: mudar frontend para chamar `/projects/...` em vez de `/api/ui/project/...`, ou criar wrapper que traduz os paths.
3. **Goals**: corrigir `botserver/src/analytics/goals.rs` para delegar a `botanalytics::goals::configure_goals_routes()` em vez de retornar `Router::new()` vazio.
4. **Learn**: montar `configure_learn_api_routes()` em server.rs. Adicionar endpoints faltantes (`GET /api/learn/courses/{id}`, `POST /api/learn/quizzes/{id}/submit`).
5. Para todos: adicionar logging de `501 Not Implemented` para endpoints não implementados.
