# Advisory editorial checklist

This file is the authoritative, in-repo editorial standard for the security
bulletin program (#2712). It describes the process we use today; progress and
open work live in the GitHub issues, not here. Every published advisory is a
Markdown file with YAML front matter in this directory, copied from
[`TEMPLATE.md`](TEMPLATE.md). Free-form bulletins are rejected by
`scripts/validate_advisory.py`.

## Source of truth

`TEMPLATE.md` front matter is the single source. The HTML page (#2716), CSAF
2.0 feed (#2714), OSV/VEX records (#2718) and CVE JSON 5 records (#2715) are
**generated** from it — never edit a generated format by hand. A generated
artefact must carry the same `id`, CVSS vector/score, CWE, CAPEC, reviewers
and revision history as the source.

## Pre-publication checklist

- [ ] File is named `GBSA-YYYY-NNNN.md` (stable ID). No self-assigned `CVE-` ID:
      CVE issuance is decided in #2715.
- [ ] `title`, `summary`, `product` describe the flaw without speculation.
- [ ] `affected_versions` and `fixed_versions` list exact ranges; unaffected
      products are called out in the body.
- [ ] `cwe` has at least one `CWE-N` ID; `capec` has `CAPEC-N` IDs when relevant.
- [ ] `cvss.version` is `4.0`, the vector contains all 11 base metrics, and
      `base_score` / `base_severity` / top-level `severity` agree with it.
- [ ] `attack_prerequisites`, `impact`, `remediation`, `workarounds` and
      `exploitability_status` are filled and match the vector's prose meaning.
- [ ] `timeline`, `credits`, `references` and `revision_history` are non-empty
      and chronologically ordered.
- [ ] `review.author` and `review.security_reviewer` are **two different
      people**; for `status: published`, `review.approved: true`,
      `review.reviewed_at` and the `published` date are set.
- [ ] Embargo is decided: if `embargo.early_notice: true`, both `recipients`
      and `until` are set and the government notice has been sent.
- [ ] All ten `##` body headings from the template are present.
- [ ] `python3 scripts/validate_advisory.py security/advisories/` exits 0.

## CVSS v4.0 scoring

- Calculator (authoritative, FIRST reference implementation):
  <https://www.first.org/cvss/calculator/4.0>
- Specification: <https://www.first.org/cvss/v4.0/specification-document>
- Validate locally with no network and no Python dependencies:
  ```bash
  python3 scripts/cvss4.py 'CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N'
  # -> 9.3 critical
  python3 scripts/cvss4.py --self-test   # 16 reference vectors + rejection cases
  ```
- The score is recomputed from the vector by `scripts/cvss4.py`; a hand-typed
  score that disagrees by even 0.1 fails the gate. The implementation is
  cross-checked against the FIRST calculator, **not** the printed scores in
  FIRST's examples document — two of those examples are known typos (a vector
  printed as 9.4 scores 9.3; one printed as 8.2 scores 9.2).
- Publish the Base score (CVSS-B). Threat/Environmental metrics may be added
  by consumers; if present in the vector they are scored with the spec defaults
  (E = X means Attacked, CR/IR/AR = X means High).

## Severity thresholds and response

Qualitative rating comes from the CVSS v4.0 base score; the internal targets
below set publication urgency. Government criticality may raise, never lower,
the effective priority.

| Rating | Base score | Publication | Remediation target |
|--------|-----------|-------------|--------------------|
| Critical | 9.0–10.0 | Immediate, after peer review and government early notice | Hotfix |
| High | 7.0–8.9 | Within 2 business days of review | Next patch release |
| Medium | 4.0–6.9 | Within 10 business days | Next scheduled release |
| Low | 0.1–3.9 | Bundle into the next bulletin | Best effort |
| None | 0.0 | Informational only | N/A |

An `exploitability_status: active` finding is treated as Critical for response
purposes even if the base score is lower, and is early-noticed immediately.

## Embargo and government early notice

- Standard coordinated disclosure is **90 days** from the report date, matching
  the `security.txt` and `SECURITY.md` policy (#2713). An extension requires a
  recorded `revision_history` entry and reviewer approval.
- Under embargo, advisories are `TLP:AMBER`. Do not publish the vector, affected
  versions or PoC status outside the early-notice list before `embargo.until`.
- Early notice goes to Brazilian government CSIRTs and customers — **CTIR Gov**
  (federal CSIRT), **CERT.br / NIC.br**, and the affected agency's security
  team — at least 30 days before public disclosure for Critical/High findings
  affecting government deployments. Record every recipient in
  `embargo.recipients` before clearing the gate.
- No bug bounty program exists. Reward budget is 0. Recognition in `credits`
  and the hall of fame (#2721) is the only incentive; never add bounty language.

## Publication and revisions

1. Copy `TEMPLATE.md` to `GBSA-YYYY-NNNN.md`, fill every field, run the gate.
2. Move `status: draft` → `review`; the security reviewer verifies the score,
   affected ranges, remediation and early notice, then sets `approved: true`.
3. Move to `status: published`, set `published` and `review.reviewed_at`, add
   the first revision row, and run the gate again.
4. Generate HTML, CSAF, OSV/VEX and CVE JSON 5 from the source; confirm the
   generated files carry the same ID, score and reviewers.
5. Any post-publication content change adds a new `revision_history` row, a new
   review, and regenerates every format.
