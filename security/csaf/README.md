# CSAF 2.0 feed (v1, static files only)

Consumers start at `provider-metadata.json`, then poll `changes.csv` for stable
advisory IDs and timestamps and fetch only changed documents from the versioned
year directory. Every advisory carries affected products and version ranges, a
CVSS v3.1/v4.0 vector, CWE, remediation, exploitability notes, revision history
and a publication date. The shipped record is a clearly marked **example**,
not a real vulnerability.

`python3 scripts/security/validate_csaf.py` performs structural validation
without network access. Full OASIS schema validation must run in CI when a
validator is available. Feed files are published by the static site; there is
no backend dependency.
