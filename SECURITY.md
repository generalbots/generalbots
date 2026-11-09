# Security Policy

General Bots is MIT-licensed software that runs multi-tenant bots, LLM tools and
office-document pipelines.

This file states **what must be done and how often**. It deliberately records no
credential values, credential provenance, host addresses, internal paths or scan
history: those belong in the private operator record, not in a public
repository. The continuous work is done by `.github/workflows/security.yml`,
`deny.toml`, `.gitleaks.toml`, `scripts/osv_allowlist.txt` and Dependabot; this
file is the policy they enforce.

---

## Reporting a Vulnerability

Open a private security advisory on GitHub, or write to
**security@pragmatismo.com.br**. Do not open a public issue for an unpatched
vulnerability. We aim to acknowledge within two business days.

Include the affected component, a reproduction and the observed impact. Do not
test against customer tenants or third-party hosts; tests against our own
infrastructure follow the rules of engagement in the internal tracker.

## Third-Party Runtime Components

The services botserver provisions on first boot — PostgreSQL, Valkey, MinIO and
`mc`, Qdrant, Vault, Zitadel, Caddy, Stalwart, LiveKit, NocoDB, llama.cpp — are
pinned in `botserver/3rdparty.toml` by `version`, `url` and `sha256`. That file
is a security boundary: it decides which binaries a user's machine downloads and
then executes. It applies to the server and to the desktop edition alike.

- ✅ **Update a pin as soon as upstream ships a security release**, without
  waiting for a feature reason to touch the file. Review the file monthly and on
  every upstream CVE that names one of these components.
- ✅ Change `url` and `sha256` together, and recompute the checksum from the
  artifact you actually downloaded (`sha256sum <file>`); a checksum carried over
  from a previous release silently disables the verification the provisioner
  performs.
- ✅ Read the upstream release notes before bumping, and record any residual or
  accepted risk in this file — the same way the `accelerate` entry below does —
  instead of leaving it undocumented.
- ✅ Prefer the smallest surface that fixes the issue: bump the component, keep
  the URL on the vendor's own release page, and keep the pinned release
  immutable.
- ❌ Never point a component at `latest`, a branch, or an unpinned URL. A
  floating tag makes the build unreproducible and unreviewable.
- ❌ Never bypass a checksum mismatch. The provisioner aborts on mismatch by
  design; fix the pin, do not work around it.

Desktop components resolved from a cache directory follow the same rule: an
archive is accepted only when its `sha256` matches the manifest.

Advisory review 2026-11-09: Vault 2.2.1 (pinned) postdates and covers
HCSEC-2026-43 / CVE-2026-89322 (ACL deny bypass, fixed in 2.1.2) and
HCSEC-2026-41; Caddy 2.11.9 covers CVE-2026-27590 (RCE, fixed in 2.11.1).
No action required for either; re-verify at the next cadence review or on any
new bulletin naming a pinned component.

---

## Supported Versions

Security fixes land on `main` and are deployed from there. There is no
long-term-support branch; the desktop line receives the same fixes through
`main` merges. The desktop build ships the same Rust workspace, so a Rust
advisory applies to both.

---

## Automated Gates (continuous)

`.github/workflows/security.yml` runs on every push to `main`, every pull
request, daily at 04:41 UTC and on demand. No job compiles the workspace; the
tools are pinned static binaries verified against a checked-in SHA-256, so a
compromised release asset cannot silently become code we run.

| Job | Tool | Blocks on |
|-----|------|-----------|
| `supply-chain` | cargo-deny | vulnerability, yanked crate, license, source |
| `osv` | osv-scanner | any advisory outside `scripts/osv_allowlist.txt` |
| `secrets` | gitleaks | any credential hit in the full history |
| `python` | pip-audit | vulnerability in an exact `botmodels` pin |
| `sbom` | syft | — (SPDX inventory artifact, 90 days) |
| `workflow-lint` | actionlint + pinning script | unpinned `uses:`, `pull_request_target`, syntax |
| `headers` | curl | the six IMP-09 headers on the public origin (advisory) |

Run the same checks locally:

```bash
cargo deny --all-features check advisories bans licenses sources
osv-scanner scan source --lockfile Cargo.lock --format json --output-file /tmp/osv.json
python3 scripts/check_osv_allowlist.py --report /tmp/osv.json
gitleaks git --config .gitleaks.toml --redact --log-opts="--all"
python3 scripts/check_workflow_pinning.py
```

### CodeQL (security scanning)

The security CodeQL scan is **platform-managed** (GitHub's dynamic
`CodeQL` workflow, path `dynamic/github-code-scanning/codeql`); it is not a
file in this repository, so it has no editable `concurrency` block. GitHub
cancels a superseded analysis when a newer push arrives — a run list showing
`completed / cancelled` for every push is the expected state, and two
`in_progress` runs for the same ref are the regression (#2000).

Do **not** add a checked-in `codeql.yml` next to it: that produces two scans of
every push. If the scan is ever moved to advanced setup, either delete the
dynamic workflow first or add `concurrency: {group: codeql-${{ github.ref }},
cancel-in-progress: true}` to the checked-in file — `scripts/check_workflow_pinning.py`
fails any workflow that runs `github/codeql-action/init|analyze` without it.

`CodeQL - Code Quality` (path `dynamic/github-code-quality/codeql`) is a
separate product and runs independently; do not disable it while working on the
security scan.

---

## Security Maintenance Cadence

A policy file alone is not enough: security is the recurring execution of the
actions below. Automation covers the daily and weekly rows; a named operator
owns the monthly, quarterly and annual rows.

| When | Action | Evidence |
|------|--------|----------|
| **Every push / daily** | Security workflow runs (advisories, OSV, secrets, SBOM, workflow pinning, headers). A red gate blocks the merge. | Actions run + Security tab |
| **Weekly** | Triage and merge Dependabot PRs (Cargo, Actions, pip, npm). Read new OSV/advisory entries; accept or fix each. | Merged PRs, updated `deny.toml`/allowlist |
| **Monthly** | Apply OS security updates to every host and container; reboot where a kernel/library update requires it. Review open CodeQL and Dependabot alerts. Re-check the production baseline below. | Patch log, alert counts, baseline output |
| **Quarterly** | Credential review: rotate what can be rotated, verify the rest at the provider, remove unused accounts/keys/tokens. Review Vault policies, identity-provider users and SSH authorized keys. Restore drill for the secrets store and database backup. | Rotation entries, access list, restore test |
| **Annually** | Review this policy, the residual-risk register and the accepted advisory dates. Re-run an authorized external assessment of the public estate. | Updated file + assessment report |
| **Every release** | Build through CI, verify the artifact checksum, keep the SBOM, deploy the same commit that passed the gates. | Release manifest |

If a gate fails, fixing it is the priority; do not merge around it. If a
provider-side rotation is required, the operator performs it first, then updates
Vault, then restarts the consuming service.

---

## Credential Hygiene

- **Never commit a credential.** Documentation and examples always use
  placeholders; real values live only in Vault.
- **Storage:** service credentials live in Vault by service path; per-bot model,
  channel and integration keys live under the bot's Vault path. Operator-only
  material (unseal shares, recovery keys) stays outside the repository with
  mode `0600`.
- **Rotation:** rotate on operator change, suspected exposure, provider
  requirement, or the quarterly review — whichever comes first. Rotate at the
  provider first, then update Vault, then restart the consumer.
- **Verification:** after rotating, prove the old value is dead at the provider
  and the new value works through the real consumer.
- **History rewrite is not a fix.** It breaks every clone and does not remove
  caches, forks or provider logs. Rotate instead and keep history for audit.

Current credential classes and their rotation action:

| Class | Storage | Action |
|-------|---------|--------|
| Vault unseal shares | operator-only, `0600` | `vault operator rekey`, then prove a seal/unseal cycle before destroying the old shares |
| Identity-provider admin/service tokens and client secrets | Vault | rotate in the provider, update consumers, verify the old value is rejected |
| Cloud provider API keys (GCP, Azure) | Vault | revoke the old key, issue a new one, store it in Vault; decommission unused services |
| Messaging channel tokens (WhatsApp, mail) | per-bot Vault | revoke at the provider, issue a new token, verify the webhook/send path |
| Database, cache, object storage | Vault | rotate the password, update the service and the pool, verify health |

---

## Dependency Policy

- `Cargo.lock` is tracked for application crates; library crates do not track it.
- `cargo-deny` audits the union of every shipping feature graph; a
  graph-specific advisory still fails because the union includes it.
- Licenses: MIT project policy. AGPL is acceptable only for bundled binaries
  whose notices are recorded; no GPL-family crate may be linked.
- SBOM: produced in CI and kept as an artifact; never committed.
- Two office-document crates are **vendored** under `3rdparty/` (`docx-rs`,
  `umya-spreadsheet`) as path dependencies. They are unmodified upstream
  sources except for raising their `zip` requirement to 9.x and, in
  `umya-spreadsheet`, adapting one `ZipFile::name()` call to the zip-9
  `Result` return. This keeps a single patched zip in the graph. The vendored
  copies are removed as soon as a stable upstream release accepts zip ≥ 9; the
  removal condition is recorded in each vendored `Cargo.toml`.
- Vendored frontend assets are local (no CDN) and inventoried in the security
  review. An unreferenced library is removed, not kept "just in case".
- Desktop installer payloads are verified against the checksums in
  `botserver/3rdparty.toml`; a mismatch aborts the install (fail-closed).

### Accepted residual risk

Every acceptance carries a reason, a reachability argument and a review date.
The same IDs are mirrored in `deny.toml` / `scripts/osv_allowlist.txt` so a new
advisory still fails the gate.

| Advisory family | Component | Why accepted | Review |
|-----------------|-----------|--------------|--------|
| glib iterator unsoundness | glib on the Tauri Linux desktop stack | Desktop-only; no server runtime exposure. Removed by the GTK4 desktop upgrade. | 2027-11-06 |
| Unmaintained build-time crates | proc-macro-error, paste, unic-*, ttf-parser, smartstring, zvariant, pin-utils | Build-time macros, Unicode tables, font metrics or the desktop D-Bus stack; no attacker-reachable runtime sink identified. | 2027-11-06 |
| accelerate | Python model tooling | No patched release for the sharded-checkpoint path traversal; only operator-run local tooling, not exposed to end users. | 2027-11-06 |
| Unmaintained Windows cache port | Redis-x64-3.0.504 (microsoftarchive) | Desktop-edition Windows payload only; unmaintained upstream, digest now pinned fail-closed in the release build. Replace with a maintained cache strategy or drop before shipping Windows broadly. | 2027-05-09 |
| Ecosystem single-owner crates | syn, quote, proc-macro2, serde (one owner); ring; jsonwebtoken; reqwest | crates.io-level concentration, not fixable per consumer: the crates are single-owner upstream (verified via the owners API, 2026-11-09, self-review in #2001). Mitigation is lockfile pinning, the cargo-deny/OSV gate on every push and dependency-bump PR review; monitor ownership changes at review. A `cargo vet` attestation pass is the candidate future gate. | 2027-05-09 |

Open upgrade clusters (each changes a public API and needs its own regression
pass):

| Cluster | Current | Removed when |
|---------|---------|--------------|
| GTK / Tauri desktop | glib 0.18 | GTK4 track lands |
| Python model tooling | accelerate 1.1.1 | a release carries the fix |

---

## Production Baseline

Re-check these on every monthly review and after any infrastructure change:

- [ ] Six IMP-09 headers present on the public origin.
- [ ] Unauthenticated protected APIs return `401`; no wildcard CORS.
- [ ] Database, cache, object storage, orchestrator and admin ports are not
      reachable from the internet.
- [ ] Secrets store unsealed, audit enabled, unseal material `0600`.
- [ ] Cache requires authentication; database host rules are least-privilege.
- [ ] Containers are not privileged; orchestrator and browser-debug sockets are
      reachable only by their client containers.
- [ ] Every OS package and service binary is on a supported, patched release.
- [ ] Authentication rate limits are active.
- [ ] TLS certificates and DNS records are current.
- [ ] Backups exist and a restore was tested this quarter.

The operator record (private) holds the host inventory, addresses, versions and
per-finding evidence; this repository holds only the policy and the code.

---

## Repository Configuration

- Every remote GitHub Action is pinned to a 40-character commit SHA with a
  version comment, enforced by `scripts/check_workflow_pinning.py`.
- `pull_request_target` is rejected outright.
- Dependabot opens grouped update PRs weekly for Cargo, GitHub Actions, pip and
  npm; security updates are not grouped so they can merge fast.
- `CODEOWNERS` routes security-sensitive paths (security modules, workflows,
  vendored assets, `deny.toml`, `.gitleaks.toml`, this file) to the security
  owners.
- Release artifacts are built by CI and verified by checksum before deploy.

Open repository action: require reviews and passing status checks on `main`,
disallow force-pushes, and enforce signed commits. This is a repository setting,
not code, and needs the maintainers' agreement because it changes the direct-push
workflow.

---

## Coverage Gaps

- Windows, macOS and alternate-architecture installer checksums must be filled
  before those installers ship.
- Active scanning (DAST) of production is a manual, scheduled exercise by
  design; it is not part of the push gate.
- The authoring host cannot run a full local `cargo check`; compile
  verification is the CI release build.
- Provider-side rotations require operator access and cannot run from CI; the
  quarterly review tracks them.
