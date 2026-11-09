---
name: invoicing-brazil
description: |
  Explain and support the NF-e/NFS-e/CT-e flow and the taxes involved. Use for Brazilian invoicing
  questions.
version: 1.0.0
source: in-repo
---

# invoicing-brazil

## Workflow

1. Identify the document type by operation: goods (NF-e), services (NFS-e), transport (CT-e).
2. Collect issuer/recipient tax ids, CFOP/NCM or service code, and the tax regime.
3. Compute ICMS/ISS/PIS/COFINS according to the regime, showing the base and rate.
4. Validate against the municipality/state rules before issuing.
5. Store the authorised XML and the protocol number.
6. Never invent a tax rate; ask for the regime and municipality.

## Quality checks

- Tax bases and rates are shown, not just totals.
- The document's legal fields match the issuer's registration.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
