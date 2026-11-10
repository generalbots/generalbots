//! Offline unit tests for the dashboard row mappers (#2075).
//!
//! The storage layer converts Diesel rows into the API model. Every conversion
//! has a fallback for a JSON column written by an older release (`unwrap_or_default`),
//! and an unknown enum string must degrade to a safe default instead of failing
//! the whole dashboard. Those fallbacks are the tested contract.

use super::*;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

fn widget_row(widget_type: &str, config: serde_json::Value) -> DbWidget {
    DbWidget {
        id: Uuid::from_u128(1),
        dashboard_id: Uuid::from_u128(2),
        widget_type: widget_type.to_string(),
        title: "Revenue".to_string(),
        position_x: 1,
        position_y: 2,
        width: 3,
        height: 4,
        config,
        data_query: None,
        style: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn filter_row(filter_type: &str, options: serde_json::Value, linked: serde_json::Value) -> DbFilter {
    DbFilter {
        id: Uuid::from_u128(3),
        dashboard_id: Uuid::from_u128(2),
        name: "Region".to_string(),
        field: "region".to_string(),
        filter_type: filter_type.to_string(),
        default_value: None,
        options,
        linked_widgets: linked,
        created_at: Utc::now(),
    }
}

#[test]
fn a_known_widget_type_and_config_are_decoded() {
    let widget = db_widget_to_widget(widget_row("line_chart", json!({"text_content": "hi"})));
    assert_eq!(widget.widget_type, WidgetType::LineChart);
    assert_eq!(widget.config.text_content.as_deref(), Some("hi"));
    assert_eq!(widget.title, "Revenue");
    assert_eq!(widget.position.x, 1);
    assert_eq!(widget.position.width, 3);
    assert_eq!(widget.position.height, 4);
}

#[test]
fn an_unknown_widget_type_falls_back_to_text() {
    for raw in ["", "sparkline", "LINE_CHART"] {
        let widget = db_widget_to_widget(widget_row(raw, json!({})));
        assert_eq!(widget.widget_type, WidgetType::Text, "{raw:?}");
    }
}

#[test]
fn a_malformed_config_falls_back_to_the_default() {
    for config in [json!("not-an-object"), json!([]), json!(null)] {
        let widget = db_widget_to_widget(widget_row("kpi", config));
        assert_eq!(widget.config.text_content, None);
        assert_eq!(widget.config.chart_config, None);
        // The row itself still maps: a broken config must not hide the widget.
        assert_eq!(widget.title, "Revenue");
    }
}

#[test]
fn a_data_query_is_decoded_only_when_it_is_valid() {
    let mut row = widget_row("table", json!({}));
    row.data_query = Some(json!({"query_type": "table", "table": "sales"}));
    let widget = db_widget_to_widget(row);
    let query = widget.data_query.expect("valid query");
    assert_eq!(query.table.as_deref(), Some("sales"));

    let mut row = widget_row("table", json!({}));
    row.data_query = Some(json!({"nonsense": true}));
    assert!(db_widget_to_widget(row).data_query.is_none());
}

#[test]
fn filter_options_and_links_are_decoded() {
    let filter = db_filter_to_filter(filter_row(
        "select",
        json!([{"value": "br", "label": "Brazil"}]),
        json!([Uuid::from_u128(7).to_string()]),
    ));
    assert_eq!(filter.filter_type, DashboardFilterType::Select);
    assert_eq!(filter.options.len(), 1);
    assert_eq!(filter.options[0].label, "Brazil");
    assert_eq!(filter.linked_widgets, vec![Uuid::from_u128(7)]);
}

#[test]
fn an_unknown_filter_type_and_malformed_options_degrade_safely() {
    let filter = db_filter_to_filter(filter_row("radar", json!("nope"), json!({})));
    assert_eq!(filter.filter_type, DashboardFilterType::Text);
    assert!(filter.options.is_empty());
    assert!(filter.linked_widgets.is_empty());
    assert_eq!(filter.field, "region");
}
