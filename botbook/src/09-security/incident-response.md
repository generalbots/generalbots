# Incident Response and Publication Authority

Issue #2721. This chapter documents how a security report becomes a public
advisory, who owns each decision, and how we escalate to Brazilian government
CSIRTs. It is an operations runbook: progress and open work live in the
security-bulletin epic (#2712), not in this file.

**There is no bug bounty program. Reward budget is 0.** Recognition — advisory
credit and the [hall of fame](https://generalbots.org/security/hall-of-fame.html)
— is the only incentive. Never add bounty or reward language to a published
page, advisory or notice.

## Roles, not people

Authority is held by roles. The named people filling each role are kept in the
private operator record, reviewed quarterly and rotated on leave; no personal
addresses appear in advisories, the status page or these docs. Contact the
roles through the public channel in
[`/.well-known/security.txt`](https://generalbots.org/.well-known/security.txt),
never through a private address copied into a repository.

| Role | Owns |
|------|------|
| **Triage Owner** | First response, acknowledgement, severity scoring, duplicate detection and the decision to open an incident record. |
| **Incident Commander (IC)** | Declares an incident, names the Break-Glass Publisher when the normal queue is too slow, keeps the private incident log and records every decision timestamp. |
| **Advisory Author** | Writes the advisory from the canonical template (#2724), recomputes CVSS v4.0 with `scripts/cvss4.py` and drafts the early-notice text. |
| **Verifier (peer reviewer)** | Independent reviewer: confirms score, affected ranges, fix, embargo and early notice. Must be a different person from the Advisory Author. |
| **Publication Approver** | The role that clears `review.approved: true` and releases an advisory to publication. Clears publication only after the Verifier's approval. |
| **Advisory Repository Owner** | Holds admin of the static site and mirror repos, branch protection and signed-commit enforcement (#2726). Never publishes alone in an emergency. |
| **Advisory Publisher (primary / secondary)** | Executes the reviewed publication and mirror dispatch; the secondary holds the backup token. |
| **Government Liaison** | Sends embargoed early notice to CTIR Gov, CERT.br/NIC.br and the affected agency, and records recipients in the advisory front matter. |
| **Mirror Operator** | Owns the two hosting surfaces, DNS and the public status page; responds to uptime alerts. |

Separation of duties is mandatory: the Author, the Verifier and the
Publication Approver are three distinct people for every published advisory.
Break-glass is the only exception, and the IC records who authorised it.

## Intake and triage

1. Reports arrive through the private channel published in `security.txt` and
   `SECURITY.md`. The **Triage Owner** acknowledges within **3 business days**
   and gives a severity assessment within **10 business days**.
2. Score with CVSS v4.0 (`python3 scripts/cvss4.py 'CVSS:4.0/...'`). Government
   criticality may raise, never lower, the effective response priority. An
   actively exploited finding is treated as Critical regardless of score.
3. The Triage Owner opens a private incident record, assigns an IC when the
   finding is High/Critical or affects a government deployment, and marks the
   reporter's consent state for credit. **Consent to be named is opt-in** and
   never assumed from the fact that a report was submitted.
4. Under embargo the case is `TLP:AMBER`. The vector, affected versions, PoC
   status and early-notice list are restricted to the named early-notice
   recipients until `embargo.until`. The standard embargo is **90 days**; an
   extension needs a recorded revision entry and reviewer approval.

## Who approves publication

1. **Author** drafts `security/advisories/GBSA-YYYY-NNNN.md` from
   `TEMPLATE.md` and runs `python3 scripts/validate_advisory.py`.
2. **Verifier** independently checks the recomputed score, affected ranges,
   remediation, credits and early notice, and sets `review.approved: true`.
3. **Publication Approver** clears the advisory for `status: published`, sets
   `published` and `review.reviewed_at`, and hands it to the Publisher.
4. **Advisory Publisher** lands the reviewed commit and dispatches the signed
   mirror workflow (#2726). The Verifier re-checks both surfaces.
5. **Publication surfaces** are updated from the same source in the same
   review: the canonical advisory, the CSAF/OSV/VEX feeds (#2714, #2718), the
   incident history (`security/incidents/index.json`), the
   [status page](https://generalbots.org/security/status.html), and — when the
   reporter opted in — the hall of fame registry.

No person or bot outside this chain can create, amend or silently suppress an
advisory. If the main site is down or a critical advisory must beat the normal
queue, the IC declares break-glass, names one Break-Glass Publisher and records
it in the private log; normal review resumes immediately afterwards (#2726).

## Escalation to CERT.br / CTIR Gov

The **Government Liaison** acts on behalf of the Publication Approver and uses
only the public contact channels published by the receiving organisations:

| Organisation | Role | When |
|--------------|------|------|
| **CTIR Gov** | Federal CSIRT — Brazilian government incident response | Critical/High findings affecting government deployments: early notice **at least 30 days** before public disclosure, and immediate notice for active exploitation. |
| **CERT.br / NIC.br** | National CSIRT — coordination with Brazilian operators | Same early-notice window; coordination when third-party or supply-chain components are involved. |
| **Affected agency security team** | Local owner of the deployment | Early notice before disclosure; receives the remediation and verification steps. |

Escalation steps:

1. Government Liaison sends the `TLP:AMBER` notice through the CSIRT's published
   channel, with advisory ID, CVSS vector, affected ranges, fixed version and
   the agreed disclosure date. No exploit steps or PoC are attached.
2. Record each recipient in `embargo.recipients` and the send time in
   `timeline` before the publication gate can clear.
3. If a CSIRT requests an extension, log it in `revision_history`, obtain
   Verifier approval and update `embargo.until`. Do not publish early.
4. If a CSIRT reports active exploitation of a lower-scored finding, the IC
   reclassifies it as Critical and the 30-day window is replaced by immediate
   notice.

No private e-mail address, internal IP, token or credential is stored in the
status page, the incident index, the hall of fame registry or this chapter.
Addresses used for escalation live in the private operator record only.

## Incident history and the status page

The [status page](https://generalbots.org/security/status.html) and its
machine-readable source `security/incidents/index.json` publish **resolved**
advisories only: date, severity, product/component, affected ranges and fixed
version. A row is added only when the advisory is published, two-person
reviewed and past its embargo. Exploit detail, PoC, reporter e-mail, internal
IPs and tokens are never rendered. Example records are marked `example: true`
so illustrative schema data cannot be mistaken for a real incident; the data
contract is in `security/incidents/README.md`.

After every publication the Publisher verifies the status page still reflects
the advisory source by running:

```bash
python3 scripts/check_security_status_pages.py
python3 scripts/validate_advisory.py security/advisories/
```

## Hall of fame: opt-in and removal

The [hall of fame](https://generalbots.org/security/hall-of-fame.html) is
backed by `security/pages/security/hall-of-fame.json`. An entry is created only after
the reporter explicitly opts in to being named, using a chosen handle. Entries
with `opt_in: false` must never be rendered, and no e-mail address is stored in
the public registry. Any reporter may request removal through
`security@generalbots.org`: acknowledge within 3 business days, withdraw the
credit from the live site within 5 business days, and omit it from the next
signed mirror publication. Previously published content-addressed bundles
cannot be edited; the Mirror Operator retires the affected bundle from the
surface indexes and publishes a new revision with the credit removed, as
described in [immutable bulletin mirrors](./immutable-bulletin-mirrors.md).
Reporter-data handling, retention and erasure follow the reporter-data policy
(#2722).

## Site integration

The status page must be linked from **both** the canonical bulletin index
(`/security/`) and the **main site footer** of `generalbots.org`, so a reader
can reach the track record without knowing the URL. Both surfaces use the same
three links:

```html
<a href="/security/">Advisories</a>
<a href="/security/status.html">Status &amp; history</a>
<a href="/security/hall-of-fame.html">Hall of fame</a>
```

PT-BR is the authoritative locale for these labels; the site i18n layer maps
the keys on every locale (#2716). The bulletin index is owned by the forum
boundary workstream (#2727); the main site footer is owned by the website
repository. This chapter records the requirement; the live link check is part
of browser UAT.

## Acceptance mapping (issue #2721)

| Criterion | Evidence |
|-----------|----------|
| Status page lists resolved advisories with dates, severity, affected versions | `security/pages/security/status.html` table, generated from `security/incidents/index.json` |
| No exploit detail before embargo ends | `security/incidents/README.md` publication rules; `embargo`/`example` checks in `scripts/check_security_status_pages.py` |
| Hall of fame requires explicit opt-in and a removal process | `hall-of-fame.json` `entry_schema.opt_in`, page opt-in/removal sections, this chapter |
| Owners and escalation contacts named as roles | Roles table and CTIR Gov / CERT.br escalation table above |
| No internal IP, token or reporter email published | No-IP/no-email/no-token gate in `scripts/check_security_status_pages.py` |
| No bug bounty, reward budget 0 | Explicit statement in both pages, the JSON registry, this chapter and `security/advisories/CHECKLIST.md` |
| Linked from bulletin and main site footer | "Site integration" section above; live link check needs browser UAT |

Part of the security bulletin epic (#2712). Related: reporter data (#2722),
advisory template and review (#2724), immutable mirrors (#2726), forum boundary
(#2727).
