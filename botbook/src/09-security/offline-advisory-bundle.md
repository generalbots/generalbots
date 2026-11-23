# Signed Offline Advisory Bundle (air-gapped deployments)

Issue #2720. Brazilian government deployments often run in networks with **no
outbound internet**. An advisory that can only be read online is useless to the
operators who need it. This chapter documents the v1 offline bundle: what it
contains, how it is named and signed, and how to verify and mirror it on a
machine with networking disabled. The implementation is
`scripts/security/build_offline_bundle.py` (CLI) and
`scripts/security/offline_bundle_lib.py` (stdlib-only library); the fixed files
shipped in every archive live under `security/offline/`.

## What the bundle is

A deterministic tarball built from a snapshot of the online advisory feeds at
one immutable source commit:

```
generalbots-advisories-v1-<first-16-of-tree-digest>.tar.gz
  index.html          static index; inline CSS, no CDN, no remote fonts, no JS
  csaf/               CSAF 2.0 metadata, advisories and changes.csv
  osv/                OSV records with matching advisory IDs
  VERIFYING.md        checksum + signature verification instructions (offline)
  README.md           layout, naming and mirroring rules
  keys/README.md      key-pinning guidance
  keys/release.pub.asc  release public key (when supplied at build time)
  SHA256SUMS          canonical checksums of all files above
```

Published **next to** the archive:

| File | Purpose |
|------|---------|
| `SHA256SUMS` | the exact object that is signed; one `sha256  path` line per file |
| `SHA256SUMS.sig` | detached PGP signature or cosign blob signature |
| `MANIFEST.json` | publication date, advisory IDs, source commit, digests, counts |
| `MANIFEST.sha256` | checksum of `MANIFEST.json` |
| `latest` / `latest.json` | stable pointer to the current bundle filename |

The signature and checksum stay outside the immutable archive so the release
key holder can sign the exact published bytes without rebuilding the archive.
`VERIFYING.md` and the public-key guidance are *inside* the archive, so an
operator who only received the tarball still has the verification procedure.

## Name and immutability

- `v1` is the format version. A breaking layout change increments it and
  produces `v2-...` archives while v1 remains verifiable.
- The 16-hex prefix is `SHA-256(SHA256SUMS)[:16]`. `SHA256SUMS` is canonical
  (sorted paths, two spaces, LF, UTF-8), so the same feed snapshot always
  yields the same bytes and the same filename. Rebuilding with a different
  publication date, commit or feed content yields a different digest and
  therefore a different filename.
- `latest` is a text file with the current archive filename and a newline;
  `latest.json` carries the tree digest, source commit, publication date and
  advisory count. Mirrors copy all files; a consumer reads `latest` and then
  verifies the named archive. The archive is never renamed.

## Build

The build tool consumes an exported feed snapshot (the same static files that
are published online at that revision). It never fetches anything.

```sh
python3 scripts/security/build_offline_bundle.py build \
  --feed-dir /path/to/exported-feed \
  --static-dir security/offline \
  --template security/offline/index.template.html \
  --out /tmp/offline-bundle \
  --source-commit "$(git rev-parse HEAD)" \
  --publication-date 2026-11-23 \
  --public-key security/release.pub.asc \
  --key-fingerprint <full-fingerprint> \
  --sign-command 'cosign sign-blob --yes --output-signature {sig} {file}'
```

`--sign-command` is optional at build time on a developer machine; it is
mandatory in the publication job. It is executed without a shell via
`shlex.split`, with `{file}` substituted by the absolute path of `SHA256SUMS`
and `{sig}` by the output path `SHA256SUMS.sig`. `{file}` and `{sig}` are the
only supported placeholders. For detached PGP use, for example:

```
--sign-command 'gpg --batch --yes --armor --detach-sign --output {sig} {file}'
```

A detached signature must ultimately be produced by the release key holder;
CI cannot mint the organisation's release-key signature. The published
manifest records `signature_file` and the pinned `key_fingerprint` so a
consumer can tell whether a signature is expected.

## Verify (works offline)

```sh
python3 scripts/security/build_offline_bundle.py verify \
  --bundle /tmp/offline-bundle \
  --expected-revision <40-hex-source-commit> \
  --require-signature \
  --verify-command 'gpg --verify {sig} {file}'
python3 scripts/security/build_offline_bundle.py check-parity \
  --bundle /tmp/offline-bundle --feed-dir /path/to/exported-feed \
  --expected-revision <40-hex-source-commit>
```

`verify` re-extracts the archive, recomputes every SHA-256, checks the tree
digest, manifest, `MANIFEST.sha256`, `latest` pointer, advisory IDs, signature
and the absence of remote HTML assets. It rejects symlinks, path traversal and
non-regular tar members. `check-parity` proves that the bundle's advisory IDs
and file bytes match the feed snapshot at the same revision. Both commands use
only local files; the `--offline` flag blocks all socket creation as a
self-check.

Manual verification steps for operators are in `security/offline/VERIFYING.md`.
Key pinning is covered in `security/offline/keys/README.md`.

## Acceptance evidence (#2720)

| Criterion | Evidence |
|-----------|----------|
| v1 bundle readable offline, no external assets | `index.template.html` uses inline CSS only; `check_offline_html` rejects remote assets; `--self-test` includes a CDN-link rejection case |
| Signature and checksum instructions inside the bundle | `VERIFYING.md` and `keys/README.md` are copied into every archive; `SHA256SUMS` is included too |
| Same advisories as the online feed at the same revision | `MANIFEST.json` records `advisory_ids` + `source_commit`; `check-parity` compares IDs and feed bytes; `verify` re-derives IDs from the extracted archive |
| Tested with networking disabled | `--offline` blocks `socket.socket`; `--self-test` runs build, verify, signing and parity under that guard |

## Release-key requirement

The detached signature is produced by the release key holder (or by CI with a
provisioned signing key); it cannot be generated offline by this tool alone.
Until a signature is published, `verify --require-signature` fails closed, so
an unsigned or untrusted bundle can never be mistaken for an authenticated
one. The remaining work is operational, not code: provision the signing key in
the publication job and publish `SHA256SUMS.sig` on both bulletin surfaces.
