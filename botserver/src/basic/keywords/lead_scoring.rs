

























use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::crm::register_crm_keywords;












































pub fn register_lead_scoring_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering lead scoring keywords...");


    register_crm_keywords(state, user, engine);

    debug!("Lead scoring keywords registered successfully");
}
