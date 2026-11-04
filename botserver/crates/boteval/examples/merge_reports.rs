//! Merge sharded `boteval-run` reports and evaluate the official CI gate.
//!
//! Sharding parallelises entries; it must not change the verdict. This example
//! reconstructs one `EvalReport` from the shard JSON files so the exact
//! `CiGate` used by CI can judge the combined run.
//!
//! ```text
//! cargo run -p boteval --features http --example merge_reports -- \
//!     crates/boteval/ci/gate.toml /tmp/eval1.json /tmp/eval2.json ...
//! ```

use boteval::ci_gate::{CiGate, CiGateConfig};
use boteval::runner::EvalReport;
use boteval::scoring::Score;
use std::collections::HashMap;

fn main() {
    let mut args = std::env::args().skip(1);
    let gate_path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("usage: merge_reports <gate.toml> <report.json>...");
            std::process::exit(2);
        }
    };
    let gate_bytes = match std::fs::read_to_string(&gate_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read gate {gate_path}: {error}");
            std::process::exit(2);
        }
    };
    let config: CiGateConfig = match toml::from_str(&gate_bytes) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("invalid gate {gate_path}: {error}");
            std::process::exit(2);
        }
    };

    let mut merged = EvalReport {
        dataset_name: "merged".to_string(),
        score: Score {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            rate: 0.0,
        },
        per_entry: Vec::new(),
        per_tag: HashMap::new(),
        total_cost: 0.0,
        total_tool_calls: 0,
    };

    for path in args {
        let bytes = match std::fs::read_to_string(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("cannot read report {path}: {error}");
                std::process::exit(2);
            }
        };
        let report: EvalReport = match serde_json::from_str(&bytes) {
            Ok(report) => report,
            Err(error) => {
                eprintln!("invalid report {path}: {error}");
                std::process::exit(2);
            }
        };
        merged.score.total += report.score.total;
        merged.score.passed += report.score.passed;
        merged.score.failed += report.score.failed;
        merged.score.skipped += report.score.skipped;
        merged.total_cost += report.total_cost;
        merged.total_tool_calls += report.total_tool_calls;
        merged.per_entry.extend(report.per_entry);
        for (tag, score) in report.per_tag {
            let entry = merged.per_tag.entry(tag).or_insert(Score {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                rate: 0.0,
            });
            entry.total += score.total;
            entry.passed += score.passed;
            entry.failed += score.failed;
            entry.skipped += score.skipped;
        }
    }

    if merged.score.total > 0 {
        merged.score.rate = merged.score.passed as f64 / merged.score.total as f64;
    }
    for score in merged.per_tag.values_mut() {
        if score.total > 0 {
            score.rate = score.passed as f64 / score.total as f64;
        }
    }

    let gate = CiGate::new(config);
    let verdict = gate.evaluate(&merged);
    println!(
        "{}",
        serde_json::json!({
            "score": merged.score,
            "per_tag": merged.per_tag,
            "total_cost": merged.total_cost,
            "total_tool_calls": merged.total_tool_calls,
            "entries": merged.per_entry.len(),
            "verdict": verdict,
        })
    );
    std::process::exit(verdict.exit_code);
}
