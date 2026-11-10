//! Insights wire contract: enum spellings, optional request fields and the
//! HTTP mapping of [`InsightsError`].

use axum::response::IntoResponse;
use serde_json::json;

use super::{
    InsightsError, InsightsPeriod, RecommendationCategory, RecommendationPriority, TrackUsageRequest,
    TrendDirection, UpdateFocusModeRequest, UpdateSettingsRequest, UsageEventType, Weekday,
};

fn status_of(error: InsightsError) -> u16 {
    error.into_response().status().as_u16()
}

#[test]
fn insights_errors_map_to_the_published_http_status() {
    assert_eq!(status_of(InsightsError::NotFound("n".into())), 404);
    assert_eq!(status_of(InsightsError::Unauthorized("u".into())), 401);
    assert_eq!(status_of(InsightsError::Validation("v".into())), 400);
    assert_eq!(status_of(InsightsError::Database("d".into())), 500);
}

#[test]
fn insights_errors_render_their_message() {
    assert_eq!(InsightsError::NotFound("x".into()).to_string(), "Not found: x");
    assert_eq!(
        InsightsError::Validation("bad".into()).to_string(),
        "Validation error: bad"
    );
}

#[test]
fn trend_directions_use_snake_case() {
    for (value, wire) in [
        (TrendDirection::Up, "up"),
        (TrendDirection::Down, "down"),
        (TrendDirection::Stable, "stable"),
    ] {
        assert_eq!(serde_json::to_value(value).expect("serializes"), json!(wire));
    }
    assert!(serde_json::from_value::<TrendDirection>(json!("sideways")).is_err());
}

#[test]
fn recommendation_categories_use_snake_case() {
    let cases = [
        (RecommendationCategory::FocusTime, "focus_time"),
        (RecommendationCategory::MeetingLoad, "meeting_load"),
        (RecommendationCategory::WorkLifeBalance, "work_life_balance"),
        (RecommendationCategory::Collaboration, "collaboration"),
        (RecommendationCategory::Breaks, "breaks"),
        (RecommendationCategory::AfterHours, "after_hours"),
    ];
    for (value, wire) in cases {
        assert_eq!(serde_json::to_value(value).expect("serializes"), json!(wire));
    }
}

#[test]
fn recommendation_priorities_use_snake_case() {
    for (value, wire) in [
        (RecommendationPriority::Low, "low"),
        (RecommendationPriority::Medium, "medium"),
        (RecommendationPriority::High, "high"),
    ] {
        assert_eq!(serde_json::to_value(value).expect("serializes"), json!(wire));
    }
}

#[test]
fn weekdays_use_snake_case() {
    let all = [
        (Weekday::Monday, "monday"),
        (Weekday::Tuesday, "tuesday"),
        (Weekday::Wednesday, "wednesday"),
        (Weekday::Thursday, "thursday"),
        (Weekday::Friday, "friday"),
        (Weekday::Saturday, "saturday"),
        (Weekday::Sunday, "sunday"),
    ];
    for (value, wire) in all {
        assert_eq!(serde_json::to_value(&value).expect("serializes"), json!(wire));
        let back: Weekday = serde_json::from_value(json!(wire)).expect("deserializes");
        assert_eq!(back, value);
    }
}

#[test]
fn usage_event_types_and_periods_use_snake_case() {
    for (value, wire) in [
        (UsageEventType::SessionStart, "session_start"),
        (UsageEventType::SessionEnd, "session_end"),
        (UsageEventType::ActivePing, "active_ping"),
    ] {
        let back: UsageEventType = serde_json::from_value(json!(wire)).expect("deserializes");
        assert_eq!(back, value);
    }
    for (value, wire) in [
        (InsightsPeriod::Daily, "daily"),
        (InsightsPeriod::Weekly, "weekly"),
        (InsightsPeriod::Monthly, "monthly"),
    ] {
        let back: InsightsPeriod = serde_json::from_value(json!(wire)).expect("deserializes");
        assert_eq!(back, value);
    }
}

#[test]
fn a_track_usage_request_needs_only_its_required_fields() {
    let request: TrackUsageRequest = serde_json::from_value(json!({
        "app_name": "chat",
        "event_type": "session_start",
    }))
    .expect("parses");
    assert_eq!(request.app_name, "chat");
    assert!(request.timestamp.is_none());
    assert!(request.metadata.is_none());
}

#[test]
fn a_track_usage_request_rejects_an_unknown_event() {
    assert!(serde_json::from_value::<TrackUsageRequest>(json!({
        "app_name": "chat",
        "event_type": "click",
    }))
    .is_err());
}

#[test]
fn an_empty_settings_patch_parses() {
    let request: UpdateSettingsRequest = serde_json::from_value(json!({})).expect("parses");
    assert!(request.tracking_enabled.is_none());
    assert!(request.daily_digest.is_none());
    assert!(request.weekly_digest.is_none());
    assert!(request.digest_time.is_none());
    assert!(request.goals.is_none());
}

#[test]
fn a_focus_mode_request_requires_enabled() {
    assert!(serde_json::from_value::<UpdateFocusModeRequest>(json!({})).is_err());
    let request: UpdateFocusModeRequest =
        serde_json::from_value(json!({ "enabled": true })).expect("parses");
    assert!(request.enabled);
    assert!(request.block_notifications.is_none());
    assert!(request.schedule.is_none());
}

#[test]
fn a_focus_schedule_parses_its_days_and_window() {
    let request: UpdateFocusModeRequest = serde_json::from_value(json!({
        "enabled": true,
        "schedule": {
            "days": ["monday", "friday"],
            "start_time": "09:00",
            "end_time": "12:00",
            "timezone": "America/Sao_Paulo"
        }
    }))
    .expect("parses");
    let schedule = request.schedule.expect("schedule");
    assert_eq!(schedule.days, vec![Weekday::Monday, Weekday::Friday]);
    assert_eq!(schedule.start_time, "09:00");
}
