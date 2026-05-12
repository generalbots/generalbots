pub mod archive;
pub mod basic_io;
pub mod copy_move;
pub mod handlers;
pub mod pdf;
pub mod transfer;
pub mod utils;

use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_file_ops_keywords(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    handlers::register_file_operations(&state, user, engine);
}
