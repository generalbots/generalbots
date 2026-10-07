# [BOTMODELS] 1520 — Remove dead code

**Status:** implemented — commit `329c4d600`
**Priority:** P2
**Kind:** cleanup
**Depends on:** — · **Blocks:** —

## Problem

### `botmodels/app.py` — dead Flask + allennlp app

A 50-line legacy file: `/reading-comprehension` via
`Predictor.from_path("https://storage.googleapis.com/allennlp-public-models/transformer-qa-2020-10-03.tar.gz")`
— a **remote tarball fetched at runtime** — auth via
`hmac.compare_digest(key, 'starter')`, and `app.run(debug=True)` **at module
scope**, so merely importing it starts a server.

Not referenced by `main.py`, not in `requirements.txt` (no Flask, no allennlp),
not in the README. This is exactly the "deprecated legacy allennlp" the README
tells developers to move away from — still sitting in the tree root. It is also
the only place `hmac.compare_digest` is used today, which 1515 needs.

### `src/anomaly_detection.py` — orphaned second app

A **second, standalone FastAPI app** on port 8082, not mounted in `main.py`, with
its own `/health` and `POST /api/detect`. Nothing in `botmodels` imports it.

It returns a **different schema** from the mounted endpoint
(`anomaly_percentage` 0–100 + `statistics{}` vs `anomaly_rate` 0–1 + `summary{}`).
Its `os` import sits *after* the `health()` function that uses `os.environ` —
harmless today only because module import completes first.

### Port 8082 ambiguity

`botbook/.../multimodal.md:192` tells users to set `BOTMODELS_HOST` to port
**8082** for the vision/DETECT flow, while line 34 documents
`botmodels-port` default **8085**. `installer_regs.rs:163` also claims 8082 for a
*different* service (the embedding llama.cpp server). Three meanings, one port.

## Goal

- Delete `app.py` and `src/anomaly_detection.py`.
- Resolve the 8082 collision in the botbook.
- `/api/scoring` keeps working but stops reporting fabricated metadata.

## `/api/scoring` — 575 lines of rules pretending to be ML

Despite a docstring claiming "ML-powered lead scoring", there is no torch, no
sklearn, no checkpoint. It is a hand-written weighted rules engine.

- `/api/scoring/model-info` returns `last_trained=datetime(2025, 1, 1)` and
  `accuracy_metrics = {mql_precision: 0.85, sql_precision: 0.92,
  conversion_correlation: 0.78}` as **hardcoded literals** for a model that
  does not exist.
- `confidence` is `data_points / 8.0` — a field-completeness count, not model
  confidence.
- `custom_weights` is **accepted by the schema and completely ignored** — every
  scoring function reads `ScoringWeights` class constants.
- `GET /api/scoring/health` has **no auth** (1515).
- No Rust caller exists: `grep '/api/scoring'` across the tree returns nothing.
- Substring matching is fragile (`if key in size_lower`) and dict-order
  dependent.

## Decision needed

Delete `/api/scoring` outright, or keep it and label it honestly as a rules
engine? A public `/model-info` reporting precision metrics that were never
measured is a liability in a CRM context. If it stays: rename the fields,
return `kind: "rules-engine"`, drop the fabricated metrics, honour
`custom_weights`.

## Acceptance criteria

- [x] `app.py` and `src/anomaly_detection.py` deleted; nothing references them.
- [ ] Port `8082` documented once, with one meaning.
- [x] `/api/scoring` either removed or reports no unmeasured metrics.
- [ ] `custom_weights` either honoured or removed from the schema.

### What landed

Both files deleted; repo-wide grep confirms no dangling references.
`/model-info` now returns `kind: "rules-engine"` via `/health`,
`last_trained: None`, and `accuracy_metrics: {}` — the fabricated
`mql_precision: 0.85` / `sql_precision: 0.92` / `conversion_correlation: 0.78`
and the hardcoded `datetime(2025, 1, 1)` are gone. The docstring states plainly
that no model is trained.

`scoring.py` was 577 lines, over the 450-line rule. Split into
`services/scoring/engine.py` (340) + `services/scoring/schemas.py` (90) + a
thin 182-line route layer that imports and delegates. Every file is now within
the limit.

Separately fixed while in this file: `/api/detect` indexed `request.data[i]`
against a value list that had been filtered of non-numeric rows, so every
anomaly after a gap pointed at the wrong record. Now tracks `valid_indices`
explicitly. The `std` computation no longer calls `detect_zscore` as a
truthiness guard, and `median` averages the two middle values for even-length
input rather than taking the upper one.

### Outstanding

- **`custom_weights` is still ignored.** Accepted by the schema, read by nothing
  — every scoring function reads `ScoringWeights` class constants. Left
  unchanged because honouring it properly means threading overrides through
  five functions, which is a behaviour change deserving its own review.
- **Port 8082** in the botbook is still unresolved (see 1514).
- `anomaly_service.py:114-152` labels a modified z-score
  `detect_isolation_forest` — genuinely misleading, and both that and the
  `votes >= 1` union in `/api/detect` (making `confidence` only ever 0.5 or
  1.0) remain open as separate issues.

## Non-goals

- Rewriting the anomaly algorithm (the `votes >= 1` union in `/api/detect` makes
  `confidence` only ever 0.5 or 1.0, and `anomaly_service.py:114-152` labels a
  modified z-score `detect_isolation_forest` — both are separate issues).