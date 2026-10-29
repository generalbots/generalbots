# CLAUDE.md — Edição desktop local (Chat + Drive)

<!-- Salve em .claude/CLAUDE.md (o AGENTS.md proíbe arquivos novos na raiz). -->

@../AGENTS.md

## Exceções ao AGENTS.md para este trabalho

- Responda ao usuário em **português** (o AGENTS.md pede inglês; aqui o usuário pediu português).
- Mensagens de commit e PR continuam em **inglês**, como no resto do repositório.
- Portas do projeto: API 8080, Suite 3000, login 5000.

## Objetivo deste branch

Entregar uma edição **desktop totalmente local** do General Bots contendo apenas
**Chat** e **Drive**, para distribuição em um programa de governo.
Branch de trabalho: `release/desktop-chat-drive`.

Princípio: preferir mudanças que possam voltar para o `main` (feature flags,
config, `.product`) em vez de apagar código. Nada de fork divergente.

## Estado atual (levantado no código)

- `botapp/` é só a casca Tauri: aponta para `../botui/ui/suite` e monitora
  `localhost:9000`/`3000`, mas **não inicia o botserver**.
- `botserver/Cargo.toml`: a feature `chat` puxa `automation` + `cache` (Redis);
  `drive` usa `rust-s3` (precisa de S3/MinIO). Diesel é só `postgres`.
- `botcore/src/bootstrap/bootstrap_manager/install.rs` instala no primeiro
  boot: vault, tables, cache, drive, directory, llm, vector_db, alm, alm-ci —
  baixando binários da internet (`botserver/3rdparty.toml`).
- `3rdparty.toml` só tem URLs linux-amd64 e windows (sem macOS).
- Existe a feature `no-security` (sem RBAC/directory).

## Metas, em ordem

1. **Feature bundle** `desktop-local = ["chat", "drive", "embed-ui", ...]`
   no `botserver/Cargo.toml`; build limpo com `--no-default-features`.
2. **Bootstrap enxuto**: quando compilado com `desktop-local`, instalar só
   tables, cache e drive (sem alm, alm-ci, vector_db, directory, llm, a menos
   que decidido abaixo).
3. **Instalação offline**: os binários de terceiros vêm dentro do instalador
   (sem download no primeiro boot). Checksums verificados.
4. **Sidecar**: `botapp` inicia o botserver (e a pilha local) ao abrir e
   encerra tudo ao fechar; janela abre só quando `/api/product` responder.
5. **UI**: launcher da Suite mostra só os apps de `/api/product`; `.product`
   com `apps=chat,drive`; `suite-default-app=chat`.
6. **Instaladores**: Windows (MSI/NSIS) e Linux (deb/AppImage) via Tauri.

## Decisões em aberto (perguntar antes de implementar)

- Login: manter `directory` (Zitadel) ou modo usuário único (`no-security`)?
- Segredos: manter Vault ou trocar por keyring do SO no modo local?
- LLM: modelo local (llama.cpp) ou provedor externo configurável?

## Regras de trabalho

- Um commit por meta, mensagem em inglês, prefixo `desktop-local:`.
- Antes de cada commit: `cargo build -p botserver --no-default-features --features desktop-local`
  e `cargo test` nos crates tocados.
- Não remover features nem apps do `main`; só condicionar com `#[cfg(feature = ...)]`.
- Não adicionar dependência nova sem registrar licença em `LICENSES-desktop.md`.
- Registrar a licença de todo binário de terceiro empacotado (exigência do programa).
