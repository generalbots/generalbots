use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_clear_tools_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, _engine: &mut Engine) {}
