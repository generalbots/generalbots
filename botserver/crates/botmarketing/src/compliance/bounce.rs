//! Bounce classification, bounded retry and bounce webhook (#2536).
//!
//! A transient failure is retried with exponential backoff; a hard bounce or
//! spam complaint is terminal, auto-suppresses the address and never retries.
//! The attempt counter is persisted in `marketing_recipients.response` (jsonb)
//! because no migration may be added from this crate. A terminal recipient is
//! moved to `dead_letter` / `bounced` / `complaint` status.

use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use super::bounce_signing::{
    authorized_branch, signing_key, verify_bounce_signature, BounceClaims,
};
use super::suppression::{
    persist_suppression, MarketingChannel, SuppressionKey, SuppressionStore,
};
use crate::schema::{marketing_contacts, marketing_recipients};
use crate::state::AppState;

/// Maximum number of delivery attempts for one recipient.
pub const MAX_SEND_ATTEMPTS: u32 = 3;
/// Delay before the first retry (doubled each subsequent attempt): 2s, 4s, 8s…
pub const RETRY_BASE_MILLIS: u64 = 2_000;

/// Provider-independent bounce/failure classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BounceClass {
    Hard,
    Soft,
    Complaint,
    Transient,
    Unknown,
}

impl BounceClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hard => "hard",
            Self::Soft => "soft",
            Self::Complaint => "complaint",
            Self::Transient => "transient",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "hard" | "hard_bounce" | "permanent" => Some(Self::Hard),
            "soft" | "soft_bounce" => Some(Self::Soft),
            "complaint" | "spam_complaint" | "abuse" => Some(Self::Complaint),
            "transient" | "temporary" => Some(Self::Transient),
            _ => None,
        }
    }

    /// Hard bounces and complaints must never be retried. Unknown failures
    /// fail closed as terminal so a provider outage cannot mail an address
    /// whose status we cannot interpret.
    pub fn is_retryable(self) -> bool {
        matches!(self, Self::Soft | Self::Transient)
    }

    pub fn is_terminal(self) -> bool {
        !self.is_retryable()
    }
}

/// Classifies a provider error string. Keyword lists are deliberately small
/// and provider-agnostic; the webhook payload can override with an explicit
/// typed `bounce_type`.
pub fn classify_bounce(raw: &str) -> BounceClass {
    let text = raw.to_ascii_lowercase();
    const COMPLAINT: [&str; 5] = [
        "spam complaint",
        "complaint",
        "abuse report",
        "marked as spam",
        "reported as spam",
    ];
    const HARD: [&str; 12] = [
        "hard bounce",
        "5.1.1",
        "550",
        "mailbox does not exist",
        "recipient not found",
        "invalid recipient",
        "user unknown",
        "domain not found",
        "address not found",
        "permanent failure",
        "no such user",
        "bad destination",
    ];
    const SOFT: [&str; 9] = [
        "soft bounce",
        "mailbox full",
        "4.2.2",
        "over quota",
        "temporary failure",
        "greylist",
        "try again later",
        "mailbox unavailable",
        "deferred",
    ];
    const TRANSIENT: [&str; 8] = [
        "timeout",
        "timed out",
        "connection reset",
        "rate limit",
        "429",
        "503",
        "temporarily unavailable",
        "try again",
    ];
    if COMPLAINT.iter().any(|needle| text.contains(needle)) {
        return BounceClass::Complaint;
    }
    if HARD.iter().any(|needle| text.contains(needle)) {
        return BounceClass::Hard;
    }
    if SOFT.iter().any(|needle| text.contains(needle)) {
        return BounceClass::Soft;
    }
    if TRANSIENT.iter().any(|needle| text.contains(needle)) {
        return BounceClass::Transient;
    }
    BounceClass::Unknown
}

/// Exponential backoff: 2s, 4s after attempts 1 and 2 (capped).
pub fn retry_delay_millis(attempt: u32) -> u64 {
    let shift = attempt.min(6);
    RETRY_BASE_MILLIS.saturating_mul(1u64 << shift)
}

fn truncate_error(raw: &str) -> String {
    const MAX_LEN: usize = 512;
    if raw.len() <= MAX_LEN {
        return raw.to_string();
    }
    let mut end = MAX_LEN;
    while end > 0 && !raw.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &raw[..end])
}

/// Persisted per-recipient delivery attempt state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttemptState {
    pub attempts: u32,
    pub last_error: Option<String>,
    pub bounce_class: Option<BounceClass>,
}

impl AttemptState {
    /// Reads the counter from `marketing_recipients.response`. A malformed or
    /// missing value starts a fresh attempt rather than failing the send.
    pub fn from_json(value: Option<&Value>) -> Self {
        let Some(value) = value else {
            return Self::default();
        };
        let attempts = value
            .get("attempts")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .min(MAX_SEND_ATTEMPTS as u64) as u32;
        let last_error = value
            .get("last_error")
            .and_then(Value::as_str)
            .map(truncate_error);
        let bounce_class = value
            .get("bounce_class")
            .and_then(Value::as_str)
            .and_then(BounceClass::parse);
        Self {
            attempts,
            last_error,
            bounce_class,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("attempts".into(), json!(self.attempts));
        if let Some(error) = &self.last_error {
            map.insert("last_error".into(), json!(error));
        }
        if let Some(class) = self.bounce_class {
            map.insert("bounce_class".into(), json!(class.as_str()));
        }
        Value::Object(map)
    }

    /// Increments the counter. A terminal classification exhausts the budget
    /// immediately so no further attempt is scheduled.
    pub fn record(&mut self, error: &str, class: BounceClass) {
        self.attempts = self.attempts.saturating_add(1);
        self.last_error = Some(truncate_error(error));
        self.bounce_class = Some(class);
        if class.is_terminal() {
            self.attempts = MAX_SEND_ATTEMPTS;
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.attempts >= MAX_SEND_ATTEMPTS
    }
}

/// Persists the attempt counter without needing a new column.
pub fn persist_attempt(
    conn: &mut PgConnection,
    recipient_id: Uuid,
    state: &AttemptState,
) -> Result<(), String> {
    diesel::update(
        marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
    )
    .set(marketing_recipients::response.eq(Some(state.to_json())))
    .execute(conn)
    .map(|_| ())
    .map_err(|error| format!("Attempt update error: {error}"))
}

/// Webhook body accepted by `POST /api/marketing/bounce`.
///
/// `signature` is the HMAC capability over the other fields (see
/// [`super::bounce_signing`]). The endpoint makes no state change until it has
/// been verified, and the tenant used for suppression comes from the signed
/// claim plus the persisted recipient row, never from these raw fields.
#[derive(Debug, Clone, Deserialize)]
pub struct BounceWebhookPayload {
    pub recipient_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub address: Option<String>,
    pub channel: Option<String>,
    pub reason: Option<String>,
    pub bounce_type: Option<String>,
    pub signature: Option<String>,
}

/// Typed result of processing a bounce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BounceResult {
    pub recipient_id: Uuid,
    pub classification: String,
    pub suppressed: bool,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BounceError {
    MissingIdentifier,
    NotFound,
    Database,
}

impl BounceError {
    fn reason(self) -> &'static str {
        match self {
            Self::MissingIdentifier => "recipient_id or address is required",
            Self::NotFound => "recipient not found",
            Self::Database => "bounce storage unavailable",
        }
    }
}

type RecipientRow = (Uuid, Uuid, Option<String>, Option<String>, Option<Value>);

/// Loads the recipient using the **verified** claim only.
///
/// #2886 — the branch filter is always applied from the signed claim
/// (`caller_branch`), so a capability issued for tenant A can never resolve a
/// recipient belonging to tenant B. The raw body's `branch_id` is never used to
/// select rows.
fn load_recipient(conn: &mut PgConnection, claims: &BounceClaims) -> Result<RecipientRow, BounceError> {
    let mut query = marketing_recipients::table
        .select((
            marketing_recipients::id,
            marketing_recipients::branch_id,
            marketing_recipients::email,
            marketing_recipients::channel,
            marketing_recipients::response,
        ))
        .filter(marketing_recipients::branch_id.eq(claims.branch_id))
        .into_boxed();
    match (claims.recipient_id, claims.address.as_deref()) {
        (Some(id), _) => {
            query = query.filter(marketing_recipients::id.eq(id));
        }
        (None, Some(address)) => {
            query = query.filter(marketing_recipients::email.eq(address));
        }
        _ => return Err(BounceError::MissingIdentifier),
    }
    query
        .first::<RecipientRow>(conn)
        .map_err(|error| match error {
            diesel::result::Error::NotFound => BounceError::NotFound,
            _ => {
                log::error!("bounce recipient lookup failed: {error}");
                BounceError::Database
            }
        })
}

fn mark_contact_suppressed(conn: &mut PgConnection, branch_id: Uuid, address: &str) {
    if let Err(error) = diesel::update(
        marketing_contacts::table
            .filter(marketing_contacts::branch_id.eq(branch_id))
            .filter(marketing_contacts::email.eq(address)),
    )
    .set((
        marketing_contacts::subscribed.eq(Some(false)),
        marketing_contacts::updated_at.eq(chrono::Utc::now()),
    ))
    .execute(conn)
    {
        log::error!("bounce: could not persist contact suppression: {error}");
    }
}

/// `POST /api/marketing/bounce` — records a bounce/complaint, auto-suppresses
/// terminal addresses and updates the recipient's status and attempt counter.
///
/// #2886 — the signed capability is verified **before** the connection is even
/// acquired, so a forged or missing signature cannot mutate consent anywhere.
/// Every decision below uses the verified claim; the raw body is only a
/// transport envelope.
pub async fn record_bounce(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BounceWebhookPayload>,
) -> Result<Json<BounceResult>, (StatusCode, String)> {
    let claims = verify_bounce_signature(signing_key(), &body).map_err(|error| {
        log::warn!("bounce capability rejected: {}", error.reason());
        (error.status(), error.reason().to_string())
    })?;

    let mut conn = state
        .conn
        .get()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, BounceError::Database.reason().to_string()))?;
    let (recipient_id, recipient_branch, stored_email, stored_channel, stored_response) =
        load_recipient(&mut conn, &claims)
            .map_err(|error| (StatusCode::BAD_REQUEST, error.reason().to_string()))?;
    // The tenant is fixed by the persisted recipient row the capability
    // selected, never by the raw body. Cross-check it against the signed claim
    // so a capability for tenant A cannot suppress a recipient in tenant B.
    let branch_id = authorized_branch(&claims, recipient_branch).map_err(|error| {
        log::warn!("bounce tenant mismatch rejected: {}", error.reason());
        (error.status(), error.reason().to_string())
    })?;

    let reason = claims.reason.as_deref().unwrap_or("");
    let class = claims
        .bounce_type
        .as_deref()
        .and_then(BounceClass::parse)
        .unwrap_or_else(|| classify_bounce(reason));
    let channel = claims
        .channel
        .as_deref()
        .and_then(MarketingChannel::parse)
        .or_else(|| stored_channel.as_deref().and_then(MarketingChannel::parse))
        .unwrap_or(MarketingChannel::Email);
    let address = claims
        .address
        .as_deref()
        .or(stored_email.as_deref())
        .unwrap_or("");
    // Keep the durable contact update and the in-memory suppression key on the
    // same normalized spelling (the key normalizes internally).
    let normalized_address = address.trim().to_ascii_lowercase();

    let mut attempt = AttemptState::from_json(stored_response.as_ref());
    attempt.record(reason, class);
    if let Err(error) = persist_attempt(&mut conn, recipient_id, &attempt) {
        // #2536 — the persisted error carries provider/DB detail; log it and
        // return only a caller-safe envelope to the webhook sender.
        let sanitized =
            botsecurity_core::error_sanitizer::log_and_sanitize_str(&error, "bounce_persist_attempt", None);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, sanitized.error));
    }

    let mut suppressed = false;
    let status = match class {
        BounceClass::Hard => "bounced",
        BounceClass::Complaint => "complaint",
        _ => "failed",
    };
    if class.is_terminal() && !normalized_address.is_empty() {
        let key = SuppressionKey::new(branch_id, channel, &normalized_address);
        match SuppressionStore::global().add(key.clone()) {
            Ok(_) => {
                suppressed = true;
                mark_contact_suppressed(&mut conn, branch_id, &normalized_address);
            }
            Err(error) => log::error!("bounce suppression failed: {}", error.reason()),
        }
        // Durable record so a restart cannot retry a hard-bounced address.
        match persist_suppression(&mut conn, &key, class.as_str()) {
            Ok(_) => log::info!("bounce suppression persisted: class={}", class.as_str()),
            Err(error) => log::error!("bounce suppression not persisted: {error}"),
        }
    }

    if let Err(error) = diesel::update(
        marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
    )
    .set(marketing_recipients::status.eq(Some(status.to_string())))
    .execute(&mut conn)
    {
        log::error!("bounce status update failed for {recipient_id}: {error}");
    }

    log::info!("bounce recorded: recipient={recipient_id} class={} status={status}", class.as_str());
    Ok(Json(BounceResult {
        recipient_id,
        classification: class.as_str().to_string(),
        suppressed,
        status: status.to_string(),
    }))
}
