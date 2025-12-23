//! Social Media Keywords - Wrapper module
//!
//! This module serves as a wrapper for social media functionality,
//! re-exporting the functions from the social module for backward compatibility.
//!
//! BASIC Keywords provided:
//! - POST TO - Post content to social media platforms
//! - POST TO AT - Schedule posts for later
//! - GET METRICS - Retrieve engagement metrics
//! - GET POSTS - List posts from a platform
//! - DELETE POST - Remove a post
//!
//! Supported Platforms:
//! - Instagram (via Graph API)
//! - Facebook (via Graph API)
//! - LinkedIn (via LinkedIn API)
//! - Twitter/X (via Twitter API v2)
//!
//! Examples:
//!   ' Post to Instagram
//!   POST TO "instagram" WITH
//!     image = "https://example.com/image.jpg"
//!     caption = "Check out our new product! #launch"
//!   END WITH
//!
//!   ' Schedule a post for later
//!   POST TO "facebook" AT "2024-12-25 09:00:00" WITH
//!     text = "Merry Christmas from our team!"
//!   END WITH
//!
//!   ' Get engagement metrics
//!   metrics = GET METRICS FROM "instagram" FOR post_id
//!   TALK "Likes: " + metrics.likes + ", Comments: " + metrics.comments
//!
//!   ' Get recent posts
//!   posts = GET POSTS FROM "twitter" LIMIT 10
//!   FOR EACH post IN posts
//!     TALK post.text + " - " + post.likes + " likes"
//!   NEXT

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::social::register_social_media_keywords as register_social_keywords_impl;

/// Register all social media keywords
///
/// This function delegates to the social module's registration function,
/// providing a convenient alias for backward compatibility and clearer intent.
///
/// ## Keywords Registered
///
/// ### POST TO
/// Post content to a social media platform.
/// ```basic
/// POST TO "instagram" WITH
///   image = "path/to/image.jpg"
///   caption = "My post caption #hashtag"
/// END WITH
/// ```
///
/// ### POST TO AT (Scheduled Posting)
/// Schedule a post for a specific time.
/// ```basic
/// POST TO "facebook" AT DATEADD(NOW(), 1, "hour") WITH
///   text = "This will be posted in 1 hour!"
/// END WITH
/// ```
///
/// ### GET METRICS
/// Retrieve engagement metrics for a post.
/// ```basic
/// metrics = GET_INSTAGRAM_METRICS(post_id)
/// metrics = GET_FACEBOOK_METRICS(post_id)
/// metrics = GET_LINKEDIN_METRICS(post_id)
/// metrics = GET_TWITTER_METRICS(post_id)
/// ```
///
/// ### GET POSTS
/// List posts from a platform.
/// ```basic
/// posts = GET_INSTAGRAM_POSTS(10)  ' Get last 10 posts
/// posts = GET_FACEBOOK_POSTS(20)
/// ```
///
/// ### DELETE POST
/// Remove a post from a platform.
/// ```basic
/// DELETE_INSTAGRAM_POST(post_id)
/// DELETE_FACEBOOK_POST(post_id)
/// ```
pub fn register_social_media_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering social media keywords...");

    // Delegate to social module which contains the actual implementation
    register_social_keywords_impl(state, user, engine);

    debug!("Social media keywords registered successfully");
}
