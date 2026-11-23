# Security reporting — privacy notice and reporter data policy

Status: **published policy**. This file is the authoritative, in-repo source for
how the vulnerability reporting channel handles personal data. It describes the
practice the reporting page, the intake mailbox and the advisory pipeline must
follow. Progress and open work live in the GitHub issues (epic #2712, this
workstream #2722), not in this document.

This notice applies only to the **security reporting channel** — the private
route a reporter uses to tell us about a suspected vulnerability. It does not
cover the community forum (which is public discussion, see
[forum-advisory-boundary.md](../../botbook/src/09-security/forum-advisory-boundary.md)),
the published advisory feed, or ordinary product telemetry. The site-wide
privacy baseline is [PRIVACY.md](../../PRIVACY.md).

---

## 1. Minimum data we ask for

The reporting form and the mailbox intake collect the smallest set of fields
that lets us investigate and reply. A working contact method is sufficient.

| Field | Required | Purpose | LGPD basis |
|-------|----------|---------|------------|
| Contact method (email address) | **Yes** | Acknowledge the report, ask follow-up questions, send the outcome | Art. 7 IX — legitimate interest in securing the service; the reporter is expected to provide a contact route |
| Report description / steps to reproduce | **Yes** | Understand and verify the finding | Art. 7 IX — legitimate interest |
| Preferred name or handle | No | Credit in an advisory or hall of fame, **only if the reporter opts in** | Art. 7 I — consent (separate, revocable) |
| Product / version affected | No | Route the report to the right maintainers | Art. 7 IX — legitimate interest |

**We do not require, and the form must not request:** CPF/CNPJ, phone number,
employer or organisation, job title, physical address, date of birth, or any
government identifier. Do not add these fields "just in case" — minimisation
under LGPD Art. 6 §1 III means we collect only what is needed for the stated
purpose.

A reporter may submit anonymously through the PGP option in §8. If no working
contact method is provided and the report cannot be clarified, we may close it
as unactionable; no personal data is retained for such a report (see §5).

## 2. Purpose and lawful basis

- **Purpose (Art. 6 §1 II):** receive, triage, investigate and remediate
  security vulnerabilities in General Bots products; notify affected operators
  under coordinated disclosure; and keep a factual record of the handling.
- **Lawful basis:** the primary basis is **legitimate interest** (Art. 7 IX) —
  the controller's interest in protecting its software and its users, which a
  reporter acting in good faith shares. Where a reporter asks for public credit
  we rely on **consent** (Art. 7 I), collected separately and withdrawable at
  any time. Where we must disclose to a competent authority we rely on
  **legal obligation** (Art. 7 III).
- **Transparency (Art. 9):** this notice is linked from the reporting page and
  from `security.txt` (see §10), and is published before any data is collected.

We do not use reporter data for marketing, profiling, automated decision-making,
or any purpose incompatible with security handling.

## 3. Confidentiality, access and encryption

- **In transit.** Reports sent by email must use the PGP key published in
  `security.txt` when the reporter wants confidentiality; the mailbox also
  accepts TLS. Reporters who do not want to use PGP may still use the standard
  contact route — they are told that email without PGP is not end-to-end
  encrypted.
- **At rest.** Intake mail and any exported case files are stored in the
  access-controlled security mailbox / encrypted drive bucket. Storage uses
  server-side encryption with keys held in the deployment's secret manager
  (Vault); keys are never written to the repository or to a report file.
  Report bodies are never copied into public issues, logs, CI artefacts,
  advisory drafts or the hall of fame unless they contain no personal data.
- **Access.** Read access is limited to the named security-team members who
  need it to triage the case ("need to know"). Publication of an advisory
  requires two different reviewers (`security/advisories/CHECKLIST.md`). Access
  grants are reviewed when staff leave the team and at the annual policy
  review. Contractors/processors may see data only under a data-processing
  agreement and only for the task they perform.
- **No local copies.** Reporters and staff must not download case material to
  unmanaged devices. Case data stays in the encrypted intake system.

## 4. Retention and deletion schedule

Retention runs from the date a report is **closed**, not from the date it is
received. Clock starts when the case state changes; state changes are recorded
in the private case tracker.

| Report state | Retention after closure | Action at expiry |
|--------------|-------------------------|------------------|
| Accepted / fixed (advisory published) | 24 months | Delete the case file and contact details; retain only the already-published advisory, which carries no personal data unless credit was consented to |
| Rejected (out of scope, no vulnerability, or cannot be reproduced) | 90 days | Irreversibly delete the report, attachments and contact details |
| Duplicate of an existing open case | 30 days | Merge into the original case, then delete the duplicate record |
| Unactionable / anonymous with no contact route | 30 days | Delete; no personal data is kept |
| Withdrawn consent for credit | immediately on request | Remove the name/handle from the advisory and hall of fame; keep the non-identifying advisory text |

Exceptions: a case under a legal hold, law-enforcement request or active
incident is retained until the hold is lifted, then deleted on the normal
schedule. Backups follow the same clock: a deleted case must age out of backup
storage within the standard backup rotation (90 days, per the security policy)
and must not be restored into active storage after deletion.

Deletion must be **implementable**, so the intake system needs: a `closed_at`
timestamp and a `state` flag on every case; a scheduled job that selects cases
past the thresholds above; and a deletion log that records the case ID and
deletion date **without personal data**. If a deployment cannot run the job,
the operator must perform and record the deletion manually at the same
cadence.

## 5. Recipients and disclosures

- **Internal:** the security team and, strictly case by case, the product
  maintainer needed to reproduce or fix the issue.
- **Processors:** the mail provider and any encrypted storage used by the
  deployment, bound by a data-processing agreement; they act only on our
  instructions.
- **External, only when necessary:** affected operators during coordinated
  disclosure, and competent authorities where legally required (Art. 7 III).
  Government early notice (CTIR Gov, CERT.br/NIC.br) is described in
  `security/advisories/CHECKLIST.md`; it shares the finding, not the reporter's
  personal data, unless the reporter consented to credit.
- We never sell, rent or publish the reporter list. There is no monetary
  incentive program; recognition (advisory credit, hall of fame) is the only
  incentive and requires opt-in consent.

## 6. Data subject rights

Under LGPD Art. 18 a reporter may request:

| Right | Article | How to exercise |
|-------|---------|-----------------|
| Confirmation of processing | Art. 18 I | Write to the security mailbox with the case reference |
| Access to the data | Art. 18 II | Same route; we provide a machine-readable summary |
| Correction of inaccurate data | Art. 18 III | Same route |
| Anonymisation, blocking or deletion | Art. 18 IV, VI | Same route; deletion follows §4 unless a legal hold applies |
| Portability | Art. 18 V | Same route |
| Information about shared recipients | Art. 18 VII | Same route |
| Withdrawal of consent (credit) | Art. 8 §5 | Same route; credit is removed prospectively |
| Review of a decision made solely by automated means | Art. 20 | Same route; triage is human-reviewed, so this right rarely applies |

Requests are answered within the statutory period (15 days for LGPD). Identity
is verified against the contact method already on the case — we do **not**
request a CPF or identity document just to honour a rights request. Requests
are free; a manifestly abusive or repetitive request may be answered with a
short explanation instead of a new copy of the data.

The controller for this processing is the operator of the General Bots
deployment that received the report; the security team acts on its behalf. The
contact route is the same mailbox published in `security.txt` — no separate
personal address is hardcoded here.

## 7. PGP-encrypted reporting (confidential option)

Reporters who want confidentiality should encrypt the message to the key whose
URL and fingerprint are published in `/.well-known/security.txt` (`Encryption:`
field) and mirrored in `SECURITY.md`. The fingerprint is the trust anchor:
compare it out of band before sending, and rotate only with an overlapping
validity window, as stated in #2713.

- Never put a private key, passphrase or the key's origin IP in the repository.
- Plaintext must not be pasted into a public GitHub issue or forum thread —
  the forum is discussion only (see the boundary document linked above).
- If a reporter has no PGP tooling, the ordinary contact route remains
  available; §3 explains the confidentiality limitation honestly.

## 8. No tracking on the reporting page

The reporting page and intake flow must not:

- log the reporter's IP address or `User-Agent` in application, access or
  client-error logs;
- set cookies (including "consent", session or analytics cookies) for an
  unidentified reporter, or store reporter data in `localStorage`;
- load third-party analytics, tag managers, pixels, session replay or external
  fonts/scripts — all assets are first-party, per the project's no-CDN rule;
- forward reporter data to the generic `/api/client-errors` sink, whose
  `CLIENT:` lines are described in `PRIVACY.md` — that sink is for product
  errors, not for report content or reporter identity.

The privacy guard (`python3 scripts/adequacy_guard.py --baseline
issues/adequacy/baseline.json`) must stay green after any change to the
reporting page.

## 9. How this notice is linked

Acceptance requires three links. This repository owns two of them; the third
belongs to the `security.txt` workstream (#2713).

1. **Reporting page:** show a short line next to the submit control:
   `How we handle your data (LGPD notice)` → this file's published URL
   (`https://generalbots.org/security/reporter-privacy/`).
2. **`security.txt`:** add an RFC 9116 field pointing at the same URL:
   `Policy: https://generalbots.org/security/reporter-privacy/`
   (the `Contact:` and `Encryption:` fields in that file already route the
   report and expose the PGP key). Ship this together with #2713.
3. **Site-wide privacy policy:** [PRIVACY.md](../../PRIVACY.md) links to this
   notice from its "Security reporting" section.

The notice itself is linked from the published bulletin page near the contact
instructions, using the same wording as item 1 so the text cannot drift.

## 10. Acceptance mapping

| Criterion (#2722) | Where satisfied |
|-------------------|-----------------|
| Privacy notice linked from the reporting page and from `security.txt` | §9 items 1–2 (copy-ready link and `Policy:` field; `security.txt` itself is #2713) |
| Reporting form collects no unnecessary personal fields | §1 field table and "do not request" list |
| No tracking cookies / no IP or UA logging / no analytics | §8; site-wide baseline in `PRIVACY.md` |
| Retention and deletion rules written down and implementable | §4 table, clock rules and scheduled-job requirements |
| Who can access, encryption at rest, deletion of rejected/duplicates | §3 access/encryption; §4 state-specific schedule |
| PGP option | §7 (key published through `security.txt` / `SECURITY.md`) |
| Site-wide privacy policy links to the notice | §9 item 3; the link is in `PRIVACY.md` |

Part of the security bulletin epic (#2712). Related: RFC 9116 `security.txt` +
coordinated disclosure (#2713), advisory template and peer review (#2724),
hall of fame (#2721), PT-BR-first WCAG pages (#2716).
