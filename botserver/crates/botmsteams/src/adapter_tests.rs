//! Offline tests for the Teams inbound payload normaliser (issue #2075).
//! `receive_message` performs no I/O: it only inspects the parsed activity.

use super::*;
use crate::channel::ChannelAdapter;
use crate::state::GetConfigFn;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

fn adapter(bot_id: &str) -> TeamsAdapter {
    // `build_unchecked` never opens a connection, so the test stays offline.
    let manager = ConnectionManager::<PgConnection>::new("postgres://offline/offline");
    let pool = Arc::new(
        Pool::builder()
            .build_unchecked(manager),
    );
    let bot = bot_id.to_string();
    let get_config: GetConfigFn = Arc::new(move |_, key, default| match key {
        "teams-bot-id" => Ok(bot.clone()),
        "teams-app-id" => Ok("app-id".to_string()),
        "teams-app-password" => Ok("app-password".to_string()),
        _ => default.map(str::to_string).ok_or_else(|| "missing".to_string()),
    });
    TeamsAdapter::new(pool, uuid::Uuid::nil(), get_config)
}

fn receive(adapter: &TeamsAdapter, payload: serde_json::Value) -> Option<String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(adapter.receive_message(payload))
        .expect("receive_message never fails")
}

#[test]
fn configured_adapter_reports_its_identity() {
    let adapter = adapter("28:bot");
    assert!(adapter.is_configured());
    assert_eq!(adapter.channel_type(), "msteams");
    assert_eq!(adapter.display_name(), "Teams");
}

#[test]
fn missing_app_credentials_leave_the_adapter_unconfigured() {
    let manager = ConnectionManager::<PgConnection>::new("postgres://offline/offline");
    let pool = Arc::new(Pool::builder().build_unchecked(manager));
    let get_config: GetConfigFn = Arc::new(|_, _, default| {
        default.map(str::to_string).ok_or_else(|| "missing".to_string())
    });
    let adapter = TeamsAdapter::new(pool, uuid::Uuid::nil(), get_config);

    assert!(!adapter.is_configured());
}

#[test]
fn message_text_has_the_bot_mention_removed() {
    let payload = serde_json::json!({
        "type": "message",
        "text": "<at>28:bot</at>   run the report  "
    });
    assert_eq!(
        receive(&adapter("28:bot"), payload).as_deref(),
        Some("run the report")
    );
}

#[test]
fn message_without_text_but_with_attachment_reports_the_content_type() {
    let payload = serde_json::json!({
        "type": "message",
        "attachments": [{ "contentType": "application/vnd.microsoft.card.adaptive" }]
    });
    assert_eq!(
        receive(&adapter("28:bot"), payload).as_deref(),
        Some("Received attachment: application/vnd.microsoft.card.adaptive")
    );
}

#[test]
fn message_with_neither_text_nor_attachment_is_ignored() {
    let payload = serde_json::json!({ "type": "message", "attachments": [] });
    assert_eq!(receive(&adapter("28:bot"), payload), None);
}

#[test]
fn reaction_activities_are_reported_as_reaction_text() {
    let payload = serde_json::json!({
        "type": "messageReaction",
        "reactionsAdded": [{ "type": "like" }]
    });
    assert_eq!(
        receive(&adapter("28:bot"), payload).as_deref(),
        Some("Reaction: like")
    );
}

#[test]
fn reaction_without_a_known_type_falls_back_to_unknown() {
    let payload = serde_json::json!({ "type": "messageReaction" });
    assert_eq!(
        receive(&adapter("28:bot"), payload).as_deref(),
        Some("Reaction: unknown")
    );
}

#[test]
fn non_message_activities_are_not_turned_into_chat_text() {
    for activity_type in ["conversationUpdate", "invoke", "typing", ""] {
        let payload = serde_json::json!({ "type": activity_type, "text": "ignored" });
        assert_eq!(receive(&adapter("28:bot"), payload), None, "{activity_type}");
    }
}

#[test]
fn user_info_returns_the_id_and_platform_without_a_network_call() {
    let value = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(adapter("28:bot").get_user_info("29:user"))
        .expect("user info");

    assert_eq!(value["id"], "29:user");
    assert_eq!(value["platform"], "teams");
}
