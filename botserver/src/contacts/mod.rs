use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botcontacts::{
    ContactsError, ContactsService, CrateState, CreateAccountRequest, CreateActivityRequest,
    CreateCampaignRequest, CreateContactRequest, CreateDealRequest, CreateLeadForm,
    CreateLeadRequest, CreateOpportunityRequest, CrmAccount, CrmActivity, CrmCampaign,
    CrmContact, CrmDeal, CrmNote, CrmOpportunity, CrmPipelineStage, Contact,
    ContactActivity, ContactGroup, ContactSource, ContactStatus, ActivityType,
    ListQuery, PipelineStats, StageStats, CrmStats, UpdateDealRequest, UpdateLeadRequest,
    UpdateOpportunityRequest, ImportPostgresRequest, UpdateCampaignRequest, LeadStageQuery,
    CountStageQuery, ContactsApiCreateRequest, ContactsApiUpdateRequest, ContactListQuery,
    ContactListResponse, ImportRequest, ImportFormat, ImportResult, ImportError,
    create_contacts_tables_migration,
};

fn make_crate_state(app_state: &Arc<AppState>) -> CrateState {
    CrateState::new(
        app_state.conn.clone(),
        Arc::new(|conn| get_default_bot(conn)),
        Arc::new(|_conn, _contact_id, _action, _bot_id| {}),
        Arc::new(|_conn, _deal_id, _old_stage, _new_stage, _bot_id| {}),
    )
}

pub fn configure_crm_routes(app_state: Arc<AppState>) -> Router {
    botcontacts::routes::configure_crm_ui_routes()
        .with_state(Arc::new(make_crate_state(&app_state)))
}

pub fn configure_crm_api_routes(app_state: Arc<AppState>) -> Router {
    botcontacts::routes::configure_crm_api_routes()
        .with_state(Arc::new(make_crate_state(&app_state)))
}
