# Plano de Compilação Individual de Features

## Objetivo
Compilar cada feature individualmente do botserver com `cargo check --no-default-features --features <feature>` para identificar todos os erros de dependência e compilação, consolidando os logs para análise sistemática.

## Features a Testar

### Grupo 1: Comunicação
- [x] `chat`
- [x] `people`
- [x] `mail`
- [ ] `meet` (Failed: webrtc-sys C++ build error: missing absl/container/inlined_vector.h)
- [x] `social`

### Grupo 2: Produtividade
- [x] `calendar`
- [x] `tasks`
- [x] `project`
- [x] `goals`
- [x] `workspaces`
- [x] `tickets`
- [x] `billing`
- crm
### Grupo 3: Documentos
- [x] `docs`
- [x] `sheet`
- [x] `slides`
- [x] `paper`

### Grupo 4: Mídia
- [x] `video`
- [x] `player`
- [x] `canvas`

### Grupo 5: Aprendizado
- [x] `learn`
- [ ] `research` (Failed: missing EmailDocument struct, unknown field email_db, type inference errors)
- [x] `sources`

### Grupo 6: Analytics
- [x] `analytics`
- [x] `dashboards`
- [ ] `monitoring` (Failed: E0308 type mismatch in SVG generation)

### Grupo 7: Desenvolvimento
- [x] `designer`
- [x] `editor`

### Grupo 8: Admin
- [x] `attendant`
- [x] `security`
- [x] `settings`

## Erros e Avisos Identificados

### Erros de Compilação (Bloqueios)
- [ ] **meet**: Falha no build C++ da dependência `webrtc-sys` (header `absl/container/inlined_vector.h` não encontrado).
- [ ] **research**: Diversos erros de tipo e campos ausentes:
    - `EmailDocument` não encontrado no escopo.
    - Campo `email_db` desconhecido na struct `UserIndexingJob`.
    - Erros de inferência de tipo em `vectordb_indexer.rs`.
- [ ] **monitoring**: Erro `E0308` (mismatched types) na geração de SVG em `app_generator.rs` (conflito entre `f32` e `f64`).

### Avisos Comuns (Shared)
- `botserver/src/basic/compiler/mod.rs:358:25`: `unused mut` e `unused variable` (`conn`).
- `botserver/src/basic/compiler/mod.rs:357:25`: `unused variable` (`cron`).
- `botserver/src/core/shared/state.rs:469:13`: `unused mut` (`debug`).
- `botserver/src/drive/drive_monitor/mod.rs:20:7`: `KB_INDEXING_TIMEOUT_SECS` (dead code).
- `botserver/src/drive/drive_monitor/mod.rs:39:5`: `kb_indexing_in_progress` (dead code).

### Avisos Específicos de Feature
- **mail**: Unused imports em `src/core/shared/schema/mail.rs`.
- **tasks**: Unused imports em `src/core/shared/schema/tasks.rs`.
- **project**: Unused imports em `src/core/shared/schema/project.rs`.
- **tickets**: Unused imports em `src/core/shared/schema/tickets.rs`.
- **learn**: Unused imports em `src/core/shared/schema/learn.rs`.
- **analytics**: Unused import em `src/analytics/mod.rs`.
- **designer**: Unused variable `_messages`.

