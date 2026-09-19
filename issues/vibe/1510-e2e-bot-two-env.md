# [VIBE] 1510 — E2E test plan: bot TEST/PROD two-env Run/Deploy (1504)

**Priority:** P0
**Kind:** test (E2E, browser CDP — visual evidence mandatory)
**Covers:** issue #1504 / GH #1416

## Cases
### Case A — toolbar button is "Bot" for bots
1. Open `/vibe` inside the desktop shell (never the HTML directly), select a
   bot project.
2. Expect the second toolbar command to read **Bot** (not Browser); select a
   website project → it reads **Browser** again.
3. Screenshot `/tmp/{bot}_vibe_toolbar.png`.

### Case B — Run opens the TEST tab
1. Press Run on the bot project.
2. Expect the Chat window with tabs `{NAME} (TEST)` and `{NAME} (PROD)`,
   active tab = TEST; the bot greets with the current git HEAD text.
3. Screenshot before/after.

### Case C — Deploy promotes TEST→PROD
1. Edit the bot's script in the git repo (Forgejo), Run (TEST shows the new
   text), then press Deploy.
2. Expect `POST …/bot/deploy-prod` → 200, log `[git_monitor] {bot}: TEST →
   PROD promotion queued`, switch to the PROD tab → new text; TEST tab →
   same text.

### Case D — deploy pushes uncommitted workspace edits
1. Edit a script in the Vibe workspace (editor) WITHOUT committing.
2. Press Deploy → expect an auto-commit `Deploy {bot}` on `main` and PROD
   serving it (git is the deployable truth).

## Acceptance
- TEST and PROD are independently refreshable; Run never touches PROD.
