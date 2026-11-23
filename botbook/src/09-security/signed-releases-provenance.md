# Signed Releases, SBOM and SLSA Provenance 🟡 BETA

Every product release (`vMAJOR.MINOR.PATCH`) carries verifiable software
identity. The `Release supply chain` workflow
(`.github/workflows/release-provenance.yml`) runs as soon as the release is
published: it inventories the build, signs every published asset with
Sigstore/cosign, generates an SLSA v1.0 provenance statement, verifies the
whole chain and only then attaches the trust material to the release.

## What is published next to the assets

| File | Purpose |
|------|---------|
| `MANIFEST.json` | Every asset with its SHA-256 and byte size, plus release, commit, ref and CI run id. |
| `SHA256SUMS` | One command check of the whole release (`sha256sum -c SHA256SUMS`). |
| `sbom-source.cdx.json` | CycloneDX SBOM covering Rust (`pkg:cargo/`), JavaScript (`pkg:npm/`) and Python (`pkg:pypi/`) dependencies, including the vendored trees. |
| `provenance.slsa.json` | in-toto Statement v1 with an SLSA v1.0 predicate: source repository, exact commit, tag, workflow, builder id and CI run invocation id. |
| `<asset>.sig`, `<asset>.pem`, `<asset>.bundle` | Detached signature, keyless signing certificate and Sigstore bundle for **every** asset, including the two metadata files above. |

Signing is **keyless**: cosign obtains a short-lived certificate from the
GitHub Actions OIDC identity of the workflow. There is no signing key, token
or certificate stored in this repository and no CDN-hosted key material.

## Fail closed

The CI gate re-checks the checksums, the manifest, the SBOM ecosystem coverage,
the provenance subjects/commit/run fields and every cosign signature. If any of
them disagrees, the job fails and immediately returns the release to draft
(unpublishes it). A release with a broken chain is never left available for
download.

## One-line verification command (operators)

Download the release assets into an empty directory, install
[cosign](https://github.com/sigstore/cosign/releases) and run:

```sh
sha256sum -c SHA256SUMS && for f in $(awk '{print $2}' SHA256SUMS); do cosign verify-blob --bundle "$f.bundle" --certificate-oidc-issuer https://token.actions.githubusercontent.com --certificate-identity-regexp '^https://github.com/generalbots/generalbots/\.github/workflows/release-provenance\.yml@refs/tags/.*$' "$f" || exit 1; done && echo VERIFIED
```

`VERIFIED` means every downloaded asset matches the published checksum **and**
carries a signature issued to the release-provenance workflow by GitHub's OIDC
provider. The regexp pins the exact workflow that is allowed to sign our
assets; change the owner/repository only for a fork you control.

To inspect the software identity, read `provenance.slsa.json` and confirm that
`predicate.buildDefinition.externalParameters.source.commit` is the source
commit you expect and that `predicate.runDetails.metadata.invocationId` names
the CI run that built it.

## Downloads

- Latest SBOM:
  [sbom-source.cdx.json](https://github.com/generalbots/generalbots/releases/latest/download/sbom-source.cdx.json)
- Latest provenance:
  [provenance.slsa.json](https://github.com/generalbots/generalbots/releases/latest/download/provenance.slsa.json)
- Combined checksums:
  [SHA256SUMS](https://github.com/generalbots/generalbots/releases/latest/download/SHA256SUMS)

## Reproducing locally

The offline helper has no third-party dependencies:

```sh
python3 scripts/release_supply_chain.py --self-test
python3 scripts/release_supply_chain.py verify --dir <directory-with-release-assets>
```

It exits non-zero on any mismatch and prints each violation with `::error::`.
