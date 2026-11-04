# Issue #1691 — Human review 2

Verified in Chrome (CDP :9223) against the running suite and botserver.

## 1. Audio + full-duplex + camera in the composer

The composer now has three capture controls beside the send button:

| Control | Behavior |
|---------|----------|
| Microphone | records one utterance, sends it as an audio attachment; BotModels `faster-whisper` transcribes it |
| Full duplex | opens an immersive voice panel with a live waveform and keeps the conversation hands-free |
| Camera | opens a live preview, captures a JPEG and sends it for BotModels vision analysis |

**Voice panel.** Full-screen glass surface; 32 animated bars that follow the
microphone level while listening (green), pulse while the answer is generated
(amber) and animate while the assistant speaks (blue). It shows the last
assistant line and a single **End voice conversation** action that stops both
the microphone and duplex mode.

Live state after enabling it:

```text
overlay: true       phase: listening
eyebrow: "Conversa por voz"
status:  "Ouvindo… fale naturalmente"
end:     "Encerrar conversa por voz"
bars:    32, heights 37–50 px
```

Closing it set `duplex=false`, `recording=false`, `visible=false`.

Screenshot: `1691-voice-mode.png`.

**Camera.** `getUserMedia({video:{facingMode:"environment"}})` feeds a live
preview; the shutter draws the frame to a canvas and stages a
`camera-<timestamp>.jpg` attachment. The fake camera produced 1280×720, the
photo was sent and the assistant described it (“green background with a black
and white pattern”), proving camera → attachment → BotModels vision → LLM.
Browsers without `getUserMedia` use the native `capture` file input.

Screenshots: `1691-camera-preview.png`, `1691-camera-analysis.png`.

## 2. Weather enabled and working

Weather is part of the default profile (`apps=…,weather`) but the client
registry filter kept only the always-on core when Preview mode was off, so the
window opened via the fallback `/suite/partials/weather.html` (404) and stayed
on “Loading app”. The registry now keeps every app enabled by `.product` (and
every catalog entry the backend marks `enabled`), while preview apps still
require Preview mode.

The app also used to call Open-Meteo directly. On this dual-stack network the
browser picked an unreachable IPv6 geocoding host and showed “Weather is
offline”. A new botserver endpoint proxies the lookup:

```text
GET /api/weather?city=São Paulo
status: 200 in 0.21 s
city:   São Paulo
current: temperature 29.3 °C, humidity 47 %, wind 10.1 km/h
daily:  7 days
```

The app prefers `/api/weather`, keeps a direct Open-Meteo fallback, and a city
search now cancels a pending geolocation autodiscovery (a late failure used to
overwrite the city the user just searched).

End-to-end: “Open the Weather app and search for Salvador” produced a 4-step
automation, all successful, and the window shows **28 °C, Salvador, 72 %
humidity, 19 km/h** plus the week forecast.

Screenshots: `1691-weather-app.png`, `1691-weather-automation.png`.

## 3. Dynamic app offers

`ui_automation_instructions()` now receives the client capabilities
(`preview_mode`, locale, input) and filters the app catalog, deep links and
sequence hints through the product configuration. The prompt states that an
unlisted app must be answered in prose, never with a plan. The client also
refuses a plan whose app is absent from `APPS_REGISTRY`.

Live proof with CRM absent from `.product`:

```text
user: Open the CRM app for me.
bot:  CRM isn't one of the apps enabled in this workspace, so I can't open it
      — the available apps are Clock, Chat, JukeBox, Sheets, Editor, Database,
      Integrations, Terminal, Explorer (Drive), Player, Calculator, Sticky
      Notes, Photos, Timer and Weather. …
crm windows: 0
```

The same check for weather (enabled) opened and filled the app. Executing a
plan directly for CRM returned `false`; the registry contained `weather` and
not `crm`.

Unit test for the catalogue filter:

```text
test main_module::ui_plan::ui_automation_instructions::tests::
     only_enabled_apps_enter_the_prompt_catalogue ... ok
test result: ok. 1 passed; 0 failed
```

## 4. Duplicate bot replies in parallel tabs

A parallel tab rendered every completed reply twice (scrollback had two
identical bot entries). The tab socket now drops a completion frame that
repeats the immediately preceding identical bot turn, and duplicate UI plans
are executed once. `open` steps also resolve `true` after their settle delay
instead of `undefined`, so a successful automation no longer ends with
“Automation FAILED — 1 step did not complete”.

Final automation report for the weather flow:

```text
✅ Opened weather
✅ Filled 'City' = Salvador
✅ Clicked 'search'
⚙️ Waiting 800ms
✅ Automation complete: 4/4 steps
```

## Checks

```text
cargo check -p botserver --features "sources,goals,telegram"   exit 0, 0 warnings
cargo test -p botserver --bin botserver <prompt filter>        1 passed
node --check (all modified JS)                                 pass
GET /api/weather?city=São Paulo                                200, 7-day payload
```

## 5. Final automation run

After the sequence hint and the `open`-step return fix:

```text
user: Open the Weather app and search for Curitiba.
✅ Opened weather
✅ Filled 'Search city' = Curitiba
✅ Clicked 'search'
⚙️ Waiting 800ms
✅ Automation complete: 4/4 steps
Weather window: 22 °C, Curitiba, 84 % humidity, 11 km/h
```

The model occasionally left a plan fragment in the visible reply when the JSON
opened inside a prose brace. `scrub_plan_fragments()` now drops any surviving
line carrying plan keys (`"op"`, `"steps"`, `"__ui_plan__"`), with a regression
test. The user sees only the confirmation sentence.

Screenshot: `1691-weather-chat-final.png`.

Final smoke run (Recife/Fortaleza):

```text
✅ Opened weather
✅ Filled 'Search city' = Fortaleza
✅ Clicked 'search'
⚙️ Waiting 800ms
✅ Automation complete: 4/4 steps
Weather window: 28 °C, Fortaleza, 66 % humidity, 18 km/h + 7-day forecast
raw plan leaked into chat: false
```
