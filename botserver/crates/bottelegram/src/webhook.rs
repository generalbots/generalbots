use crate::adapter::TelegramAdapter;
use crate::media::message_content;
use crate::state::ChannelState;
use crate::session::{
    find_or_create_session_for_bot, resolve_bot_scope, resolve_bot_scope_by_name,
    route_to_attendant, route_to_bot,
};
use crate::types::{TelegramCallbackQuery, TelegramMessage, TelegramUpdate};
use crate::webhook_gate::{self, DeliveryAuth};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use log::{debug, info, warn};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Telegram redelivers an update whenever its webhook client does not receive
/// the 200 in time; its read timeout is far below what media processing takes
/// (BLIP description plus LLM rounds can exceed a minute). Acknowledge the
/// delivery BEFORE any processing and handle it in the background, and drop
/// update_ids already accepted inside the dedup window, so a slow media
/// message is processed once instead of once per redelivery.
const UPDATE_DEDUP_WINDOW: std::time::Duration = std::time::Duration::from_secs(600);

/// Redelivery guard with an injectable clock.
///
/// The window used to be a process-global `HashSet<(i64, Instant)>` whose
/// eviction only ran on the next delivery, which made the 600 s boundary
/// unreachable from a test. Keeping the state in a struct lets the tests drive
/// `now` directly while the handler still owns a single instance.
pub(crate) struct DedupRegistry {
    window: std::time::Duration,
    seen: HashSet<i64>,
    order: std::collections::VecDeque<(i64, std::time::Instant)>,
}

impl DedupRegistry {
    pub(crate) fn new(window: std::time::Duration) -> Self {
        Self {
            window,
            seen: HashSet::new(),
            order: std::collections::VecDeque::new(),
        }
    }

    /// Records `update_id` at `now` and reports whether it is new. An id is a
    /// duplicate while it is still inside the window; once it ages out the same
    /// id is accepted again, which is what keeps the set from growing forever.
    pub(crate) fn register_at(&mut self, update_id: i64, now: std::time::Instant) -> bool {
        self.evict_at(now);
        if !self.seen.insert(update_id) {
            return false;
        }
        self.order.push_back((update_id, now));
        true
    }

    fn evict_at(&mut self, now: std::time::Instant) {
        while let Some((update_id, accepted_at)) = self.order.front().copied() {
            if now.saturating_duration_since(accepted_at) < self.window {
                break;
            }
            self.order.pop_front();
            self.seen.remove(&update_id);
        }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.seen.len()
    }
}

fn update_dedup_registry() -> &'static Mutex<DedupRegistry> {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<Mutex<DedupRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(DedupRegistry::new(UPDATE_DEDUP_WINDOW)))
}

/// Returns true when this update_id was not seen inside the window.
async fn register_update_id(update_id: i64) -> bool {
    let mut registry = update_dedup_registry().lock().await;
    registry.register_at(update_id, std::time::Instant::now())
}

/// Placeholder carried into the conversation when an update has no text.
///
/// These markers are shown in the chat transcript and read by the model, so
/// they are localized (#2094). `locale` is the bot/workspace default: the
/// sender is a Telegram chat, not a `users` row with a stored preference.
pub(crate) fn extract_message_content(
    locale: &botlib::i18n::Locale,
    message: &TelegramMessage,
) -> String {
    if let Some(text) = &message.text {
        return text.clone();
    }
    if let Some(caption) = &message.caption {
        return caption.clone();
    }
    if message.photo.is_some() {
        return marker_text(locale, "channel-telegram-photo-received");
    }
    if message.document.is_some() {
        return marker_text(locale, "channel-telegram-document-received");
    }
    if message.voice.is_some() {
        return marker_text(locale, "channel-telegram-voice-received");
    }
    if message.audio.is_some() {
        return marker_text(locale, "channel-telegram-audio-received");
    }
    if message.video.is_some() {
        return marker_text(locale, "channel-telegram-video-received");
    }
    if let Some(location) = &message.location {
        let latitude = location.latitude.to_string();
        let longitude = location.longitude.to_string();
        return botlib::i18n::translate(
            locale,
            "channel-telegram-location-received",
            &[("latitude", &latitude), ("longitude", &longitude)],
        );
    }
    if let Some(contact) = &message.contact {
        return botlib::i18n::translate(
            locale,
            "channel-telegram-contact-received",
            &[
                ("name", &contact.first_name),
                ("phone", &contact.phone_number),
            ],
        );
    }
    String::new()
}

fn marker_text(locale: &botlib::i18n::Locale, key: &str) -> String {
    botlib::i18n::translate(locale, key, &[])
}

pub fn configure() -> Router<Arc<ChannelState>> {
    Router::new()
        .route("/webhook/telegram", post(handle_webhook))
        // axum 0.7 path-param syntax (":name"); "{name}" is axum 0.8+.
        .route(
            "/webhook/telegram/:bot_name",
            post(handle_webhook_for_bot),
        )
        .route("/api/telegram/send", post(crate::handlers::send_message))
}

/// Reads the configured secret for `bot_id` and compares it with the delivered
/// header. Fails closed on every path: an unreadable configuration answers
/// `500` (the message would fail later anyway, and a spoofed delivery must not
/// slip through during a database outage), an unset secret answers `503`
/// naming the operator action, and a wrong header answers `401`.
fn verify_webhook_secret_for_bot(
    state: &Arc<ChannelState>,
    bot_id: Uuid,
    headers: &HeaderMap,
) -> Result<(), StatusCode> {
    let expected = match (state.get_config)(&bot_id, webhook_gate::SECRET_CONFIG_KEY, None) {
        Ok(secret) => secret,
        Err(e) => {
            warn!("Telegram webhook secret unreadable for bot {bot_id}: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let provided = headers
        .get(webhook_gate::SECRET_HEADER)
        .and_then(|value| value.to_str().ok());

    match webhook_gate::authorize_delivery(&expected, provided) {
        DeliveryAuth::Verified => Ok(()),
        DeliveryAuth::Mismatch => {
            warn!(
                "Telegram webhook delivery for bot {bot_id} rejected: {} missing or mismatched",
                webhook_gate::SECRET_HEADER
            );
            Err(StatusCode::UNAUTHORIZED)
        }
        DeliveryAuth::Unconfigured => {
            webhook_gate::warn_unconfigured_once(bot_id);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

pub async fn handle_webhook(
    State(state): State<Arc<ChannelState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_webhook_in(state, None, headers, body).await
}

/// Per-bot inbound webhook: the bot name is carried in the path so a shared
/// deployment can point every bot at its own URL instead of funneling all
/// deliveries through the workspace default bot.
pub async fn handle_webhook_for_bot(
    State(state): State<Arc<ChannelState>>,
    Path(bot_name): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_webhook_in(state, Some(bot_name), headers, body).await
}

/// Gate first, parse second: an unsigned POST must not even reach the JSON
/// decoder, so a stranger cannot probe the schema through error messages.
async fn handle_webhook_in(
    state: Arc<ChannelState>,
    bot_name: Option<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // The delivery is attributed to one bot up front: from the URL path on the
    // per-bot route, from the workspace default otherwise. The same scope then
    // drives the secret check and the session, so a delivery can never be
    // verified against one bot and processed by another.
    let (bot_id, _branch_id) = match resolve_delivery_scope(&state, bot_name.as_deref()) {
        Ok(scope) => scope,
        Err(status) => return status.into_response(),
    };

    if let Err(status) = verify_webhook_secret_for_bot(&state, bot_id, &headers) {
        return status.into_response();
    }

    let update: TelegramUpdate = match serde_json::from_slice(&body) {
        Ok(update) => update,
        Err(e) => {
            warn!("Telegram webhook payload is not a valid update: {e}");
            return (StatusCode::BAD_REQUEST, "Invalid update").into_response();
        }
    };

    // Redelivery guard first: a duplicate update costs one registry lookup.
    if !register_update_id(update.update_id).await {
        info!(
            "Telegram webhook duplicate update_id={} dropped",
            update.update_id
        );
        return StatusCode::OK.into_response();
    }

    info!("Telegram webhook received: update_id={}", update.update_id);

    // Acknowledge before processing. Media handling (Drive upload, BLIP
    // description, LLM rounds) routinely exceeds the Telegram webhook client
    // timeout, and every timeout becomes a redelivery of the same update, so
    // the 200 goes out while processing continues in the background.
    let message = update.message.or(update.edited_message);
    let callback = update.callback_query;

    tokio::spawn(async move {
        if let Some(message) = message {
            if let Err(e) = process_message(state.clone(), bot_id, &message).await {
                log::error!("Failed to process Telegram message: {}", e);
            }
        }

        if let Some(callback) = callback {
            if let Err(e) = process_callback(state, bot_id, &callback).await {
                log::error!("Failed to process Telegram callback: {}", e);
            }
        }
    });

    StatusCode::OK.into_response()
}

/// Resolves the bot a delivery belongs to. The per-bot route looks the name up
/// and answers `404` for an unknown one — falling back to the workspace
/// default would attribute the message to the wrong conversation. The plain
/// route keeps the workspace default resolution.
fn resolve_delivery_scope(
    state: &Arc<ChannelState>,
    bot_name: Option<&str>,
) -> Result<(Uuid, Uuid), StatusCode> {
    let mut conn = state.conn.get().map_err(|e| {
        log::error!("Telegram webhook scope check failed, no database connection: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match bot_name {
        Some(name) => resolve_bot_scope_by_name(&mut conn, name).ok_or_else(|| {
            warn!("Telegram webhook delivery for unknown bot '{name}' rejected");
            StatusCode::NOT_FOUND
        }),
        None => Ok(resolve_bot_scope(state, &mut conn)),
    }
}

async fn process_message(
    state: Arc<ChannelState>,
    bot_id: Uuid,
    message: &TelegramMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = message.chat.id.to_string();
    let user = message.from.as_ref();

    let user_name = user
        .map(|u| {
            let mut name = u.first_name.clone();
            if let Some(last) = &u.last_name {
                name.push(' ');
                name.push_str(last);
            }
            name
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let session = find_or_create_session_for_bot(&state, bot_id, &chat_id, &user_name)?;

    let adapter = TelegramAdapter::new(
        state.conn.clone(),
        session.bot_id,
        state.get_config.clone(),
    );

    // Media is fetched into Drive before routing, so the bot receives the
    // stored path instead of a placeholder.
    let content = message_content(&state, &adapter, message, session.bot_id).await;

    if content.is_empty() {
        debug!("Empty message content, skipping");
        return Ok(());
    }

    let preview: String = content.chars().take(50).collect();

    botlog::safe_info!(
        "Processing Telegram message from {} (chat_id={}): {}",
        user_name, chat_id, preview
    );

    let assigned_to = session
        .context_data
        .get("assigned_to")
        .and_then(|v| v.as_str());

    if assigned_to.is_some() {
        route_to_attendant(state, &session, &content, &chat_id, &user_name)?;
    } else {
        route_to_bot(state, &session, &content, &chat_id).await?;
    }

    Ok(())
}

async fn process_callback(
    state: Arc<ChannelState>,
    bot_id: Uuid,
    callback: &TelegramCallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = callback
        .message
        .as_ref()
        .map(|m| m.chat.id.to_string())
        .unwrap_or_default();

    let user_name = {
        let mut name = callback.from.first_name.clone();
        if let Some(last) = &callback.from.last_name {
            name.push(' ');
            name.push_str(last);
        }
        name
    };

    let data = callback.data.clone().unwrap_or_default();

    if data.is_empty() || chat_id.is_empty() {
        return Ok(());
    }

    info!(
        "Processing Telegram callback from {} (chat_id={}): {}",
        user_name, chat_id, data
    );

    let session = find_or_create_session_for_bot(&state, bot_id, &chat_id, &user_name)?;

    route_to_bot(state, &session, &data, &chat_id).await?;

    Ok(())
}

#[cfg(test)]
#[path = "webhook_tests.rs"]
mod webhook_tests;
