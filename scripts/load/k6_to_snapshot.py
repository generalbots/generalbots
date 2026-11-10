#!/usr/bin/env python3
"""Convert a k6 summary export into the SLO snapshot format (#2115).

k6 reports ``http_req_duration`` and custom trends/rates; ``check_slo.py`` wants
the SLO metric ids from ``ops/slo.yaml``. This adapter is the only place that
knows both shapes, so a load run feeds the gate without a hand-written file::

    k6 run --summary-export=/tmp/k6.json scripts/load/k6_http.js
    python3 scripts/load/k6_to_snapshot.py --summary /tmp/k6.json \\
        --out /tmp/slo-api.json --class api
    python3 scripts/check_slo.py --snapshot /tmp/slo-api.json --class api

Exit codes: 0 snapshot written, 2 the summary is unreadable or carries none of
the metrics for the requested class.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# k6 metric name -> (snapshot metric id, kind). `kind` is "trend" or "rate".
TREND_METRICS = {
    "http_req_duration": "rest.api_ms",
    "ws_first_token_ms": "chat.first_token_ms",
    "queue_wait_ms": "queue.wait_ms",
    "tool_duration_ms": "chat.tool_call_ms",
    "db_query_ms": "dep.db_ms",
    "cache_query_ms": "dep.cache_ms",
    "drive_request_ms": "dep.drive_ms",
}
RATE_METRICS = {
    "http_req_failed": "turn.error_pct",
    "server_error_rate": "turn.error_pct",
    "ws_unexpected_disconnect_pct": "ws.unexpected_disconnect_pct",
    "llm_provider_error_rate": "llm.provider_error_pct",
    "llm_provider_retry_rate": "llm.provider_retry_pct",
}
# Which snapshot ids belong to which SLO class, so --class keeps the snapshot honest.
CLASSES = {
    "api": {"rest.api_ms"},
    "chat": {"chat.first_token_ms", "chat.tool_call_ms", "ws.unexpected_disconnect_pct", "turn.error_pct"},
    "deps": {"dep.db_ms", "dep.cache_ms", "dep.drive_ms"},
    "queue": {"queue.wait_ms"},
    "llm": {"llm.provider_error_pct", "llm.provider_retry_pct"},
}


def pick(values: dict, *names: str) -> float | None:
    for name in names:
        if name in values:
            try:
                return float(values[name])
            except (TypeError, ValueError):
                return None
    return None


def trend(values: dict) -> dict[str, float]:
    stats: dict[str, float] = {}
    for stat, names in {
        "p50": ("p(50)", "p50", "med"),
        "p95": ("p(95)", "p95"),
        "p99": ("p(99)", "p99"),
        "max": ("max",),
    }.items():
        value = pick(values, *names)
        if value is not None:
            stats[stat] = value
    return stats


def convert(summary: dict, profile: str, klass: str | None) -> dict[str, float | dict]:
    raw_metrics = summary.get("metrics")
    if not isinstance(raw_metrics, dict):
        raise ValueError("the k6 summary has no 'metrics' object")

    metrics: dict[str, float | dict] = {}
    for name, slo_id in TREND_METRICS.items():
        entry = raw_metrics.get(name)
        if not isinstance(entry, dict):
            continue
        stats = trend(entry.get("values") or {})
        if stats:
            metrics[slo_id] = stats
    for name, slo_id in RATE_METRICS.items():
        entry = raw_metrics.get(name)
        if not isinstance(entry, dict):
            continue
        rate = pick(entry.get("values") or {}, "rate")
        if rate is None:
            continue
        # `http_req_failed` and `server_error_rate` both describe the error rate;
        # the higher of the two is the honest one to report.
        percent = rate * 100.0
        previous = metrics.get(slo_id)
        if isinstance(previous, float):
            percent = max(percent, previous)
        metrics[slo_id] = percent

    if klass:
        allowed = CLASSES.get(klass)
        if allowed is None:
            raise ValueError(f"unknown --class {klass!r}; known: {', '.join(sorted(CLASSES))}")
        metrics = {key: value for key, value in metrics.items() if key in allowed}
    if not metrics:
        raise ValueError("no SLO metric could be derived from this summary")

    return metrics


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="k6 summary -> SLO snapshot (#2115)")
    parser.add_argument("--summary", required=True, help="k6 --summary-export file, or - for stdin")
    parser.add_argument("--out", default="-", help="snapshot path, or - for stdout")
    parser.add_argument("--profile", default="local-mock", help="profile name recorded in the snapshot")
    parser.add_argument("--class", dest="klass", help="keep only the metrics of this SLO class")
    parser.add_argument("--model", help="attach a cost block for this model (price from ops/slo.yaml)")
    parser.add_argument("--conversations", type=int, default=0)
    parser.add_argument("--tokens-in", type=float, default=0.0)
    parser.add_argument("--tokens-out", type=float, default=0.0)
    args = parser.parse_args(argv)

    try:
        if args.summary == "-":
            summary = json.loads(sys.stdin.read())
        else:
            summary = json.loads(Path(args.summary).read_text(encoding="utf-8"))
        metrics = convert(summary, args.profile, args.klass)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"k6_to_snapshot: {error}", file=sys.stderr)
        return 2

    snapshot: dict[str, object] = {
        "profile": args.profile,
        "measured_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "window_days": 0,
        "source": "k6",
        "metrics": metrics,
    }
    if args.model:
        snapshot["cost"] = {
            "model": args.model,
            "conversations": args.conversations,
            "tokens_in": args.tokens_in,
            "tokens_out": args.tokens_out,
        }

    text = json.dumps(snapshot, indent=2, sort_keys=True)
    if args.out == "-":
        print(text)
    else:
        Path(args.out).write_text(text + "\n", encoding="utf-8")
        print(f"k6_to_snapshot: wrote {args.out} ({len(metrics)} metrics)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
