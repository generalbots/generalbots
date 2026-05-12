

























use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use crate::keywords::crm::register_crm_keywords;












































pub fn register_lead_scoring_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering lead scoring keywords...");


    register_crm_keywords(state, user, engine);

    debug!("Lead scoring keywords registered successfully");
}
