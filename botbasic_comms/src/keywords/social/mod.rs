use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub mod delete_post;
pub mod get_metrics;
pub mod get_posts;
pub mod post_to;
pub mod post_to_scheduled;

pub fn register_social_media_keywords(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    delete_post::delete_post_keyword(state.clone(), user.clone(), engine);
    get_metrics::get_instagram_metrics_keyword(state.clone(), user.clone(), engine);
    get_posts::get_posts_keyword(state.clone(), user.clone(), engine);
    post_to::post_to_keyword(state.clone(), user.clone(), engine);
}
