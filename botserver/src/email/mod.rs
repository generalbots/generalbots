use crate::{core::urls::ApiUrls, core::shared::state::AppState};
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

pub mod ui;
pub mod stalwart_client;
pub mod stalwart_sync;
pub mod vectordb;

pub mod types;
pub mod accounts;
pub mod messages;
pub mod tracking;
pub mod signatures;
pub mod htmx;
pub mod integration_types;
pub mod integration;
pub mod snooze;
pub mod nudges;
pub mod flags;

#[cfg(test)]
mod integration_types_test;

#[cfg(test)]
mod integration_tests;

pub use types::*;
pub use accounts::*;
pub use messages::*;
pub use tracking::*;
pub use signatures::*;
pub use htmx::*;
pub use integration_types::*;
pub use integration::*;
pub use snooze::*;
pub use nudges::*;
pub use flags::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags() {
        let flags = FeatureFlags {
            crm_enabled: true,
            campaigns_enabled: false,
        };
        assert!(flags.crm_enabled);
        assert!(!flags.campaigns_enabled);
    }

    #[test]
    fn test_lead_extraction() {
        let response = LeadExtractionResponse {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            email: "john@example.com".to_string(),
            company: Some("Acme".to_string()),
            phone: None,
            value: Some(50000.0),
        };
        assert_eq!(response.email, "john@example.com");
    }
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::EMAIL_ACCOUNTS, get(list_email_accounts))
        .route(
            &format!("{}/add", ApiUrls::EMAIL_ACCOUNTS),
            post(add_email_account),
        )
        .route(
            &ApiUrls::EMAIL_ACCOUNT_BY_ID.replace(":id", "{account_id}"),
            axum::routing::delete(delete_email_account),
        )
        .route(ApiUrls::EMAIL_LIST, post(list_emails))
        .route(ApiUrls::EMAIL_SEND, post(send_email))
        .route(ApiUrls::EMAIL_DRAFT, post(save_draft))
        .route(
            &ApiUrls::EMAIL_FOLDERS.replace(":account_id", "{account_id}"),
            get(list_folders),
        )
        .route(ApiUrls::EMAIL_LATEST, get(get_latest_email))
        .route(
            &ApiUrls::EMAIL_GET.replace(":campaign_id", "{campaign_id}"),
            get(get_email),
        )
        .route(
            &ApiUrls::EMAIL_CLICK
                .replace(":campaign_id", "{campaign_id}")
                .replace(":email", "{email}"),
            post(track_click),
        )
        .route(
            "/api/email/tracking/pixel/{tracking_id}",
            get(serve_tracking_pixel),
        )
        .route(
            "/api/email/tracking/status/{tracking_id}",
            get(get_tracking_status),
        )
        .route("/api/email/tracking/list", get(list_sent_emails_tracking))
        .route("/api/email/tracking/stats", get(get_tracking_stats))
        .route(ApiUrls::EMAIL_ACCOUNTS_HTMX, get(list_email_accounts_htmx))
        .route(ApiUrls::EMAIL_LIST_HTMX, get(list_emails_htmx))
        .route(ApiUrls::EMAIL_FOLDERS_HTMX, get(list_folders_htmx))
        .route(ApiUrls::EMAIL_COMPOSE_HTMX, get(compose_email_htmx))
        .route(ApiUrls::EMAIL_CONTENT_HTMX, get(get_email_content_htmx))
        .route("/api/ui/email/:id/delete", delete(delete_email_htmx))
        .route(ApiUrls::EMAIL_LABELS_HTMX, get(list_labels_htmx))
        .route(ApiUrls::EMAIL_TEMPLATES_HTMX, get(list_templates_htmx))
        .route(ApiUrls::EMAIL_SIGNATURES_HTMX, get(list_signatures_htmx))
        .route(ApiUrls::EMAIL_RULES_HTMX, get(list_rules_htmx))
        .route(ApiUrls::EMAIL_SEARCH_HTMX, get(search_emails_htmx))
        // Integration routes
        .route("/api/features/:org_id/enabled", get(get_feature_flags))
        .route("/api/ai/extract-lead", post(extract_lead_from_email))
        .route("/api/crm/contact/by-email/:email", get(get_crm_context_by_email))
        .route("/api/email/crm/link", post(link_email_to_crm))
        .route("/api/ai/categorize-email", post(categorize_email))
        .route("/api/ai/generate-reply", post(generate_smart_reply))
        // Email features
        .route("/api/email/snooze", post(snooze_emails))
        .route("/api/email/snoozed", get(get_snoozed_emails))
        .route("/api/email/nudges", post(check_nudges))
        .route("/api/email/nudge/dismiss", post(dismiss_nudge))
        .route("/api/email/flag", post(flag_for_followup))
        .route("/api/email/flag/clear", post(clear_flag))
        .route(ApiUrls::EMAIL_AUTO_RESPONDER_HTMX, post(save_auto_responder))
        .route("/api/email/signatures", get(list_signatures).post(create_signature))
        .route("/api/email/signatures/default", get(get_default_signature))
        .route("/api/email/signatures/{id}", get(get_signature).put(update_signature).delete(delete_signature))
}
