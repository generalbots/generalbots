//! Error frame construction for the LLM stream pipeline (#2562).
//!
//! Extracted from `stream.rs` to keep files under 450 lines.

use super::super::sink::ChannelSink;
use super::super::types::SessionIdentity;
use super::error_classify::LlmErrorCode;

/// Sends a classified LLM error frame to the client.
///
/// The frame carries stable error codes so the client renders the right card
/// with retry and settings buttons.
pub(super) async fn send_classified_error(
    sink: &dyn ChannelSink,
    identity: &SessionIdentity<'_>,
    code: LlmErrorCode,
) {
    let safe_msg = code.safe_message();
    let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, identity.user_id);
    let i18n_key = match code {
        LlmErrorCode::ModelUnreachable => "error-model-unreachable",
        LlmErrorCode::ModelAuth => "error-model-auth",
        LlmErrorCode::ModelRateLimited => "error-model-rate-limited",
        LlmErrorCode::ModelError => "error-model-error",
        LlmErrorCode::Timeout => "error-model-timeout",
        LlmErrorCode::ContextExceeded => "error-model-context",
    };
    let msg = {
        let translated = botcore::i18n::t(&locale, i18n_key);
        if translated.starts_with('[') && translated.ends_with(']') {
            safe_msg.to_string()
        } else {
            translated
        }
    };
    let bot_uuid_s = identity.bot_uuid.to_string();
    let session_id_s = identity.session_id.to_string();
    let err_frame = serde_json::json!({
        "bot_id": bot_uuid_s,
        "user_id": identity.user_id.to_string(),
        "session_id": session_id_s,
        "channel": "web",
        "content": msg,
        "message_type": botlib::message_types::MessageType::BOT_RESPONSE,
        "is_complete": true,
        "is_error": true,
        "error": true,
        "error_code": code,
        "retryable": code.is_retryable(),
        "status": "failed",
        "turn_status": "failed",
        "suggestions": [],
        "switchers": [],
        "context_length": 0,
        "context_max_length": 0,
    });
    if sink.supports_raw_frames() {
        if let Err(error) = sink.send_raw_json(&err_frame).await {
            log::error!("LLM error frame not delivered: {error}");
        }
    } else {
        let err_resp = botlib::models::BotResponse::new(
            bot_uuid_s,
            session_id_s,
            identity.user_id.to_string(),
            &msg,
            "web",
        );
        if let Err(error) = sink.send_bot_response(&err_resp).await {
            log::error!("LLM error response not delivered: {error}");
        }
    }
}

/// Sends an empty-response error frame after all retries are exhausted.
pub(super) async fn send_empty_response_error(
    sink: &dyn ChannelSink,
    identity: &SessionIdentity<'_>,
) {
    let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, identity.user_id);
    let err_msg = super::llm_empty_message(&locale);
    let bot_uuid_s = identity.bot_uuid.to_string();
    let session_id_s = identity.session_id.to_string();
    let err_frame = serde_json::json!({
        "bot_id": bot_uuid_s,
        "user_id": identity.user_id.to_string(),
        "session_id": session_id_s,
        "channel": "web",
        "content": err_msg,
        "message_type": botlib::message_types::MessageType::BOT_RESPONSE,
        "is_complete": true,
        "is_error": true,
        "error": true,
        "retryable": true,
        "status": "failed",
        "turn_status": "failed",
        "suggestions": [],
        "switchers": [],
        "context_length": 0,
        "context_max_length": 0,
    });
    if sink.supports_raw_frames() {
        if let Err(error) = sink.send_raw_json(&err_frame).await {
            log::error!("empty-response notice not delivered: {error}");
        }
    } else {
        let err_resp = botlib::models::BotResponse::new(
            &bot_uuid_s,
            &session_id_s,
            identity.user_id.to_string(),
            &err_msg,
            "web",
        );
        if let Err(error) = sink.send_bot_response(&err_resp).await {
            log::error!("empty-response notice not delivered: {error}");
        }
    }
}
