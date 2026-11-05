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
