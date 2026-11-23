# OSV, VEX and NVD advisory sync

The security bulletin publishes machine-readable advisories on three surfaces,
all rendered from one canonical source so their identifiers and scores can
never drift:

| Surface | Location | Source |
|---------|----------|--------|
| Human bulletin + CSAF 2.0 | `/security/advisories/<ID>` | canonical advisory record |
| OSV records | `/security/osv/index.json`, `/security/osv/<GBSA-ID>.json` | canonical advisory record |
| OpenVEX statements | `/security/vex/index.json`, `/security/vex/<GBVEX-ID>.json` | `security/vex/statements.json` |

The canonical advisory record is
`botlib/src/security/advisory/data/advisories.json`. Editorial authoring happens
in `security/advisories/` (see `CHECKLIST.md`); VEX claims are authored in
`security/vex/statements.json`. Generated JSON is never edited by hand.

## Generate and validate (offline)

```bash
python3 scripts/security/osv_sync.py generate       # (re)write security/osv + security/vex
python3 scripts/security/osv_sync.py check          # parity, VEX example, sync procedure
python3 scripts/security/osv_sync.py --self-test    # fail-closed tests in a temp dir
python3 scripts/security/osv_sync.py emit-submission --out /tmp/osv-queue
```

`generate` is deterministic and atomic: records are validated before any file
is replaced. `check` is the gate and fails CI when:

- an advisory in the canonical/CSAF set has no OSV record with the **same ID**
  (or a stray OSV record exists),
- an OSV record's CVSS severity, `modified` date or embedded ID does not match
  the canonical advisory,
- a VEX statement is missing, has an invalid `not_affected` justification,
  lacks a precise product version range, or is stale after a revision.

## OSV record mapping

- `id` is the stable `GBSA-YYYY-NNNN` advisory ID — identical to CSAF. Aliases
  (a CVE, once assigned) are added to the canonical record, not invented here.
- `severity[].type` is `CVSS_V4` for a 4.0 vector and `CVSS_V3` for 3.x; the
  `score` is the verbatim vector string from the advisory.
- `affected[].ranges` are SEMVER `introduced`/`fixed` events; the authored range
  text is preserved in `database_specific.authored_range`.
- `modified` is the advisory's `updated` date (or `published` for a first
  revision), so a revision is visible to OSV consumers immediately.

Records under `security/osv/` are self-published at `/security/osv/`.
`emit-submission` writes the same records as a ready-to-post payload; it never
opens a network connection. Actual publication to OSV.dev is performed by the
security team or the scheduled CI job, and only for advisories whose component
maps to an OSV-supported ecosystem (for example `npm` for botui). General Bots'
own binary releases use the `GeneralBots` ecosystem in the self-hosted feed.

## VEX statements

Issue a VEX statement when an upstream dependency CVE is known but **not
exploitable** in a specific General Bots configuration. Each statement records:

- the CVE identifier and a factual description;
- `status: not_affected` and one OpenVEX justification — `component_not_present`,
  `vulnerable_code_not_present`, `vulnerable_code_not_in_execute_path`,
  `vulnerable_code_cannot_be_controlled_by_adversary` or
  `inline_mitigations_already_exist`;
- the **precise product versions** the claim covers, as purls plus an explicit
  version range in `database_specific.products`;
- a reason in `impact_statement`, a revision number and `last_updated`.

`security/vex/GBVEX-2026-0001.json` is the worked example: Apache Log4j2
(CVE-2021-44228) is `not_affected` because `component_not_present` — no shipped
General Bots product (server, suite or desktop client, any version `>= 1.0.0`)
contains Java or `log4j-core`. A VEX claim is reissued with a new revision when
the dependency inventory or the affected version range changes.

## NVD/CVE handoff via the CNA

NVD/CVE publication is a handoff to the CNA chosen for our advisories (tracked
in the security bulletin epic), not a second authoring step:

1. Reserve the `GBSA-YYYY-NNNN` ID locally and keep the advisory in
   `status: review` until two people approve it (see `CHECKLIST.md`).
2. Request a CVE from that CNA only after the fix version is known. Record the
   assigned CVE as an alias in the canonical record; never put a self-assigned
   `CVE-` ID in the `GBSA` ID field.
3. Submit the CNA's CVE JSON 5 record and the OSV record from the **same**
   canonical advisory. Copy the title, summary, CVSS vector, CWE, affected
   ranges and references verbatim so the NVD description and CPE data match
   the bulletin. CPE names must use the product/version pairs from
   `affected_versions`, not free-text product names.
4. Submit the CNA package to NVD and, where the component maps to an
   OSV-supported ecosystem, post the OSV record to OSV.dev. Record the
   submission date and the resulting CVE/OSV IDs in the advisory's
   `references`.
5. Attach the VEX statement to the CVE's NVD entry when the not-affected claim
   is relevant to downstream scanners.

## Keeping OSV and NVD in sync on revision

A post-publication content change follows this order — never edit a generated
file or an NVD/OSV record directly:

1. Add a new `revision_history` row and a new `updated` date in the canonical
   advisory (or bump `revision` in `security/vex/statements.json`).
2. Re-run `osv_sync.py generate` and commit the regenerated OSV/VEX files with
   the source change.
3. Run `osv_sync.py check`; it fails if any ID, CVSS vector, version range or
   `modified` date is out of sync.
4. Rebuild the bulletin mirror (`scripts/bulletin_mirror.py`) so the new tree
   digest seals the revised feed.
5. Re-submit the updated record to OSV.dev and send the CNA a revised CVE JSON
   5 record with the same revision note. Ask NVD to re-promote the CVE entry;
   confirm the published description and CPE ranges match the new advisory.
6. Close the loop in the advisory's `references` and in the tracking issue.

Part of the security bulletin epic (#2712). Related: CSAF feed (#2714),
CVE JSON 5 (#2715), immutable bulletin mirrors (#2726).
