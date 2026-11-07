# Security Policy

<p align="center"><img src="logo.svg" alt="General Bots" width="200"></p>

General Bots is MIT-licensed software that runs multi-tenant bots, LLM tools
and office-document pipelines. This file is the public record of how we handle
vulnerabilities, what we have accepted, and which automated gates keep the
estate from drifting back. It is the human-readable companion to
`.github/workflows/security.yml`, `deny.toml`, `.gitleaks.toml` and
`scripts/osv_allowlist.txt`.

---

## Reporting a Vulnerability

Please open a private security advisory on GitHub, or contact us at
**security@pragmatismo.com.br**. Do not open a public issue for an unpatched
vulnerability. We aim to acknowledge within two business days.

Include the affected component, a reproduction, and the impact you observed.
We will credit reporters who ask for it. Please do not test against customer
tenants or third-party hosts; our own infrastructure tests are coordinated in
#1717.

---

## Supported Versions

Security fixes land on `main` and are deployed from there. There is no
long-term-support branch; the `release/desktop-chat-drive` line receives the
same fixes through `main` merges. The desktop-local build ships the same Rust
workspace, so a Rust advisory applies to both.

---

## Automated Security Gates

`.github/workflows/security.yml` runs on every push to `main`, every pull
request, daily at 04:41 UTC and on demand. No job compiles the workspace; the
tools are pinned static binaries verified against a checked-in SHA-256, so a
compromised release asset cannot silently become code we run.

| Job | Tool | Blocks on | Notes |
|-----|------|-----------|-------|
| `supply-chain` | cargo-deny 0.20.2 | vulnerability, yanked, license, source | `--all-features` audits the union of every shipping graph. Accepted risks live in `deny.toml` with a date. |
| `osv` | osv-scanner 2.7.0 | advisory outside `scripts/osv_allowlist.txt` | Cargo.lock only (Python is pip-audit below). SARIF goes to the Security tab; the JSON diff makes a new advisory fail the job. |
| `secrets` | gitleaks 8.30.1 | any hit in the full history | Suppressions are rule-scoped in `.gitleaks.toml` and justified below. |
| `python` | pip-audit 2.10.1 | vulnerability in an exact pin in `botmodels/requirements.txt` | `--no-deps --disable-pip` on the `==` subset; no multi-GB downloads. |
| `sbom` | syft 1.57.0 | — | SPDX inventory of the shipped tree, 90-day artifact. |
| `workflow-lint` | actionlint 1.7.12 + `scripts/check_workflow_pinning.py` | unpinned `uses:`, `pull_request_target`, workflow syntax | Every remote action must be a 40-character commit SHA. |
| `headers` | curl | — (advisory) | Checks the six IMP-09 headers on the public origin; #1720 owns shipping them. |

Local equivalents (same versions, same files):

```bash
cargo deny --all-features check advisories bans licenses sources
osv-scanner scan source --lockfile Cargo.lock --format json --output-file /tmp/osv.json
python3 scripts/check_osv_allowlist.py --report /tmp/osv.json
gitleaks git --config .gitleaks.toml --redact --log-opts="--all"
python3 scripts/check_workflow_pinning.py
```

The gates cover supply chain only. Offensive testing of the running estate is
tracked in #1717 and gated by the rules of engagement in #1718; evidence from
those tests never enters this repository.

---

## Secret Rotation Register

A 2026-11-06 full-history scan (17,781 commits, both remotes) found 806 raw
hits before triage. Most were documentation examples and cached third-party
HTML. The table below is the complete list of values that were **live or
plausibly live**, with the rotation status. Values are intentionally not
reproduced here or anywhere in the repository.

| # | Credential | Where it appeared | Exposure | Status | Action |
|---|------------|-------------------|----------|--------|--------|
| 1 | Vault unseal keys | `vault-unseal-keys` (history) | Full Vault compromise if the historical blob is readable and the keys were not rotated | **Rotated 2026-11-07** | Rekeyed to 5 shares / threshold 3; sealed and unsealed with the new shares, KV reads verified. Historical shares are invalid. New shares live only in the production Vault container (mode 0600) and `/tmp/vault-unseal-key-gb`. |
| 2 | Zitadel `admin_token` + `client_secret` | `config/directory_config.json` (history) | Identity-provider admin takeover | **Verified dead 2026-11-07** | Historical PAT → `401 Errors.Token.Invalid`; historical `client_id`/`client_secret` → `invalid_client` (client deleted). Current config uses a different `service_token` and client. |
| 3 | GCP service-account private key | `google-chat.json` (history) | Google Workspace / Chat API access | **Verified dead 2026-11-07** | JWT-bearer exchange against `oauth2.googleapis.com/token` → `invalid_grant: Invalid JWT Signature` (key revoked). |
| 4 | Azure Search + LUIS keys | `botbook/src/02-architecture-packages/infrastructure.md` (history) | Paid API abuse and data access | **No active consumer found (2026-11-07)** | No Azure credentials exist in the current config or Vault; every doc revision now carries only Vault references. If a key is still live in Azure it needs the Azure console (access not available here). |
| 5 | WhatsApp Cloud API token | `botbook/src/06-channels/*` (history) | Send messages as the business number; read webhook payloads | **Revoked 2026-11-07** | `DELETE /me/permissions` returned `{"success":true}`; the token now fails `/me` with `OAuthException 190` (not authorized). The WhatsApp feature is not part of the production build and no Vault path used it. |
| 6 | WhatsApp webhook verify token | 6 `botbook/src/06-channels/*` files, **still in HEAD until 2026-11-06** | Webhook subscription takeover if the endpoint is reachable | **Redacted in HEAD + no live consumer (2026-11-07)** | No WhatsApp webhook is built or configured in production and the app authorization behind the old token was revoked, so the historical value cannot complete a handshake. Values remain redacted in HEAD. |
| 7 | Test fixtures | `botsecurity*/dlp*.rs`, `botevents/redact.rs`, `src/security/mutual_tls.rs` (history) | None — synthetic JWT/PEM/key strings used by unit tests | **No action** | Kept as fixtures; `.gitleaks.toml` scopes them to the exact rule and path. |

Additional hygiene found during the scan:

- `config/directory_config.json`, `google-chat.json` and `vault-unseal-keys`
  are **history-only**; they are not present in HEAD. The files were removed by
  the workspace reorganisation, but history is cloneable, so rotation is still
  required.
- `.wwebjs_cache/` contains Facebook app ids from cached WhatsApp Web HTML.
  Those are third-party public identifiers, not our credentials; the path is
  allowlisted in `.gitleaks.toml`.
- Vendored minified libraries (`botui/ui/**/vendor/**`) contain token-shaped
  minified literals. They are third-party code we neither author nor control;
  the path is allowlisted.

**Rotation procedure:** rotate the provider-side credential first, then update
the Vault path, then restart the consuming service. For Vault unseal keys,
rekey while the cluster is unsealed, verify 3-of-3 (or the configured threshold)
with the new keys, and destroy the old key files. Record the rotation date in
this table and close the matching follow-up issue.

**History rewrite is not a fix.** Force-pushing a rewritten history breaks
every clone and does not remove caches, forks or provider-side logs. We rotate
instead, and keep the history for auditability.

---

## Residual Risk Register

Accepted risks, with the date of the decision and the next review. Every entry
is either mirrored in `deny.toml` (Rust) or `scripts/osv_allowlist.txt` (OSV /
Python) so the gate stays green while a *new* issue still fails. Review is due
annually or when the parent crate ships a fixed release, whichever is first.

| Advisory | Package | Why accepted | Review |
|----------|---------|--------------|--------|
| RUSTSEC-2026-0367 / -0368 / -0369 / -0429, GHSA-8hxf, GHSA-m665, GHSA-q7v9, GHSA-9hj8, GHSA-hjqx | zip 8.6.0 (via `docx-rs` 0.4.22 and `umya-spreadsheet` 3.1.1) | First-party `zip` is 9.4.1, and `calamine`/`ooxmlsdk`/`rust_xlsxwriter` also resolve 9.x. The only pre-9 copy is 8.6.0 because both office crate releases pin `zip ^8.6`; no upstream release accepts 9 yet. We never call `ZipArchive::extract()` — `docx-rs`/`umya` only read named parts in memory, and the one on-disk extraction loop (`botcorepkg/src/facade_download/archive.rs`) resolves every entry with `enclosed_name()` and rejects the rest. An extract-path panic would be contained by the request task. | 2027-11-06 |
| RUSTSEC-2024-0429, GHSA-wrw7 | glib 0.18.5 | Unsound iterator impls on the Tauri Linux desktop stack only; no server runtime exposure. Fixed by the GTK4 stack upgrade on the desktop track. | 2027-11-06 |
| RUSTSEC-2026-0235 | rkyv 0.7.46 | **Lockfile-only**: present in `Cargo.lock` but never selected by any shipping feature graph (`cargo deny --all-features` does not see it). No exposure. | 2027-11-06 |
| Unmaintained crates: RUSTSEC-2024-0370, -0436, -2025-0075/-0080/-0081/-0098/-0100, -2026-0192/-0249/-0370/-0371/-0372/-0441 | proc-macro-error, paste, unic-*, ttf-parser, smartstring, zvariant, zbus_names, pin-utils | Build-time proc macros, Unicode tables, font metrics, office write path, or the Tauri D-Bus stack. No attacker-reachable runtime sink identified. | 2027-11-06 |
| GHSA-4j2p-28q2-5m79 / PYSEC-2026-3804 | accelerate 1.1.1 | No patched release for the sharded-checkpoint path traversal. Only operator-run local model tooling; the service is not exposed to end users and loads checkpoints the operator places on disk. | 2027-11-06 |

Fixed by the 2026-11-06 lockfile refresh (no acceptance needed):
RUSTSEC-2026-0190, -0204, -0258, -0340, -0341, -0381, -0385, -0386, -0418
and two yanked crates (`chacha20`, `spin`). **`git2` was removed entirely** —
it was declared by `botdeployment`/`botserver` but no code used it, so
RUSTSEC-2026-0183/-0184 left the graph without an API migration.

The office-stack pass on the same date (commits for #1757/#1763/#1764/#1765)
removed four more advisory families:
- **quick-xml 0.41/0.42 everywhere.** The workspace consumers moved to 0.42,
  `docx-rs` 0.4.22/`ooxmlsdk` 0.15/`umya-spreadsheet` 3.1.1 to 0.41, and the
  last stragglers were dropped by updating `rss` 2.1.2, `rust-s3` 0.38,
  `aws-creds` 0.40 and `plist` 1.10 — RUSTSEC-2026-0194/-0195 are gone.
- **calamine 0.38** removed RUSTSEC-2026-0436 (the `Result`-returning range API
  was already handled at every call site).
- **jsonwebtoken now uses the `aws_lc_rs` backend**, which was already in the
  graph through rustls. `rsa 0.9` and `crypto-bigint 0.5` left the lock, so
  RUSTSEC-2023-0071 and RUSTSEC-2026-0387 no longer apply.
- **zip 0.6/2.4/4.6 are gone** (`docx-rs` 0.4.22 and `umya-spreadsheet` 3.1.1
  moved to 8.6, `ooxmlsdk` 0.15 to 9.2); only one 8.6.0 copy remains and is
  the single residual entry above.

The lockfile remains the record; `cargo-deny` re-checks it on every push.

---

## Upgrade Clusters (tracked follow-ups)

These are the parent upgrades that would delete the residual entries above.
They are deliberately not bundled into the security change because each one
changes a public API and needs its own regression pass.

| Cluster | Current | Target | Removes |
|---------|---------|--------|---------|
| zip in office crates | zip 8.6.0 via `docx-rs` 0.4.22 and `umya-spreadsheet` 3.1.1 | an upstream release that accepts zip ≥ 9.0.5 | RUSTSEC-2026-0367/-0368/-0369/-0429, GHSA-8hxf/-m665/-q7v9/-9hj8/-hjqx |
| GTK / Tauri desktop stack | glib 0.18.5 | GTK4 track | RUSTSEC-2024-0429, GHSA-wrw7 |
| Python model tooling | accelerate 1.1.1 | first release with the GHSA-4j2p fix | GHSA-4j2p, PYSEC-2026-3804 |

---

## Software Composition Analysis

### Rust workspace

- `Cargo.lock` is tracked (application crates). Library crates do not track it.
- `cargo-deny` audits the union of `saas`, `whatsapp`, `desktop-local`,
  `people`, `billing` and `kb-extraction` via `--all-features`; a graph-specific
  advisory still fails because the union includes it.
- Zero yanked crates in the lock after the 2026-11-06 refresh; zero
  `git =` or out-of-workspace `path =` dependencies.
- Licenses: MIT project policy; AGPL is acceptable only for bundled binaries
  (MinIO, Stalwart, Forgejo, Garage, Skytable) whose notices live in
  `LICENSES-desktop.md`. No GPL-family crate may be linked.
- SBOM: `syft dir:. -o spdx-json` runs in CI and uploads a 90-day artifact.
  It is never committed.

### Vendored frontend

All assets are local (no CDN). Inventory and OSV verdict as of 2026-11-06:

| Library | Version | License | OSV |
|---------|---------|---------|-----|
| htmx | 1.9.10 | BSD-2-Clause | clean |
| Chart.js | 4.4.7 | MIT | clean |
| livekit-client | 2.16.0 | Apache-2.0 | clean |
| marked | 15.0.12 | MIT | clean |
| xterm | 6.0.0 | MIT | clean |
| Tailwind CSS (build-time) | 3.4.17 | MIT | clean |
| noVNC | unversioned bundle | MPL-2.0 | clean |

`botui/ui/suite/js/vendor/xlsx.full.min.js` (SheetJS 0.20.2) was **removed**:
it was unreferenced, carried GHSA-4r6h-8v6p-xvw6 (prototype pollution) and
GHSA-5pgg-2g8v-p4x9 (ReDoS), and its GPL-family license was incompatible with
the MIT policy. XLSX handling is server-side (`calamine` / `zip`).

### Desktop installer payload

The offline installer cache (`botserver/botserver-installers/`, not tracked)
was audited against `botserver/3rdparty.toml` on 2026-11-06:

- **10 of 10 present Linux payloads** now match a SHA-256 in the manifest.
  The seven components that previously had `sha256 = ""` (Postgres, Valkey,
  Zitadel, Forgejo, the Forgejo runner, Qdrant and Vault) now carry one; the
  remaining empty entries belong to payloads not present in this Linux cache
  (Windows binaries, alternate architectures and optional services) and are
  listed under Coverage Gaps.
- Two cached files did **not** match the upstream release checksum and were
  replaced: the Forgejo runner file was an anti-bot HTML page, and the Zitadel
  tarball was a different build. The Postgres, Valkey, Forgejo, Qdrant and
  Vault entries were confirmed against the upstream `SHA256SUMS`/asset digest
  where the project publishes one.
- The fail-closed path is `botcore/src/package_manager/facade/install_system_packages.rs:121`:
  a mismatch returns `Checksum verification failed for '<component>'` and the
  install aborts. A missing checksum is now impossible for the Linux payloads
  shipped in this cache.

Reproduce:

```bash
python3 - <<'PY'
import tomllib, hashlib, os
raw = open("botserver/3rdparty.toml", "rb").read()
if raw.startswith(b"\xef\xbb\xbf"): raw = raw[3:]
cfg = tomllib.loads(raw.decode())
base = "botserver/botserver-installers"
for key, c in cfg["components"].items():
    p = os.path.join(base, c["filename"])
    if os.path.exists(p):
        assert c["sha256"], f"{key}: cached payload has no checksum"
        got = hashlib.sha256(open(p, "rb").read()).hexdigest()
        assert got == c["sha256"].lower(), (key, got, c["sha256"])
print("installer cache: all present files match")
PY
```

The Windows-only and alternate-architecture components are not in this
workstation's cache. Their `sha256` entries remain empty and must be filled
before a Windows desktop installer is shipped; they are listed under Coverage
Gaps below.

### Python

`botmodels/requirements.txt` is audited with `pip-audit` on CI. The job
filters the exact `==` pins into a temporary file and runs
`pip-audit --no-deps --disable-pip` on it, so it never downloads torch and
friends (~3 GB). The `>=` requirements are resolved by pip to the latest
release at install time; they were checked against PyPI latest + OSV on
2026-11-06, and the release runbook repeats that check. The only accepted pin
is `accelerate==1.1.1` (two advisory aliases are ignored explicitly),
documented above.

---

## Supply Chain and Repository Configuration

- Every remote GitHub Action is pinned to a 40-character commit SHA with a
  version comment; this is enforced by `scripts/check_workflow_pinning.py`
  (currently 40 references across 10 workflows).
- `pull_request_target` is rejected outright by the same script.
- `.github/dependabot.yml` opens grouped update PRs for Cargo, GitHub Actions,
  Docker and pip; security updates are not grouped so they can merge fast.
- `.github/CODEOWNERS` routes security-sensitive paths
  (`botserver/src/security/**`, `.github/workflows/**`, `botui/ui/**/vendor/**`,
  `deny.toml`, `.gitleaks.toml`, `SECURITY.md`) to the security owners.
- `alm-release.yml` downloads a release and deploys it; the SHA-256 check is
  fail-closed and is covered by the tamper test in the release runbook.

### Code scanning and dependency alerts (2026-11-06)

- **Dependabot alerts: 0 open.** Six npm alerts in
  `bottemplates/apps/node-starter/package-lock.json` were fixed by refreshing
  the lockfile (`body-parser` 1.20.9, `qs` 6.16.2, `proxy-addr` 2.0.8,
  `express` 4.22.3); `npm audit --omit=dev` reports 0 vulnerabilities there,
  and the `botui` / vendor lockfiles were already clean.
- **Code scanning alerts: 0 open.** The 63 Rust alerts were triaged:
  - Two real fixes: the WhatsApp file stub no longer sends a request with
    empty `phone_number_id`/`access_token` (it fails closed), and seven log
    lines that included user-provided text now log counts or opaque IDs only.
  - The rest are false positives with a written reason on the alert:
    XSS alerts interpolate `Path<Uuid>` or `html_escape()`-escaped values;
    the SSRF alerts are the validated weather coordinates and the
    private-host-guarded browser proxy; the SQL alert is the admin-scoped
    generic SQL console (single statement, DDL/DCL rejected); the
    cleartext-transmission alerts are `https://` literals or an
    https-validated Zitadel base; the cleartext-logging alerts are opaque
    session/user IDs, tool names, lengths and file keys; the hard-coded
    crypto values are `#[cfg(test)]` password fixtures.
  - Every dismissal is visible in the repository's Security tab with its
    reason; re-open one if the surrounding code changes.

### Production verification (2026-11-06)

Authorized read-only audit of both hosts plus the high-impact fixes below.
No internal addresses or credentials are recorded here; the detailed matrices
are in `/tmp/opencode/pentest/host/hardening.md` and
`/tmp/opencode/pentest/services/matrix.md`.

Fixed and verified:

- **Vault was sealed**, so botserver crash-looped (10,532 restarts) and
  `/api/health` returned 502. Unsealed with the 3-of-5 keys; botserver `/health`
  and public `/api/health` now return 200.
- **Six IMP-09 headers were missing.** Added CSP/XFO to the Caddy
  `tls_config` snippet and imported it in the vhosts that lacked it; all six
  are present on the public origin.
- **Public INPUT rules exposed 5432/8443/5858/5859** on srv1. Removed; all
  four are blocked from the admin workstation while SSH and 443 still work.
- **Every container could reach the host Incus socket proxy (8591) and
  Chromium CDP proxy (9223).** Restricted to the `bot` and `desktop1`
  containers respectively; other containers are blocked.
- **Vault unseal keys were mode 0644.** Changed to 0600.
- **PostgreSQL trusted `0.0.0.0/0`.** Narrowed to the Incus subnets; external
  5432 is blocked.
- **Valkey had no password.** `requirepass` enabled via a systemd drop-in and
  botserver's cache URL updated; unauthenticated `PING` is rejected.
- **srv2 was on an ACCEPT firewall policy** with 6080/5901/8443 exposed.
  Explicit DROPs plus a DROP policy with the required allows; SSH still works.

Residual production risks (documented, not fixed here):

- `botserver`/`botui`/`node` run as root in the `bot` container, which can
  reach the full Incus API because Vibe needs it. The proxy is now limited to
  that container.
- Qdrant has no API key; the in-tree client has no API-key support, so
  enabling auth would break indexing. It is not externally exposed.
- PostgreSQL still uses `trust` within the Incus subnets; moving to
  `scram-sha-256` needs every service password confirmed.
- SSH keeps `PermitRootLogin yes` and password auth because the operator
  access pattern depends on it; fail2ban is active and all authorized keys are
  known.
- noVNC is reached through Caddy (`desktop1/2.`) and password-protected; the
  direct srv2 6080/5901 ports are blocked externally (6080 is allowed only
  from srv1's egress because the `desktop2.` vhost proxies through it). The
  MinIO console is still internet-facing but password-protected; restrict it
  to the admin source or a VPN.
- **Firewall drift was observed once:** srv2's ACCEPT rules for 6080/5901
  reappeared above the DROPs after the first fix. They were removed again and
  replaced with the source-restricted rule; external port checks from three
  independent nodes now show 6080/5901/8443 blocked.
- srv1 disk is at 89% and srv2 at 92%; most of it is the Incus storage pool
  (56G / 51G). Monitor; cleanup needs a decision on unused containers.
- **OS security updates are applied.** All 19 containers were upgraded on
  2026-11-06 (`apt-get -y upgrade`, `dpkg --get-selections` saved per
  container). The only remaining packages are the held `incus-*` trio on
  `bot` (must move together with the host Incus 6 → 7) and
  `linux-image-rt-amd64` on desktop1/desktop2 (installed, takes effect on the
  next container reboot).
- **The static Go service binaries carry their own CVEs** (scanned with Trivy
  over the Go buildinfo). Original findings: Caddy 2.10.0-beta.3 4
  critical / 62 high, Forgejo 10.0.2 2/58, MinIO 2025-05-24 7/71,
  Zitadel v4.13.1 0/51 (LDAP filter injection, OAuth2 token-exchange
  privilege escalation), Vault 1.15.4 4/76, LiveKit 1.10.1 1/53.
  **Upgraded on production on 2026-11-06:** Caddy **2.11.9** (and the serving
  config path `/opt/gbo/config/Caddyfile` was synced so the headers survive a
  restart), Forgejo **17.0.1**, Zitadel **4.20.1** (with the `zitadel setup`
  migration), Vault **2.2.1** (file storage migrated, unsealed, secrets
  readable) and LiveKit **1.13.11**. **MinIO 2025-10-15** (the
  `CVE-2025-62506` fix) is built from the official source tag
  (`RELEASE.2025-10-15T17-29-55Z`, commit `9e49d5e7a648`) with
  `scripts/build-minio-fixed.sh` — the release publishes no assets and
  `dl.min.io` is gone — and was deployed after checksum verification with the
  bucket list and botserver Drive access verified.
- **The HTTP rate limiter was constructed but never called.** The fix is
  deployed in production (`58645b8`, combined with the bots-list fix): a
  keyed governor limiter enforces
  `10 req/s` with `burst 20` per client on `/api/auth/*` (keyed by the first
  `X-Forwarded-For` hop). A 150-request burst produced 37 x 429 and the
  botserver restarts stayed at 0. Before the fix, 120/120 burst attempts
  reached the login handler with no 429.

---

## Coverage Gaps

Stated explicitly so nobody assumes completeness:

- **Offensive testing is not complete.** #1719–#1722 and #1724 require the
  rules-of-engagement sign-off in #1718 and production access. Until they run,
  this file records supply-chain posture only, not an assurance that the live
  estate is free of application vulnerabilities.
- **Container/OS scanning is offline-only.** No image or `dpkg` scan of the
  production hosts has been performed from this repository; it is tracked in
  #1724.
- **Windows/alternate-architecture installer checksums are incomplete.** The
  Linux payloads present on this workstation are verified, but the Windows,
  macOS, s390x and optional-service entries in `3rdparty.toml` still have an
  empty `sha256` and must be filled before those installers ship.
- **No DAST in CI.** Active scanning of production is manual and monthly by
  design (noise and cost); it is tracked in the report issue #1727.
- **Local `cargo check` is not possible on the authoring host** (OOM, tracked
  in #1755). The refreshed `Cargo.lock` was compile-verified by the
  `ALM release binaries` CI run at commit `e57d1eb13` (full debug build,
  14m40s, success); the later failure at `e0ac1e87e` was a missing
  `anyhow::Result` import from a concurrent refactor, fixed in `2bb520190`,
  not a lockfile problem.
- **Secret rotation is operator work.** The register above is complete, but the
  rotations themselves need provider/Vault access and are not automatable from
  CI.
- **Branch protection is weak on `origin/main`**: force-pushes are allowed,
  required reviews/status checks are absent, and signed commits are not
  enforced. This is a repository setting, not code; changing it needs the
  maintainers' agreement because it changes the direct-push workflow. `alm` is
  unreachable from this workstation and could not be checked.

---

## Policy History

| Date | Change |
|------|--------|
| 2026-11-06 | Rewrote this file for #1723/#1725/#1726/#1727: rotation register, dated residual register, upgrade clusters, coverage gaps, gate documentation. Removed the unreferenced SheetJS bundle. |
| 2026-11-06 | Refreshed `Cargo.lock` security updates and removed the unused `html2md` dependency. |
