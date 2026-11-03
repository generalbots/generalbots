# First-Run Wizard (Desktop Edition)

The desktop edition ships a dedicated first-run wizard that runs **before** the
local stack exists. It is static HTML/CSS/JS bundled with the application and
rendered by its own window, so it never depends on botserver, botui, Vault,
PostgreSQL, MinIO or the network.

Issues: #1586 (placement), #1587 (shell), #1588 (services), #1589
(provisioning), #1590 (privacy data map), #1591 (acceptance matrix).

## Where it lives

| Piece | Path |
|---|---|
| Static page | `botui/ui/wizard/index.html` |
| Styles | `botui/ui/wizard/wizard.css` |
| Controller | `botui/ui/wizard/wizard.js` |
| Step rendering | `botui/ui/wizard/wizard-steps.js` |
| State and provider copy | `botui/ui/wizard/wizard-state.js` |
| Tauri bridge | `botui/ui/wizard/wizard-bridge.js` |
| Headless checks | `botui/ui/wizard/selftest.html` |
| Tauri backend | `botapp/src/desktop/wizard/` |
| Provisioning mode | `botserver/src/main_module/wizard_provision.rs` |

The page is served at `/wizard/index.html`. In a packaged build it resolves
from the Tauri asset protocol; in development botui serves
`botui/ui/wizard/` at `/wizard`.

## Detection

The wizard opens only when **both** are true:

1. desktop mode — the `desktop-local` feature is compiled, `GB_DESKTOP_MODE=1`
   is set, or `stack/.product.desktop-local` exists;
2. first run — `stack/.initialized` is absent.

There are no heuristics based on a slow API. A non-desktop build never opens
the wizard.

## Lifecycle

```
app start
  ├─ first run → wizard window (main window stays hidden)
  │     steps 1–4 collect choices
  │     step 5 shows the data map and runs provisioning
  │     wizard spawns `botserver --provision-stack`
  │       vault → tables → cache → drive → vector_db → llm (if local)
  │     wizard starts the normal sidecar and waits for /api/product
  │     step 6 connects Claude Code / ChatGPT plan accounts (optional)
  │     wizard stores OAuth clients through the API (if any)
  │     handoff: write stack/.initialized, show suite, close wizard
  └─ otherwise → start sidecar and show the suite (previous behaviour)
```

Provisioning is idempotent: an installed and running component is reported
`ready` without being restarted. Progress is written to
`stack/.wizard-progress.json` after every component, so a crash resumes at the
first incomplete component. A failed component keeps the stack folder, names
the component and exposes **Open log folder**, **Retry** and **Reset stack**
(typed confirmation naming the exact folder).

Nothing is downloaded at runtime in the desktop edition. Missing bundled
binaries produce a named error from the package manager instead of a download.

## Privacy data map

Step 5 renders the data map from the draft configuration, not from static
marketing copy:

| Data | Destination | Trigger |
|---|---|---|
| Conversations, generated files, database | the chosen workspace folder | always |
| API keys, OAuth tokens | the local vault in the stack folder | on save |
| Prompts and answers | the model chosen in step 2 — *nothing* when the bundled local model is selected | every message |
| Mail and files | connected providers, only for requested actions | per request |
| Crash reports | nowhere unless the opt-in checkbox is on | opt-in |

## Assistant model options

Step 2 offers four choices. The first is the bundled local model
(llama.cpp + DeepSeek-R1-Distill-Qwen-1.5B) and is the only one that installs a
local runtime. For low-memory machines, setup can skip that bundle entirely by
choosing a hosted option:

| Option | Key | Local bundle | Where prompts go |
|---|---|---|---|
| Bundled local model | — | installed | nowhere (this machine) |
| Free hosted API | not required for Kilo Code/OVHcloud; LLM7.io needs a free account key | not installed | Kilo Code (`kilo-auto/free`), LLM7.io (`gemini-3-flash`) or OVHcloud AI Endpoints (`gpt-oss-20b`) |
| My own provider key | required | not installed | the endpoint the user enters |
| Hosted model | required | not installed | the endpoint the user enters |

The free endpoints are third-party services governed by their own terms; the
wizard states that they have no SLA and should not receive confidential data.
The Test button calls the chosen endpoint before Next is enabled. The choice is
written to the stack's LLM secret (`gbo/llm`) during provisioning, so the
assistant uses it immediately.

## Administrator credentials

The finish screen shows the administrator account created by setup (email and a
generated password) before the suite opens. Saving it writes
`stack/conf/directory/admin-credentials.json` with user-only permissions; the
password is never logged and is not persisted in the setup draft. The login
fallback reads that file from `BOTSERVER_STACK_PATH` as well as the process
working directory, so a packaged desktop install can sign in with the account
the wizard displayed.

## AI accounts step

After provisioning, the wizard offers the two subscription accounts:

| Card | Store | Login flow |
|---|---|---|
| Claude Code | the agent VM (`/home/claude/.claude`, 0700) | `claude setup-token` runs under the `claude` user; the URL is shown in the wizard, and the pasted code is forwarded to the VM |
| ChatGPT plan | the local Vault (`gbo/users/{user_id}/chatgpt-plan`) | loopback OAuth on `127.0.0.1:1455` (`OPENAI_OAUTH_CLIENT_ID` / `OPENAI_OAUTH_REDIRECT`) |

Both cards are optional: the assistant works without a subscription, and the
same accounts can be connected later in Settings → AI accounts.

## Acceptance

`botui/ui/wizard/selftest.html` runs the real controller against the in-browser
bridge with the stack stopped and no network. It asserts:

- the wizard renders and the rail exposes eight steps with `aria-current`;
- copy names no operating system;
- step 2 blocks an own-key setup without a key and passes with the local model;
- workspace writability is reported;
- the redirect URI shown matches the backend callback;
- the data map reflects the selected model;
- provisioning reaches `ready` and every component reports `ready`;
- the AI accounts step renders the Claude Code and ChatGPT plan cards;
- the finish step summarises the choices.

The matrix in #1591 (operating systems, offline, invalid keys, interruptions)
is executed per release; the headless harness covers the deterministic
scenarios on every build.
