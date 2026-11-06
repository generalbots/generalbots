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
| 1 | Vault unseal keys | `vault-unseal-keys` (history) | Full Vault compromise if the historical blob is readable and the keys were not rotated | **Open — rotation required** | Rotate the unseal keys with `vault operator rekey`; never store them in the repo or a git-adjacent directory. Store only in `/tmp/vault-unseal-key-gb` on the operator host. |
| 2 | Zitadel `admin_token` + `client_secret` | `config/directory_config.json` (history) | Identity-provider admin takeover | **Open — rotation required** | Rotate the service user token and client secret in Zitadel; confirm the current `conf/system/directory_config.json` uses new values. |
| 3 | GCP service-account private key | `google-chat.json` (history) | Google Workspace / Chat API access | **Open — rotation required** | Revoke the key in GCP IAM, create a new one, store it in Vault (`secret/gbo/...`), never in a tracked JSON file. |
| 4 | Azure Search + LUIS keys | `botbook/src/02-architecture-packages/infrastructure.md` (history) | Paid API abuse and data access | **Open — rotation required** | Rotate both keys in Azure; replace the examples with placeholders. |
| 5 | WhatsApp Cloud API token | `botbook/src/06-channels/*` (history) | Send messages as the business number; read webhook payloads | **Open — rotation required** | Rotate the permanent token in Meta Business Settings; put the new value in the per-bot Vault path. |
| 6 | WhatsApp webhook verify token | 6 `botbook/src/06-channels/*` files, **still in HEAD until 2026-11-06** | Webhook subscription takeover if the endpoint is reachable | **Redacted in HEAD (this change)** | Replace with `YOUR_VERIFY_TOKEN_HERE`; rotate the token in Meta and in the bot Vault entry because history still contains the old value. |
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
| RUSTSEC-2026-0194 / -0195 | quick-xml 0.31–0.39 (transitive) | Quadratic start-tag parse and unbounded namespace allocation. Reached only through office-document import (`docx-rs`, `ooxmlsdk`, `calamine`, `umya-spreadsheet`) of authenticated uploads, behind size limits. Parents do not accept quick-xml 0.41 yet. | 2027-11-06 |
| RUSTSEC-2026-0367 / -0368 / -0369 / -0429, GHSA-8hxf, GHSA-m665, GHSA-q7v9 | zip 0.6.6 / 2.4.2 / 4.6.1 (transitive) | First-party `zip` is 9.2.0 (patched). The transitive copies belong to `docx-rs`, `umya-spreadsheet`, `ooxmlsdk`, `calamine` and `rust_xlsxwriter`. Our code never calls `ZipArchive::extract()`; the only on-disk extraction loop (`botcorepkg/src/facade_download/archive.rs`) resolves every entry with `enclosed_name()` and rejects the rest. An extract-path panic is contained by the request task. | 2027-11-06 |
| RUSTSEC-2026-0436 | calamine 0.26.1 | CFB/OLE2 unbounded allocation. Only the authenticated spreadsheet import reaches it, behind upload size limits. The 0.38 upgrade changes the public API and is tracked as a follow-up. | 2027-11-06 |
| RUSTSEC-2023-0071, RUSTSEC-2026-0387 | rsa 0.9.10 / crypto-bigint 0.5.5 | Marvin timing and bigint timing. RSA is used for signing with our own keys, never as a decryption oracle on attacker-chosen ciphertext; timing is not remotely observable. Fix requires a major `rsa` chain upgrade. | 2027-11-06 |
| RUSTSEC-2026-0183 / -0184 | git2 0.20.4 | UB in `Remote::list()` and `Signature` from a buffer. git2 is used by the git-owned bot sync against repositories we control; 0.21 upgrade tracked. | 2027-11-06 |
| RUSTSEC-2024-0429, GHSA-wrw7 | glib 0.18.5 | Unsound iterator impls on the Tauri Linux desktop stack only; no server runtime exposure. Fixed by the GTK4 stack upgrade on the desktop track. | 2027-11-06 |
| RUSTSEC-2026-0235 | rkyv 0.7.46 | **Lockfile-only**: present in `Cargo.lock` but never selected by any shipping feature graph (`cargo deny --all-features` does not see it). No exposure. | 2027-11-06 |
| Unmaintained crates: RUSTSEC-2024-0370, -0436, -2025-0075/-0080/-0081/-0098/-0100, -2026-0192/-0249/-0370/-0371/-0372/-0441 | proc-macro-error, paste, unic-*, ttf-parser, smartstring, zvariant, zbus_names, pin-utils | Build-time proc macros, Unicode tables, font metrics, office write path, or the Tauri D-Bus stack. No attacker-reachable runtime sink identified. | 2027-11-06 |
| GHSA-4j2p-28q2-5m79 / PYSEC-2026-3804 | accelerate 1.1.1 | No patched release for the sharded-checkpoint path traversal. Only operator-run local model tooling; the service is not exposed to end users and loads checkpoints the operator places on disk. | 2027-11-06 |

Fixed by the 2026-11-06 lockfile refresh (no acceptance needed):
RUSTSEC-2026-0190, -0204, -0258, -0340, -0341, -0381, -0385, -0386, -0418
and two yanked crates (`chacha20`, `spin`). The lockfile remains the record;
`cargo-deny` re-checks it on every push.

---

## Upgrade Clusters (tracked follow-ups)

These are the parent upgrades that would delete the residual entries above.
They are deliberately not bundled into the security change because each one
changes a public API and needs its own regression pass.

| Cluster | Current | Target | Removes |
|---------|---------|--------|---------|
| office-document XML stack | quick-xml 0.31–0.39 via `docx-rs` 0.4.20, `ooxmlsdk` 0.3.0, `umya-spreadsheet` 2.3.3, `calamine` 0.26.1 | quick-xml 0.41 across the six first-party consumers + calamine 0.38 | RUSTSEC-2026-0194/-0195/-0436 |
| zip in office crates | zip 0.6.6 / 2.4.2 / 4.6.1 | a parent release that accepts zip ≥ 9.0.5 | RUSTSEC-2026-0367/-0368/-0369/-0429, GHSA-8hxf/-m665/-q7v9 |
| git2 | 0.20.4 | 0.21.0 | RUSTSEC-2026-0183/-0184 |
| rsa / crypto-bigint | rsa 0.9.10, crypto-bigint 0.5.5 | rsa 0.10 (crypto-bigint 0.7.x) | RUSTSEC-2023-0071, RUSTSEC-2026-0387 |
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
- **`cargo check` of the refreshed lockfile was not run on the authoring host**
  (OOM, tracked in #1755). The lockfile is valid TOML and `cargo-deny` resolves
  it, but the next CI build is the first full compile verification.
- **Secret rotation is operator work.** The register above is complete, but the
  rotations themselves need provider/Vault access and are not automatable from
  CI.
- **Branch protection** on `main` (required reviews, no force-push, signed
  commits) is a repository setting, not code; it must be verified in both
  remotes by an administrator.

---

## Policy History

| Date | Change |
|------|--------|
| 2026-11-06 | Rewrote this file for #1723/#1725/#1726/#1727: rotation register, dated residual register, upgrade clusters, coverage gaps, gate documentation. Removed the unreferenced SheetJS bundle. |
| 2026-11-06 | Refreshed `Cargo.lock` security updates and removed the unused `html2md` dependency. |
