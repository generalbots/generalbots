//! Memory OS hook (issue #1178): recall injection before LLM calls and
//! post-turn extraction. No-ops when the `memory-os` feature is disabled
//! or the service was never initialized, preserving legacy behavior.
use std::sync::{Arc, OnceLock};
use botcore::shared::state::AppState;
use uuid::Uuid;

static SERVICE: OnceLock<Arc<botmemory::MemoryService>> = OnceLock::new();

pub fn init(pool: botmemory::DbPool) -> bool {
    let llm: botmemory::LlmFn = Arc::new(|_system: &str, _user: &str, _params: &str| {
        Err("LLM not wired for memory extraction".to_string())
    });
    init_with(Arc::new(botmemory::MemoryService::new(pool, llm)))
}

/// Registers an already-built service so router state and hooks share it.
pub fn init_with(service: Arc<botmemory::MemoryService>) -> bool {
    SERVICE.set(service).is_ok()
}

fn service() -> Option<&'static Arc<botmemory::MemoryService>> {
    SERVICE.get()
}

/// Appends a memory context system message for this user when relevant
/// memories exist. Best-effort: any failure is logged and ignored.
pub fn inject_recall(
    state: &Arc<AppState>,
    user_id: Uuid,
    query_hint: &str,
    messages_val: &mut serde_json::Value,
) {
    let _ = state;
    let Some(service) = service() else { return };
    if !service.enabled {
        return;
    }
    let mut conn = match service.pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("memory recall pool: {e}");
            return;
        }
    };
    let block = botmemory::recall::recall_block(
        &mut conn,
        user_id,
        None,
        query_hint,
        512,
    );
    if block.is_empty() {
        return;
    }
    if let Some(arr) = messages_val.as_array_mut() {
        if let Some(idx) = arr.iter().position(|m| m["role"] == "system") {
            if let Some(content) = arr[idx]["content"].as_str() {
                arr[idx]["content"] =
                    serde_json::Value::String(format!("{content}\n\n{block}"));
                return;
            }
        }
        arr.insert(0, serde_json::json!({ "role": "system", "content": block }));
    } else {
        log::warn!("memory recall: messages payload is not an array");
    }
}

/// Post-turn extraction (sampled; explicit phrasing bypasses sampling).
pub async fn extract_from_turn(
    user_id: Uuid,
    branch_id: Uuid,
    session_id: Uuid,
    user_text: &str,
    assistant_text: &str,
) {
    let Some(service) = service() else { return };
    botmemory::extract::maybe_extract(
        service,
        user_id,
        Some(branch_id),
        &session_id.to_string(),
        user_text,
        assistant_text,
    )
    .await;
}
