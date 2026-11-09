---
name: kyc-verification
description: |
  Check identity documents with explicit human confirmation. Use for onboarding/verification flows.
version: 1.0.0
source: in-repo
---

# kyc-verification

## Workflow

1. Collect document type, number, expiry and the selfie/liveness evidence.
2. Validate format and checksums; check expiry and name consistency.
3. Compare document photo with the selfie; flag mismatches.
4. Screen against sanctions/PEP lists where the tenant provides one.
5. A human confirms the final decision; the assistant only prepares evidence.
6. Store only what policy allows; never retain raw documents longer than needed.

## Quality checks

- No automatic approval without human confirmation.
- Every check has an auditable reason.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
