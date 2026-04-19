pub mod dateadd;
pub mod datediff;
pub mod extract;
pub mod now;

use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn register_datetime_functions(state: &Arc<AppState>, user: UserSession, engine: &mut Engine) {
    now::now_keyword(state, user.clone(), engine);
    now::today_keyword(state, user.clone(), engine);
    now::time_keyword(state, user.clone(), engine);
    now::timestamp_keyword(state, user.clone(), engine);
    extract::year_keyword(state, user.clone(), engine);
    extract::month_keyword(state, user.clone(), engine);
    extract::day_keyword(state, user.clone(), engine);
    extract::hour_keyword(state, user.clone(), engine);
    extract::minute_keyword(state, user.clone(), engine);
    extract::second_keyword(state, user.clone(), engine);
    extract::weekday_keyword(state, user.clone(), engine);
    dateadd::dateadd_keyword(state, user.clone(), engine);
    datediff::datediff_keyword(state, user.clone(), engine);
    extract::format_date_keyword(state, user.clone(), engine);
    extract::isdate_keyword(state, user, engine);

    debug!("Registered all datetime functions");
}
