use super::*;

#[test]
fn window_is_clamped_to_a_drawable_range() {
    assert_eq!(window_minutes(None), 120);
    assert_eq!(window_minutes(Some(0)), 5);
    assert_eq!(window_minutes(Some(90)), 90);
    assert_eq!(window_minutes(Some(999_999)), 43_200);
}

#[test]
fn granularity_follows_the_window() {
    assert_eq!(granularity(60), "minute");
    assert_eq!(granularity(180), "minute");
    assert_eq!(granularity(181), "hour");
    assert_eq!(granularity(2_880), "hour");
    assert_eq!(granularity(2_881), "day");
}

fn empty_data(minutes: i32) -> TrafficData {
    TrafficData {
        minutes,
        series: Vec::new(),
        totals: (0, 0, 0),
        fleet: (0, 0),
        deploys: Vec::new(),
    }
}

#[test]
fn empty_series_renders_an_explicit_empty_state() {
    let html = chart(&empty_data(60));
    assert!(html.contains("No turns recorded"));
    assert!(!html.contains("ops-bar"));
}

#[test]
fn chart_draws_one_bar_per_bucket_and_marks_deploys() {
    let start = DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp is valid");
    let mut data = empty_data(60);
    data.series = vec![(start, 4), (start + chrono::Duration::minutes(1), 8)];
    data.deploys = vec![DeployRow {
        version: "7.0.1".to_string(),
        commit_hash: "abc123".to_string(),
        build_date: "2026-11-01".to_string(),
        deployed_at: start + chrono::Duration::seconds(30),
    }];
    let html = chart(&data);
    assert_eq!(html.matches("class=\"ops-bar\"").count(), 2);
    assert_eq!(html.matches("ops-deploy-line").count(), 1);
    assert!(html.contains(">7.0.1<"));
}

#[test]
fn deploy_list_states_when_no_marker_exists() {
    let html = deploy_list(&empty_data(60));
    assert!(html.contains("No deploy markers"));
    assert!(!html.contains("<li>"));
}

#[test]
fn render_reports_the_window_and_the_fleet() {
    let mut data = empty_data(90);
    data.totals = (12, 3, 2);
    data.fleet = (5, 2);
    let html = render(&data);
    assert!(html.contains("last 90 min"));
    assert!(html.contains("WS upgrades"));
    assert!(html.contains(">5<"));
}
