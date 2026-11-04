# Issue #1692 — Human review 3

Live verification of the four requests plus the layout/tab regressions found
while testing them.

## Fixed regressions

### Blank page / half-height chat (composer above tabs)

Two `<main id="messages">` elements could exist after a fragment re-injection or
an interrupted tab switch. Both received `flex: 1`, splitting the window 50/50:
the first was empty, the footer collapsed under the tab strip, and the page
looked blank.

`mountActive` now keeps exactly one stock `#messages`, removes orphan mains and
restores the id/visibility of the parked stock surface. The stock session is
reloaded when switching back to Default if its DOM is empty.

Measured after the fix: one `main#messages`, height **460 px** of a 683 px
window, footer at the bottom, scroll gap **0**.

### Tab overflow

The strip keeps five tabs (three on narrow windows) and replaces the rest with
a **… +N** button. The menu lists hidden conversations, keeps the active tab
visible, and never scrolls the strip horizontally.

Verified: 5 visible + `+1`, `scrollWidth === clientWidth`, menu shows the hidden
conversation and focuses it.

### Scroll follows the newest message

The previous code measured “near bottom” after appending, so a long reply
always looked like the user had scrolled up. It now captures the position
before the append, pins the live edge for async layout (inline iframes,
suggestions), and uses a ResizeObserver/MutationObserver while the reader is
near the bottom.

Verified: `gap = -1` after reload and after bot replies; last message fully
visible.

### Output mixing / duplicate replies

- One socket per tab: `connectTab` closes the previous socket and stamps a
  generation; stale handlers ignore frames.
- The stock socket no longer paints while a parallel tab is active.
- Boot always selects `tab-default` (not whichever tab is first in the stored
  order); only that tab owns the stock surface.
- Duplicate completion frames and duplicate persisted bot turns are deduped.
- Default history is restored when returning to the tab.

## Infographic on the chat surface

- Raw HTML documents and leaked `show_app` tool-call JSON are detected and
  rendered in a **sandboxed iframe** (`allow-scripts allow-forms allow-modals`,
  no same-origin) instead of being printed into the bubble.
- The artifact spans the conversation width and has Expand/Collapse.
- Parallel tabs now handle CUSTOM frames (`gb.app`, `gb.surface`,
  `gb.question`, `gb.surface_update`) and target their own pane.
- Plan JSON (`__ui_plan__` / `ui_plan`) is stripped from user-visible text.

Verified: “Brasil em Números — Infográfico Interativo” renders at 1164 px wide
inside the chat, 32-bar/region buttons are interactive, no `__tool_call__` or
`<style>` text remains.

## Provider failure UX

The external endpoint intermittently returns 503 (`Endpoint is unavailable`)
and sometimes stalls after a 200. The stream retries are now capped (3 attempts,
max 8 s waits) and the composer shows “Waiting for the model…” after 12 s; the
pipeline's final fallback message offers a retry. This prevents minutes of
silence.

## Requests verified live (2026-11-04, external DeepSeek provider)

| Request | Result |
|---------|--------|
| Interactive Brazil infographic | inline sandboxed artifact “Brasil em Números — Infográfico Interativo”, 1164 px wide, interactive |
| 4-page PDF about Rio de Janeiro | `output/rio-de-janeiro.pdf`, **4 A4 pages**, 9,595 bytes, verified with `pdfinfo`/`pdftotext` |
| “Draw me an animation” | `gb.app` card “Flow Field — Animation”, 360 px iframe, two frames differ across 1276×464 px (running) |
| “Give me a complex plan that you can ask me” | `propose_plan` surface: 6 steps, reorder/remove controls, **Approve plan** |

### PDF request

The agent ran Python in the workspace (`reportlab`), produced
`output/rio-de-janeiro.pdf` and returned the file card with Preview / Download /
Save. `pdfinfo` reports 4 pages at A4 (595×842 pt); `pdftotext` confirms real
content (founding, geography, UNESCO landscape, numbers, itinerary).

The inline preview was a blank broken-file box because the global security
headers forced `X-Frame-Options: DENY` on `/api/agent/vm/file`. Handler-scoped
headers now win (`entry().or_insert`) and the raw/preview responses send
`SAMEORIGIN`, so PDFs and images preview in the chat iframe. CSP still pins
`frame-ancestors 'self'`.

### Animation request

`show_app` rendered a self-contained canvas animation. The app card header shows
title, status, Reload / Expand / Stop. The status used to stay “Starting…”
forever because generated apps do not implement the JSON-RPC handshake; it now
settles to “Running” after 1.2 s. Frames captured 900 ms apart differ, proving
the animation loop is alive.

### Plan request

`propose_plan` rendered a six-step plan with Move up / Move down / Remove per
step and an **Approve plan** action. The request also asks to be questioned, so
the tool now accepts an optional `questions` array (same field schema as
`ask_user`): the plan is followed by a `gb.question` form and the run waits for
input. Previously answered fields are not asked twice.
