---
name: diagrams
description: |
  Draw flowcharts, sequence diagrams and ERDs with Mermaid or drawio, with consistent notation. Use
  for process, architecture and data-model explanations.
version: 1.0.0
source: in-repo
---

# diagrams

## Workflow

1. Pick the diagram type by question: flow (process), sequence (interaction), ERD (data).
2. Write the Mermaid source in a fenced block so it can be re-rendered and versioned.
3. Keep node labels short (≤5 words); put detail in notes.
4. Use consistent shapes: rectangle=step, diamond=decision, cylinder=store.
5. Render to SVG and embed in the document with alt text.
6. Verify the diagram parses (mermaid CLI) and that arrows do not cross without reason.

## Quality checks

- Diagram parses without syntax errors.
- Every decision has both branches labelled.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
