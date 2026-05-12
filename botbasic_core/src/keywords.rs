pub mod arrays;
pub mod core_functions;
pub mod datetime;
pub mod errors;
pub mod first;
pub mod for_next;
pub mod format;
pub mod hearing;
pub mod hear_talk;
pub mod last;
pub mod math;
pub mod print;
pub mod procedures;
pub mod set_context;
pub mod set_user;
pub mod string_functions;
pub mod switch_case;
pub mod validation;
pub mod wait;

use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::Engine;
use std::sync::Arc;

pub fn register_core_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    arrays::register_array_functions(&state, user.clone(), engine);
    core_functions::register_core_functions(&state, user.clone(), engine);
    datetime::register_datetime_functions(&state, user.clone(), engine);
    errors::register_error_functions(&state, user.clone(), engine);
    first::first_keyword(engine);
    format::format_keyword(engine);
    for_next::for_keyword(&state, user.clone(), engine);
    hearing::hear_keyword(&state, user.clone(), engine);
    hearing::talk_keyword(&state, user.clone(), engine);
    last::last_keyword(engine);
    math::register_math_functions(&state, user.clone(), engine);
    print::print_keyword(&state, user.clone(), engine);
    procedures::register_procedure_keywords(&state, user.clone(), engine);
    set_context::set_context_keyword(&state, user.clone(), engine);
    set_user::set_user_keyword(&state, user.clone(), engine);
    string_functions::register_string_functions(&state, user.clone(), engine);
    switch_case::switch_keyword(&state, user.clone(), engine);
    validation::register_validation_functions(&state, user.clone(), engine);
    wait::wait_keyword(&state, user, engine);
}
