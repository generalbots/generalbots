# Forum vs. Authoritative Advisory Channel 🟡 BETA

## Purpose

The General Bots community forum (a Go-based discussion board) and the security
bulletin serve two different purposes. Conflating them destroys clarity exactly
when clarity matters most — during an active vulnerability disclosure.

This page defines the boundary, gives forum operators copy-ready category text,
and gives moderators a written rule for conflicting claims.

> **Policy.** The forum is for **discussion**. The security bulletin at
> [`/security/`](https://generalbots.org/security/) is the **single authoritative
> source** for security advisories. A forum post never creates, amends, confirms
> or refutes an advisory.

## Channel matrix

| Channel | Purpose | Authoritative for advisories? | Who may publish |
|---------|---------|-------------------------------|-----------------|
| **Security bulletin** — [`generalbots.org/security/`](https://generalbots.org/security/) | Published advisories, CSAF feed, incident history | **Yes — canonical** | Security team only, through the reviewed advisory pipeline |
| [`/.well-known/security.txt`](https://generalbots.org/.well-known/security.txt) | RFC 9116 machine-readable reporting entry point | Routes reports; does not publish advisories | Security team only |
| `SECURITY.md` (repository root, linked from `security.txt`) | Scope, reporting channels, disclosure SLA and embargo | States policy; does not publish advisories | Security team only |
| **Community forum** → Security category | Questions, discussion, community support | **No — user-generated content** | Any community member |
| `security@generalbots.org` | Private vulnerability reports | Intake only | Reporters → security team |

### What this means in practice

- A CVE identifier, CVSS score or "we are affected" statement only becomes an
  advisory when the security team publishes it in the bulletin through the
  advisory program's review process. The advisory feed and template are tracked
  in #2714 and #2724; until a feature ships, the bulletin is the authority.
- No advisory is ever **published first** on the forum. If a researcher posts a
  finding in public, moderators point them to the private reporting channel and
  the case enters the normal disclosure process; the bulletin, not the thread,
  is where the outcome is published.
- Forum posts must not be edited to "update" an advisory. Corrections to an
  advisory go through the bulletin's correction/changelog process.
- General Bots offers **recognition** (advisory credit, hall of fame) for
  reporters, not monetary rewards. Do not promise a bounty in forum messaging.

## Forum security category — category description / pinned post

Forum operators: set the **Security** category description to the text below and
pin it as the first post. It supplies the two required canonical links and makes
the boundary explicit before anyone posts.

```text
SECURITY — DISCUSSION ONLY

This category is for discussion, questions and community support. It is NOT an
authoritative security channel, and posts here do not create or amend advisories.

• Canonical advisories & bulletin: https://generalbots.org/security/
• How to report privately (RFC 9116): https://generalbots.org/.well-known/security.txt
  (also linked from SECURITY.md in the repository)

Never post an unpatched vulnerability, proof-of-concept or personal data in a
public thread. Report it privately via security@generalbots.org or the contact
in security.txt. For a confirmed advisory, always follow the canonical bulletin
link above — not a forum thread.
```

Operational checks for this category:

- [ ] The category description contains the canonical bulletin link.
- [ ] The category description contains the `security.txt` link.
- [ ] A pinned post repeats both links and the private-reporting instruction.
- [ ] No forum UI, template or workflow can create or amend a bulletin advisory.
      Publishing remains a security-team-only pipeline action.

## Moderation rule for conflicting claims

**Rule R-SEC-1 — Conflicting or contradictory security claims.**

When a thread contradicts, disputes or misstates a **published** advisory:

1. Verify the claim against the canonical bulletin at
   `generalbots.org/security/`. The bulletin wins; the thread does not amend it.
2. Apply a visible label such as **"Community claim — not authoritative"** (do
   not silently delete the post — the discussion is valuable).
3. **Lock** the thread when the claim could cause someone to skip a patch or
   ignore a workaround.
4. Post the standard reply below with the direct link to the specific advisory:

```text
This thread is community discussion and is not an authoritative security
statement. The verified advisory and remediation steps are published at:
https://generalbots.org/security/<advisory-id>/

Please follow the canonical bulletin. If you believe the advisory is wrong,
or have a new finding, report it privately via security@generalbots.org or
https://generalbots.org/.well-known/security.txt — do not open a new public
thread. This thread is locked to prevent outdated guidance from spreading.
```

5. If the thread contains a **previously unreported** vulnerability, treat it as
   a report: ask the author to move to the private channel, start the disclosure
   process, and only publish the outcome in the bulletin.
6. Escalate repeated misinformation or deliberate advisory-spoofing to the
   security team; do not debate advisory contents in the thread.

## Note for the bulletin page

The advisory page/component owner should place this short note near the advisory
list (collection copy is provided here so wording stays consistent):

> **Discussion happens elsewhere.** General Bots advisories are published only
> here. Community questions and discussion are welcome on the community forum,
> but forum threads are user-generated and are never authoritative — always rely
> on the bulletin, the CSAF feed or `security.txt` linked above. Found a
> vulnerability? Report it privately using `security.txt`; recognition is offered,
> there is no bounty.

## Acceptance mapping

| Criterion | Where satisfied |
|-----------|-----------------|
| Forum security category links to the canonical advisory feed | Category description / pinned post above (bulletin + `security.txt`) |
| No advisory is published first on the forum | Channel matrix + operational checks; publication stays in the reviewed bulletin pipeline |
| Documentation states which channel is authoritative | "Policy" note and channel matrix on this page |
| Moderators have a written rule for conflicting claims | Rule R-SEC-1 and the standard reply above |

Part of the security bulletin epic (#2712). Related: RFC 9116 `security.txt` +
coordinated disclosure (#2713), CSAF feed (#2714), advisory template (#2724).
