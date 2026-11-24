# Offline advisory bundle / Pacote offline de avisos (v1)

This directory is copied **into** the offline advisory tarball. The tarball is
built by `scripts/security/build_offline_bundle.py` from a snapshot of the
online advisory feeds (CSAF + OSV) at one immutable source commit. It exists so
an operator in a network with no outbound internet can still read, verify and
mirror authoritative General Bots security advisories.

Este diretório é copiado **para dentro** do tarball. O pacote é gerado por
`scripts/security/build_offline_bundle.py` a partir de um snapshot dos feeds
online (CSAF + OSV) em um commit imutável.

## Layout inside the archive / Conteúdo do arquivo

```
index.html                 static index — inline CSS, no CDN, no remote fonts, no JS
csaf/...                   CSAF 2.0 provider metadata, advisories, changes.csv
osv/...                    OSV records (ids matching the CSAF advisories)
VERIFYING.md               checksum + signature verification instructions
README.md                  this file
keys/README.md             public-key pinning guidance
keys/release.pub.asc       release public key (only when supplied at build time)
SHA256SUMS                 canonical checksums of every file above
```

Published **next to** the archive (not inside it): `SHA256SUMS.sig`,
`MANIFEST.json`, `MANIFEST.sha256`, `latest`, `latest.json`. The signature and
checksum are kept outside the immutable archive so a key holder can sign the
exact bytes that were published without rebuilding the archive.

## Stable naming and `latest` / Nomenclatura estável e `latest`

```
generalbots-advisories-v1-<first-16-of-tree-digest>.tar.gz
```

- `v1` is the bundle format version. A breaking layout change increments it.
- The 16-hex prefix is derived from `SHA-256(SHA256SUMS)`, so the filename is a
  content address: identical feed snapshots always produce the identical
  filename and bytes; any change produces a different name.
- `latest` is a plain-text file containing only the current bundle filename
  plus a newline. `latest.json` adds the tree digest, source commit,
  publication date and advisory count. Mirrors copy the whole directory; a
  consumer reads `latest` to find the current archive.

To mirror: copy every published file to each surface, verify the copy with the
`verify` command in `VERIFYING.md`, then publish. Never rename the archive.

## Offline guarantees / Garantias offline

- The HTML index uses inline CSS only: no `<link>` stylesheets, no external
  scripts, no remote fonts, no CDN URLs. The build rejects any HTML feed file
  that loads a remote asset.
- Building, verifying and parity-checking use only the Python standard library
  and local files. `python3 scripts/security/build_offline_bundle.py --offline
  verify ...` blocks all socket creation and therefore proves the verification
  path needs no network.
- The `MANIFEST.json` records the publication date, the advisory IDs and the
  full source commit, so an operator can tie the copy in the isolated network
  to the exact revision published online.

See `VERIFYING.md` for the acceptance-test procedure on a machine with
networking disabled.
