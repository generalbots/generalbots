# Mobile and compact form factors

General Bots is a desktop-first suite, but every app is expected to stay usable
when the window shrinks. A mobile claim is not an opinion: it is a browser
result recorded against the five viewports in the matrix below (#2421).

## The pane contract

At desktop widths the shell shows the launcher, a pane for the active app and
its secondary panels. As the viewport narrows the shell collapses in one
direction only:

| Width | Panes | Navigation |
|-------|-------|------------|
| 390 × 844 (`phone`) | one | drawer + back button |
| 320 × 568 (`phone-small`) | one | drawer + back button, no horizontal scroll |
| 768 × 1024 (`tablet-p`) | two | no drag needed to reach the second pane |
| 1024 × 768 (`tablet-l`) | two | window chrome stays reachable |
| 1440 × 900 (`desktop`) | full | unchanged from `main` |

Touch targets should be at least 44 CSS pixels. Primary actions are reachable
without hover; anything exposed only through `mouseenter`/`mouseover` is debt.
The composer and a Sheet cell must both open the on-screen keyboard on a coarse
pointer.

## Verification matrix

`scripts/check_viewports.py` is the repeatable gate. It connects to a running
Chrome CDP endpoint (default `http://127.0.0.1:9223`), opens one new tab per
viewport and prints one compact JSON line per row: `inner_width`,
`inner_height`, `scroll_width`, `horizontal_overflow`, `pointer` and a
screenshot path under `/tmp/opencode/`. It never navigates or closes a tab it
does not own.

```bash
python3 scripts/check_viewports.py --cdp http://127.0.0.1:9223 \
  --url http://localhost:3000/default --out-dir /tmp/opencode
```

Each mobile claim must attach a `criterion | observed | PASS/FAIL` table with
the screenshot paths to the relevant issue before it is closed. The full
flow-level matrix (login, Drive round trip, app switching, rotation, keyboard,
deep links) lives in `UAT.md`.

## Ratchet

`scripts/check_mobile_budget.py --check` counts four structural signals —
`100vh`, hover-only handlers, sub-44px interactive targets and malformed
breakpoint spellings — and fails when any of them rises above the checked-in
baseline in `scripts/mobile_budget_baseline.json`. The baseline only ever goes
down. `--self-test` demonstrates that a raised count exits non-zero without
requiring a browser.

## Native thin clients (in development, #2584)

The `botapp` `mobile` / `thin-client` Cargo features build a WebView-only client:
it renders the existing suite from a remote botserver and does not start the
desktop sidecar, setup wizard, tray or host-filesystem commands. Both Android
and iPhone builds are being added; compiled artifacts are not yet published in
a release.

The origin is the only server the client may contact. It is resolved from the
`GB_MOBILE_SERVER_URL` build-time value first, then the same runtime environment
variable. A missing or invalid value is a hard start error — there is no public
default. The value must be an HTTPS origin with a non-empty host and no user
information or fragment; the loopback `http://127.0.0.1:8080` is accepted only
in debug builds. An optional runtime `GB_MOBILE_SESSION_TOKEN` is appended to
the suite URL as `?token=`; no token is embedded in the binary.

Artifacts are produced on demand by the manual `mobile-artifacts` workflow, not
by the release publish gate. Android yields a `.apk` and a `.aab`; iOS yields an
ad-hoc-signed `.ipa`. Binary-inspection and device/browser evidence for each
platform are tracked in the child issues before the client is considered ready.
