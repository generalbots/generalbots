//! Offline unit tests for the billing alert formatters and thresholds (#2075).
//!
//! Alert text is what an operator reads at 3am: the byte/number formatters must
//! be exact, the severity ladder must be monotonic, and a threshold must only
//! fire at or above its own percentage.

use super::super::*;
use super::{format_bytes, format_number};

fn window(start_hour: u8, end_hour: u8) -> QuietHours {
    QuietHours {
        start_hour,
        end_hour,
        timezone: "UTC".to_string(),
        days: Vec::new(),
    }
}

#[test]
fn bytes_are_rendered_in_the_largest_fitting_unit() {
    assert_eq!(format_bytes(0), "0 bytes");
    assert_eq!(format_bytes(1), "1 bytes");
    assert_eq!(format_bytes(1023), "1023 bytes");
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(1536), "1.50 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(format_bytes(1024_u64.pow(4)), "1.00 TB");
    assert_eq!(format_bytes(1024_u64.pow(4) * 3 / 2), "1.50 TB");
}

#[test]
fn numbers_get_thousands_separators() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(999), "999");
    assert_eq!(format_number(1000), "1,000");
    assert_eq!(format_number(1234567), "1,234,567");
    assert_eq!(format_number(u64::MAX), "18,446,744,073,709,551,615");
}

#[test]
fn severity_labels_and_priorities_are_distinct_and_ordered() {
    let all = [
        AlertSeverity::Warning,
        AlertSeverity::Critical,
        AlertSeverity::Exceeded,
    ];
    let mut labels: Vec<&str> = all.iter().map(AlertSeverity::as_str).collect();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), 3);

    assert!(AlertSeverity::Warning.priority() < AlertSeverity::Critical.priority());
    assert!(AlertSeverity::Critical.priority() < AlertSeverity::Exceeded.priority());
    assert_eq!(AlertSeverity::Warning.to_string(), "warning");
    assert_eq!(AlertSeverity::Exceeded.to_string(), "exceeded");
    assert!(!AlertSeverity::Warning.emoji().is_empty());
}

#[test]
fn thresholds_fire_at_or_above_their_percentage() {
    let thresholds = AlertThresholds::default();
    assert_eq!(thresholds.warning, 80.0);
    assert_eq!(thresholds.critical, 90.0);
    assert_eq!(thresholds.exceeded, 100.0);

    assert_eq!(thresholds.get_severity(0.0), None);
    assert_eq!(thresholds.get_severity(79.99), None);
    assert_eq!(thresholds.get_severity(80.0), Some(AlertSeverity::Warning));
    assert_eq!(thresholds.get_severity(89.99), Some(AlertSeverity::Warning));
    assert_eq!(thresholds.get_severity(90.0), Some(AlertSeverity::Critical));
    assert_eq!(thresholds.get_severity(99.99), Some(AlertSeverity::Critical));
    assert_eq!(thresholds.get_severity(100.0), Some(AlertSeverity::Exceeded));
    assert_eq!(thresholds.get_severity(250.0), Some(AlertSeverity::Exceeded));
}

#[test]
fn custom_thresholds_are_honoured() {
    let thresholds = AlertThresholds::new(10.0, 20.0, 30.0);
    assert_eq!(thresholds.get_severity(9.0), None);
    assert_eq!(thresholds.get_severity(10.0), Some(AlertSeverity::Warning));
    assert_eq!(thresholds.get_severity(20.0), Some(AlertSeverity::Critical));
    assert_eq!(thresholds.get_severity(30.0), Some(AlertSeverity::Exceeded));
}

#[test]
fn notification_preferences_filter_by_minimum_severity() {
    let mut preferences = NotificationPreferences::default();
    assert!(preferences.enabled);
    assert_eq!(preferences.min_severity, AlertSeverity::Warning);

    // Default floor is Warning: every severity notifies.
    assert!(preferences.should_notify(AlertSeverity::Warning));
    assert!(preferences.should_notify(AlertSeverity::Critical));
    assert!(preferences.should_notify(AlertSeverity::Exceeded));

    preferences.min_severity = AlertSeverity::Critical;
    assert!(!preferences.should_notify(AlertSeverity::Warning));
    assert!(preferences.should_notify(AlertSeverity::Critical));
    assert!(preferences.should_notify(AlertSeverity::Exceeded));

    preferences.min_severity = AlertSeverity::Exceeded;
    assert!(!preferences.should_notify(AlertSeverity::Critical));
    assert!(preferences.should_notify(AlertSeverity::Exceeded));
}

#[test]
fn quiet_hours_without_a_window_are_never_active() {
    let preferences = NotificationPreferences::default();
    assert!(preferences.quiet_hours.is_none());
    assert!(!preferences.is_in_quiet_hours());
}

#[test]
fn quiet_hour_windows_handle_the_midnight_wrap() {
    // 09:00-17:00 is a same-day window; 22:00-06:00 wraps midnight.
    let day = window(9, 17);
    assert!(!day.is_active_at(8));
    assert!(day.is_active_at(9));
    assert!(day.is_active_at(16));
    assert!(!day.is_active_at(17));
    assert!(!day.is_active_at(0));

    let night = window(22, 6);
    assert!(night.is_active_at(22));
    assert!(night.is_active_at(23));
    assert!(night.is_active_at(0));
    assert!(night.is_active_at(5));
    assert!(!night.is_active_at(6));
    assert!(!night.is_active_at(12));
}
