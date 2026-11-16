# Shared rules for the gbgov agents (read first, every time)

Project: General Bots monorepo at C:\Users\thipa\gbstandalone, branch `gbgov`: a government desktop
edition, Chat + Drive on the person's own machine. The product is a NEW chat-first assistant built on the
existing back end. Benchmark: the best desktop AI assistants. Owner's words: "a new product on top of the
technology stack we have available", with General Bots' features and the edition's visual identity.

Know before you start (do not re-explore the repository):
- Codebase map: C:\Users\thipa\.claude\projects\C--Users-thipa-gbstandalone\handoff-2026-11-14\CODEBASE-MAP.md
  (read only the sections your task needs; line numbers drift, reopen a file before editing it).
- The assistant shell: `botui/ui/suite/assistant/` (plain scripts `modules/NN_*.js`, own `i18n/pt-BR.js`
  and `en.js`, `assistant.css`, `assistant-chat.css`). Identity: `botui/ui/suite/identity/` (`--id-*`
  tokens, `window.GBIdentity`). Wizard: `botui/ui/wizard/`. Legacy chat engine, for protocol reference
  only: `botui/ui/suite/chat/` (`messages/70_stream.js`, `chat-send.js`, `modules/40_consent_cards.js`).
- Build profile: cargo feature `gbgov` (chat, drive, llm, vectordb, kb-extraction, security, directory,
  settings, consent, agent-vm). NOT compiled: terminal, database tool, office apps, research/web search,
  integrations, mail, voice. The agent workspace (`agent-vm`) is compiled but on Windows the Incus
  backend is unavailable without WSL2; agent mode is off by default and disabled gracefully at runtime
  when the backend is missing. Product file `botserver/.product.gbgov`.
- Private design references: `GBGOV/Refs/` (design system + two screenshots). Read-only.

Hard rules:
- Never commit, push, switch branches, or write to GitHub. Leave changes in the working tree; the driver
  commits. Never touch `GBGOV/` except to read it and to use `GBGOV/dev/` tools.
- Never name other companies' products, the reference brand, `GBGOV`, or the private plan in any repo
  file, comment or message meant to be public.
- The owner works on this machine: never open a visible window, never start/stop the desktop app or the
  server, never attach to the owner's browser. Headless only.
- Rust: never `cargo build`; `cargo check -j 2` only, and only if no cargo/rustc is running
  (`tasklist | grep -i -E "cargo|rustc"`). Memory is tight: one cargo at a time, ever.
- No `unwrap`/`expect`/`panic!` outside tests, no `#[allow]`, no CDN or remote asset, no file over 450
  lines (split at 350), plain `<script>` files with "use strict", absolute `/suite/...` paths.
- `main` is merged into this branch often: prefer new files; keep edits to existing shared files tiny
  and list every one in your report.
- Every user-visible string in pt-BR (with accents) and en. Every animation has a reduced-motion form.

Testing a UI change (the installed server embeds the UI at build time, so use the dev proxy):
- `node GBGOV/dev/devproxy.js botui/ui/suite 8090` (start it in the background if port 8090 is not
  listening; leave it running) serves `/suite/*` live from the repo and forwards API/WebSocket to 8080.
- One step = one command: `python GBGOV/dev/open_page.py --port 8090 [--say "…"] [--eval "js"] [--shot f]`.
  It signs in with the test administrator. Never print or copy the token or the password.
- Real model calls cost money and may leave the machine: at most 4 prompts per task unless the brief says
  otherwise, each prefixed "[teste]".
- Screenshots and scratch files go under the scratchpad folder named in your brief, never in the repo.
  Look at your own screenshots before reporting.

Report (compact, no narrative): what changed (files, one line each); evidence (screenshot paths, command
output in one line); what does NOT work yet, exactly; anything in another agent's area that blocked you.
Stop and report as soon as the brief is met or you are blocked; do not expand the scope.

## Commit messages (owner's rule, 2026-11-15)

A commit message describes the change and nothing else: a subject line plus,
when needed, two to five lines on what changed and why. Never put in a commit
message: plans, next steps, "still open", "not done", queues, limitations to
fix later, verification narratives (measurements, "verified live", screenshot
names), test clean-up notes, or how the work was organised. Those belong in
the report to the driver. Never commit a plan, notes or report file. Never
amend or rewrite commits that are already pushed unless the owner asks.
