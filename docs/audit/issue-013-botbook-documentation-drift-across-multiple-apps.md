# Issue #013: BOTBOOK — Documentation drift from actual implementation

**Severity:** MEDIUM
**Components:** `botbook/src/` (multiple files)
**Type:** Documentation drift

## Description

Comparison between botbook documentation and actual frontend/backend implementation (after auditing all 80+ crates) reveals several discrepancies. Some endpoints that botbook says don't exist actually DO exist, just in different crates.

---

### DRIVE (most critical)

| Botbook says | Backend has | Notes |
|-------------|-------------|-------|
| `/api/drive/list` | `/api/files/list` | Different namespace entirely |
| `/api/drive/upload` | `/api/files/write` | Different path + method semantics |
| `/api/drive/download/:path` | `/api/files/download` (POST) | Different path + method |
| `/api/drive/delete/:path` | `/api/files/delete` (POST) | Different path + method |
| `/api/drive/mkdir` | `/api/files/createFolder` | Different path |

Botbook documenta `/api/drive/*` mas a SPA moderna em `drive.js` usa `/api/files/*` (que funciona). O HTMX legado (`index.html`) usa `/api/drive/*` (não funciona). O backend (handler no crate `botdrive`, rotas registradas em `server.rs`) só registra `/api/files/*`.

---

### M365

**Correção:** Ao contrário do reportado inicialmente, os endpoints `/api/m365/*` **EXISTEM** no crate `botapps` (mounted em server.rs):

| Botbook says | Backend has (botapps) | Status |
|-------------|---------------------|--------|
| `GET /api/m365/onedrive/files` | `GET /api/m365/onedrive` ✅ | ✅ Existe (path ligeiramente diferente) |
| `POST /api/m365/onedrive/files` | ❌ Não implementado | ❌ |
| `GET /api/m365/onedrive/files/{id}/download` | ❌ Não implementado | ❌ |
| — | `GET /api/m365/sharepoint` ✅ | Extra |
| — | `GET /api/m365/calendar` ✅ | Extra |
| — | `GET /api/m365/settings` ✅ | Extra |

**Veredito:** Botbook subestima — há mais endpoints M365 do que documentado, mas faltam os específicos de download de arquivos.

---

### PLAYER

Botbook documenta:
```
GET /api/drive/{bot_id}/files/{file_path}?preview=true
GET /api/drive/{bot_id}/stream/{file_path}
GET /api/drive/{bot_id}/thumbnail/{file_path}
```
Nenhum desses endpoints existe no backend. Drive usa `/api/files/*` sem `{bot_id}` no path.

---

### Suite Apps Status

`botbook/src/07-user-interface/apps/suite-apps-status.md` marca **21 apps como "Done"**. Revisão após auditoria completa dos crates:

| App | Botbook Status | Real Status | Correction |
|-----|---------------|------------|------------|
| drive | 🟢 Done | ⚠️ Prefix mismatch | HTMX template quebrado |
| monitoring | 🟢 Done | 🟡 18/18 endpoints OK (placeholders) | Funciona mas dados placeholders |
| products | 🟢 Done | 🟢 Funciona | Ambos os prefixos existem |
| tickets | 🟢 Done | 🟢 36 rotas funcionando | SLA panel é mock, resto OK |
| m365 | 🟢 Done | 🟢 Parcial | Endpoints existem em botapps |
| compliance | 🟢 Done | 🟢 82% funcional | Bem melhor que estimativa inicial |
| minutes | 🟢 Done | 🟢 93% funcional | Quase completo |
| learn | 🟢 Done | 🟡 3 UI rotas OK, 12 API orfas | API precisa ser montada |
| project | 🟢 Done | 🟡 Prefixo sem `/api/` | Backend existe mas não bate |
| goals | 🟢 Done | ❌ Wrappers vazios bloqueiam | Código existe mas não roda |
| retail | 🟢 Done | ❌ Sem backend algum | Apenas modelos de dados |

### API Documentation

`botbook/src/08-rest-api-tools/` documenta ~50 API modules. A auditoria revelou que muitos módulos de API existem em crates mas **nunca foram documentados no botbook**:

| Crate | Rotas | Documentado? |
|-------|-------|-------------|
| botcalendar | 31 | ❌ Não |
| botpeople | 32 | ❌ Não |
| botsocial | 20 | ❌ Não |
| botproducts | 31 | ❌ Não |
| botemail | 55 | ❌ Não |
| botsheet | 90+ | ❌ Não |
| botslides | 82+ | ❌ Não |
| botdocs | 95+ | ❌ Não |
| botpaper | 24 | ❌ Não |
| botfraud (orfan) | 9 | ❌ Não |
| botanalytics goals | 23 | ❌ Não |
| botanalytics insights | 9 | ❌ Não |

---

## Impact

- Developers relying on botbook miss **centenas de endpoints** que já existem mas não estão documentados.
- Suite-apps-status.md marca apps como "Done" que estão quebrados (goals, retail) ou incompletos.
- Novos contribuidores têm visão incorreta da arquitetura real.

## Suggested Fix

1. **Auditar** toda a documentação do botbook contra as rotas reais dos crates (não apenas server.rs).
2. **Adicionar** documentação para crates não documentados: botcalendar, botpeople, botsocial, botproducts, botemail, botsheet, botslides, botdocs, botpaper.
3. **Corrigir** suite-apps-status.md: goals → ❌, retail → ❌, project → 🟡, learn → 🟡.
4. **Automatizar**: criar CI check que compara endpoints documentados contra rotas reais usando `grep -r "\.route(" botserver/crates/`.
5. **Gerar** documentação automaticamente a partir das definições de rota em cada crate.
