//! The engine that answers the current turn of a conversation, for work a
//! workspace tool does on the turn's behalf (the one-shot program repair).
//!
//! The chat stream records it before it runs the turn's tool calls; the
//! repair then talks to the same provider and model the person chose instead
//! of the bot's default model.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use uuid::Uuid;

/// Characters of the person's request kept for the repair prompt.
const REQUEST_CHARS: usize = 2_000;

/// The provider, model and key of one turn, and what the person asked.
pub struct TurnEngine {
    pub provider: Arc<dyn botlib::traits::LLMProvider>,
    pub model: String,
    pub key: String,
    /// A subscription agent fixes its own programs: no repair call is made.
    pub subscription: bool,
    pub request: String,
}

fn turns() -> &'static Mutex<HashMap<Uuid, Arc<TurnEngine>>> {
    static TURNS: OnceLock<Mutex<HashMap<Uuid, Arc<TurnEngine>>>> = OnceLock::new();
    TURNS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records the engine of the turn now running in `session_id`.
pub fn remember(
    session_id: Uuid,
    provider: &Arc<dyn botlib::traits::LLMProvider>,
    model: &str,
    key: &str,
    engine_type: Option<&str>,
    request: &str,
) {
    let engine = TurnEngine {
        provider: provider.clone(),
        model: model.to_string(),
        key: key.to_string(),
        subscription: engine_type == Some(botsettings::usage_engine::ENGINE_SUBSCRIPTION),
        request: request.chars().take(REQUEST_CHARS).collect(),
    };
    if let Ok(mut map) = turns().lock() {
        map.insert(session_id, Arc::new(engine));
    }
}

/// The engine of the latest turn in `session_id`, if one ran.
pub(super) fn current(session_id: Uuid) -> Option<Arc<TurnEngine>> {
    turns().lock().ok()?.get(&session_id).cloned()
}
