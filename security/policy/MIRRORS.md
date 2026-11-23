# Immutable mirrors and always-available hosting

The authoritative feed is static: `security/csaf/`, `security/osv/` and
`security/pages/` are published verbatim to the website host. Mirrors are
append-only, content-addressed copies: a path includes the advisory ID and
revision, so a published document is never overwritten. Hosts serve the same
bytes from independent networks; operators verify checksums from
`MANIFEST.sha256` before trusting a mirror.

`scripts/security/verify_mirror.py <mirror-dir>` compares every local advisory
file with the mirror copy and reports missing or modified documents. A mirror
that fails verification must be removed from rotation. The bulletin status page
records mirror availability; no advisory depends on a single host.
