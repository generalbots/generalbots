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
- [x] `research` (Fixed: gated email dependencies, added missing imports)
- [x] `sources`

### Grupo 6: Analytics
- [x] `analytics`
- [x] `dashboards`
- [x] `monitoring` (Fixed: E0308 type mismatch in SVG generation)

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
    - Requer instalação de dependências de sistema (não resolvido neste ambiente).

### Avisos Comuns (Shared)
- [x] Fixed all shared warnings (unused variables/mut/imports in compiler, state, drive_monitor).

### Avisos Específicos de Feature
- [x] **mail**: Fixed unused imports.
- [x] **tasks**: Fixed unused imports.
- [x] **project**: Fixed unused imports.
- [x] **tickets**: Fixed unused imports.
- [x] **learn**: Fixed unused imports.
- [x] **analytics**: Fixed unused imports.
- [x] **designer**: Fixed unused variable `messages`.


## Remaining Warnings Plan (From TODO.tmp)
1.  **Automated Fixes**: Run `cargo clippy --fix --workspace` to resolve simple warnings (unused imports/variables/mut).
    - [ ] Execution in progress.
2.  **Manual Fixes**: Address warnings not resolvable by auto-fix.
    - [ ] Complex logic changes.
    - [ ] Feature gating adjustments.
3.  **Verification**: Run `cargo check --workspace` to ensure zero warnings.
