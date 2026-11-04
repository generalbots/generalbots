# GB7 test runs — 2026-11-04

Environment: botserver `:8080` (rebuilt with the surface/terminal fixes),
BotModels `:8085` (min tier, CPU), botui `:3000/4000/5000`, browser Chrome
CDP `:9223`. Chat model is the externally configured completion endpoint;
speech/vision/OCR are local.

## TS-06 — durable runs (PARTIAL)

- A `vm_exec` shell loop (`for i in 1..8; echo step $i; sleep 2`) was started
  from chat, the browser was reloaded mid-run, and the same session finished
  with `stdout: step 1 … step 8`, `exit code: 0`.
- Server-side `run_events` for the thread: 20 events, including
  `RUN_STARTED` / `STEP_STARTED` / `STEP_FINISHED` / `RUN_FINISHED`
  (`duration_ms: 25849`, `state: "succeeded"`).
- A second reload left the count at **20** (no duplicated events).
- Takeover replay (`?session=`, active takeover bar) was verified earlier in
  #1677.
- Not run: mid-run steering, 2-hour goal checkpoints, 100-turn TTFT.

Evidence: `docs/e2e/2026-11-04/ts06_reload.png`.

## TS-07 — terminal and browser audit (PASS, ledger partial)

Terminal:
- User prompt: “Run this shell command with vm_exec: ls /nope …”.
- The consent card proposed exactly `ls /nope` after the parser fix
  (`propose_terminal_from_text` now takes the payload after `command:` /
  `vm_exec:` and cuts “Report/Show/Do not” clauses).
- Approval wrote a `terminal_audit` row:
  `command=ls /nope`, `decision=approved`, `exit_code=2`,
  `output_excerpt=ls: cannot access '/nope': No such file or directory`.
- The first command (`date; ls -la`) ran through `vm_exec` with `ok=true` and
  printed stdout.

Browser:
- Asking to open `https://example.com` produced a **Browser Action** consent
  card; Allow returned “Opened: Example Domain … Example Domain …”.
- The visible browser is the running Chrome/CDP session.

Ledger note: the terminal ledger carries actor/tool/args/decision/exit/output.
The browser action is visible and gated, but its result is not stored with a
screenshot path in a queryable ledger row; that remains a gap for the
“screenshot path” column of TS-07.

Evidence: `docs/e2e/2026-11-04/ts07_terminal_audit.png`,
`docs/e2e/2026-11-04/ts07_browser.png`.

## TS-09 — Drive search, previews, share, versions, trash (PASS)

Uploaded `parity-search-unique.pdf` (unique phrase `ZEBRA-PARITY-9147`),
`parity-preview.png` and `parity-preview.xlsx` through `/api/files/write`.

- Content search: `GET /api/files/search?query=ZEBRA-PARITY-9147&scope=bot`
  returned the PDF (the phrase exists only in its body).
- Inline previews: PDF `application/pdf` 1446 B, PNG `image/png` 2421 B,
  XLSX `application/octet-stream` 5406 B. The XLSX MIME is generic; the UI
  previews it by extension.
- Share: `POST /api/files/share-link` minted `/api/files/public/<token>`;
  anonymous GET returned the exact PDF (`application/pdf`, 1446 B) and a
  POST to the same URL returned **403** (read-only).
- Versions: overwriting the PDF produced
  `.versions/parity-search-unique.pdf/<stamp>`; restore returned 200 and the
  restored bytes equal the original v1 (`cmp` clean).
- Trash: `POST /api/files/trash/move` created a row with `original_path`,
  `bucket=default.gborg`; `POST /api/files/trash/restore` returned 200 and the
  file was searchable again.
- Storage: the bucket is MinIO `default.gborg`; `/api/files/shared` is empty
  and no external drive is connected.

## TS-10 — trust/cost/ops (PARTIAL)

- Login `:5000` → 200; Zitadel `:8300/healthz` → 200.
- RBAC: a member session (`roles: ["User"]`) calling the admin-only
  `POST /api/files/bots` got **403** `{"code":"RBAC_DENIED"}`; the admin
  session passed the gate (422 for the intentionally empty body).
- Zitadel bootstrap still logs “Failed to ensure default organization/admin
  (non-critical): No access token” for the admin PAT; creating a directory
  user through the API is blocked until that PAT is valid.
- Not run: per-run spend caps, pooled budget reports, cross-tenant file id
  probe, axe/375px/Lighthouse, CI manifest gate, golden Q/A eval set,
  self-hosted egress-blocked install.

## Blockers outside this repository

- **TS-05**: the published page still lists locked conversations and skill
  import as gaps; both shipped. A corrected script is in
  `docs/e2e/2026-11-04/ts05-rebaseline.md`; the site owner must publish it.
- **TS-08**: a Telegram bot token is required for the channel leg. MCP
  resources and the Settings toggle are verified.
- **TS-10**: a valid Zitadel admin PAT is required to create the member user
  and drive the SSO leg end to end.

## Follow-up — steering, a11y, tenant isolation

### TS-06 steering

A `vm_exec` loop ran while a second message was sent. The server queued it
(`bus.is_running`) and started a **second run** at the turn boundary; its
`RUN_STARTED` title is the steering text (`Do not run any command. Reply with
exactly: STEER-OK`). The model's second turn produced reasoning-only content
(no visible final `STEER-OK`), so the delivery mechanism is verified but the
model instruction following remains partial. The earlier takeover replay
(`?session=`, active bar) still stands (#1677). The 2-hour goal and 100-turn
TTFT checks were not run.

### TS-10 RBAC / cross-tenant / a11y

- Member token (`roles: ["User"]`) → `POST /api/files/bots`: **403**
  `{"code":"RBAC_DENIED"}`. Admin token passes the middleware (422 for the
  empty request body).
- Cross-tenant: member and admin requests for bucket `salesianos.gborg`
  return **403** `Access denied for this bucket` for both listing and
  `download-inline`.
- 375 px emulation: `innerWidth=375`, `scrollWidth=375`, no horizontal
  overflow; 0 images without `alt`; key composer present. One cached icon
  (`#scrollToBottom`) still lacks an accessible name in the rendered page;
  `aria-label` was added to the source along with the three taskbar tray
  icons (`#trayUsage`, `#trayNotifications`, `#trayLock`).
- Caps: `botbrowserpolicy::BudgetTracker` enforces step and milli-unit cost
  caps for **browser tasks**; no per-run chat/VM spend cap was found.
- Evals: `boteval-run` exists with `ci/gate.toml`
  (`max_cost_per_task = 0.05`, `min_pass_rate = 0.95`); the full suite was not
  executed (it requires building the runner and spending model calls).
- SSO remains blocked by the invalid Zitadel admin PAT (the bootstrap logs
  “No access token”).
