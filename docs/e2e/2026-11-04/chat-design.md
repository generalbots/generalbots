# Chat surface redesign — 2026-11-04

Tracking: [#1689](https://github.com/generalbots/generalbots/issues/1689).

## What changed

- User messages: neutral gray surface (`#f4f4f5`), explicit gray border
  (`#e4e4e7`), dark text. No accent fill, no white text, no black border.
- Bot messages: white surface, `#ececef` border, 18 px radius, 1 px shadow.
- Hover actions: copy and listen sit beside the bot bubble and appear on
  hover/focus; timestamps appear on hover. Copy was verified against the
  clipboard (`Hello there — good to see you!`).
- Run cards: compact, gray-outlined (12 px radius, 12.5 px title). Replayed
  finished cards from `run_resume` are removed instead of being parked at the
  bottom; live cards keep their turn position. Direct-child cards no longer
  shrink to 2.5 px lines.
- Composer: one gray border and one soft shadow; focus changes the border
  colour only (no double outline).
- Window chrome around `#chat-app` uses `#d4d4d8`; the theme's near-black
  focused border is overridden.
- Dark theme: dark-gray borders and surfaces (`#3f3f46`, `#313138`,
  `#1b1b1f`), dark header/status bar, dark thinking block.

## Verification (computed styles, live browser)

Light:

```text
user    bg rgb(244,244,245)  border rgb(228,228,231)  text rgb(24,24,27)
bot     bg rgb(255,255,255)  border rgb(236,236,239)  text rgb(24,24,27)
window                       border rgb(212,212,216)
near-black border audit: 0 elements
```

Dark:

```text
user    bg rgb(38,38,43)    border rgb(63,63,70)    text rgb(237,237,240)
bot     bg rgb(27,27,31)    border rgb(49,49,56)    text rgb(237,237,240)
window                      border rgb(63,63,70)
header  bg rgb(27,27,31)
```

Mobile (375 px emulation): `innerWidth = scrollWidth = 375`, no horizontal
overflow; user border still `1px solid rgb(228,228,231)`.

Screenshots:

- `chat-design-light.png`
- `chat-design-dark.png`
- `chat-design-mobile.png`

## Files

- `botui/ui/suite/css/chat-modern.css` — tokens, canvas, bubbles, actions,
  timestamps, thinking block (242 lines)
- `botui/ui/suite/css/chat-modern-composer.css` — composer, suggestions,
  tabs, states (195 lines)
- `botui/ui/suite/css/chat-modern-chrome.css` — rail, window chrome, run
  cards, floating controls (288 lines)
- `botui/ui/suite/css/chat-modern-dark.css` — dark overrides (146 lines)
- `botui/ui/suite/chat/chat-messages.js` — hover timestamp, copy action
- `botui/ui/suite/chat/modules/71_run_timeline.js` — replay cleanup
- `botui/ui/suite/desktop.html`, `partials/chat.html` — cache busters
- `botbook/src/07-user-interface/conversation-workspace.md` — visual language
  contract
