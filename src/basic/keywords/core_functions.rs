use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{debug, info};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

use super::arrays::register_array_functions;
use super::datetime::register_datetime_functions;
use super::errors::register_error_functions;
use super::math::register_math_functions;
use super::validation::register_validation_functions;

pub fn register_core_functions(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    debug!("Registering core BASIC functions...");

    register_math_functions(&state, user.clone(), engine);
    debug!("  * Math functions registered");

    register_datetime_functions(&state, user.clone(), engine);
    debug!("  * Date/Time functions registered");

    register_validation_functions(&state, user.clone(), engine);
    debug!("  * Validation functions registered");

    register_array_functions(state.clone(), user.clone(), engine);
    debug!("  * Array functions registered");

    register_error_functions(state, user, engine);
    debug!("  * Error handling functions registered");

    // Register send_mail stub function for tools (traces when mail feature is not available)
    engine.register_fn("send_mail", move |to: Dynamic, subject: Dynamic, body: Dynamic, _attachments: Dynamic| -> String {
        let to_str = to.to_string();
        let subject_str = subject.to_string();
        let body_str = body.to_string();
        info!(
            "[TOOL] send_mail called (mail feature not enabled): to='{}', subject='{}', body_len={}",
            to_str,
            subject_str,
            body_str.len()
        );
        // Return a fake message ID
        format!("MSG-{:0X}", chrono::Utc::now().timestamp())
    });
    debug!("  * send_mail stub function registered");

    debug!("All core BASIC functions registered successfully");
}
