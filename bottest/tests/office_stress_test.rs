//! Office large-file load-stress harness (#2531) — offline contract tests.
//!
//! These tests do not run cargo, a server or a browser. They drive the Python
//! harness as a subprocess and assert its *offline* contract: the self-test is
//! green, a runtime-less run emits an explicit SKIP (never PASS), SKIP can be
//! promoted to a hard failure, and the gate fails closed on a breaching report.
//! Real RSS/soak measurement needs hardware and is deliberately not faked here.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const HARNESS: &str = "tools/officestress/os_measure.py";

fn harness() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(HARNESS)
}

fn python3() -> Option<Command> {
    let probe = Command::new("python3").arg("--version").output().ok()?;
    if probe.status.success() {
        Some(Command::new("python3"))
    } else {
        None
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "officestress-rs-{}-{}-{}",
        label,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn run_harness(args: &[&str]) -> Output {
    let mut command = match python3() {
        Some(command) => command,
        None => {
            eprintln!("python3 unavailable; skipping offline harness contract test");
            return Command::new("true").output().expect("true binary");
        }
    };
    command.arg(harness()).args(args).output().expect("run harness")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn harness_self_test_passes_offline() {
    let output = run_harness(&["--self-test"]);
    assert!(
        output.status.success(),
        "harness self-test failed:\n{}\n{}",
        stdout(&output),
        stderr(&output)
    );
    assert!(
        stdout(&output).contains("checks passed"),
        "self-test did not report a check count: {}",
        stdout(&output)
    );
}

#[test]
fn runtime_less_run_is_an_explicit_skip() {
    let dir = temp_dir("skip");
    let report = dir.join("report.json");
    let report_arg = report.to_string_lossy().into_owned();
    let output = run_harness(&["--run", "--report", &report_arg]);
    assert!(
        output.status.success(),
        "SKIP run must not fail the pipeline:\n{}\n{}",
        stdout(&output),
        stderr(&output)
    );
    let document: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
            .expect("report is JSON");
    assert_eq!(document["status"], "SKIP");
    assert_eq!(document["gate"]["verdict"], "SKIP");
    assert_eq!(document["cases"].as_array().map(Vec::len), Some(0));
    assert!(document["skip_reason"].as_str().map_or(false, |reason| !reason.is_empty()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn require_runtime_promotes_skip_to_failure() {
    let dir = temp_dir("strict");
    let report = dir.join("report.json");
    let report_arg = report.to_string_lossy().into_owned();
    let output = run_harness(&["--run", "--report", &report_arg, "--require-runtime"]);
    assert_eq!(output.status.code(), Some(2), "strict SKIP must exit 2");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn gate_fails_closed_on_a_breach() {
    let dir = temp_dir("gate");
    let report = dir.join("breach.json");
    let case = serde_json::json!({
        "id": "docs-large",
        "baseline_rss_mb": 300.0,
        "peak_rss_mb": 2000.0,
        "return_rss_mb": 310.0,
        "exit_code": 0,
        "cycles": 1,
        "session_cache_growth_mb": 1.0,
        "events_with_content": 0,
        "browser_heap_mb": null,
        "wire_bytes": null
    });
    let document = serde_json::json!({
        "schema": 1,
        "issue": 2531,
        "status": "MEASURED",
        "mode": "measure",
        "profile": "laptop-4gb",
        "cases": [case]
    });
    fs::write(&report, document.to_string()).expect("write breaching report");
    let report_arg = report.to_string_lossy().into_owned();
    let output = run_harness(&["--gate", "--report", &report_arg]);
    assert_eq!(output.status.code(), Some(1), "a ceiling breach must fail the gate");
    assert!(
        stdout(&output).contains("BREACH") || stdout(&output).contains("FAIL"),
        "gate output should explain the breach: {}",
        stdout(&output)
    );
    let _ = fs::remove_dir_all(&dir);
}
