use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
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

    debug!("All core BASIC functions registered successfully");
}
