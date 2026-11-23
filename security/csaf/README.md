# CSAF 2.0 feed (v1, static files only)

Consumers start at `provider-metadata.json`, then poll `changes.csv` for stable
advisory IDs and timestamps and fetch only changed documents from the versioned
year directory. The served metadata must also be aliased at
`/.well-known/csaf/provider-metadata.json`; its `canonical_url` points there.
Every advisory carries affected products and version ranges, a CVSS v3.1/v4.0
vector, CWE, remediation, an exploitability assessment derived from the CVSS
base metrics, revision history and a publication date.

None of these files is hand-authored. They are rendered from the canonical
advisory record `botlib/src/security/advisory/data/advisories.json`:

```bash
python3 scripts/security/csaf_sync.py generate   # (re)write provider metadata, advisories, changes.csv
python3 scripts/security/csaf_sync.py check      # parity + offline validation gate
python3 scripts/security/csaf_sync.py --self-test
python3 scripts/security/validate_csaf.py        # same gate, legacy entry point
```

`generate` is deterministic and atomic and removes advisories that no longer
exist in the canonical source. `check` fails when a published file is missing,
stale, hand-edited, carries an invented CVE or contains secret-like material,
so a new advisory never needs a second list to keep in sync.

Advisory IDs are the canonical `GBSA-YYYY-NNNN` tokens. No `cve` field is
emitted: a CVE is added to the canonical record only when a CNA actually
assigns one. CVSS v3.1 uses the standard `cvss_v3` object; CSAF 2.0 predates
CVSS v4.0, so a v4.0 vector is published as a `cvss_v4_0` extension and repeated
verbatim in a note and an `exploit_status` threat for strict consumers.

The feed is static files only; there is no backend dependency. Full OASIS
schema validation remains a CI concern; the commands above are offline gates
with no third-party packages and no network access.
