use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

mod get_default_bot_fn;
mod comment;
mod handle_get_feed;
mod handle_delete_post;
mod handle_add_comment;
mod handle_trending_topics;

pub use get_default_bot_fn::*;
pub use comment::*;
pub use handle_get_feed::*;
pub use handle_delete_post::*;
pub use handle_add_comment::*;
pub use handle_trending_topics::*;
