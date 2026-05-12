use axum::{routing::{get, post, put}, Router};
use std::sync::Arc;

use crate::handlers;
use crate::ui;
use crate::CrateState;

pub fn configure_crm_api_routes() -> Router<Arc<CrateState>> {
    Router::new()
        .route("/api/crm/contacts", get(handlers::contacts::list_contacts).post(handlers::contacts::create_contact))
        .route("/api/crm/contacts/{id}", get(handlers::contacts::get_contact).put(handlers::contacts::update_contact).delete(handlers::contacts::delete_contact))
        .route("/api/crm/accounts", get(handlers::accounts::list_accounts).post(handlers::accounts::create_account))
        .route("/api/crm/accounts/{id}", get(handlers::accounts::get_account).delete(handlers::accounts::delete_account))
        .route("/api/crm/leads", get(handlers::deals::list_leads).post(handlers::deals::create_lead_form))
        .route("/api/crm/leads/{id}", get(handlers::deals::get_lead).put(handlers::deals::update_lead).delete(handlers::deals::delete_lead))
        .route("/api/crm/leads/{id}/stage", put(handlers::deals::update_lead_stage))
        .route("/api/crm/leads/{id}/convert", post(handlers::deals::convert_lead_to_opportunity))
        .route("/api/crm/opportunities", get(handlers::opportunities::list_opportunities).post(handlers::opportunities::create_opportunity))
        .route("/api/crm/opportunities/{id}", get(handlers::opportunities::get_opportunity).put(handlers::opportunities::update_opportunity).delete(handlers::opportunities::delete_opportunity))
        .route("/api/crm/opportunities/{id}/close", post(handlers::opportunities::close_opportunity))
        .route("/api/crm/deals", get(handlers::crm::list_deals).post(handlers::crm::create_deal))
        .route("/api/crm/deals/{id}", get(handlers::crm::get_deal).put(handlers::crm::update_deal).delete(handlers::crm::delete_deal))
        .route("/api/crm/activities", get(handlers::crm::list_activities).post(handlers::crm::create_activity))
        .route("/api/crm/pipeline/stages", get(handlers::crm::get_pipeline_stages))
        .route("/api/crm/stats", get(handlers::crm::get_crm_stats))
}

pub fn configure_crm_ui_routes() -> Router<Arc<CrateState>> {
    Router::new()
        .route("/api/ui/crm/count", get(ui::crm_ui::handle_crm_count))
        .route("/api/ui/crm/pipeline", get(ui::crm_ui::handle_crm_pipeline))
        .route("/api/ui/crm/contacts", get(ui::crm_ui::handle_crm_contacts))
        .route("/api/ui/crm/accounts", get(ui::crm_ui::handle_crm_accounts))
        .route("/api/ui/crm/deals", get(ui::crm_ui::handle_crm_deals))
}

#[cfg(feature = "calendar")]
pub fn configure_calendar_routes() -> Router<Arc<CrateState>> {
    crate::calendar_routes::calendar_integration_routes()
}

#[cfg(feature = "tasks")]
pub fn configure_tasks_routes() -> Router<Arc<CrateState>> {
    crate::tasks_routes::tasks_integration_routes()
}

#[cfg(feature = "external_sync")]
pub fn configure_external_sync_routes() -> Router<Arc<CrateState>> {
    crate::sync_routes::external_sync_routes()
}

pub fn configure_all_routes() -> Router<Arc<CrateState>> {
    let router = Router::new()
        .merge(configure_crm_api_routes())
        .merge(configure_crm_ui_routes());

    #[cfg(feature = "calendar")]
    let router = router.merge(configure_calendar_routes());

    #[cfg(feature = "tasks")]
    let router = router.merge(configure_tasks_routes());

    #[cfg(feature = "external_sync")]
    let router = router.merge(configure_external_sync_routes());

    #[cfg(not(any(feature = "calendar", feature = "tasks", feature = "external_sync")))]
    let router = router;

    router
}
