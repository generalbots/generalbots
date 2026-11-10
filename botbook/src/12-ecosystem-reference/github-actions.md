# GitHub Actions Workflows 🟡 BETA

The repository runs CI on GitHub Actions as well as on the self-hosted Forgejo
(ALM) instance described in [CI/CD Integration](./ci-cd.md). This page covers
only the GitHub side, because the two pipelines enforce different things.

Authority: `.github/workflows/` and the shared action
`.github/actions/rust-setup`.

## Workflows

| File | Purpose |
|---|---|
| `ci.yml` | Unit tests, feature-manifest guard, served-desktop smoke test, paid `boteval` evaluation |
| `release.yml` | Version-tag release: binaries, tarball, single-file extractor, desktop packages |
| `security.yml` | Dependency and workflow-pinning checks |
| `codeql.yml` | CodeQL analysis |
| `desktop-macos.yml` | macOS desktop build and unsigned `.dmg` pre-release |
| `release-check.yml` | Release links check |
| `download.yml` | Download page for all platform variants |
| `desktop-os-iso.yml` | Kiosk appliance image (General Bots OS) |

## Jobs in `ci.yml`

| Job | What it runs |
|---|---|
| `security` | Calls `security.yml` with `security-events: write` so the SARIF upload works |
| `changes` | Path filter that emits `migrations` and `botbook` outputs for later jobs |
| `windows` | Windows agent-adapter compile check on `windows-latest` |
| `guard` | `scripts/adequacy_guard.py` against the structural baseline |
| `reindex` | Verifies the committed BotBook: installs the pinned mdBook and runs the book guards (only when `botbook` changed) |
| `migrate` | Fresh-database migration run against a `postgres:16` service (only when `migrations` changed) |
| `check` | `scripts/product_profile_check.py` — always/preview/hidden classification |
| `static-precheck` | Lighthouse/a11y gate on a static copy of the desktop markup |
| `served-desktop` | Same gate against a running `botserver` desktop-local build, with `postgres:16` and `redis:7-alpine` services |
| `evaluate` | `boteval` gate against a real model endpoint (paid) |
| `unit-tests` | `cargo test -p botserver --bin botserver` |
| `workspace-tests` | `cargo test -p <crate> --lib` over 17 crates |
| `bottest-unit` | `cargo test -p bottest --test unit` |
| `feature-manifest` | Asserts a slim build reports only its requested features |

The test jobs carry
`if: ${{ github.server_url == 'https://github.com' }}`, so a self-hosted mirror
of the repository does not attempt them.

## Conventions

- **Actions are pinned to commit SHAs**, not tags. `security.yml` checks the
  pinning.
- **Toolchain and cache come from `./.github/actions/rust-setup`**, with a
  `cache-scope` input so each job gets its own cache namespace.
- **Timeouts are explicit** (60–120 minutes for the test jobs).
- **`clippy` and `rustfmt` are not part of `ci.yml`.** The project rule is that
  clippy must pass with zero warnings locally; the workflow does not enforce it.

```yaml
# Shape of a test job in ci.yml (abridged)
unit-tests:
  if: ${{ github.server_url == 'https://github.com' }}
  runs-on: ubuntu-latest
  timeout-minutes: 120
  steps:
    - uses: actions/checkout@<pinned-sha>
    - uses: ./.github/actions/rust-setup
      with:
        cache-scope: unit-tests
    - run: cargo test -p botserver --bin botserver
```

## Why `--bin botserver` matters

`botserver` is a binary crate. `cargo test -p botserver --lib` fails with
"no library targets found" and tests nothing, which is how stale expectations
once accumulated unnoticed. The `unit-tests` job exists to keep that regression
from returning; the same form is required locally.

## Related pages

- [Testing](./testing.md) — test layout and local commands.
- [End-to-End Testing](./e2e-testing.md) — the browser-driven suite.
- [CI/CD Integration](./ci-cd.md) — the Forgejo/ALM pipeline and deploy path.
