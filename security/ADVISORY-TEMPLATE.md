# Advisory template (single source of truth)

Every advisory starts from this front matter. The HTML page, CSAF, OSV and CVE
JSON records are generated from it; do not hand-write a parallel bulletin.

```yaml
id: GB-SA-YYYY-NNNN
title: Short description
summary: What happened and who is affected
affected_products: [generalbots]
affected_versions: ["< 6.9.0"]
fixed_versions: ["6.9.0"]
cvss_v4_vector: "CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N"
cvss_v4_score: 9.3
cwe: CWE-79
capec: CAPEC-591
attack_prerequisites: Unauthenticated network access
impact: Token theft
remediation: Upgrade to 6.9.0
workarounds: Disable the affected route
exploitability: no known public exploit
timeline: ["2026-11-23 reported", "2026-11-23 fixed"]
credits: Reporter name (with consent)
references: []
revision_history: ["1 — 2026-11-23 initial"]
author: name
reviewer: name
```
`security/cvss_check.py` rejects a record whose declared score disagrees with
its vector; publication requires two-person review (author + security
reviewer) recorded above.
