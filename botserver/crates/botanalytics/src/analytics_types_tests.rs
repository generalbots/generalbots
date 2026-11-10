//! Offline tests for the analytics formatting helpers (issue #2075).
//! Both helpers are pure string/number transforms.

use super::*;

#[test]
fn format_number_keeps_small_values_verbatim() {
    let cases = [(0, "0"), (1, "1"), (999, "999"), (-5, "-5")];
    for (value, expected) in cases {
        assert_eq!(format_number(value), expected, "value {value}");
    }
}

#[test]
fn format_number_uses_k_for_thousands() {
    let cases = [
        (1_000, "1.0K"),
        (1_500, "1.5K"),
        (12_345, "12.3K"),
        (999_949, "999.9K"),
    ];
    for (value, expected) in cases {
        assert_eq!(format_number(value), expected, "value {value}");
    }
}

#[test]
fn format_number_uses_m_for_millions() {
    let cases = [
        (1_000_000, "1.0M"),
        (2_450_000, "2.5M"),
        (10_000_000, "10.0M"),
    ];
    for (value, expected) in cases {
        assert_eq!(format_number(value), expected, "value {value}");
    }
}

#[test]
fn format_number_never_renders_a_thousand_k() {
    // 999_999 used to round to "1000.0K"; it must promote to millions.
    assert_eq!(format_number(999_999), "1.0M");
    assert_eq!(format_number(999_950), "1.0M");
}

#[test]
fn html_escape_replaces_every_significant_character() {
    assert_eq!(
        html_escape(r#"<a href="x">Tom & Jerry's</a>"#),
        "&lt;a href=&quot;x&quot;&gt;Tom &amp; Jerry&#39;s&lt;/a&gt;"
    );
}

#[test]
fn html_escape_leaves_plain_text_untouched() {
    assert_eq!(html_escape("relatório 2026"), "relatório 2026");
    assert_eq!(html_escape(""), "");
}

#[test]
fn analytics_stats_default_is_all_zeroes() {
    let stats = AnalyticsStats::default();
    assert_eq!(stats.message_count, 0);
    assert_eq!(stats.session_count, 0);
    assert_eq!(stats.active_sessions, 0);
    assert_eq!(stats.avg_response_time, 0.0);
}
