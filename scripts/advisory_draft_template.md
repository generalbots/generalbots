> **DRAFT — DO NOT PUBLISH OR QUOTE EXTERNALLY.**
> Opened automatically by the scheduled dependency scan (#2725). This is a
> triage stub, not an advisory. Publication requires the two-person review
> gate in #2724 and a separate, manual publish step — this workflow never
> writes CSAF, OSV or CVE documents.

## Prefilled from the scanner

| Field | Value |
|-------|-------|
| Advisory / finding ID | `$advisory_id` |
| Ecosystem / source | `$ecosystem` (`$source_file`) |
| Package | `$package` |
| Installed version | `$installed_version` |
| Vulnerable range | `$vulnerable_range` |
| Severity | `$severity` |
| CVE(s) | $cves |
| CWE(s) | $cwes |
| Aliases | $aliases |

**Summary:** $summary

**Source links:**

$references

Full triage report: `$report_ref` (workflow artifact).

## Triage checklist (human)

- [ ] Confirm the package and version range actually ship in a supported build
      (check shipping feature graph, not just the lockfile).
- [ ] Check reachability: record a written argument if this is not affected in
      a given configuration (VEX — #2718), or accept it as a waiver below.
- [ ] Convert this stub into the canonical advisory template — CVSS v4.0
      vector + score, CWE, CAPEC, impact, remediation, timeline, credits
      (#2724). Validate the vector against the score before proceeding.
- [ ] Assign an **author** and a **second security reviewer**; record both in
      the advisory metadata. Two distinct people must approve (#2724).
- [ ] Check for duplicates against the CSAF feed (`changes.csv`, #2714),
      published OSV records (#2718) and already-open `advisory-draft` issues.
- [ ] Decide embargo / early-notice for government customers per #2724 policy.
- [ ] Only after both approvals: publish through the CSAF/OSV process. Then
      add the advisory ID to `scripts/published_advisories.txt` so the scan
      deduplicates it on the next run.

## If this is a false positive / accepted risk

Add a `[[waiver]]` to `scripts/advisory_waivers.toml` with this finding ID, a
written reason, an **approver** and an **expires** date (ISO `YYYY-MM-DD`).
An expired waiver stops suppressing and this draft is reopened automatically.
Every waiver appears in the next scan report with its approver and expiry.

/cc @security-team
