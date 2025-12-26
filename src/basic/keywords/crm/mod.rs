pub mod attendance;
pub mod score_lead;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn register_crm_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    score_lead::score_lead_keyword(state.clone(), user.clone(), engine);
    score_lead::get_lead_score_keyword(state.clone(), user.clone(), engine);
    score_lead::qualify_lead_keyword(state.clone(), user.clone(), engine);
    score_lead::update_lead_score_keyword(state.clone(), user.clone(), engine);
    score_lead::ai_score_lead_keyword(state.clone(), user.clone(), engine);

    attendance::register_attendance_keywords(state, user, engine);

    debug!("Registered all CRM keywords (lead scoring + attendance + LLM assist)");
}
