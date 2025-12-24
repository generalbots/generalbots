pub mod isempty;
pub mod isnull;
pub mod str_val;
pub mod typeof_check;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn register_validation_functions(
    state: &Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    str_val::val_keyword(&state, user.clone(), engine);
    str_val::str_keyword(&state, user.clone(), engine);
    str_val::cint_keyword(&state, user.clone(), engine);
    str_val::cdbl_keyword(&state, user.clone(), engine);
    isnull::isnull_keyword(&state, user.clone(), engine);
    isempty::isempty_keyword(&state, user.clone(), engine);
    typeof_check::typeof_keyword(&state, user.clone(), engine);
    typeof_check::isarray_keyword(&state, user.clone(), engine);
    typeof_check::isnumber_keyword(&state, user.clone(), engine);
    typeof_check::isstring_keyword(&state, user.clone(), engine);
    typeof_check::isbool_keyword(&state, user.clone(), engine);
    nvl_iif::nvl_keyword(&state, user.clone(), engine);
    nvl_iif::iif_keyword(&state, user, engine);

    debug!("Registered all validation functions");
}

pub mod nvl_iif;
