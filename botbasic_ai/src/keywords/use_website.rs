use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_use_website_function(_state: Arc<dyn BasicRuntime>, _user: UserSession, _engine: &mut Engine) {}
