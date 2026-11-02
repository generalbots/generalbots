pub mod mcp_add;
pub mod mcp_catalog;

use crate::state::AppState;
use axum::{extract::State, response::Html, routing::get, Router};
use std::sync::Arc;

mod handle_sources_list_page;
mod configure_sources_ui_routes;

pub use handle_sources_list_page::*;
pub use configure_sources_ui_routes::*;
