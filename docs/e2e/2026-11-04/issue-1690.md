# Issue #1690 — Human review UX pass

All items reported in the issue were reproduced in Chrome (CDP :9223) and
fixed. Evidence below is from the running suite at `localhost:3000`.

## 1. Taskbar — half height, Start left, tray right

`css/desktop/taskbar.css` now drives one full-width strip:

| Property | Before | After |
|----------|--------|-------|
| Height | 56 px floating dock | **30 px** (`--taskbar-h`) |
| Start | centred dock | **left edge** (`left: 6px`) |
| Search | next to Start | next to Start, 190 px, collapses on narrow viewports |
| Tray + clock | right of the centred dock | **right edge** |
| Running apps | centred dock | centred in the flexible middle |

The taskbar became a single global `<footer>`; the chat stylesheet's bare
`footer` selector was painting a white full-width bar over the dock. Chat
footer rules are now scoped to `#chat-app footer`.

Measured: `taskbar {w:1205,h:30,l:0}`, `left {l:6}`,
`right {l:1005,w:169}` at a 1205 px viewport.

Screenshot: `1690-taskbar-max-ptbr.png`.

## 2. Maximized window over the left rail

A maximized window is now `position: fixed`, `left: 0`, `width: 100vw`,
`height: calc(100vh - var(--taskbar-h))`. The rail is covered and the taskbar
stays reachable. Only the window that was maximized gets the behavior
(`.window-maximized-target`), so other open windows are not forced fullscreen.

Measured: window `{w:1205,h:610,l:0,t:0}`, viewport `1205×640`,
taskbar `y=610..640`.

The Start menu remains above it: overlay `z-index: 9999` vs window `1500`.
Screenshot: `1690-start-menu.png`.

## 3. Two scrollbars

`_makeResizable()` used to set `overflow: auto` on the window element itself
while `.window-body` and `#messages` also scrolled. The window element is now
`overflow: hidden`; only the app's list scrolls.

Long-conversation audit (`#messages` scrollHeight 4277 / clientHeight 357):

```text
window overflow-y: hidden
scrollers: [ {tag: MAIN, id: messages, sh: 4277, ch: 357} ]
```

## 4. Chat input above tabs

The tab strip is pinned with flex order (`order: -1`), messages/panes `0`,
footer/composer `1`. Switching or restoring a conversation can no longer put
the composer above the tab strip, regardless of DOM re-attachment order.

Measured on a switched conversation: strip `y=43`, messages `y=84`,
composer footer `y=416` (top-to-bottom).

## 5. Attachments open in the browser

Attachment file names are real links with `target="_blank"` and no
`download` attribute. Images, PDFs, text, audio and video use the raw endpoint;
Office documents use the sanitized `preview=1` conversion. The Download button
keeps the save-to-disk behavior.

Verified by clicking `general-bots-for-all.pdf`: the popup opened with
`document.contentType = application/pdf`.

## 6. Microphone and full duplex (voice)

New module `chat/modules/76_voice.js` and two composer controls:

- **Microphone**: records with `MediaRecorder`, auto-stops after speech +
  1.2 s silence, sends the recording as a normal audio attachment. BotModels
  `faster-whisper` transcribes it at `/api/speech/totext`; the transcript
  reaches the same chat/LLM path as uploaded audio.
- **Full duplex**: re-arms the microphone after every completed bot turn and
  reads the reply through the existing TTS layer.
- Audio/voice attachments now also work in parallel conversation tabs
  (`MC.sendInTab` forwards the `file` payload instead of dropping it).

Verified states: recording `true` with `aria-pressed=true` and title
“Parar a gravação e enviar”; duplex `active`, `autoSpeak=true`, then off.

End-to-end voice proof: BotModels generated “What is the capital of France?”
as a WAV; the file was attached in a **new conversation tab**; the bot replied
“The capital of France is Paris.”

Screenshots: `1690-voice-recording.png`, `1690-voice-transcription.png`.

## 7. Token error after sleeping

Suite sessions had a hard 3600 s TTL. The login response also advertised
`expires_in: 3600`, so a laptop sleeping for an hour woke up with a dead
session and a login redirect.

`botcoredirectory` now uses `GB_SESSION_TTL_SECS` (default **7 days**, minimum
300 s) and returns the same value as `expires_in`. Verified live:

```text
login expires_in: 604800
```

## 8. Localization (i18n)

- Desktop chrome tagged: Start, search, usage, notifications, lock, clock
  tooltip, sidebar sections/actions.
- Chat: connection states, model tiers, tab titles, voice controls.
- New catalogue keys added to `botlib/locales/{en,pt-BR,es,fr,de,ja,ko,zh-CN}/ui.ftl`
  and to the exposed key list in `botcore`.
- `i18n.translateElement` now keeps the authored English text when a key is
  missing instead of exposing the raw key.
- Late-rendered sidebar labels refresh on `localeChanged`.
- `GBDateTime` picks a locale-aware date default: `en-US` → MM/DD/YYYY,
  `ja/zh/ko` → YYYY-MM-DD, others → DD/MM/YYYY. Settings overrides still win.
- Model tiers are localized after the `/api/chat/models` hydration.

Verified in `pt-BR`:

```text
lang=pt
search=Pesquisar...      start=Iniciar
newChat=Nova Conversa    sections=Ações rápidas / Arquivos rápidos / Fixados
actions=Nova Conversa / Explorador / Configurações
model=Rápido · ágil / Equilibrado · padrão / Qualidade · melhor
date=04/11/2026
```

Screenshot: `1690-sidebar-i18n.png`.

## Commands / checks

```text
cargo check -p botserver --features "sources,goals,telegram"   exit 0, 0 warnings
node --check (all modified JS files)                           pass
GET /api/i18n/pt-BR                                            new keys translated
POST /api/auth/login                                           expires_in 604800
```

Botbook updated: `07-user-interface/apps/chat.md` (voice, tabs, attachments,
localization) and `apps/suite.md` (taskbar, maximize, single scrollbar,
session lifetime).
