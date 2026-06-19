pub mod tax;
pub mod tax_storage;
pub mod video;
pub mod vision;
pub mod fraud;
pub mod erp;
pub mod integrations;
pub mod itsm;
pub mod itsm_storage;
pub mod hr;
pub mod banking;
pub mod sales;
pub mod pos;
pub mod handoff;
pub mod kyc;
pub mod timeclock;
pub mod m365;
pub mod minutes;
pub mod templates_app;
pub mod database;
pub mod db;
pub mod ui_fragments;

use axum::routing::{get, post, put, delete, patch};
use axum::Router;

pub fn register<S: Clone + Send + Sync + 'static>(r: Router<S>) -> Router<S> {
    r // tax
        .route("/api/tax/nfe", get(tax::list_nfe).post(tax::create_nfe))
        .route("/api/tax/nfe/{id}/authorize", post(tax::authorize_nfe))
        .route("/api/tax/nfse", get(tax::list_nfse).post(tax::create_nfse))
        .route("/api/tax/cte", get(tax::list_cte).post(tax::create_cte))
        .route("/api/tax/sped", get(tax::list_sped))
        // video
        .route("/api/video/cameras", get(video::list_cameras).post(video::create_camera))
        .route("/api/video/cameras/{id}", delete(video::delete_camera))
        .route("/api/video/alerts", get(video::list_alerts))
        .route("/api/video/analytics", get(video::list_analytics))
        // vision
        .route("/api/vision/analyze", post(vision::analyze_image))
        .route("/api/vision/history", get(vision::list_history))
        // fraud
        .route("/api/fraud/transactions", get(fraud::list_transactions).post(fraud::create_transaction))
        .route("/api/fraud/rules", get(fraud::list_rules).post(fraud::create_rule))
        .route("/api/fraud/rules/{id}", put(fraud::update_rule))
        .route("/api/fraud/blocklist", get(fraud::list_blocklist).post(fraud::add_blocklist))
        .route("/api/fraud/blocklist/{id}", delete(fraud::remove_blocklist))
        // erp
        .route("/api/erp/financial", get(erp::get_financial))
        .route("/api/erp/inventory", get(erp::list_inventory))
        .route("/api/erp/procurement", get(erp::list_procurement))
        .route("/api/erp/branches", get(erp::list_branches))
        // integrations
        .route("/api/integrations/connectors", get(integrations::list_connectors))
        .route("/api/integrations/connectors/{id}/connect", post(integrations::connect_connector))
        .route("/api/integrations/connectors/{id}/disconnect", post(integrations::disconnect_connector))
        .route("/api/integrations/etl", get(integrations::list_etl))
        // itsm
        .route("/api/itsm/incidents", get(itsm::list_incidents).post(itsm::create_incident))
        .route("/api/itsm/incidents/{id}", put(itsm::update_incident))
        .route("/api/itsm/requests", get(itsm::list_requests).post(itsm::create_request))
        .route("/api/itsm/cmdb", get(itsm::list_cmdb))
        .route("/api/itsm/kb", get(itsm::list_kb))
        // hr
        .route("/api/hr/employees", get(hr::list_employees).post(hr::create_employee))
        .route("/api/hr/employees/{id}", put(hr::update_employee))
        .route("/api/hr/recruitment", get(hr::list_recruitment))
        .route("/api/hr/attendance", get(hr::list_attendance))
        // banking
        .route("/api/banking/transactions", get(banking::list_transactions).post(banking::create_transaction))
        .route("/api/banking/platforms", get(banking::list_platforms))
        .route("/api/banking/reconcile", get(banking::list_reconcile_pairs).post(banking::reconcile))
        .route("/api/banking/reconcile/match", post(banking::manual_match))
        .route("/api/banking/platforms/{id}/sync", post(banking::sync_platform))
        .route("/api/banking/reports", get(banking::get_report))
        // sales
        .route("/api/sales/deals", get(sales::list_deals).post(sales::create_deal))
        .route("/api/sales/deals/{id}", put(sales::update_deal))
        .route("/api/sales/contacts", get(sales::list_contacts))
        .route("/api/sales/activities", get(sales::list_activities))
        .route("/api/sales/forecast", get(sales::get_forecast))
        // pos
        .route("/api/pos/products", get(pos::list_products).post(pos::create_product))
        .route("/api/pos/orders", get(pos::list_orders).post(pos::create_order))
        .route("/api/pos/orders/{id}", get(pos::get_order))
        // handoff
        .route("/api/handoff/queue", get(handoff::list_queue))
        .route("/api/handoff/transfer/{id}", post(handoff::transfer_item))
        .route("/api/handoff/analytics", get(handoff::get_analytics))
        .route("/api/handoff/channels", get(handoff::list_channels))
        .route("/api/handoff/csat", get(handoff::list_csat))
        // kyc
        .route("/api/kyc/verifications", get(kyc::list_verifications))
        .route("/api/kyc/verifications/{id}", put(kyc::update_verification))
        .route("/api/kyc/signatures", get(kyc::list_signatures))
        .route("/api/kyc/signatures/{id}/sign", post(kyc::sign_document))
        .route("/api/kyc/certificates", get(kyc::list_certificates))
        // timeclock
        .route("/api/timeclock/clock", post(timeclock::clock_in_out))
        .route("/api/timeclock/records", get(timeclock::list_records))
        .route("/api/timeclock/overtime", get(timeclock::list_overtime))
        .route("/api/timeclock/overtime/{id}/approve", post(timeclock::approve_overtime))
        .route("/api/timeclock/reports", get(timeclock::get_reports))
        // m365
        .route("/api/m365/sharepoint", get(m365::list_sharepoint))
        .route("/api/m365/calendar", get(m365::list_calendar))
        .route("/api/m365/onedrive", get(m365::list_onedrive))
        .route("/api/m365/settings", get(m365::get_settings))
        // minutes
        .route("/api/minutes/meetings", get(minutes::list_meetings))
        .route("/api/minutes/transcripts", get(minutes::list_transcripts))
        .route("/api/minutes/documents", get(minutes::list_documents))
        .route("/api/minutes/documents/{id}", put(minutes::update_document))
        .route("/api/minutes/forms/meeting/start/{id}", post(minutes::start_meeting))
        .route("/api/minutes/forms/meeting/{id}", patch(minutes::update_meeting))
        // templates
        .route("/api/templates/list", get(templates_app::list_templates))
        .route("/api/templates/preview/{id}", get(templates_app::preview_template))
        .route("/api/templates/deploy/{id}", post(templates_app::deploy_template))
        // HTMX fragment handlers for brazil, timeclock, minutes apps
        .merge(ui_fragments::configure())
}
