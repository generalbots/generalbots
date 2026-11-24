# Incident history — data contract

`index.json` is the public, machine-readable incident history behind the
[status page](../pages/status.html). It is a **trust product**: it records what
was resolved, never how it was exploited.

## What may be published

A row enters `index.json` only when **all** of the following are true:

1. The advisory is `status: published` in its canonical source
   (`security/advisories/GBSA-YYYY-NNNN.md`, mirrored into
   `botlib/src/security/advisory/data/advisories.json`).
2. `review.approved: true` with two different reviewers (author and security
   reviewer), and `review.reviewed_at` is set.
3. The embargo has ended (`embargo.until` has passed, or `early_notice: false`).
   `embargo: "ended"` must appear on the row before it is rendered.
4. A fixed version exists; `status: "resolved"` is then set.

Before the embargo ends the finding is `TLP:AMBER`: it is **not** in this file,
not in the status page, and not in any public feed. Early notice goes only to
the named government CSIRTs (CTIR Gov, CERT.br / NIC.br) and affected agencies.

## Row schema

| Field | Meaning |
|-------|---------|
| `advisory` | Canonical `GBSA-YYYY-NNNN` id |
| `date` | ISO `YYYY-MM-DD` publication date |
| `severity` | CVSS v4.0 qualitative rating: critical / high / medium / low / info |
| `product`, `component` | Affected scope, matching the advisory front matter |
| `affected_versions` | Version ranges, verbatim from the advisory |
| `fixed_version` | First release known not to be affected |
| `status` | `resolved` (only value published here) |
| `embargo` | `ended` (only value published here) |
| `example` | `true` for illustrative records that describe no real flaw |
| `advisory_url` | Link to the canonical advisory, the source of further detail |

**Never store here:** exploit steps, proof-of-concept code, CVSS attack
prerequisites in prose, reporter email addresses, internal IP addresses,
credentials, tokens or any embargoed detail. The advisory itself carries the
factual summary; this index carries only the minimum public metadata needed
for a version/date/severity history.

## Update procedure

1. After a two-person review approves publication, copy the published fields
   into a new row. Keep IDs and dates identical to the advisory source; do not
   retype a severity — derive it from the recomputed CVSS v4.0 score.
2. Regenerate the status page table from this file. Do not hand-edit the HTML
   table and the JSON independently.
3. Run `python3 scripts/check_security_status_pages.py`. It exits non-zero on a
   mismatch with the advisory source, an embargoed row, a missing fixed
   version, or any email/IP/token-shaped string.
4. The advisory publication workflow mirrors the whole static tree, including
   this file and the status page, to both independent surfaces (#2726).

Example rows in this repository are marked `"example": true` and are rendered
as examples, so no reader can mistake illustrative schema data for a real
incident. Real records replace them only through the reviewed pipeline above.

Part of the security bulletin epic (#2712). Related: status page and hall of
fame (#2721), advisory template and peer review (#2724), immutable mirrors
(#2726).
