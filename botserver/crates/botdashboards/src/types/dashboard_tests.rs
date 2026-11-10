//! Offline unit tests for the dashboard type vocabulary (#2069).
//!
//! Every enum that reaches the database or the browser is parsed from a string
//! and rendered back to one. `FromStr` must be exact (no case folding, no
//! silent default) and `Display` must round-trip, otherwise a widget saved by
//! one release silently changes type on the next load.

use super::{DashboardFilterType, DataSourceStatus, DataSourceType, WidgetType};

#[test]
fn widget_type_round_trips_every_variant() {
    let variants = [
        WidgetType::LineChart,
        WidgetType::BarChart,
        WidgetType::PieChart,
        WidgetType::DonutChart,
        WidgetType::AreaChart,
        WidgetType::ScatterPlot,
        WidgetType::Heatmap,
        WidgetType::Table,
        WidgetType::Kpi,
        WidgetType::Gauge,
        WidgetType::Map,
        WidgetType::Text,
        WidgetType::Image,
        WidgetType::Iframe,
        WidgetType::Filter,
        WidgetType::DateRange,
    ];
    for variant in variants {
        let rendered = variant.to_string();
        assert!(!rendered.is_empty());
        assert!(!rendered.contains(' '), "{rendered} is not a wire token");
        let parsed: WidgetType = rendered.parse().expect("display output must parse");
        assert_eq!(parsed.to_string(), rendered);
    }
}

#[test]
fn widget_type_rejects_unknown_and_miscased_tokens() {
    for token in ["", "line", "LineChart", "line-chart", "bogus", "kpi "] {
        assert!(
            token.parse::<WidgetType>().is_err(),
            "{token:?} must not be a widget type"
        );
    }
    let err = "bogus".parse::<WidgetType>().expect_err("must fail");
    assert_eq!(err, "Unknown widget type: bogus");
}

#[test]
fn filter_type_round_trips_and_reports_the_bad_token() {
    for variant in [
        DashboardFilterType::Text,
        DashboardFilterType::Number,
        DashboardFilterType::Date,
        DashboardFilterType::DateRange,
        DashboardFilterType::Select,
        DashboardFilterType::MultiSelect,
        DashboardFilterType::Checkbox,
        DashboardFilterType::Slider,
    ] {
        let rendered = variant.to_string();
        let parsed: DashboardFilterType = rendered.parse().expect("round trip");
        assert_eq!(parsed.to_string(), rendered);
    }
    assert_eq!(
        "nope".parse::<DashboardFilterType>().expect_err("must fail"),
        "Unknown filter type: nope"
    );
}

#[test]
fn data_source_type_round_trips_and_rejects_unknown() {
    for variant in [
        DataSourceType::Postgresql,
        DataSourceType::Mysql,
        DataSourceType::Sqlserver,
        DataSourceType::Oracle,
        DataSourceType::Mongodb,
        DataSourceType::Bigquery,
        DataSourceType::Snowflake,
        DataSourceType::Redshift,
        DataSourceType::Elasticsearch,
        DataSourceType::RestApi,
        DataSourceType::GraphqlApi,
        DataSourceType::Csv,
        DataSourceType::Excel,
        DataSourceType::GoogleSheets,
        DataSourceType::Airtable,
        DataSourceType::InternalTables,
    ] {
        let rendered = variant.to_string();
        let parsed: DataSourceType = rendered.parse().expect("round trip");
        assert_eq!(parsed.to_string(), rendered);
    }
    assert_eq!(
        "postgres".parse::<DataSourceType>().expect_err("must fail"),
        "Unknown data source type: postgres"
    );
}

#[test]
fn data_source_status_round_trips_and_rejects_unknown() {
    for variant in [
        DataSourceStatus::Active,
        DataSourceStatus::Inactive,
        DataSourceStatus::Error,
        DataSourceStatus::Syncing,
    ] {
        let rendered = variant.to_string();
        let parsed: DataSourceStatus = rendered.parse().expect("round trip");
        assert_eq!(parsed.to_string(), rendered);
    }
    assert_eq!(
        "paused".parse::<DataSourceStatus>().expect_err("must fail"),
        "Unknown status: paused"
    );
}
