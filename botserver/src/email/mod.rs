use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub mod stalwart_client;
pub mod stalwart_sync;
pub mod ui;
pub mod vectordb;

pub use botemail::{
    AppState as EmailAppState, CreateSignatureRequest, DeleteEmailRequest, EmailAccountRequest,
    EmailAccountResponse, EmailConfig, EmailError, EmailResponse, EmailSignature,
    FlagRequest, FlagResponse, FolderInfo, ListEmailsRequest, MarkEmailRequest,
    Nudge, NudgeCheckRequest, NudgesResponse, SaveDraftRequest, SaveDraftResponse,
    SendEmailRequest, SentEmailTracking, SmartReplyRequest, SmartReplyResponse,
    SnoozeRequest, SnoozeResponse, TrackingPixelQuery, TrackingStatsResponse,
    TrackingStatusResponse, UpdateSignatureRequest, ApiResponse as EmailApiResponse,
    AuthenticatedUser, EmailCategoryResponse, EmailCrmLink, EmailTrackingParams,
    FeatureFlags, LeadExtractionRequest, LeadExtractionResponse,
};

pub use ui::configure_email_ui_routes;
pub use vectordb::{EmailDocument, UserEmailVectorDB};

use botemail::models::{AppState as BotEmailAppState, SecretsProvider};

fn make_email_state(app_state: &Arc<AppState>) -> Arc<BotEmailAppState> {
    Arc::new(BotEmailAppState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn| get_default_bot(conn)),
        secrets_provider: Arc::new(|key| {
            crate::core::shared::utils::get_secrets_manager_sync()
                .and_then(|s| s.get_secret(key).ok())
                .ok_or_else(|| format!("Secret '{}' not found", key))
        }),
    })
}

pub fn configure(app_state: Arc<AppState>) -> Router {
    botemail::routes::configure(make_email_state(&app_state))
}
