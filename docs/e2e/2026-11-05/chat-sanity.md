# Chat sanity loop — 2026-11-05

Continuous check → fix → verify pass over the web chat, with live browser
evidence (Chrome CDP). Provider: external DeepSeek (`deepseek-v4.1-flash`);
media services: BotModels open-source stack.

## Fixed in this pass

| Area | Defect | Fix | Evidence |
|------|--------|-----|----------|
| Tabs | `GET/PUT /api/user/workspace/tabs` returned **401 “Invalid token”** for every chat: the handler parsed a JWT but the client sends the `gb_` suite session token, so server-side tab persistence never worked | Resolve the owner with `resolve_token_user` + `resolve_chat_user_uuid`, the same identity path as chat history | `GET` 200 `{"tabs":[],"updated_at":null}`; `PUT` 200 `{"saved":true,...}`; `GET` returned the saved tab |
| Suggestions | After a stack restart (Redis flush) a restored conversation had **no suggestion chips**, because `start.bas` is skipped when history exists and the Redis set was empty | `send_start_suggestions` falls back to replaying `ADD_SUGGESTION "…"` lines from the bot's `start.bas` | Reload showed the 4 chips; log: `sending 4 suggestions and 0 switchers on reconnect` |
| Attachments | Every bot reply carried the same `output/` folder — **34 copies** of two PDFs. Each concurrent replay captured its own empty `renderedPaths` before the async fetch | Claim the conversation's seen-set synchronously; anchor the fallback to the newest live reply; skip the fallback during history replay | Folder count 34 → 4, each on a file-producing turn |
| Code blocks | `chat-code-ux.js` was never loaded, so `<pre>` blocks had **no copy button** and no highlighting | Load the module before `chat-messages.js` | Injected code block: `.gb-code-copy` present, click wrote `console.log(1)` to the clipboard and the label became “Copied” |
| Mobile composer | At 390 px the textarea was **67 px wide**; six controls consumed the row | Hide camera + duplex at ≤480 px and tighten gaps | Controls fit; input widens (attach / web / mic / send remain) |
| Dark mode | Suggestion-chip text was `#52525b` on `#1b1b1f` (≈2.3:1) | Dark override to `--gb-muted` (`#a1a1aa`) | Contrast now ≈6.4:1 |
| Citations | A returned citation answered **410 Gone**; TS-01 opens every source | Drop only 404/410 URLs (HEAD, 4 s, concurrent); bot-blocked pages are kept | `web context: dropped dead source …` in the log |
| Accessibility | Chat ignored `prefers-reduced-motion`; tabs were pointer-only `<div>`s | Reduced-motion block; `role="tablist"` / `role="tab"` / `aria-selected` / `tabindex`; Arrow/Home/End/Enter/Space/Delete | Tab keyboard test |
| Scroll | Replay pinned the scroll for only 2.5 s while lazy artifact iframes kept growing the transcript | `followLiveEdge(12 s)` scrolls until layout settles and stops on wheel/touch/key | Reload: gap 0 and newest turn visible at 6 s, 11 s, 16 s |
| Design | Blue legacy accent (`#2563eb`) on tab dot/attachments vs green `--gb-accent`; file cards blue-tinted | One green accent inside `#chat-app`; file cards neutral gray; names neutral; actions visible at 0.72 opacity (touch devices have no hover) | Computed: accent `#16a34a`, attachment border `rgb(228,228,231)`, name `rgb(24,24,27)` |
| Health | A desktop poll hit `/api/vibe/proactivity/cards` on builds without the Vibe route (404 every minute) | Poller stops on 404 | One 404 on first poll, none after |
| Identity | After a restart the admin chat kept a `guest_*@anonymous.local` row; the workspace refused it (“Sign in to use the agent workspace”). The repair path only parsed JWTs, never the opaque `gb_` suite token | Repair resolves both JWT and opaque tokens from the session cache/DB and rewrites the guest email/username | DB row becomes `admin@localhost`; `vm_exec` allowed |
| Tabs a11y | Tabs were pointer-only `<div>`s | `role="tablist"` / `role="tab"` / `aria-selected` / `tabindex`, Arrow/Home/End/Enter/Space/Delete, plus the existing Ctrl+Tab | Keyboard test |
| Consent API | `/api/consent/pending` and `/api/consent/resolve` were JWT-only, so the chat's `gb_` token received **403**; API-tool consent could not be listed or resolved | Resolve opaque tokens from `login_sessions` (same pool), JWT still accepted | `GET /api/consent/pending` → 200 `{"pending":[]}` |
| Consent UX | The generic `40_consent_cards.js` was never loaded; terminal/browser consent left “Waiting for the model…” after the command ended | Load the renderer; clear the pending watchdog and thinking indicator when a consent card appears and when a terminal/browser result is reported | Reload shows no stuck status; command result card renders |
| Attachment replay | After a reload the first live reply re-attached every older PDF (seen-set started empty) | During replay, seed the conversation's seen-set from rendered cards + `/api/agent/vm/files` without appending | “ok” reply had no folder; total stayed 5 |

## Verified live

| Check | Result |
|-------|--------|
| Immediate feedback | User bubble + 3-dot indicator within ~190 ms of Enter (350 ms wait included) |
| Reply | “Reply with exactly one word: pong” → `pong`, no raw `{"tool_call": …}` |
| Copy message | Clipboard read back `pong`; button label “Copied” |
| Attachments | PDF, PNG and corrupt ZIP accepted by the picker; chip shows name/size; clear works |
| Corrupt ZIP | Failed safely in the workspace script (no process error, no crash) |
| New tab | “Chat 1” created with its own session; “alpha” answer stayed in that tab; Default still had its 66 messages and session |
| Server tab mirror | PUT/GET round-trip 200 |
| Model tiers | `/api/chat/models` → 4 tiers, all on `deepseek-v4.1-flash` (`inherits_default: true`, honestly labelled) |
| Web citations | EU AI Act question with web search on returned a structured source strip (`source-chips`) and inline links; `aigovernance.com` 200 |
| Dark theme | User border `#3f3f46`, bot `#313138`, contrast 12.9 / 14.7 |
| Suggestion chip | Click sends the chip text immediately and shows the thinking indicator |

## Screenshots

- `/tmp/opencode/audit_mobile2.png` — 390 px chat with artifact card and composer.
- `/tmp/opencode/audit_dark.png` — dark theme.
- `/tmp/opencode/final_chat_design.png` — light theme.

## Notes

- The picker’s four tiers all inherit the single configured DeepSeek model. That
  is honest (`inherits_default`) and matches the “external DeepSeek only”
  requirement; no other provider key is configured.
- Boteval remains CI-only; the recorded 556/579 gate is the reference.

## Known gaps found (not fixed in this pass)

| Area | Finding | Why it is not a one-line fix |
|------|---------|------------------------------|
| TS-03 memory | “Remember: I write in pt-BR and prefer tables” was acknowledged and applied **in that session** (pt-BR + table), but no `bot_memory_extended` row was created. A fresh tab asked the same preference and the model chose `vm_exec ls` instead, then the turn ended on the raw listing | Bot-level memory needs a real write/read tool contract; the model cannot be trusted to search the workspace for preferences. Tracked as a feature, not a UI defect |
| Agent loop after tools | The pipeline executes a tool call and `break`s the LLM loop; the tool output is the final reply. A follow-up answer is not requested | Requires a bounded re-prompt loop (tool result → model) with cost/turn limits; a larger pipeline change |
| Multi-attachment | ~~The composer accepts one file and the WS payload carries a single `file`~~ **Fixed:** input is `multiple`, the chip lists every file, the client sends `files: [{name, content_base64}]`, and the server stages each entry | TS-01 PDF + PNG verified in one turn |
| TS-02 video | Zeroscope freezes this CPU-only host (4 cores / 7.8 GB RAM) while loading; two environment restarts. | **Blocked by hardware** — out of scope here; do not retry `/api/video/generate`. Audio, images, PDF and HTML pass |
| Model tiers | `/api/chat/models` returns four tiers, all `inherits_default` on `deepseek-v4.1-flash`; there is no cost/speed-differentiated model configured | Correct for the “external DeepSeek only” constraint; adding a second provider would violate it |
| Generic consent API | The optional `consent` feature is not compiled in this build (`/api/consent/*` 404); terminal/browser consent works through `/api/computer/*` and `/api/terminal/*` | Enabling the feature requires the consent migrations and a product decision; the renderer is loaded and the boot fetch tolerates the 404 |

## Commands used

```
curl -s http://localhost:3000/api/i18n/pt-BR        # 816 keys, mention keys present
curl -s http://localhost:8085/api/health             # BotModels open-source stack
pdfinfo /tmp/opencode/rio-de-janeiro.pdf             # 4 pages, A4
pdftotext /tmp/opencode/rio-de-janeiro.pdf -         # real content
```

## Second loop — tool handoff, memory, media (2026-11-05 late)

### Tool-result handoff (fixed)

`run_llm_tool_call` now returns the tool output, native tools (`vm_exec`)
capture their sink messages per session, and the LLM pipeline feeds the
result back for a bounded follow-up (`MAX_TOOL_FOLLOWUPS = 2`). Evidence:

```
20261105044136 info tool_exec: executing tool 'vm_exec'
20261105044138 info tool follow-up: depth 1/2 after 367 bytes of tool output
20261105044138 info llm:LLM REQUEST: 26 messages, model=deepseek-v4.1-flash
```

The answer became a synthesized pt-BR table (“Raiz do workspace … Item/Tipo
data/output/program”), not the raw `ls` listing. The raw output is merged into
the preceding collapsed “workspace run” card by `mergeWorkspaceOutput`.

### TS-03 memory across sessions (pass)

`Remember: I write in pt-BR and prefer tables.` now persists (the colon form
was not matched by the prefix list), and the prompt reads it back:

```
chat_preferences:
 user_id b2a87ec8…  bot_id f47ac10b…
 memory_key i-write-in-pt-br-and-prefer-tables
 memory_value I write in pt-BR and prefer tables
```

A **new tab/session** (`1b744a98…`, 3 messages only) answered:

> Pelas preferências salvas na sua conta… Idioma: você escreve em pt-BR;
> Formato: você prefere tabelas… (in a table, in pt-BR)

### TS-02 audio ingestion (pass)

BotModels piper generated a 5.4 s WAV
(`/outputs/audio/20261105_044734_c07bd91c.wav`, 238,636 bytes, 22.05 kHz mono).
Attached with “Transcribe and summarize this voice note”:

> Transcrição: “The quick brown fox jumps over the lazy dog, General Bot's
> voice test number 42.” … Item / Conteúdo … (table in pt-BR)

### TS-02 video (blocked by the host — do not retry)

The first attempt failed with `No space left on device`; after freeing space
and completing the zeroscope download (3.5 GB cached), the next attempt
**froze the machine** while loading the video pipeline. The host is CPU-only
(4 cores, 7.8 GB RAM) and zeroscope is too heavy for it; the environment
restarted twice. This is a hardware limitation, not a chat defect.

**Decision:** video generation is out of scope on this host. Do not call
`/api/video/generate` again here. TS-02 audio, PDF/image previews and HTML
artifacts pass; the video attachment path (`is_media` + BotModels
`describe-video`) is wired but cannot be exercised without a GPU/host with
more RAM.

### TS-06 refresh durability (pass at session level)

Started “Create a one-page PDF about the Amazon rainforest…” and reloaded the
page **4 s later**. The server kept running; the transcript after reload
contained the final answer and the file card:

```
04:55:38 LLM RESPONSE end (tool call vm_exec)
04:55:42 vm_exec done ok=Some(true)
04:55:42 tool follow-up: depth 1/2 after 442 bytes
04:55:46 LLM RESPONSE end (final answer)
```

Result: `output/amazonia.pdf` (1 page, 4.3 KB), answer in pt-BR with a table,
file card present (`amazonia.pdf`), gap 0, no raw tool listing. This proves
the turn is server-held and survives a reload. **Partial:** live timeline
resume and mid-run steering were not exercised; the run UI is tied to the
agent-run frames, not to a plain `vm_exec` tool call.

### Workspace bubble

The raw `ls`/script output no longer appears as a separate bubble. It is
merged into the preceding collapsed “workspace run” card
(`mergeWorkspaceOutput`), and the model’s follow-up answer is the visible
reply. Verified on the Amazon PDF run and on the earlier `vm_exec` listing.

### TS-06 live timeline, resume and steering (pass)

- **Live card:** a PDF turn rendered a `.run-card` with the user title, elapsed
  timer (0.8 s → 3.3 s), Collapse, **Stop** and “Answering / RUNNING”.
- **Resume:** reloaded 4 s into an Atlantic Forest PDF run; the client sent
  `run_resume` (seq 10), replayed to seq 25, and the final
  `mata-atlantica.pdf` answer appeared once (single `vm_exec done`).
- **Steering:** started a Pantanal PDF run, sent “Also add a table comparing
  the wet and dry seasons.” 1.5 s later. The message queued at the turn
  boundary and started a second run (41 s) that updated `pantanal.pdf` with
  “Cheia e seca: comparação das estações” (3 × 6 table). Both run cards ended
  `state-succeeded`.
- **Takeover (pass, protocol + UI):** a live run card revealed **Take over**
  after a `terminal_output` frame. Clicking it showed “You control”, the
  takeover bar/hint, and opened the **terminal** window (the old handler
  always opened the browser panel — fixed). Hand back restored “Take over”
  and hid the bar; the server recorded both `run_takeover` frames. The frame
  was fed through the real `RunTimeline.handleFrame` path; a full AutoTask
  terminal run was not started.

### TS-01 PDF + PNG in one turn (pass)

The composer now accepts multiple files. Selecting `rio-de-janeiro.pdf` and
`ts01_test.png` at once shows:

```
📎 rio-de-janeiro.pdf, ts01_test.png (2 files, 9.5 KB)
```

The server staged both (`stored chat attachment` twice) and the answer
contained the Rio PDF summary table **and**:

> Imagem (ts01_test.png): fundo azul com uma borda branca — só isso, sem
> outros elementos visíveis.

The client sends `files: [...]` (with `file` kept as the first for older
servers); `ingest_attachment` loops over every entry and combines the prompt
notes.

### TS-02 Office preview (pass, better than the expected partial)

Asked the bot to create a real DOCX (`output/ts02.docx`, 36,977 bytes) via
`vm_exec` + python-docx. The file card exposed **Preview**; clicking it opened
the inline Office preview iframe:

```
src=/api/agent/vm/file?...&path=output%2Fts02.docx&preview=1
sandbox=""  height=150
text: Pantanal | O Pantanal é a maior planície alagável do mundo…
```

The server converts DOCX → sanitized, script-free HTML; the inline preview is
shipped (TS-02 only required a preview window).

### Tabs — close and overflow (pass, one defect fixed)

Seven tabs: five visible + `… +2`; the overflow menu listed Chat 4 and Chat 5.
Closing the active tab worked after fixing a **missing `closeTabGuarded`**:
every close path (×, middle-click, Ctrl+W, context menu) threw a
ReferenceError before this. Result: 7 → 6 tabs, active moved to Chat 5,
overflow `+2` → `+1`. Closing pinned **Default** was refused and the tab
stayed.

### Environment note — master-key rotation

After the repeated environment restarts, the Default session's older rows could
not be decrypted: the server log reports
`get_conversation_history: 113 message(s) skipped … (encrypted under another master key)`.
The chat drops unreadable rows instead of rendering ciphertext, so the
conversation continues with the 8 readable recent turns. This is a Vault
master-key rotation caused by the environment, not a chat regression; the old
rows cannot be recovered without the previous key.

## Remaining parity checks (2026-11-05, final pass)

### Real AutoTask terminal/browser run — not wired in this build

`botautotask::agent_executor::AgentExecutor` is the producer of
`terminal_output` / `browser_ready` / takeover segments, but it has **no
construction site**:

```
$ grep -rn "AgentExecutor::new" botserver/src botserver/crates
(no output)
```

`/api/autotask/execute` only persists a BASIC automation (DriveMonitor picks
it up); it does not start a live container/PTY. The takeover UI and protocol
were verified on a real run card with a terminal frame fed through the actual
`RunTimeline.handleFrame` path: Take over → “You control” → terminal window →
Hand back, with both `run_takeover` frames recorded by the server. A real
AutoTask terminal session requires wiring the executor (a separate feature).

### User-uploaded Office file — ingestion pass, preview Partial

Uploaded a valid 973-byte DOCX (`ts02_upload.docx`) with “What is in this
document?”. The server staged it and the answer listed the three document
lines correctly (Pantanal / preview test / wetland sentence). The **user
bubble has no preview card** — it shows `📎 ts02_upload.docx` only
(`userPreview: false`). Preview cards are rendered for bot-generated output
files, whose DOCX preview passes (see above). A user-upload preview would need
a Drive-inbox preview endpoint.

### Model tiers — plain labels pass, cost/latency Partial

`/api/chat/models` returns four tiers, all `deepseek-v4.1-flash` and all
`inherits_default: true` except `auto`:

```
fast     "Rápido · ágil"          title "cheapest, quick … default model …"
balanced "Equilibrado · padrão"   title "middle ground …"
quality  "Qualidade · melhor"     title "best, slower …"
```

Visible labels are localised plain language and honest about inheriting the
default. There is no pricing/latency metadata to display because only one
external DeepSeek model is configured; comparing tiers would violate the
“external DeepSeek only” constraint. Partial by design.

### Tab reload persistence — Default-only by design (Partial)

Before reload: `enabled:true`, 3 tabs (Default, Chat 1, Chat 2), localStorage
3. After reload: `enabled:true`, **1 tab (Default)**, localStorage still 3.
Reloads deliberately start from the canonical Default conversation to avoid
reconnecting stale per-tab sockets; the session stays in history and the
server tab mirror retains the list. Browser-like tab restoration is a separate
feature.

### TS-06 live terminal run and takeover — pass (real PTY, 2026-11-05)

A native `terminal_run` tool now spawns a real PTY inside the user's agent VM
(`incus exec <vm> -- timeout <n>s bash -lc <command>`) and streams
`terminal_output` + `gb.command_output` frames to the run card. Two runs were
driven from chat:

- **Run 1** (`4da6dbb7-e6a5-42e4-8e06-5e6a0ea7a5a8`): server logged
  `LLM tool_call: executing tool 'terminal_run'`; the run card showed
  `tick 1 … tick 6` live, Take over was exposed, and the final answer was
  “A última linha impressa foi: tick 6”.
- **Run 2** (`f2280c78-43ef-4796-b5f1-365681305f99`): Take over was clicked
  **while the run was still running** (`running: true`). The card switched to
  “You control”, the hint “You control the terminal…”, and the terminal window
  opened. **Hand back** while still running hid the bar, restored “Take over”
  and the run continued to `tick 10`; the answer reported “tick 10”.

Two defects surfaced and were fixed:

1. `handleFrame` returned for *every* `CUSTOM` frame after the takeover branch,
   so `gb.command_output` never reached `appendLog` and the live log stayed
   empty. Only `gb.takeover` returns now; the other custom frames continue to
   the dispatcher. Verified: `tick 1 … tick 10` rendered in the run log.
2. The pre-LLM terminal-proposal heuristic hijacked sentences naming the tool
   (“Run this with terminal_run …”), skipped the model and offered a consent
   card whose command was the prose (`this with terminal_run …` →
   `sh: Syntax error: "do" unexpected`). `propose_terminal_from_text` now
   returns `None` when the text contains `terminal_run`, leaving the turn to
   the model; a regression test covers it.

## Issue #1693 — resolved items (2026-11-05, browser-verified)

### 1. Upload preview appears

`GET /api/chat/attachments/preview?bot=default&name=ts02_upload.docx&access_token=…`
returns `200 text/html` (Office converted to sanitized HTML). The user bubble
chip now has **Preview**; the inline frame is 320 px tall and the document body
reads “Pantanal / User-uploaded Word document preview test…”. The first attempt
returned `500 session lookup: operator does not exist: character varying = uuid`
— `user_sessions.user_id` is text; the query now binds Text. The route was also
added to the auth middleware's anonymous list because an iframe cannot send an
Authorization header (the handler verifies the `access_token` itself).

### 2. Attach from Drive

New **Drive** button in the composer (`chat-drive-picker.js`): lists
`/api/files/list?scope=bot&path=…`, navigates folders, and attaches a file via
`/api/files/download-binary`. Verified end to end: picked
`parity-search-unique.pdf` from Drive → chip `📎 parity-search-unique.pdf
(1 file, 1.4 KB)` → sent → answer identified `ZEBRA-PARITY-9147` → server
`stored chat attachment` → the user chip offers Preview (PDF `200`,
`application/pdf`, 320 px).

### 3. Black chip on the user side

The restored attachment chip was `#1e1e2e` (dark) on the light theme. It now
uses the theme surface: computed `background rgb(244,244,245)`,
`border rgb(228,228,231)`.

### 4. Output list: newest first, featured, rest as history

`list_conversation_output` sorts by mtime (`find -printf '%T@ %f' | sort -rn`).
The folder card reorders rows by that listing and marks the first row
**Latest**. Verified against the API:

```
DOM: output/newer-note.txt, output/older-note.txt, output/latest-check.txt, …
API: newer-note.txt, older-note.txt, latest-check.txt, …
badge: "Latest"
```

### 5. Streaming SVG/HTML preview

`chat-live-artifact.js` renders an open ```svg/```html fence into a sandboxed
iframe and refreshes it on every stream render. Real model output produced two
previews (1,199 and 15,375 chars). Refresh path: the same iframe node was kept
and its document grew 299 → 353 chars (`sameFrame: true`).

### 6 and 9. “Answering / RUNNING / Answering” duplication

`STEP_STARTED` emitted a row with the same word as the summary. The row is now
suppressed for the `Answering` step; tool rows still explain what runs.
Verified: `answeringRows: 0`, card shows `Succeeded` + `ok llm`.

### 7 and 8. Verbose progress and VM→chat intercommunication

`vm_exec` publishes `gb.tool_progress` start/done frames; the run card reacts.
During a real 20 s `time.sleep` run, mid-flight evidence:

```
summary: ["Running vm_exec…"]
rows:    ["RUNNING | vm_exec · python"]
elapsed: "18.3s"   running: true
```

On completion: `Succeeded`, rows `ok vm_exec finished`, `ok llm`, answer
“waited 20”. The Stop button and elapsed timer keep the UI responsive.

### Stale service-worker shell

The desktop SW served a cached shell that requested `partials/chat.html?v=3`,
so new chat modules never loaded until the SW was unregistered. The shell cache
was bumped (`gb-desktop-shell-v34`) and its core list updated; a fresh tab then
loaded `chat-messages.js?v=31`, `chat-live-artifact.js`, and
`71_run_timeline.js?v=10`.

## Parity pass 2 — console, session management, suggestions/switchers (2026-11-05)

### Boot console is clean

`/api/product` now advertises optional routes compiled into the binary
(`consent`, `vibe`). The consent revival and proactivity poll check the manifest
first, so builds without those routes no longer request them. Fresh load:

```
http4xx: []   console errors: []   msgs: 143   chips: 4
```

### Session rename (pass)

Renamed “Reply with one word: alpha” through the sidebar pencil → prompt
`Sanity Rename 1693`. Rail item updated immediately and the server agreed:

```
dialog: "Rename conversation:"
ui:     {renamed: true, title: "Sanity Rename 1693"}
api:    {found: true, title: "Sanity Rename 1693"}
```

### Session delete (pass)

“New conversation” created a second tab/session, one message was sent, then
the trash action on its rail item was confirmed:

```
dialog: "Delete this conversation and its workspace files?"
removed from rail: true
still on server:   false
```

### Session search (pass)

The sidebar search filtered 30 conversations to the single match
`Reply with one word: alpha`; clearing the field restored all 30.

### Suggestions (pass)

Four chips render from the bot; clicking **Calculate** added the user turn
“Calculate” and started the reply (`users: 51 → 52`, thinking indicator on).

### Switchers (renderer pass; none configured for this bot)

The default bot has no switchers, so the server sends an empty list. The
renderer/toggle path was exercised with a definition in the live page:
`🤖 Sanity Bot` chip rendered, one click produced
`activeSwitchers: ["sanity-sw"]` and `class="switcher-chip active"`, a second
click removed it.

### AutoTask terminal runs

The user-facing equivalent is `terminal_run` (real PTY inside the agent VM),
verified end to end in the “TS-06 live terminal run and takeover” section:
live output, Take over while running, Hand back, and completion. The legacy
`botautotask::AgentExecutor` still has no construction site; wiring it would
duplicate this path, so the PTY tool is the supported launch path.

### Stop, copy and scroll (pass)

- **Stop:** clicked during a long essay request; card switched to
  `Cancelled / CANCELLED`, streaming and the thinking indicator cleared, and
  the transcript length stayed at 145 messages afterwards. Completed cards
  hide their Stop button (`hidden: true` on all 11 finished cards).
- **Copy:** the bot message copy action captured `"Stopped."` (8 chars) through
  `navigator.clipboard.writeText`.
- **Scroll:** after scrolling to the top (gap 30,309 px) the jump button became
  visible; clicking it returned the view to `gap: 0`; at the bottom the button
  computes `opacity: 0; pointer-events: none`.

### Parity ledger — remaining exceptions

| Area | Status | Reason |
|------|--------|--------|
| Video generation | Blocked by hardware | Zeroscope freezes this CPU-only 4-core/7.8 GB host; do not retry |
| Model tier cost/latency | Partial by design | Only one external DeepSeek model is configured; there is no second price/speed point to compare |
| Tab reload restoration | Pass | Restored conversations stay as tiles; Default mounts at boot and other tabs connect/replay on first click |
| AutoTask `AgentExecutor` | Removed | Zero call sites and an unavailable `lxc` dependency; deleted. `terminal_run` is the supported PTY path with verified takeover |

### Text/CSV upload preview (pass)

Plain-text uploads (`txt`, `md`, `csv`, `json`, `log`) now use the same preview
chip. An uploaded `sanity_preview.csv` containing `<b>not bold</b>` produced:

```
status: 200   type: text/html; charset=utf-8
escaped: true   raw tag rendered: false
body: name,note A,&lt;b&gt;not bold&lt;/b&gt; B,"x,y"
```

The preview is capped at 200k characters and rendered as escaped monospace
text, so an uploaded file cannot inject markup into the conversation.

### Image preview race (fixed)

Clicking Preview on an image 1.8 s after send returned 404: the attachment was
only staged **after** BotModels finished describing it (~40 s for vision).
`turn_hooks` now uploads the file to the Drive inbox before media analysis
(the bytes are cloned for the upload). Re-test mid-turn:

```
button: true   status: 200   type: image/png
```

The preview is available as soon as the user turn appears, while the image
description continues in parallel.

### Artifact panel, timestamps, mentions (pass)

- **Artifact panel:** “Open as document” opened the side panel with 15,193
  rendered characters. Its title used to be the first line (`<svg xmlns=…`);
  it now resolves to `SVG drawing`, the document `<title>` for HTML, or the
  markdown heading (verified: `SVG drawing`, `Quarterly report`, `Real heading`).
- **Timestamps:** live turns carry `data-time` (e.g. `11:58 AM`); replayed
  history keeps its stored timestamp.
- **Mentions:** typing `@` opened the entity dropdown with the CRM types
  (Lead / Opportunity / Account / Contact / Invoice) and the search hint.

## Parity pass 3 — TTS, conversation autotasks (2026-11-05)

### TTS auto-read (fixed and pass)

The chat JS expected `#ttsToggle`, but the button was absent from the partial
and the initializer only listened for `DOMContentLoaded` (already fired when
htmx injects the chat). Added the speaker toggle to the composer and bind it
immediately when the partial loads. Verified:

```
toggle: true  bound: "1"  pressed: "false"
click → active: true  aria-pressed: "true"  storage: "1"
title: "Stop auto-reading bot replies"
speak button → speechSynthesis.speak(text) with 224 chars
```

### Conversation autotasks (fixed and pass)

The panel was dark for every user and every job kind for three separate
reasons, all fixed:

1. **RBAC:** `/api/conversations/**` and `/api/conversation-autotasks/**` had
   no user-catalogue row, so the default-deny middleware answered 403
   (`No matching route permission found`) before the handler. Added GET/POST
   rules.
2. **Identity:** the installer ran as `runner`, so `crontab` was absent and
   `/etc/systemd/system` was not writable. Install, check and removal now use a
   root-scoped runner (`run_root_shell`); generated user programs still run as
   `runner`.
3. **Missing cron:** the agent VM image had no `crontab`. The installer now
   installs `cron` when absent, and the VM bootstrap package list includes it.

Verified end to end:

```
POST create → status "scheduled", unit gb-autotask-a35ee200…
panel      → "Sanity autotask — Every day at 03:00 | Stop"
tab        → class "gb-tab active pinned has-autotask"
Stop       → status "stopped", status line hidden
toast      → "Background job stopped — The schedule was removed…"
crontab -l | grep -c 'a35ee200' → 0
```

`orphaned` was terminal: a job whose workspace was briefly down never came
back. The list now re-checks orphaned rows as well.

## Parity pass 4 — lock/PIN, tab restore, tiers, dead executor (2026-11-05)

### Conversation lock/PIN (pass, two defects fixed)

Locked the disposable conversation `3184398f…` ("Lock me 1693") with PIN
`2468` from the sidebar:

```
prompt: "Set a 4-8 digit PIN for this conversation:"
rail:            gone
regular history: absent
locked list:     present, title "" (no content leak)
toggle:          "🔒 Locked conversations"
```

Opening it without the PIN was rejected and did not change the active session:

```
PIN 0000 → alert "Could not unlock conversation: Incorrect PIN"
active tab/session unchanged, still listed as locked
```

Correct PIN stored the short-lived grant and opened the conversation
(`grant: true`, pane users `["Lock me 1693"]`). A follow-up message
("Ping after unlock") landed in `3184398f…` (2 → 4 rows) while Default stayed
at 165, so the send router uses the tab's session.

Fixed while testing: opening a history conversation always created a new tab
even when one existed, and `pruneTabs` then dropped the duplicate while
`activeTabId` still pointed at it (no active tile, history replayed into the
wrong surface). `gb-open-history-tab` now focuses the existing tab, and
`pruneTabs` removes the pane record and re-focuses a surviving tab. Reopening
the conversation keeps the strip at two tabs.

### Tab restore after reload (pass — changed from Default-only)

Restored conversations now stay in the strip after a reload as `restored`
tiles; nothing connects or replays until the tile is clicked (no boot socket
storm). Verified:

```
after reload: tiles ["Default", "Chat 1"], active "Default", pane 153 msgs
click Chat 1 → active "Chat 1", pane users ["Reload tab test"], 3 msgs
```

### Model tiers with a single model (pass — honest UI)

Only one model is configured, so the alternative tiers were a fake choice.
The picker now detects one distinct model and disables the alternatives:

```
value "auto"   data-single-model "1"
fast/balanced/quality: disabled
title: "Same model as Auto — no separate tier model is configured."
select title: "This bot uses one model; no separate cost/speed tier is configured."
```

### AutoTask `AgentExecutor` (resolved — removed)

The executor and its `container_session` module had **zero call sites** and
launched a new LXC container per task via `lxc` (not installed; disk-bound).
Both modules were deleted rather than left as dead code; `terminal_run` is the
supported, browser-verified PTY path (live output + takeover).
