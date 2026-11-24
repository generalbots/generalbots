---
# Canonical security advisory source. Copy this file to GBSA-YYYY-NNNN.md and
# replace every placeholder. This template is itself a valid draft: run
# `python3 scripts/validate_advisory.py security/advisories/` before publishing.
# HTML, CSAF (#2714), OSV (#2718) and CVE JSON 5 (#2715) are GENERATED from
# this front matter — never write those formats by hand.
id: GBSA-2026-0000                 # stable advisory ID; never self-assign CVE-YYYY-NNNN
title: "TEMPLATE — short vulnerability title"
summary: "One paragraph: what the flaw is, who is affected and the worst outcome."
status: draft                       # draft | review | published | withdrawn
severity: critical                  # MUST equal the CVSS v4.0 qualitative rating below
published: null                     # ISO date; required when status: published
updated: 2026-11-23

product: "General Bots Server"
affected_versions:
  - ">= 6.8.0, < 6.8.12"           # version ranges, not single builds
fixed_versions:
  - "6.8.12"

cwe:
  - CWE-79                          # one or more CWE IDs (https://cwe.mitre.org)
capec:                              # optional; delete the key when not applicable
  - CAPEC-63

cvss:
  version: "4.0"
  vector: "CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N"
  base_score: 9.3                   # computed by scripts/cvss4.py — never typed by hand
  base_severity: critical           # none|low|medium|high|critical (0.0/0.1-3.9/4.0-6.9/7.0-8.9/9.0-10.0)

attack_prerequisites: "What an attacker needs before exploitation (AV/AC/AT/PR/UI in prose)."
impact: "Confidentiality / integrity / availability consequences (VC/VI/VA/SC/SI/SA in prose)."
remediation: "Exact fixed version and upgrade command."
workarounds: "Temporary mitigations if upgrading is not immediately possible, or 'None'."
exploitability_status: none         # none | poc | active

timeline:
  - date: 2026-11-01
    event: "Report received"
  - date: 2026-11-22
    event: "Fix verified by security reviewer"
credits:
  - name: "Reporter name or handle"
    role: reporter                  # reporter | researcher | internal
references:
  - "https://www.first.org/cvss/v4.0/"
embargo:
  early_notice: false               # true -> government early notice; recipients + until required
  recipients: []
  until: null                       # ISO date; see security/advisories/CHECKLIST.md

review:
  author: "TBD — advisory author"
  security_reviewer: "TBD — different person, security team"
  approved: false                   # MUST be true before status: published
  reviewed_at: null                 # ISO date of the two-person review

revision_history:
  - version: 1
    date: 2026-11-23
    change: "Initial draft"
---

# TEMPLATE — short vulnerability title

## Summary

One paragraph describing the vulnerability, the affected product and the
worst-case outcome. Keep it factual; the severity rating comes from the CVSS
vector above, not from adjectives here.

## Affected products

List every affected product and version range from `affected_versions`, plus
the `fixed_versions` that resolve it. State explicitly when a product or
platform is **not** affected.

## Impact

Explain the confidentiality, integrity and availability impact on the
vulnerable system and on any subsequent systems, matching the VC/VI/VA and
SC/SI/SA values in the vector.

## Remediation

Give the exact upgrade path and verification step. Prefer a version number
over "latest".

## Workarounds

Temporary mitigations when upgrading is not immediately possible, or `None`.
Do not claim a workaround that has not been tested by the reviewer.

## Exploitability

State the `exploitability_status` (`none`, `poc` or `active`) and cite the
evidence. Any change here requires a new revision entry and a re-review.

## Timeline

Reproduce the `timeline` entries in chronological order, including the report,
the fix and the peer review. Dates are ISO `YYYY-MM-DD`.

## Credits

Reproduce the `credits` entries. There is no bug bounty program and the reward
budget is 0; recognition is the only incentive.

## References

Reproduce `references` (advisory URL, fix commit, CWE/CAPEC pages). Never add
a CDN link or an unverified third-party write-up.

## Revision history

Reproduce `revision_history` with monotonically increasing version numbers.
Every content change after publication adds a row; metadata-only corrections
are a new revision too.
