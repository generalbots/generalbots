---
name: pandas-analysis
description: |
  Answer data questions with pandas: joins, groupby, time series and honest plots. Use for CSV/Excel
  analysis.
version: 1.0.0
source: in-repo
---

# pandas-analysis

## Workflow

1. Load with explicit dtypes and date parsing; check shape and nulls first.
2. Clean: deduplicate by key, normalise column names, document dropped rows.
3. Join on verified keys; report unmatched rows on both sides.
4. Aggregate with groupby and name outputs; avoid chained indexing.
5. Plots: label axes and units, one message per chart, no misleading truncated axes.
6. Summarise findings with the numbers that support them.

## Quality checks

- Every join reports match counts.
- The summary can be recomputed from the script output.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
