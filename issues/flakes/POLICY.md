# Flake policy (#2119)

A flaky test reports both `ok` and `FAILED` for the same code. It is worse than
a failing test: it teaches the team to re-run CI, which is how a real failure
gets merged. This page is the policy; `scripts/flake_report.py` is the
enforcement, and `issues/flakes/quarantine.toml` is the registry.

## Retry

* **One retry.** A test that fails may be re-run once. Two consecutive failures
  are a real failure and block the PR.
* A test that only passes on the retry is **counted as a flake** (it flipped),
  not as a pass. The count is what triggers quarantine.
* Never retry a whole job to "see if it goes green". Re-run the failing job once
  and record the outcome — the nightly history scan reads those runs.

## Quarantine

A test that flakes **3 times in the last 20 runs** of the same workflow must be
quarantined. The gate fails when it is not:

```toml
# issues/flakes/quarantine.toml
[[flake]]
test = "botcore::tests::websocket_handshake_retries"
issue = 1234                 # mandatory: a PR cannot add a flake without an issue
owner = "quality"            # mandatory: who fixes it
since = "2026-11-10"         # mandatory: start of the SLA
reason = "times out when the shared runner cache is cold, unrelated to the change"
```

and in the source, in the same PR:

```rust
#[ignore = "flaky: #1234"]
```

Rules the script enforces:

| Rule | Effect |
|------|--------|
| `test`, `issue`, `owner`, `since`, `reason` present | otherwise the gate fails |
| `reason` >= 20 characters | "flaky" is not a reason |
| `since` within `sla_days` (14) | past the SLA the gate fails: fix it or delete it |
| a flake over `max_flakes` (3) without an entry | gate fails, naming the test |
| an entry that stopped flaking | reported so the entry is removed, never silently kept |

Never delete a quarantined test silently: fix it, or delete it in a PR that says
so and closes the issue. A test that is quarantined past its SLA is treated as a
failure, not as a permanent exemption.

## Running the scan

The registry is validated on every PR (`python3 scripts/flake_report.py
--quarantine-only`). The history scan needs CI logs and runs nightly
(`flake-history` in `.github/workflows/ci.yml`):

```bash
mkdir -p /tmp/flakes
gh run list --workflow ci.yml --limit 20 --json databaseId \
  --jq '.[].databaseId' | while read -r id; do
    gh run view "$id" --log > "/tmp/flakes/$id.log"
  done
python3 scripts/flake_report.py --logs /tmp/flakes --window 20
```

`--runs <file>` accepts the Actions API JSON (job/step granularity) for the same
analysis when per-test lines are not available.

## Where a new test belongs

| Kind | Budget | Location |
|------|--------|----------|
| unit | < 2 min | `bottest/tests/unit/` or the crate's `#[cfg(test)]` |
| contract | < 5 min | `bottest/tests/*.rs` with a `[[test]]` entry |
| integration | < 15 min | `bottest/tests/integration/` |
| E2E | schedule/manual | `bottest/tests/e2e/` |

No test may sleep > 1 s without a fake clock, bind a fixed port, or mutate
process-wide environment variables — those three are the usual source of the
flakes this policy quarantines.
