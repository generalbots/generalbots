# Windows Desktop Port: Stabilisation Plan

This page is the working execution plan for stabilising General Bots on Windows 11,
chat first. It is grounded in findings recorded on 2026-11-14 at commit `7da5f529d`.
Every file reference and issue number comes from that study; claims without a cited
source are not stated here.

---

## Goal and Definition of Done

The work is complete when a user on Windows 11 can:

1. **Start the desktop application** (`botapp.exe`, which runs `botserver.exe`
   as its sidecar), pass the first-run wizard on a new machine, and reach the
   chat window without any manual step.
2. **Sign in** with the credentials created during provisioning.
3. **Send a chat message** to the default bot and **receive a streamed reply**
   — tokens appearing incrementally, not in one burst at the end.
4. **Stop generation mid-reply** and have the next message accepted immediately.
5. **Restart the application** and find the previous conversation in history.

That is the Phase 2 exit. Later phases extend the scope:

- **Phase 3 exit:** the local GPU model (RTX 4070 or similar) answers the chat
  with GPU layers applied; the user can change the model endpoint and key in
  Settings.
- **Phase 4 exit:** Drive and the knowledge base load correctly; at least ten of
  the 100-scenario UAT cases in issue #2031 pass in a browser test.

Out of scope for this plan: cloud SaaS, kiosk OS, mobile, and any suite app not
listed above.

---

## Current State — 2026-11-14

The first pass of this plan was executed on a Windows 11 machine against a
stack that an older build had provisioned. Every line below was observed in
that run.

| Dimension | Result | Evidence |
|---|---|---|
| Compile | `cargo check` and a debug build of `botserver` (`--no-default-features --features desktop-local`) and of `botapp` (`--features desktop-local,custom-protocol`) succeed. | build logs |
| Boot | The rebuilt server starts against the existing stack: about 80 s when the services have to be started, 15–25 s when they are already running. | server log |
| Sign-in | The desktop application signs in with the session token it keeps in `conf/directory/desktop-session.json`; no password prompt. | `botapp/src/desktop/wizard/session.rs` |
| Chat | The chat window loads, the WebSocket connects, the default bot's `start.bas` greeting arrives, and a reply from a remote OpenAI-compatible endpoint streams into the bubble (14 → 837 characters over about 2.5 s). | browser test over Chrome CDP |
| History | After a server restart the same conversation is listed and the reply is readable. | browser test |
| Application | `botapp.exe` starts its own sidecar (API ready in 16.5 s), opens the suite signed in, and answers a message. | application log; WebView2 CDP |
| Upstream CI | Failed or cancelled on every recent `main` commit; no Windows release binary exists. | `gh run list` |

The product profile is unchanged: `desktop-local` enables the full suite
(`all-apps` + `embed-ui` + `mail`), sign-in is required (`no_security=false`)
and the Windows application still ships the kiosk settings (`edition=os`,
`shell_mode=true`).

### Defects Found by Running It

These were not visible from reading the code; each one stopped the smoke test
until it was fixed or worked around.

| Defect | Cause | State |
|---|---|---|
| The suite's own scripts were answered with `429`. Chat reported a failed connection because `chat-websocket.js` and `chat-send.js` never loaded. | When the server also serves the UI, one cold page load (a few hundred files) exhausts the process-wide API quota (100 requests per second, burst 150). | Fixed: static files draw from their own quota (`botsecurity-auth/src/rate_limiter_static.rs`). |
| Vault defaults were never seeded on Windows, so drive storage was "not configured" and no message encryption key existed. | `SafeCommand` rejected the full path `…\vault.exe`: the allowlist compared exact file names. | Fixed: the name is lower-cased and `.exe` stripped before the lookup. |
| Once seeding ran, it replaced the LLM endpoint chosen in the wizard with an empty one. | `seed_vault_defaults` used `vault kv put`, which overwrites, and it runs whenever the probe path (`secret/gbo/drive`) is missing — the state of a partly seeded stack. | Fixed: `-cas=0`, create only. |
| The remote model answered `401 INVALID_TOKEN`. | The seeded placeholder `openai_key=none` was sent as `Authorization: Bearer none`. | Fixed: the placeholder is treated as no key (`botllm/src/retry.rs::bearer_header`). |
| Messages of the first session after seeding would be unreadable after a restart. | The master key is read from Vault once at start-up, before a new stack is seeded; the fallback is a random key. | Fixed: Vault is asked once more before the ephemeral fallback. |
| `vault secrets enable` failed on Windows. | `VAULT_CACERT` pointed at a CA file that does not exist when Vault runs over HTTP. | Fixed: the variable is set only when the file exists. |
| The compiler ran out of memory with the default job count. | Many `rustc` processes at once on a machine with little commit headroom ("memory allocation failed", "failed to mmap rmeta"). | Work-around: `cargo build -j 2` (or `CARGO_BUILD_JOBS`). |

---

## Operating Model

A small, sustained team of three to four AI agents keeps the work moving without
introducing conflicts or redundant cargo invocations.

### Roles

| Role | Model class | Responsibilities |
|---|---|---|
| **Orchestrator** | Strongest available | Plans each sprint, assigns files to workers, runs the single shared `cargo check` and browser tests, reviews every diff, is the only agent that commits and pushes. Edits source only for small integration fixes found in review. |
| **Worker A** | Opus / Sonnet | Owns Workstream A (Windows process and probes). |
| **Worker B** | Opus / Sonnet | Owns Workstream B (Windows paths and product profile). |
| **Worker C** | Opus / Sonnet | Owns Workstream C (chat pipeline and local LLM). Activated only when A and B have landed their Phase 1 changes — three workers in parallel only when file sets are fully disjoint. |

No more than three workers run simultaneously. The orchestrator serialises the
cargo build; workers produce diffs and wait for the orchestrator's check result
before the next edit.

### Workstream File Ownership

| Workstream | Files | Phase |
|---|---|---|
| **A — Windows process & probes** | `botlib/src/security/command_guard.rs` | 1 |
| | `botserver/crates/botcore/src/bootstrap/bootstrap_utils/procs.rs` | 1 |
| | `botserver/crates/botcore/src/bootstrap/bootstrap_utils/health.rs` | 1 |
| **B — Windows paths & product profile** | `botserver/crates/botcore/src/shared/utils/init_secrets_manager.rs` | 1 |
| | `botserver/crates/botcore/src/product/load.rs` | 1 |
| | `botapp/src/desktop/sidecar.rs` | 1 |
| | All 137 `/tmp` sites (batch script) | 1 |
| **C — Chat pipeline & local LLM** | `botserver/crates/botllm/src/openai_stream.rs` | 2 |
| | `botserver/src/core/bot/pipeline/llm/attempt.rs` | 2 |
| | `botserver/src/core/bot/pipeline/exec.rs` | 2 |
| | `botserver/crates/botllm/src/local/ensure_llama_servers_running.rs` | 3 |
| | `botserver/src/core/bot/pipeline/chat_controls.rs` | 2 |
| | `botui/ui/suite/chat/modules/60_computer_use.js` | 2 |

A worker must not edit a file owned by another workstream. If a fix requires
touching both, the orchestrator merges the two tasks and assigns them to one
worker for that cycle.

### Worker Rules

- No `cargo` invocations. Submit the diff; wait for the orchestrator's check log.
- No `git` state-changing commands (no add, commit, push, checkout, branch).
- Minimal diffs: change only what the task requires.
- Windows-specific behaviour goes behind `#[cfg(windows)]` or `#[cfg(not(unix))]`
  unless the bug exists on all platforms.
- Rust rules: no `unwrap()`, `expect()`, `panic!()`, `todo!()`, or
  `unimplemented!()` outside `#[cfg(test)]` blocks; no `#[allow(...)]`; files
  must stay under 450 lines (split when approaching 350).
- JS rules: functions in globally-visible plain `<script>` files, ordered by the
  numeric prefix convention; `"use strict"` at the top of every module.

### Review Gates (in order)

1. **Syntax gate** — `scripts/verify-rust-syntax.sh` (or the equivalent
   `rustfmt --emit stdout` check on changed `.rs` files). Read-only; no compile.
2. **Type check** — `cargo check -p botserver --no-default-features --features desktop-local`
   in the background, with the log written outside the repository; the orchestrator reads the log.
3. **Debug build** — `cargo build -p botserver --no-default-features --features desktop-local`
   after type check passes. Only the orchestrator runs this. No release builds.
4. **Smoke checklist** — orchestrator opens one Chrome tab per step (Chrome CDP
   port 9222; one script = one action; see the checklist below).

### Commit and Push Cadence

- Small, focused commits to `main`; rebase on `origin/main` before each push
  (upstream lands ~70 commits per day).
- Commit message prefix: `fix(windows):`, `fix(chat):`, or `fix(llm):` per workstream.
- Push to every configured remote in the same step.
- Never amend a commit that has been pushed.

---

## Findings That Drive the Plan

The table below lists verified defects. Confidence codes: **[run]** = reproduced
by running something; **[read]** = code was read; **[agent]** = subagent-read,
treat as a strong lead; **[inferred]** = reasoning from code or logs.

### Windows Foundations

| Finding | File:line | Confidence |
|---|---|---|
| Work root is a `\\?\` verbatim path; joined with `/` → Windows rejects it (os error 123). `PROMPT.md`, `start.bas` and template copies all fail silently; chat uses the generic fallback prompt. | `init_secrets_manager.rs:255`; callers `:144`; `ws/prompt.rs:68,80`; `ws/message.rs:79-97` | [agent] × 2 + log [run] |
| Four separate `get_work_path` / `get_stack_path` definitions reading different env vars. | `botlib/src/work_path.rs:8`; `botbasic_core/src/utils/to_array.rs:97`; `botcorebot/src/utils.rs:3`; `botbasic_core/src/security_utils.rs:79` | [agent] |
| Command allowlist matches the exact file name with no `.exe` stripping and no case folding; `taskkill` is not listed; killing a component is a silent no-op. | `command_guard.rs:13-125`; `bootstrap_utils/procs.rs:83-98` | [agent] × 2 |
| `SafeCommand` strips the child environment to PATH, USERPROFILE, SYSTEMROOT only. | `command_guard.rs:392-414` | [agent] |
| No Windows Job Objects; services started via `cmd /C start /B` outlive the server; liveness check is `tasklist` by image name, so orphans count as healthy. | bootstrap crates | [agent] |
| Readiness probe falls back to `ss -tln`, absent on Windows; every boot logged "vector_db did not respond after 45 s". | `bootstrap_utils/health.rs:38-73` | [agent] + log [run] |
| Cert generation and Vault init shell out to `sh`, `bash`, `scp`, `ssh` with no `cfg` gate. | `certs_utils.rs:164`; `initialize_vault.rs`; `bootstrap/utils.rs:29` | [agent], #2381 |
| 137 hardcoded `/tmp` sites; bot upload and seven debug dumps per script run are host-affecting. | `load_template_from_directory.rs:198`; `basic/mod.rs:112-198`; `boteditor/src/handlers.rs:97` | [agent], #2380 |
| `.product.desktop-local` is searched only relative to cwd; sidecar copies it only if absent, so a stale stack copy wins after an upgrade. | `product/load.rs:104-124`; `sidecar.rs:139-164` | [agent] × 2 |
| Server binds `0.0.0.0` when `no_security=false`; triggers Windows Firewall prompt on first run. | `listen_impl.rs:30-34` | [agent], #2477 |
| Sidecar reveals the window after 120 s even if the API never answered. | `sidecar.rs:110-129`; `main.rs:175-196` | [agent] |

### Chat Correctness

| Finding | File:line | Confidence |
|---|---|---|
| No real streaming: the byte-read loop drains the whole HTTP body into a 100,000-slot channel before the SSE parser starts. Replies arrive in one burst. Total timeout is 180 s. | `openai_stream.rs:142,239-271`; `openai_client.rs:300` | **[read]** |
| Cancel or timeout does not abort the detached generation task; it holds a process-wide one-permit gate, blocking the next message. | `pipeline/llm/attempt.rs:76`; `retry.rs:103-106`; `openai_stream.rs:145` | [agent] × 2 |
| Outer turn timeout only logs; the run is marked Succeeded. An empty assistant row is saved after exhausted retries. | `pipeline/exec.rs:500-518`; `pipeline/llm/stream.rs:272` | [agent] |
| Agent layer is on by default; every non-informational turn calls the Incus VM check, which fails fast on Windows but is swallowed. | `pipeline/chat_controls.rs:13`; `botagent/src/vm/incus.rs:30` | [agent] × 2 |
| Chat UI polls `/api/computer/actions/pending` every 3 s regardless of whether Computer Use is active (568 hits recorded in last week's log). | `chat/modules/60_computer_use.js:288` | [agent] + log [run] |
| A fresh install can end up with a default bot that has a DB row but no files on disk, because the template copy hits the path bug above. | `bootstrap_manager/templates.rs`; `bot/mod.rs:166-168` | [agent] × 2 |

### Local Model

| Finding | File:line | Confidence |
|---|---|---|
| Two separate launchers; boot-time launcher skips starting if servers already answer, so `llm-server-gpu-layers` (default `0`) is never applied to a running instance. | `installer/with_base_path.rs:152-196`; `ensure_llama_servers_running.rs` | [agent] × 3 |
| Boot-time launcher restarts via `sh -c pkill llama-server` (not valid on Windows); no readiness wait; no check that the model file exists before launch. | `ensure_llama_servers_running.rs:163-165` | [agent] × 2 |

---

## Phases

### Phase 0 — Known-Good Baseline (1–2 days)

**Objective:** a debug build exists, the stack is clean, and the smoke checklist
passes with a remote LLM endpoint.

**Tasks:**

| # | Task | File | Issue |
|---|---|---|---|
| 0.1 | Restore the last compiling version of `database_verbs.rs` (from `8fb7514b7`) and delete the unreferenced `db_verbs_error.rs`. | `src/core/bot/api_catalog/database_verbs.rs` | — |
| 0.2 | Confirm the debug build succeeds with `cargo check -p botserver --no-default-features --features desktop-local`. | — | — |
| 0.3 | Reuse the provisioned stack when one exists: replace its sidecar binary and product file with the rebuilt ones. Reset the stack only when it cannot be recovered, and back it up first. | stack directory | — |
| 0.4 | Boot with a remote LLM endpoint (the wizard's choice, or the `LLM_URL` / `LLM_KEY` / `LLM_MODEL` environment variables, which override every other source). | `pipeline/enriched_llm.rs:320-334` | — |
| 0.5 | Execute the smoke checklist (see below). Record results as Phase 0 baseline. | — | — |

**Decisions taken:** the work is committed to `main`. The existing stack is kept;
a reset deletes the local PostgreSQL data, Vault and Zitadel state and needs the
owner's explicit approval.

**Exit criteria:** smoke checklist passes; build is green; baseline is recorded.

---

### Phase 1 — Windows Foundations (~2 weeks)

**Objective:** the server starts cleanly on Windows, services stop when the
application closes, and no `/tmp` or Unix-only shell assumption reaches the
Windows code path.

**Tasks:**

| # | Task | File | Issues |
|---|---|---|---|
| 1.1 | Strip `\\?\` verbatim prefix when building the work root; consolidate `get_work_path` / `get_stack_path` to one function returning `PathBuf`. | `init_secrets_manager.rs:255`; `work_path.rs`; `botbasic_core/src/utils/` | — |
| 1.2 | Normalise `.exe` suffix and fold case in the command allowlist; add `taskkill`; pass through `SystemDrive`, `ProgramData`, `TEMP`, `APPDATA`, `COMSPEC`, `PATHEXT`. | `command_guard.rs:13-125` | — |
| 1.3 | Kill only processes whose executable lives inside the stack directory (never by image name machine-wide). Later: assign services to a Windows Job Object so they stop with the server, and switch liveness to a PID-based check. | `bootstrap_utils/procs.rs` | #2379 |
| 1.4 | Replace `ss -tln` probe with a TCP connect to `127.0.0.1:<port>`; replace `localhost` with `127.0.0.1` in all health probes. | `bootstrap_utils/health.rs:38-73` | — |
| 1.5 | Gate cert-generation and Vault-init shell-outs behind `#[cfg(not(windows))]`; provide a Windows path using `cmd.exe` equivalents or native Rust. | `certs_utils.rs:164`; `initialize_vault.rs`; `bootstrap/utils.rs:29` | #2381 |
| 1.6 | Replace the host-affecting `/tmp` hardcodes with `std::env::temp_dir()` and delete the per-run debug dumps; the remaining sites (137 in total, many of them inside the agent VM) follow in batches. | `basic/mod.rs:112-198`; `load_template_from_directory.rs:198`; `boteditor/src/handlers.rs:97` | #2380 |
| 1.7 | Load `.product.desktop-local` from the stack directory first, then the executable's directory, so an upgrade replaces the stale copy. | `product/load.rs:104-124`; `sidecar.rs:139-164` | — |
| 1.8 | Bind to `127.0.0.1` when `no_security=false` and `edition=os`; suppress the Windows Firewall prompt. | `listen_impl.rs:30-34` | #2477 |
| 1.9 | Fail the sidecar with a user-visible error when the API does not respond within the timeout; do not reveal the window on a silent timeout. | `sidecar.rs:110-129`; `main.rs:175-196` | — |
| 1.10 | Windows product profile: hide Linux-only Settings panels and apps with no Windows backend. | product file; app registry | #2466, #2479 |
| 1.11 | Add a free-disk check to the wizard (`free_bytes()` returns 0 on Windows today) and refuse to provision below a stated threshold. | `botapp/src/desktop/wizard/system.rs:73-76` | #2442 |
| 1.12 | Serve the suite's static files from their own rate-limit quota; `/api/*` and `/ws` keep the API quota. | `botsecurity-auth/src/rate_limiter_static.rs`; `rate_limiter.rs` | — |
| 1.13 | Seed Vault defaults with `-cas=0` so existing secrets are never replaced; set `VAULT_CACERT` only when the CA file exists. | `installer/seed_vault_defaults.rs`; `installer/initialize_vault_local.rs` | — |
| 1.14 | Ask Vault for the message encryption key once more before falling back to an ephemeral key. | `botsecurity-crypto/src/encryption/master_key_lock.rs` | — |

**Exit criteria:** `cargo check` passes; server starts and stops cleanly in a
test run; no `/tmp` paths appear in logs on Windows; sidecar error is visible
when the server fails to start.

---

### Phase 2 — Chat Correctness (1–2 weeks)

**Objective:** the user sends a message, tokens appear incrementally, Stop works
immediately, and no dead state blocks the next message.

**Tasks:**

| # | Task | File | Issues |
|---|---|---|---|
| 2.1 | Implement true streaming: forward SSE tokens to the WebSocket as they arrive from the HTTP body; do not buffer the full response. | `openai_stream.rs:142,239-271` | — |
| 2.2 | Abort the detached generation task on cancel or timeout; release the one-permit gate; mark the run Failed not Succeeded. | `pipeline/llm/attempt.rs:76`; `retry.rs:103-106` | — |
| 2.3 | Surface the real failure cause in the chat UI; do not save an empty assistant row after exhausted retries. | `pipeline/exec.rs:500-518`; `pipeline/llm/stream.rs:272` | — |
| 2.4 | Default the per-turn Agent/VM switch to off on Windows, where the Incus backend cannot exist; stop the unconditional 3 s Computer Use poll. | `pipeline/chat_controls.rs:13`; `60_computer_use.js:288` | — |
| 2.5 | Ensure the default bot has `start.bas` and `PROMPT.md` on disk at first boot without requiring MinIO to be reachable. | `bootstrap_manager/templates.rs`; `bot/mod.rs:166-168` | — |
| 2.6 | Surface script compile errors in the chat bubble instead of silently logging them. | `basic/mod.rs:153-156`; `pipeline/start_bas.rs:134` | — |
| 2.7 | Treat the seeded placeholder key `none` as "no key" when building the `Authorization` header. | `botllm/src/retry.rs`; `openai_client.rs`; `openai_stream.rs`; `openai_stream_generate.rs` | — |
| 2.8 | Use a read timeout on the OpenAI-compatible client; a total timeout of 180 s cut long streamed replies. | `botllm/src/openai_client.rs` | — |

**Exit criteria:** tokens appear in the UI before the full reply is complete;
pressing Stop unblocks the next message within 2 s; the smoke checklist
(including the negative cases) passes.

---

### Phase 3 — Local Model on the GPU (~1 week)

**Objective:** the bundled model runs on the GPU; the user can change the
endpoint and key without touching configuration files.

**Tasks:**

| # | Task | File | Issues |
|---|---|---|---|
| 3.1 | Merge the two launchers into one; use Windows-native process kill instead of `sh -c pkill`; wait for `/health` before marking the server ready; verify the model file exists before launching. | `ensure_llama_servers_running.rs`; `with_base_path.rs:152-196` | — |
| 3.2 | Apply the `llm-server-gpu-layers` configuration value at every launch (not just the first). | `ensure_llama_servers_running.rs`; `extract_base_url.rs:60-257` | — |
| 3.3 | Replace the 1.5B Q3 model with one that fits within 8 GB VRAM and supports reliable tool calling; update `3rdparty.toml` and the checksum. | `botserver/3rdparty.toml:275-285` | — |
| 3.4 | Unify the default embedding model and dimension across all three code paths. | `llm_setup.rs:25`; `ensure_llama_servers_running.rs:102`; `botqdrant/src/embedding.rs` | — |
| 3.5 | Add a Settings panel for LLM endpoint, key and model; write to the bot configuration store (not to Vault directly from the UI). | Settings app | — |

**Exit criteria:** `llm-server-gpu-layers` is nonzero in the launcher call;
`nvidia-smi` shows VRAM usage while the model is running; the smoke checklist
passes with the local model.

---

### Phase 4 — Lock It In (ongoing)

**Objective:** per-PR automated gate, security hardening, Drive, and the first
ten UAT scenarios.

**Tasks:**

| # | Task | File | Issues |
|---|---|---|---|
| 4.1 | Add a WebSocket chat smoke test to `windows-testbuild.yml`; block merges when it fails. | `.github/workflows/windows-testbuild.yml` | #2382 |
| 4.2 | Publish a readiness contract (which endpoints and DB tables must exist before the first chat message). | `bootstrap_utils/health.rs`; docs | #2385 |
| 4.3 | Add unit tests for `process_message_internal`, `stream_llm_response`, `stream_attempt` and the HTTP/retry loop. | `bottest/tests/` | — |
| 4.4 | Tauri file commands: scope allowed paths to the stack and user data directories. | Tauri capabilities config | #2497 |
| 4.5 | Empty checksum must not pass as verified; fix the eight affected components. | package manager checksum code | #2506 |
| 4.6 | Move the Vault root token and admin password out of plaintext files in the stack. | stack layout | #2388, #2498 |
| 4.7 | Drive and knowledge-base load: verify bot files arrive in MinIO before the first chat; pass 10 UAT scenarios from #2031. | drive monitor; chat pipeline | #2031 |

---

## Smoke Checklist

Run each step as a separate single-action Chrome CDP script. Record a screenshot
at the end of each positive case.

### Positive Cases

| Step | Action | Pass criterion |
|---|---|---|
| 1 Boot | Launch the application; wait for the window to appear. | Window visible; no error dialog. |
| 2 Sign in | Enter credentials from the first-run wizard. | Dashboard or chat window loads. |
| 3 Send message | Type "Hello" in the chat input; press Enter. | The input clears; a "typing" indicator appears. |
| 4 Streamed reply | Wait for the first token. | Text appears before the reply is complete (not all at once). |
| 5 Full reply | Wait for completion. | Reply ends cleanly; no empty bubble; no Rust debug text visible. |
| 6 Restart | Close and reopen the application. | Previous conversation is visible in the history. |
| 7 Drive | Open Drive from the launcher. | File list loads; no "bucket not found" error in logs. |

### Negative Cases

| Step | Scenario | Pass criterion |
|---|---|---|
| N1 LLM unreachable | Set `LLM_URL` to a closed port; send a message. | A user-visible error message appears in the chat bubble, not a blank screen or spinner that runs forever. |
| N2 Stop | Click Stop during a long reply. | Typing stops within 2 s; the next message field is active immediately. |
| N3 Second message | Send a second message immediately after the first reply. | Second reply starts without requiring a page reload. |

---

## Issue Map

### Windows Port (9) — all still present

#2379 (process cleanup cannot match or kill a component), #2380 (`/tmp`
paths), #2381 (Unix shell-outs), #2382 (Windows CI gate), #2388 (admin
password in plaintext), #2442 (free-disk check), #2466 (Linux-only Settings
panel), #2479 (launcher apps with no backend), #2480 (licence register for
bundled binaries).

### Chat (21)

#2284 (dead chat after one missing bot), #2285 (second chat window never
boots), #2286 (deep-link prompt re-sent on reconnect), #2287, #2301 (silent
failure paths), #2305 (oversized chat modules), #2307, #2308, #2342 (composer
commands and status strip), #2343 (per-message actions), #2344 (conversation
title, clear and export), #2350, #2353, #2360, #2361, #2362, #2363, #2401,
#2495, #2516, #2542.

### Boot / Enablement (20)

#2281, #2282, #2283, #2334, #2364, #2365, #2367, #2371, #2385 (readiness
contract), #2390, #2414, #2441, #2467, #2506 (empty checksum), #2532,
#2541, #2543, #2544, #2546, #2547.

### Desktop Security (18)

#2347, #2348, #2351, #2366, #2368, #2370, #2373–#2378, #2383, #2384,
#2446 (cosmetic lock screen), #2477 (bind address), #2497 (Tauri file
commands), #2498 (Vault token plaintext).

### Reported Fixed on `main` — Can Close After Browser Verification

#2281, #2283, #2284, #2285, #2286, #2334, #2348, #2351, #2353, #2360,
#2361, #2364, #2365, #2368. Partly fixed: #2282, #2347, #2350, #2363,
#2385, #2467.

### Defects Found in This Study — No Upstream Issue Yet

| Defect | File |
|---|---|
| `\\?\` verbatim path in work root | `init_secrets_manager.rs:255` |
| Missing `taskkill` from allowlist | `command_guard.rs` |
| No Windows Job Objects for child services | `bootstrap_utils/procs.rs` |
| Non-streaming HTTP body buffering before SSE parse | `openai_stream.rs:142` |
| Double model launcher; GPU layers never applied at runtime | `ensure_llama_servers_running.rs` |
| Inner retry failures are pushed into the same channel as the response, so a request that fails once and then succeeds is reported as an error | `openai_stream.rs:164,185` |
| Stale stack `3rdparty.toml` shadows embedded pins | `package_manager/cache.rs:84-102` |
| The desktop session depends on one cached token: the wizard removes the administrator password from `admin-credentials.json` after the first sign-in, so a stack whose `desktop-session.json` is lost or rejected has no way to sign in | `botapp/src/desktop/wizard/session.rs`; `botcoredirectory/src/auth_routes/login.rs` |
| `no_security=true` no longer gives a usable suite: the shell redirects to the sign-in page because the API still answers `401` | `desktop-inline/04_auth_fullscreen.js`; server auth middleware |
| `/api/files/recent` answers `500` and `/manifest.webmanifest` and `/api/i18n/preference` answer `401` for a signed-in desktop session | route handlers |
| About two thousand "Translation key not found" warnings are written on every page load | `translation_parser` |
| Bot import resolves `/opt/gbo/data` to `C:\opt\gbo\data` on Windows | `git_bot_monitor/import.rs` |
| The storage alias step runs `SafeCommand::new("")` when no `mc` binary is resolved, and the marketplace seed then fails with an unknown access key | `feature_routers/make_saas_router.rs`; `botmarketplace/src/blobstore.rs` |
| The splash page polls `api/auth` on the application's asset origin every three seconds | `botui/ui/splash.html` |

---

## Risks and Open Questions

These can only be answered by running the rebuilt application on Windows.

| Question | Why it matters |
|---|---|
| Does a rebuilt binary boot against an existing stack, and does Zitadel provisioning complete on a new one? | Migrations and identity state move with the code; a stack provisioned by an older build may not match. |
| Actual cold-boot time. | Theoretical worst case from retry counts in the code exceeds 240 s; the Windows Firewall prompt may add further delay. |
| Is the one-permit LLM gate held for the full duration of an orphaned generation? | If yes, the user must wait for the timeout (up to 180 s) after pressing Stop before the next message goes through. |
| Does sign-in survive closing and reopening the app? | Tokens are currently stored in `sessionStorage` only; clearing the session on close would force re-authentication on every boot. |
| Does `TALK` from a `USE TOOL` script reach the browser? | The channel registration in `ws/session.rs:70` works for the first tab; dropping on a second tab or full channel is a known risk. |
| Does any child process still start without `SystemDrive` in its environment? | A literal `%SystemDrive%` directory appears in the working directory when it does. Task 1.2 passes the variable through `SafeCommand`; other spawn sites have not been audited. |

---

## Status Table

"Done" means implemented and exercised in the browser test of 2026-11-14;
"Implemented" means the code is in place and compiles but the behaviour was not
exercised (for example Stop during a reply, or a turn timeout).

| Task | Workstream | Issue | Status |
|---|---|---|---|
| 0.1 Restore `database_verbs.rs` compile | — | — | Done |
| 0.2 Confirm desktop-local check is green | — | — | Done |
| 0.3 Reuse the provisioned stack | — | — | Done |
| 0.4 Boot with a remote LLM | — | — | Done |
| 0.5 Record Phase 0 baseline | — | — | Done (this page) |
| 1.1 Strip `\\?\` prefix from the work path | B | — | Done |
| 1.1 Consolidate the path helpers into one module | B | — | Planned |
| 1.2 Normalise allowlist; add `taskkill`; pass Windows environment | A | — | Done |
| 1.3 Stack-scoped process kill | A | #2379 | Implemented |
| 1.3 Job Objects; PID-based liveness | A | #2379 | Planned |
| 1.4 TCP probe on Windows | A | — | Done |
| 1.5 Gate Unix shell-outs | B | #2381 | Planned |
| 1.6 Host-affecting `/tmp` sites and debug dumps | B | #2380 | Done |
| 1.6 Remaining `/tmp` sites | B | #2380 | Planned |
| 1.7 Product-file search order and refresh | B | — | Done |
| 1.8 Bind to loopback | A | #2477 | Planned |
| 1.9 Sidecar error on timeout | B | — | Planned |
| 1.10 Windows product profile | B | #2466, #2479 | Planned |
| 1.11 Free-disk check in wizard | B | #2442 | Planned |
| 1.12 Static files outside the API rate limit | A | — | Done |
| 1.13 Vault seeding: create only; CA file guard | A | — | Done |
| 1.14 Encryption key re-read before the ephemeral fallback | A | — | Done |
| 2.1 True SSE streaming | C | — | Done |
| 2.1 Retry failure no longer reported after a successful retry | C | — | Implemented |
| 2.2 Abort generation on cancel; release gate | C | — | Implemented |
| 2.3 Turn timeout reported as a failed run | C | — | Implemented |
| 2.3 No empty assistant row; real cause shown to the user | C | — | Planned |
| 2.4 Agent switch off on Windows; Computer Use poll skipped | C | — | Done |
| 2.5 Default bot files at first boot without MinIO | B | — | Planned |
| 2.6 Script compile errors in chat | C | — | Planned |
| 2.7 Placeholder API key not sent as a bearer token | C | — | Done |
| 2.8 Read timeout instead of a 180 s total timeout | C | — | Implemented |
| 3.1 Windows restart of the stack's own `llama-server`; model-file check | C | — | Implemented |
| 3.1 Single launcher; readiness wait | C | — | Planned |
| 3.2 Apply `gpu-layers` on every launch | C | — | Planned |
| 3.3 Replace 1.5B model | C | — | Planned |
| 3.4 Unify embedding model | C | — | Planned |
| 3.5 LLM settings panel | C | — | Planned |
| 4.1 WebSocket smoke test in CI | — | #2382 | Planned |
| 4.2 Readiness contract | A | #2385 | Planned |
| 4.3 Chat pipeline unit tests | C | — | Planned |
| 4.4 Tauri file-command path scoping | — | #2497 | Planned |
| 4.5 Non-empty checksum enforcement | — | #2506 | Planned |
| 4.6 Vault token / admin password hardening | — | #2388, #2498 | Planned |
| 4.7 Drive and 10 UAT scenarios | — | #2031 | Planned |
