//! Offline tests for the log-level labels used by the AutoTask app log
//! (issue #2075).

use super::*;

#[test]
fn log_levels_render_their_lowercase_label() {
    let cases = [
        (LogLevel::Debug, "debug"),
        (LogLevel::Info, "info"),
        (LogLevel::Warn, "warn"),
        (LogLevel::Error, "error"),
        (LogLevel::Critical, "critical"),
    ];
    for (level, label) in cases {
        assert_eq!(level.to_string(), label);
    }
}

#[test]
fn log_level_labels_are_unique() {
    let levels = [
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Critical,
    ];
    let mut labels: Vec<String> = levels.iter().map(ToString::to_string).collect();
    labels.sort();
    labels.dedup();
    assert_eq!(labels.len(), levels.len());
}
