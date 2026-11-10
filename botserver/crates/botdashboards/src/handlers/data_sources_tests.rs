//! Offline unit tests for data-source endpoint resolution and the
//! conversational query classifier (#2069).
//!
//! `host_port_from_url`/`connection_endpoint` decide which TCP endpoint a probe
//! would hit, and `analyze_query_intent` picks the widget type for a natural
//! language question. Both are pure: no socket is opened here.

use super::{analyze_query_intent, connection_endpoint, host_port_from_url};
use crate::types::WidgetType;
use serde_json::json;

#[test]
fn url_with_scheme_userinfo_and_port_is_parsed() {
    let endpoint = host_port_from_url("postgres://user:secret@db.example.test:5432/app", 1)
        .expect("scheme present");
    assert_eq!(endpoint, ("db.example.test".to_string(), 5432));
}

#[test]
fn url_without_port_falls_back_to_the_caller_default() {
    assert_eq!(
        host_port_from_url("https://api.example.test/v1/query?x=1", 443),
        Some(("api.example.test".to_string(), 443))
    );
    assert_eq!(
        host_port_from_url("redis://cache.example.test", 6379),
        Some(("cache.example.test".to_string(), 6379))
    );
}

#[test]
fn url_without_a_scheme_is_rejected() {
    assert_eq!(host_port_from_url("db.example.test:5432/app", 1), None);
    assert_eq!(host_port_from_url("", 1), None);
    assert_eq!(host_port_from_url("/relative/path", 1), None);
}

#[test]
fn unparsable_port_falls_back_instead_of_failing() {
    assert_eq!(
        host_port_from_url("http://host.example.test:not-a-port/x", 8080),
        Some(("host.example.test".to_string(), 8080))
    );
}

#[test]
fn query_string_terminates_the_authority() {
    assert_eq!(
        host_port_from_url("http://host.example.test:81?q=1", 80),
        Some(("host.example.test".to_string(), 81))
    );
}

#[test]
fn connection_endpoint_prefers_a_url_over_host_and_port() {
    let config = json!({
        "connection": {
            "url": "postgres://u:p@from-url.example.test:5433/db",
            "host": "ignored.example.test",
            "port": 1111
        }
    });
    assert_eq!(
        connection_endpoint(&config),
        Some(("from-url.example.test".to_string(), 5433))
    );
}

#[test]
fn connection_endpoint_reads_host_and_port_when_no_url_is_given() {
    let config = json!({ "connection": { "host": "db.example.test", "port": 6543 } });
    assert_eq!(
        connection_endpoint(&config),
        Some(("db.example.test".to_string(), 6543))
    );

    // Missing port defaults to 80, matching the URL branch.
    let no_port = json!({ "connection": { "host": "db.example.test" } });
    assert_eq!(
        connection_endpoint(&no_port),
        Some(("db.example.test".to_string(), 80))
    );
}

#[test]
fn connection_endpoint_accepts_a_flat_config() {
    let flat = json!({ "host": "flat.example.test", "port": 9000 });
    assert_eq!(
        connection_endpoint(&flat),
        Some(("flat.example.test".to_string(), 9000))
    );
    let flat_url = json!({ "url": "http://flat.example.test:1234/x" });
    assert_eq!(
        connection_endpoint(&flat_url),
        Some(("flat.example.test".to_string(), 1234))
    );
}

#[test]
fn connection_endpoint_returns_none_without_an_endpoint() {
    assert_eq!(connection_endpoint(&json!({})), None);
    assert_eq!(connection_endpoint(&json!({ "connection": {} })), None);
    assert_eq!(connection_endpoint(&json!({ "connection": "jdbc:unknown" })), None);
    assert_eq!(connection_endpoint(&json!({ "connection": { "port": 5432 } })), None);
}

#[test]
fn trend_wording_selects_a_line_chart() {
    for query in ["sales trend", "revenue OVER TIME", "timeline of tickets"] {
        let (widget, reason) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::LineChart, "{query}");
        assert!(reason.contains("line chart"));
    }
}

#[test]
fn comparison_wording_selects_a_bar_chart() {
    for query in ["compare regions", "sales by category", "cost breakdown"] {
        let (widget, _) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::BarChart, "{query}");
    }
}

#[test]
fn share_wording_selects_a_pie_chart() {
    for query in ["distribution of plans", "percentage of churn", "market share"] {
        let (widget, _) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::PieChart, "{query}");
    }
}

#[test]
fn aggregate_wording_selects_a_kpi() {
    for query in ["total revenue", "count of orders", "sum of invoices", "kpi dashboard"] {
        let (widget, _) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::Kpi, "{query}");
    }
}

#[test]
fn listing_wording_selects_a_table() {
    for query in ["table of leads", "list every ticket", "customer details"] {
        let (widget, _) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::Table, "{query}");
    }
}

#[test]
fn geo_wording_selects_a_map() {
    for query in ["sales map", "location heat", "geographic split"] {
        let (widget, _) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::Map, "{query}");
    }
}

#[test]
fn goal_wording_selects_a_gauge() {
    for query in ["quota gauge", "progress to goal", "target attainment"] {
        let (widget, _) = analyze_query_intent(query);
        assert_eq!(widget, WidgetType::Gauge, "{query}");
    }
}

#[test]
fn unmatched_wording_defaults_to_a_bar_chart() {
    let (widget, reason) = analyze_query_intent("something unspecified");
    assert_eq!(widget, WidgetType::BarChart);
    assert!(reason.contains("Defaulting"));
}

#[test]
fn classification_is_case_insensitive_and_first_match_wins() {
    // Case folding: "TREND" still matches the line-chart branch.
    assert_eq!(analyze_query_intent("TREND").0, WidgetType::LineChart);
    // Precedence: "trend" is checked before "compare" and "total".
    assert_eq!(
        analyze_query_intent("compare total trend").0,
        WidgetType::LineChart
    );
    assert_eq!(analyze_query_intent("compare total").0, WidgetType::BarChart);
}
