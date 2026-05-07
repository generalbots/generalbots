use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botattendance::{
    AttendanceConfig, AttendanceDriveConfig, AttendanceDriveService, RecordMetadata, SyncResult,
    AttendanceCommand, AttendanceRecord, AttendanceResponse, AttendanceService, KeywordConfig,
    KeywordParser, ParsedCommand, AttendantStats, AttendantStatus, QueueFilters, QueueItem,
    QueueStatus, AssignRequest, TransferRequest, DbPool, GetDefaultBotFn, LlmGenerateFn,
    ProcessContentFn, ConfigGetFn, SendBotResponseFn, BroadcastNotificationFn, SaveMessageFn,
};
#[cfg(feature = "llm")]
pub use botattendance::{
    LlmAssistConfig, LlmAssistMode, LlmAssistSettings,
    AttendantTip as ParsedTip, SmartReply as ParsedSmartReply,
    ConversationSummary as ParsedConversationSummary, SentimentAnalysis as ParsedSentimentAnalysis,
    parse_tips_response as parse_tips_response_alt,
    parse_polish_response as parse_polish_response_alt,
    parse_smart_replies_response as parse_smart_replies_response_alt,
    parse_summary_response as parse_summary_response_alt,
    parse_sentiment_response as parse_sentiment_response_alt,
    extract_json as extract_json_alt,
};

pub mod llm_assist {
    use super::*;

    pub async fn process_attendant_command(
        state: &Arc<AppState>,
        attendant_phone: &str,
        command: &str,
        current_session: Option<uuid::Uuid>,
    ) -> Result<String, String> {
        let config = build_attendance_config(state);
        botattendance::llm_assist_commands::process_attendant_command(
            &Arc::new(config),
            attendant_phone,
            command,
            current_session,
        )
        .await
    }
}

fn build_attendance_config(state: &Arc<AppState>) -> AttendanceConfig {
    AttendanceConfig {
        pool: Arc::new(state.conn.clone()),
        get_default_bot: Arc::new(|_user_id| Err("not configured".to_string())),
        llm_generate: Arc::new(|_prompt| Err("not configured".to_string())),
        process_content: Arc::new(|_content| Err("not configured".to_string())),
        config_get: Arc::new(|_key| None),
        send_bot_response: None,
        broadcast_notification: None,
        save_message: None,
    }
}

pub fn configure_attendance_routes(state: &Arc<AppState>) -> Router<()> {
    let config = Arc::new(build_attendance_config(state));
    botattendance::configure_attendance_routes().with_state(config)
}
