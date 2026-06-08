use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers::*;
use crate::unified_inbox;
use crate::threading;
use crate::search_handler;
use crate::draft_handler;

pub fn configure(state: Arc<crate::models::AppState>) -> Router {
    Router::new()
        .route("/api/email/accounts", get(accounts::list_email_accounts))
        .route("/api/email/accounts/add", post(accounts::add_email_account))
        .route("/api/email/accounts/{account_id}", delete(accounts::delete_email_account))
        .route("/api/email/list", post(messages::list_emails))
        .route("/api/email/send", post(messages::send_email))
        .route("/api/email/draft", post(messages::save_draft))
        .route("/api/email/bulk/archive", post(messages::archive_emails_bulk))
        .route("/api/email/bulk/mark-read", post(messages::mark_emails_read_bulk))
        .route("/api/email/bulk/delete", post(messages::delete_emails_bulk))
        .route("/api/email/labels", post(messages::create_label))
        .route("/api/email/bulk/add-label", post(messages::add_label_to_emails_bulk))
        .route("/api/email/rules", post(htmx::create_rule))
        .route("/api/email/templates", post(htmx::create_template))
        .route("/api/email/folders/{account_id}", get(messages::list_folders))
        .route("/api/email/latest", get(tracking::get_latest_email))
        .route("/api/email/campaign/{campaign_id}", get(tracking::get_email))
        .route("/api/email/campaign/{campaign_id}/{email}", post(tracking::track_click))
        .route("/api/email/tracking/pixel/{tracking_id}", get(tracking::serve_tracking_pixel))
        .route("/api/email/tracking/status/{tracking_id}", get(tracking::get_tracking_status))
        .route("/api/email/tracking/list", get(tracking::list_sent_emails_tracking))
        .route("/api/email/tracking/stats", get(tracking::get_tracking_stats))
        .route("/api/ui/email/accounts", get(accounts::list_email_accounts_htmx))
        .route("/api/ui/email/list", get(htmx::list_emails_htmx))
        .route("/api/ui/email/folders", get(htmx::list_folders_htmx))
        .route("/api/ui/email/compose", get(htmx::compose_email_htmx))
        .route("/api/ui/email/content/{id}", get(htmx::get_email_content_htmx))
        .route("/api/ui/email/{id}/delete", delete(htmx::delete_email_htmx))
        .route("/api/ui/email/labels", get(htmx::list_labels_htmx))
        .route("/api/ui/email/templates", get(htmx::list_templates_htmx))
        .route("/api/ui/email/signatures", get(htmx::list_signatures_htmx))
        .route("/api/ui/email/rules", get(htmx::list_rules_htmx))
        .route("/api/ui/email/search", get(htmx::search_emails_htmx))
        .route("/api/ui/email/unified-list", get(unified_inbox::list_unified_inbox))
        .route("/api/ui/email/unified-stats", get(unified_inbox::unified_stats))
        .route("/api/ui/email/search-all", get(search_handler::search_emails))
        .route("/api/ui/email/thread/{thread_id}", get(threading::get_thread))
        .route("/api/email/draft/auto", post(draft_handler::upsert_draft))
        .route("/api/ui/email/auto-responder", post(htmx::save_auto_responder))
        .route("/api/features/{org_id}/enabled", get(integration::get_feature_flags))
        .route("/api/ai/extract-lead", post(integration::extract_lead_from_email))
        .route("/api/crm/contact/by-email/{email}", get(integration::get_crm_context_by_email))
        .route("/api/email/crm/link", post(integration::link_email_to_crm))
        .route("/api/ai/categorize-email", post(integration::categorize_email))
        .route("/api/ai/generate-reply", post(integration::generate_smart_reply))
        .route("/api/email/snooze", post(snooze::snooze_emails))
        .route("/api/email/snoozed", get(snooze::get_snoozed_emails))
        .route("/api/email/nudges", post(nudges::check_nudges))
        .route("/api/email/nudge/dismiss", post(nudges::dismiss_nudge))
        .route("/api/email/flag", post(flags::flag_for_followup))
        .route("/api/email/flag/clear", post(flags::clear_flag))
        .route("/api/email/signatures", get(signatures::list_signatures).post(signatures::create_signature))
        .route("/api/email/signatures/default", get(signatures::get_default_signature))
        .route("/api/email/signatures/{id}", get(signatures::get_signature).put(signatures::update_signature).delete(signatures::delete_signature))
        .with_state(state)
}
