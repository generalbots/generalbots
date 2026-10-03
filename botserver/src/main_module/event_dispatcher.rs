//! Channel event dispatcher (#1507).
//!
//! `ON EVENT "<event>" CALL "<tool>"` in a bot script writes a row in
//! `basic_event_handlers`; the channel pipeline publishes the occurrence in
//! `workflow_events`. This loop joins the two: for every unprocessed channel
//! event it runs the subscribed tools with the event payload injected as script
//! variables, then marks the event processed.
//!
//! It is a background task on purpose. A tool may take minutes (video
//! perception) and may fail; neither may block the conversation that triggered
//! the event, so failures are logged and the event is still marked processed —
//! a retry storm on a broken tool is worse than one dropped run.

use std::sync::Arc;
use std::time::Duration;

use botcore::shared::basic_events::{self, PendingEvent};
use botcore::shared::state::AppState;
use botcore::shared::utils::{get_work_path, DbPool};
use uuid::Uuid;

/// Poll cadence. Fast enough to feel immediate on an upload, slow enough to be
/// invisible next to the 5 s drive/compile loops.
const TICK: Duration = Duration::from_secs(5);
/// Events handled per tick; a burst is drained over the following ticks.
const BATCH: i64 = 20;

/// Start the dispatcher loop.
pub fn start_event_dispatcher(state: Arc<AppState>) {
    tokio::spawn(async move {
        let pool = state.conn.clone();
        loop {
            tokio::time::sleep(TICK).await;
            let events = {
                let pool = pool.clone();
                match tokio::task::spawn_blocking(move || basic_events::pending_events(&pool, BATCH))
                    .await
                {
                    Ok(events) => events,
                    Err(e) => {
                        log::warn!("[event_dispatcher] poll failed: {e}");
                        continue;
                    }
                }
            };
            for event in events {
                dispatch_one(&state, &pool, event).await;
            }
        }
    });
    log::info!("[event_dispatcher] started (tick {TICK:?}) — ON EVENT subscriptions active");
}

/// Run every tool subscribed to one event, then mark it processed.
async fn dispatch_one(state: &Arc<AppState>, pool: &DbPool, event: PendingEvent) {
    let tools = {
        let lookup_pool = pool.clone();
        let bot_id = event.bot_id;
        let name = event.event_name.clone();
        let lookup = tokio::task::spawn_blocking(move || {
            basic_events::handlers_for(&lookup_pool, bot_id, &name)
        })
        .await;
        match lookup {
            Ok(tools) => tools,
            Err(e) => {
                log::warn!("[event_dispatcher] handler lookup failed: {e}");
                basic_events::mark_processed(pool, event.id);
                return;
            }
        }
    };
    if tools.is_empty() {
        // Nothing subscribed: still mark it, or the batch window fills with
        // events no one will ever consume.
        basic_events::mark_processed(pool, event.id);
        return;
    }
    log::info!(
        "[event_dispatcher] event {} for bot {} → {} tool(s): {}",
        event.event_name,
        event.bot_id,
        tools.len(),
        tools.join(", ")
    );
    for tool in tools {
        run_tool(state, pool, &event, &tool).await;
    }
    basic_events::mark_processed(pool, event.id);
}

/// Execute one subscribed tool with the event payload in scope.
async fn run_tool(
    state: &Arc<AppState>,
    pool: &DbPool,
    event: &PendingEvent,
    tool: &str,
) {
    let bot_name = {
        let pool = pool.clone();
        let bot_id = event.bot_id;
        match tokio::task::spawn_blocking(move || bot_name_for(&pool, bot_id)).await {
            Ok(Some(name)) => name,
            Ok(None) => {
                log::warn!("[event_dispatcher] no bot row for {}", event.bot_id);
                return;
            }
            Err(e) => {
                log::warn!("[event_dispatcher] bot lookup failed: {e}");
                return;
            }
        }
    };
    let rel = format!(
        "{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool}.ast",
    );
    if !crate::core::bot::ws::handler::verify_path_within_workdir(&rel) {
        log::error!("[event_dispatcher] path traversal rejected for tool {tool}");
        return;
    }
    let work = get_work_path();
    let ast_path = std::path::Path::new(&work).join(&rel);
    let source = match tokio::fs::read_to_string(&ast_path).await {
        Ok(c) if !c.trim().is_empty() => c,
        _ => {
            let bas = ast_path.with_extension("bas");
            match tokio::fs::read_to_string(&bas).await {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[event_dispatcher] {bot_name}/{tool}: no compiled source: {e}");
                    return;
                }
            }
        }
    };

    // The payload keys become script variables, so a tool reads `path` and
    // `caption` exactly as it would when the model calls it.
    let mut context = serde_json::Map::new();
    if let Some(obj) = event.payload.as_object() {
        for (key, value) in obj {
            let text = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            context.insert(key.clone(), serde_json::Value::String(text));
        }
    }
    context.insert("event".to_string(), serde_json::Value::String(event.event_name.clone()));
    context.insert("bot".to_string(), serde_json::Value::String(bot_name.clone()));
    let variables: Vec<(String, String)> = context
        .into_iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
        .collect();

    let tool_session_id = event_session_id(&event.payload);

    let state_for_tool = state.clone();
    let bot_id = event.bot_id;
    let session = botlib::models::UserSession {
        id: tool_session_id,
        user_id: Uuid::nil(),
        branch_id: Uuid::nil(),
        bot_id: event.bot_id,
        title: String::new(),
        context_data: serde_json::Value::Object(
            variables
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        ),
        current_tool: Some(tool.to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let tool_name = tool.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        let mut svc = crate::basic::ScriptService::new(state_for_tool.clone(), session);
        svc.load_bot_config_params(&state_for_tool, bot_id);
        for (key, value) in variables {
            let _ = svc.set_variable(&key, &value);
        }
        if let Err(e) = svc.run(&source) {
            log::warn!("[event_dispatcher] {tool_name} failed: {e}");
        }
    })
    .await;
}

/// Session a subscribed tool runs under: the conversation that published the
/// event, so the tool's `TALK` has somewhere to go. Running under a fresh
/// session dropped every confirmation (`send_message: NO channel for session
/// …`) because the channel of that conversation is keyed by the session id.
/// Events published without the field keep the previous behavior.
fn event_session_id(payload: &serde_json::Value) -> Uuid {
    payload
        .get("session_id")
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(Uuid::new_v4)
}

fn bot_name_for(pool: &DbPool, bot_id: Uuid) -> Option<String> {
    use botcore::shared::models::schema::bots;
    use diesel::prelude::*;
    let mut conn = pool.get().ok()?;
    bots::table
        .filter(bots::id.eq(bot_id))
        .select(bots::name)
        .first(&mut conn)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_session_is_the_conversation_that_published_the_event() {
        let session_id = Uuid::new_v4();
        let payload = serde_json::json!({
            "channel": "telegram",
            "session_id": session_id.to_string(),
            "channel_user_id": "6676512312",
        });

        assert_eq!(event_session_id(&payload), session_id);
    }

    #[test]
    fn an_event_without_a_session_gets_a_fresh_one() {
        let payload = serde_json::json!({"path": "inbox/voice_test.wav"});

        assert!(!event_session_id(&payload).is_nil());
    }
}
