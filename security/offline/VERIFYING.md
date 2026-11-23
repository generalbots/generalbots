# Verifying the offline advisory bundle / Verificando o pacote offline

This file is shipped **inside** every bundle so an air-gapped operator can
verify it after copying it into an isolated network. Nothing below downloads
anything: `sha256sum`, `gpg`, `cosign` (with a local trust bundle) and the
included Python tool all work offline.

Este arquivo acompanha **todos** os pacotes para que um operador em rede
isolada possa verificar o conteúdo após copiá-lo. Nenhum comando abaixo acessa
a internet.

## 1. What you should have / O que você deve ter

Published next to the archive (on the mirror or release asset):

| File | Purpose |
|------|---------|
| `generalbots-advisories-v1-<prefix>.tar.gz` | immutable content bundle |
| `SHA256SUMS` | SHA-256 of every file inside the archive (the signed object) |
| `SHA256SUMS.sig` | detached signature over `SHA256SUMS` (PGP or cosign) |
| `MANIFEST.json` | publication date, advisory IDs, source commit, digests |
| `MANIFEST.sha256` | checksum of `MANIFEST.json` |
| `latest` / `latest.json` | pointer to the current stable filename |
| `keys/release.pub.asc` | release public key, when supplied at build time |

The filename prefix is the first 16 hex characters of the tree digest recorded
in `MANIFEST.json`. A different file can never reuse that name.

## 2. Verify the checksum / Verifique o checksum

```sh
# SHA256SUMS is inside the archive and next to it; sha256sum -c checks every file.
mkdir unpacked
tar -xzf generalbots-advisories-v1-*.tar.gz -C unpacked
cd unpacked
sha256sum -c SHA256SUMS
```

If `sha256sum` reports any `FAILED`, stop: do not use the bundle.

## 3. Verify the signature / Verifique a assinatura

### Detached PGP (primary offline path)

1. Obtain the release key fingerprint **out of band** (published in `SECURITY.md`
   and the bulletin) and pin it. Do not trust a key that only appears inside a
   downloaded bundle.
2. If `keys/release.pub.asc` is present, confirm its fingerprint matches the
   pinned value, then import it from the local copy:

   ```sh
   gpg --show-keys --with-fingerprint keys/release.pub.asc
   gpg --import keys/release.pub.asc
   gpg --verify SHA256SUMS.sig SHA256SUMS
   ```

   Export `GNUPGHOME` to a local directory to keep the keyring offline.

### cosign (alternative)

```sh
# The Sigstore bundle must be published next to SHA256SUMS. For an air-gapped
# host, copy the Fulcio root CA and Rekor public key into COSIGN_TRUST_ROOT and
# set COSIGN_EXPERIMENTAL=0; cosign then verifies without contacting Rekor.
cosign verify-blob --signature SHA256SUMS.sig SHA256SUMS
```

## 4. Verify with the bundled tool / Verifique com a ferramenta

From a checkout of this repository copied into the isolated network:

```sh
python3 scripts/security/build_offline_bundle.py verify \
  --bundle /path/to/published-bundle \
  --expected-revision <40-hex-source-commit> \
  --require-signature \
  --verify-command 'gpg --verify {sig} {file}'
```

`verify` re-extracts the archive, recomputes every digest, checks the tree
digest, the manifest, the `latest` pointer, the advisory IDs and that the HTML
index contains no remote assets. It makes **no network calls**. Add
`--offline`-style isolation at the OS level (unplug the cable / disable the
interface) for the acceptance test.

To prove the bundle matches the online feed at the same revision, point the
parity check at the exported feed snapshot:

```sh
python3 scripts/security/build_offline_bundle.py check-parity \
  --bundle /path/to/published-bundle --feed-dir /path/to/exported-feed \
  --expected-revision <40-hex-source-commit>
```

## 5. What to do on failure / Em caso de falha

Treat any failed check as a security incident: preserve the files, record the
command output and the observed digest, and report privately using the contact
in `security.txt` / `SECURITY.md`. Never "fix" a checksum by regenerating it on
the isolated machine — that destroys the evidence.
