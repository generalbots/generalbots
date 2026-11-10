//! Contract tests for the pure half of SYNCHRONIZE (#2064): the error
//! taxonomy, the argument parser and the human-readable report.
//!
//! The retry decision (`is_retryable`) is what makes an ETL run resilient, so
//! every variant is pinned — a new variant that silently becomes retryable
//! would hammer a failing remote endpoint.

use super::*;

#[test]
fn every_error_variant_maps_to_its_taxonomy_entry() {
    let cases = [
        (SynchronizeError::FetchFailed("x".into()), SyncErrorType::FetchError),
        (SynchronizeError::ParseFailed("x".into()), SyncErrorType::ParseError),
        (SynchronizeError::TransformFailed("x".into()), SyncErrorType::TransformError),
        (SynchronizeError::InsertFailed("x".into()), SyncErrorType::InsertError),
        (SynchronizeError::UpdateFailed("x".into()), SyncErrorType::UpdateError),
        (SynchronizeError::ConflictDetected("x".into()), SyncErrorType::ConflictError),
        (SynchronizeError::ValidationFailed("x".into()), SyncErrorType::ValidationError),
        (SynchronizeError::ConnectionFailed("x".into()), SyncErrorType::ConnectionError),
        (SynchronizeError::Timeout, SyncErrorType::TimeoutError),
        (SynchronizeError::RateLimited, SyncErrorType::RateLimitError),
        // A bad configuration is reported as a validation problem: there is no
        // dedicated config error type in the taxonomy.
        (SynchronizeError::InvalidConfig("x".into()), SyncErrorType::ValidationError),
    ];
    for (error, expected) in cases {
        assert_eq!(error.error_type(), expected);
    }
}

#[test]
fn only_transient_failures_are_retryable() {
    assert!(SynchronizeError::Timeout.is_retryable());
    assert!(SynchronizeError::RateLimited.is_retryable());
    assert!(SynchronizeError::ConnectionFailed("reset".into()).is_retryable());

    for error in [
        SynchronizeError::FetchFailed("404".into()),
        SynchronizeError::ParseFailed("json".into()),
        SynchronizeError::TransformFailed("map".into()),
        SynchronizeError::InsertFailed("constraint".into()),
        SynchronizeError::UpdateFailed("missing".into()),
        SynchronizeError::ConflictDetected("id".into()),
        SynchronizeError::ValidationFailed("type".into()),
        SynchronizeError::InvalidConfig("args".into()),
    ] {
        assert!(!error.is_retryable(), "{error:?} must not be retried");
    }
}

#[test]
fn the_display_messages_are_stable() {
    assert_eq!(
        SynchronizeError::FetchFailed("timeout".into()).to_string(),
        "Fetch failed: timeout"
    );
    assert_eq!(
        SynchronizeError::ConflictDetected("id=1".into()).to_string(),
        "Conflict detected for key: id=1"
    );
    assert_eq!(
        SynchronizeError::Timeout.to_string(),
        "Operation timed out"
    );
    assert_eq!(
        SynchronizeError::RateLimited.to_string(),
        "Rate limited by remote server"
    );
}

#[test]
fn a_synchronize_error_is_a_std_error() {
    let error: Box<dyn std::error::Error> = Box::new(SynchronizeError::Timeout);
    assert_eq!(error.to_string(), "Operation timed out");
}

#[test]
fn the_error_type_names_are_snake_case() {
    assert_eq!(SyncErrorType::FetchError.to_string(), "fetch_error");
    assert_eq!(SyncErrorType::RateLimitError.to_string(), "rate_limit_error");
    assert_eq!(SyncErrorType::TimeoutError.to_string(), "timeout_error");
}

#[test]
fn five_arguments_are_required() {
    let args = |n: usize| -> Vec<String> { (0..n).map(|i| format!("a{i}")).collect() };

    for count in 0..5 {
        assert!(
            parse_synchronize_args(&args(count)).is_err(),
            "{count} arguments must be rejected"
        );
    }

    let config = parse_synchronize_args(&args(5)).expect("five arguments are enough");
    assert_eq!(config.endpoint, "a0");
    assert_eq!(config.table_name, "a1");
    assert_eq!(config.key_field, "a2");
    assert_eq!(config.page_param, "a3");
    assert_eq!(config.limit_param, "a4");
    assert!(config.page_size.is_none());
    assert!(config.headers.is_none());
    assert!(config.dry_run.is_none());
}

#[test]
fn extra_arguments_are_ignored() {
    let args: Vec<String> = (0..8).map(|i| format!("a{i}")).collect();
    let config = parse_synchronize_args(&args).expect("valid");
    assert_eq!(config.endpoint, "a0");
    assert_eq!(config.limit_param, "a4");
}

#[test]
fn the_report_lists_the_status_and_every_counter() {
    let now = Utc::now();
    let result = SyncResult {
        success: true,
        records_fetched: 10,
        records_inserted: 6,
        records_updated: 3,
        records_skipped: 1,
        records_failed: 0,
        pages_processed: 2,
        duration_ms: 1234,
        errors: vec![],
        started_at: now,
        completed_at: now,
    };

    let report = generate_sync_report(&result);
    assert!(report.contains("Status: SUCCESS"));
    assert!(report.contains("Duration: 1234ms"));
    assert!(report.contains("Pages Processed: 2"));
    assert!(report.contains("Fetched: 10"));
    assert!(report.contains("Inserted: 6"));
    assert!(report.contains("Updated: 3"));
    assert!(report.contains("Skipped: 1"));
    assert!(report.contains("Failed: 0"));
    assert!(!report.contains("--- Errors ---"));
}

#[test]
fn the_report_truncates_after_ten_errors() {
    let now = Utc::now();
    let errors: Vec<SyncError> = (0..12)
        .map(|i| SyncError {
            record_key: Some(format!("k{i}")),
            page: Some(1),
            error_type: SyncErrorType::InsertError,
            message: format!("boom {i}"),
            retryable: false,
        })
        .collect();
    let result = SyncResult {
        success: false,
        records_fetched: 12,
        records_inserted: 0,
        records_updated: 0,
        records_skipped: 0,
        records_failed: 12,
        pages_processed: 1,
        duration_ms: 5,
        errors,
        started_at: now,
        completed_at: now,
    };

    let report = generate_sync_report(&result);
    assert!(report.contains("Status: FAILED"));
    assert!(report.contains("1. [insert_error] boom 0"));
    assert!(report.contains("10. [insert_error] boom 9"));
    assert!(!report.contains("boom 10"));
    assert!(report.contains("... and 2 more errors"));
}
