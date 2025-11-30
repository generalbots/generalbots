pub mod delete_post;
pub mod get_metrics;
pub mod get_posts;
pub mod post_to;
pub mod post_to_scheduled;

pub use delete_post::delete_post_keyword;
pub use get_metrics::{
    get_facebook_metrics_keyword, get_instagram_metrics_keyword, get_linkedin_metrics_keyword,
    get_twitter_metrics_keyword, PostEngagement,
};
pub use get_posts::get_posts_keyword;
pub use post_to::post_to_keyword;
pub use post_to_scheduled::post_to_at_keyword;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use rhai::Engine;
use std::sync::Arc;

pub fn register_social_media_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    post_to_keyword(state.clone(), user.clone(), engine);
    post_to_at_keyword(state.clone(), user.clone(), engine);
    get_instagram_metrics_keyword(state.clone(), user.clone(), engine);
    get_facebook_metrics_keyword(state.clone(), user.clone(), engine);
    get_linkedin_metrics_keyword(state.clone(), user.clone(), engine);
    get_twitter_metrics_keyword(state.clone(), user.clone(), engine);
    get_posts_keyword(state.clone(), user.clone(), engine);
    delete_post_keyword(state, user, engine);
}
