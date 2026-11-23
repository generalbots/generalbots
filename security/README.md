# Security bulletin program

Static, authoritative advisory channel for government deployments. PT-BR is
the authoritative locale; English is the second locale.

| Path | Purpose |
|---|---|
| `.well-known/security.txt` | RFC 9116 contact record |
| `../SECURITY.md` | Coordinated disclosure policy, scope, SLAs, supported versions |
| `csaf/` | CSAF 2.0 provider metadata, per-year advisories, `changes.csv` |
| `osv/`, `vex/` | OSV records and OpenVEX statements |
| `pages/` | PT-BR-first advisory, status and hall-of-fame pages |
| `feeds/` | Atom/RSS feeds generated from advisory front matter |
| `advisories/` | Advisory template instances (single source of truth) |
| `policy/` | CVE strategy, subscriptions, mirrors, LGPD reporter data, API contract |
| `offline/` | Air-gapped bundle instructions |
| `ADVISORY-TEMPLATE.md`, `EDITORIAL-CHECKLIST.md` | Front-matter schema and two-person review gate |
| `FORUM-BOUNDARY.md` | The forum is discussion; this tree is authoritative |

`scripts/security/` holds the offline validators and generators. There is no
backend dependency: the bulletin is published as static files, and the
advisory API is a read-only contract that consumes the same records.

No bug bounty exists. Reward budget is zero; public credit is the only
incentive. No personal data, credentials or internal addresses are stored in
this tree.
