pub mod a2a_protocol;
pub mod app_server;
pub mod create_site;
pub mod create_task;
pub mod face_api;
pub mod file_operations;
pub mod file_ops;
pub mod on_change;
pub mod on;
pub mod security_protection;
pub mod set_schedule;
pub mod synchronize;

use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::Engine;
use std::sync::Arc;

pub fn register_system_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    a2a_protocol::register_a2a_keywords(state.clone(), user.clone(), engine);
    app_server::register_app_server_keywords(state.clone(), user.clone(), engine);
    create_site::register_create_site_keywords(state.clone(), user.clone(), engine);
    create_task::create_task_keyword(state.clone(), user.clone(), engine);
    face_api::register_face_api_keywords(state.clone(), user.clone(), engine);
    file_operations::register_file_operations_keyword(state.clone(), user.clone(), engine);
    file_ops::register_file_ops_keywords(state.clone(), user.clone(), engine);
    on_change::register_on_change_keywords(state.clone(), user.clone(), engine);
    on::register_on_keywords(state.clone(), user.clone(), engine);
    security_protection::register_security_keywords(state.clone(), user.clone(), engine);
    set_schedule::register_schedule_keywords(state.clone(), user.clone(), engine);
    synchronize::register_synchronize_keywords(state.clone(), user.clone(), engine);
}
