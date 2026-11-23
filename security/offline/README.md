# Air-gapped advisory bundle

Government deployments with no outbound network receive the same advisories as
a signed, versioned tarball. Build it on a connected host with
`python3 scripts/security/build_offline_bundle.py --out /tmp/advisories.tar.gz`;
the bundle contains the CSAF feed, OSV/VEX records, the HTML pages and a
`MANIFEST.sha256` with one digest per file.

The connected host signs the tarball with a detached minisign or PGP signature
(fingerprint published in `security.txt`). On the air-gapped host, verify the
signature with the published key, then verify every manifest entry with
`sha256sum -c MANIFEST.sha256` before importing. No key material is stored in
this repository.
