# Immutable Bulletin Mirrors and Always-Available Hosting

Issue #2726. The advisory site is a **trust product**: an incident is the worst
possible moment for it to be unreachable. The site is a static tree with no
server-side code, no database and no admin panel, published to independent
hosting providers. There is no dynamic path an attacker can exploit to silence
an advisory, and no single provider whose failure can take the bulletin down.

This chapter documents the hosting model, the named roles, the verification
commands and the break-glass path. It is an operations runbook; the code that
implements it is `scripts/bulletin_mirror.py` and
`.github/workflows/bulletin-mirror.yml`, with an independent probe from
`.forgejo/workflows/bulletin-mirror.yml`.

## Hosting model

```
generalbots/website @ <immutable commit>
        │  (static files only: security/, .well-known/, SECURITY.md)
        ▼
scripts/bulletin_mirror.py build
        │  SHA256SUMS + manifest.json + bulletin-<tree-prefix>.tar.gz
        │  cosign sign-blob (keyless OIDC) → SHA256SUMS.bundle
        ├──────────────────────────────────────┐
        ▼                                      ▼
Surface A: GitHub release asset         Surface B: independent git host
bulletin-<tree-prefix> tag               <tree-prefix>/ path + matching tag
github.com/generalbots/website           (second provider, e.g. Codeberg)
```

- The **revision** is the full 40-character commit SHA of `generalbots/website`.
  It is recorded in `manifest.json`; a different commit can never claim the
  same manifest.
- The **tree digest** is `SHA-256(SHA256SUMS)`, where `SHA256SUMS` is the sorted
  list of every file digest in the tree. Adding, removing or changing one file
  changes the digest, so the bundle filename
  `bulletin-<first 16 hex of tree digest>.tar.gz` is a content address.
- The bundle is a reproducible tar.gz (fixed modes, `mtime=0`, sorted entries,
  gzip without a timestamp). Rebuilding the same source produces the same
  bytes; a rebuild with different bytes is a different revision.
- `cosign sign-blob` signs `SHA256SUMS` with a keyless GitHub OIDC identity.
  The Sigstore bundle is published alongside the files and verified *in CI*
  before anything is uploaded. Checksums alone are not enough: an attacker who
  can write to one host could replace a file and its checksum together; the
  signature binds the checksum to our CI identity.
- Neither surface runs code. The git-owned advisory repo has branch protection
  (required review, passing status checks, no force push, no deletion) and
  requires signed commits. These are **repository settings**, not files; the
  Advisory Repository Owner owns them (see roles below).

## Published artifacts

Every publication carries these files, on both surfaces:

| File | Purpose |
|------|---------|
| `SHA256SUMS` | one `sha256  path` line per tree file, sorted; the signed object |
| `SHA256SUMS.bundle` | Sigstore bundle: keyless signature over `SHA256SUMS` |
| `SHA256SUMS.sig` | detached signature, convenience copy |
| `manifest.json` | schema, revision, tree digest, counts, bundle name and digest |
| `bulletin-<tree-prefix>.tar.gz` | deterministic, content-addressed static tree |

A consumer verifies without trusting the host:

```bash
python3 scripts/bulletin_mirror.py verify --bundle /path/to/downloaded/bundle
python3 scripts/bulletin_mirror.py check-mirrors \
  --url "$BULLETIN_SURFACE_A" --url "$BULLETIN_SURFACE_B" \
  --expected-revision "$BULLETIN_REVISION"
cosign verify-blob --bundle SHA256SUMS.bundle \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp 'bulletin-mirror\.yml@' SHA256SUMS
```

`verify` re-extracts the tar, recomputes every file digest and compares the
canonical `SHA256SUMS`, the tree digest, the counts and the bundle digest. It
rejects symlinks, traversal paths and non-regular tar members. `check-mirrors`
fetches metadata over HTTPS only, refuses a redirect that downgrades to
plaintext, and fails on revision drift or differing tree digests. Its offline
self-test (`--self-test`) proves determinism, tamper detection and drift
detection; CI runs it on every trigger.

## Roles

Roles, not individuals, hold the authority; the named people filling them are
recorded in the private operator record and reviewed quarterly.

| Role | Responsibility |
|------|----------------|
| **Advisory Repository Owner** | Holds admin on `generalbots/website` and the mirror repo; owns branch protection, signed-commit enforcement and the small write-access list. Never publishes alone in an emergency — that is the Break-Glass Publisher's job. |
| **Advisory Publisher (primary)** | Normal path: lands the advisory commit, dispatches the mirror workflow with its full SHA, records the resulting revision and tree digest in the repository variables. |
| **Advisory Publisher (secondary)** | Holds the second token; can publish when the primary identity is unavailable. Reviews the advisory content before dispatching. |
| **Verifier (peer reviewer)** | Required code review on every advisory and on any change to the mirror tooling. Independently runs the verification commands after publication. |
| **Incident Commander (IC)** | Declares a break-glass event, names the Break-Glass Publisher for that incident and records the decision in the private incident log. |
| **Break-Glass Publisher** | Named by the IC for one incident; the only identity allowed to bypass the normal review queue under time pressure. Publishes and then hands back to the normal path. |
| **Mirror Operator** | Owns the two hosting surfaces, DNS and the status page; responds to an uptime alert from either CI host. |

Write access to the advisory repo and to the mirror repo is limited to the
Advisory Repository Owner plus the two Advisory Publishers. Every commit is
signed; branch protection requires one Verifier review and green status
checks. No other identity, bot or service account can push to `main`.

## Normal publication

1. Land the advisory on `generalbots/website` `main` through reviewed, signed
   commits (the advisory template from #2724 applies).
2. Copy the full commit SHA from the merge.
3. Dispatch `bulletin-mirror.yml` with `website_ref` set to that SHA. The
   workflow checks out exactly that commit, builds the bundle, signs with
   cosign, verifies the signature, then publishes to both surfaces.
4. Confirm the workflow summary shows the revision and tree prefix. Set the
   repository variables `BULLETIN_REVISION` and `BULLETIN_TREE_DIGEST` to the
   published values so the daily drift job has a fixed expectation.
5. The Verifier independently runs `check-mirrors` and the cosign verification
   against the two public URLs and records the output in the incident/advisory
   record.

Required repository configuration (values live in the repository settings,
never in the source tree):

| Name | Kind | Purpose |
|------|------|---------|
| `BULLETIN_SURFACE_A` | variable | primary base URL containing the `<PREFIX>` placeholder |
| `BULLETIN_SURFACE_B` | variable | second-provider base URL with `<PREFIX>` |
| `BULLETIN_REVISION` | variable | 40-hex website commit currently published |
| `BULLETIN_TREE_DIGEST` | variable | full tree digest; first 16 hex are `<PREFIX>` |
| `BULLETIN_STATUS_URL` | variable | public status page probed from both CI hosts |
| `BULLETIN_PUBLISH_TOKEN` | secret | write access to `generalbots/website` |
| `BULLETIN_MIRROR_REPO` | secret | git URL of the second-host mirror repo |
| `BULLETIN_MIRROR_TOKEN` | secret | push access to that mirror repo |

## Break-glass procedure

Use only when the main site is unavailable, the primary identity cannot act,
or a critical advisory must be published faster than the normal review queue.
The IC declares the event and names one Break-Glass Publisher; every action is
recorded in the private incident log with timestamps. The budget is zero —
there is no bounty and no reward to announce in the advisory.

1. **Declare.** IC opens the private incident record, states the impact and
   names the Break-Glass Publisher and the Verifier for this incident.
2. **Author.** The Break-Glass Publisher creates the advisory commit on
   `generalbots/website` from a signed local commit. If branch protection
   cannot be satisfied in time, the Repository Owner temporarily admits the
   named publisher to the write list — no widening to other identities and no
   disabling of signed commits.
3. **Build and sign locally** (the CI path is preferred; this is the fallback):

   ```bash
   git -C website rev-parse HEAD                       # full SHA
   python3 scripts/bulletin_mirror.py build \
     --source website --revision <SHA> --out /tmp/bulletin-breakglass
   python3 scripts/bulletin_mirror.py verify --bundle /tmp/bulletin-breakglass
   ```

4. **Publish to both surfaces.** Prefer dispatching `bulletin-mirror.yml` with
   that SHA. If GitHub Actions is itself unavailable, publish the same
   `/tmp/bulletin-breakglass` files manually to both surfaces under the tree
   prefix from `manifest.json`, and attach a detached signature produced with
   the organisation's release key (keyless OIDC can only be minted by CI).
   Never publish to only one surface except as an explicitly time-boxed first
   step while the second is being restored.
5. **Verify before announcing.** The Verifier runs `verify` on the downloaded
   bundle, `check-mirrors` across both public URLs and the signature check.
   The advisory is announced only after both surfaces agree on the same
   revision and tree digest.
6. **Restore the normal path.** Within one business day the IC closes the
   exception, branch protection and the write list are back to baseline, the
   repository variables are updated to the break-glass revision, and the
   Verifier files the evidence (file:line, command output, screenshot paths if
   a page was involved) in the private record.

## Uptime and independence

- The GitHub workflow verifies the two surfaces daily from a GitHub-hosted
  runner, outside the primary hosting provider; `.forgejo/workflows/` repeats
  the probe from the ALM host. Two independent networks must both see HTTP 200
  on the status page and identical digests on the mirrors.
- A probe failure pages the Mirror Operator. A digest mismatch is treated as a
  security incident: preserve the artifacts, compare them with the signed
  `SHA256SUMS`, and involve the IC before republishing.
- Restore drills follow the quarterly cadence in `SECURITY.md`: rebuild a
  bundle from the pinned commit, verify it, and confirm both URLs still serve
  that revision.

## Acceptance evidence (#2726)

| Criterion | Evidence |
|-----------|----------|
| Two URLs, same content and revision | `check-mirrors` output from both CI hosts; immutable tags on both surfaces |
| Tree-level checksum/signature published and verified in CI | `manifest.json`, `SHA256SUMS`, `SHA256SUMS.bundle`; `verify` + cosign `verify-blob` steps in the publish job; `--self-test` result |
| Break-glass procedure with named roles | Roles and six-step procedure above; private incident record holds the named people |
| Uptime from outside the primary provider | Scheduled probes from the GitHub and ALM runners, logged in each workflow run |
