pub(crate) use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
pub(crate) use botcore::shared::state::AppState;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;

mod handle_meet_list_page;
mod handle_meet_room_page;
mod handle_meet_schedule_page;

pub use handle_meet_list_page::*;
pub use handle_meet_room_page::*;
pub use handle_meet_schedule_page::*;
