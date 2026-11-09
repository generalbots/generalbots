---
name: analytics-kpis
description: |
  Define metrics and build dashboards that support decisions. Use for product/business analytics.
version: 1.0.0
source: in-repo
---

# analytics-kpis

## Workflow

1. Write the metric definition: formula, source table, filters, period.
2. Name the owner and the decision the metric informs.
3. Check the event/instrumentation exists before building the chart.
4. Dashboard: one question per row, trend + comparison + target.
5. Annotate releases/incidents so changes are explainable.
6. Review definitions quarterly; deprecate unused metrics.

## Quality checks

- Every chart traces to a definition.
- No metric changes formula silently.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
