use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn get_instagram_metrics_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("GET_INSTAGRAM_METRICS", || -> String { "GET_INSTAGRAM_METRICS (stub)".to_string() });
}

pub fn get_facebook_metrics_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("GET_FACEBOOK_METRICS", || -> String { "GET_FACEBOOK_METRICS (stub)".to_string() });
}

pub fn get_linkedin_metrics_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("GET_LINKEDIN_METRICS", || -> String { "GET_LINKEDIN_METRICS (stub)".to_string() });
}

pub fn get_twitter_metrics_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("GET_TWITTER_METRICS", || -> String { "GET_TWITTER_METRICS (stub)".to_string() });
}
