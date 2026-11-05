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
