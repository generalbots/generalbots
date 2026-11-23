use botcore::shared::state::AppState;
use botsecurity::constant_time::eq_str;
use botsecurity::webhook::SignatureValidation;
use chrono::Utc;
use diesel::RunQueryDsl;
use hmac::{Hmac, Mac};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

mod webhook_audit;
pub use webhook_audit::{
    format_audit_signature, parse_audit_signature, sign_delivery, DeliverySignature,
};

pub const WEBHOOK_VAULT_ROOT: &str = "secret/gbo/settings/webhooks";

/// Header carrying the versioned HMAC signature.
pub const SIGNATURE_HEADER: &str = "X-Webhook-Signature";
/// Header carrying the unix timestamp that was folded into the signature.
pub const TIMESTAMP_HEADER: &str = "X-Webhook-Timestamp";
/// Header carrying the per-delivery random nonce that was folded into the signature.
pub const NONCE_HEADER: &str = "X-Webhook-Nonce";
/// Maximum accepted clock skew, in seconds, for an inbound delivery (#2645).
pub const CLOCK_SKEW_TOLERANCE_SECONDS: i64 = 300;

const SIGNATURE_V2: &str = "v2";

pub fn webhook_vault_path(webhook_id: Uuid) -> String {
    format!("{WEBHOOK_VAULT_ROOT}/{webhook_id}")
}

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Validates a webhook URL. HTTPS is required in production; private/local
/// destinations are always rejected to prevent SSRF.
pub fn validate_webhook_url(url: &str) -> Result<(), &'static str> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Webhook URL is required");
    }
    let is_prod = std::env::var("GB_ENV")
        .map(|v| v == "production")
        .unwrap_or(false);
    if is_prod && !trimmed.starts_with("https://") {
        return Err("Webhook URL must use HTTPS in production");
    }
    botsecurity::validation::validate_url_ssrf_strict(trimmed)
        .map(|_| ())
        .map_err(|_| "Webhook URL cannot point to localhost or private networks")
}

pub fn parse_events(raw: Option<&str>) -> serde_json::Value {
    let events: Vec<String> = raw
        .map(|s| {
            s.split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if events.is_empty() {
        serde_json::json!(["*"])
    } else {
        serde_json::Value::Array(events.into_iter().map(serde_json::Value::String).collect())
    }
}

type HmacSha256 = Hmac<sha2::Sha256>;

/// HMAC-SHA256 over `material`, returning the hex digest. An unusable key fails
/// closed: `None` becomes an empty signature that verification rejects, never a
/// process abort (#1368).
fn hmac_hex(material: &str, secret: &str) -> Option<String> {
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        log::error!("webhook sign: HMAC key rejected");
        return None;
    };
    mac.update(material.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

/// Canonical v2 signed material: version, unix timestamp, nonce and payload
/// (#2645). Binding the timestamp and nonce makes each delivery unique and lets
/// a receiver enforce a freshness window and a replay cache.
fn signed_material(timestamp: i64, nonce: &str, payload: &str) -> String {
    format!("{SIGNATURE_V2}.{timestamp}.{nonce}.{payload}")
}

/// Signs one delivery with an explicit timestamp and nonce. Keeping this pure
/// (no wall-clock read) makes the signature reproducible for tests and for
/// receivers replaying the exact headers they received.
pub fn sign_payload_at(payload: &str, secret: &str, timestamp: i64, nonce: &str) -> String {
    let material = signed_material(timestamp, nonce, payload);
    match hmac_hex(&material, secret) {
        Some(digest) => format!("{SIGNATURE_V2}={digest}"),
        None => format!("{SIGNATURE_V2}="),
    }
}

/// Creates a fresh timestamp, nonce and signature for one outbound delivery.
/// Returns `(signature, timestamp_header_value, nonce)`; send them in
/// [`SIGNATURE_HEADER`], [`TIMESTAMP_HEADER`] and [`NONCE_HEADER`].
pub fn create_signature_headers(payload: &str, secret: &str) -> (String, String, String) {
    let timestamp = Utc::now().timestamp();
    let nonce = Uuid::new_v4().to_string();
    let signature = sign_payload_at(payload, secret, timestamp, &nonce);
    (signature, timestamp.to_string(), nonce)
}

/// Receiving helper for signed deliveries (#2645).
///
/// Verifies the v2 signature over `timestamp.nonce.payload`, refuses timestamps
/// outside [`CLOCK_SKEW_TOLERANCE_SECONDS`] and refuses a nonce seen before.
/// Keys are cached for the width of the freshness window and pruned on each
/// verification, so the cache stays bounded while still catching replays.
pub struct WebhookSignatureVerifier {
    seen_nonces: HashMap<String, i64>,
}

impl Default for WebhookSignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookSignatureVerifier {
    pub fn new() -> Self {
        Self {
            seen_nonces: HashMap::new(),
        }
    }

    /// Verify a delivery using the current wall clock.
    pub fn verify(
        &mut self,
        payload: &str,
        signature: &str,
        timestamp_header: &str,
        nonce: &str,
        secret: &str,
    ) -> SignatureValidation {
        self.verify_at(
            payload,
            signature,
            timestamp_header,
            nonce,
            secret,
            Utc::now().timestamp(),
        )
    }

    /// Verify against a supplied clock value. Public so receivers can inject
    /// their own clock and so tests are deterministic.
    pub fn verify_at(
        &mut self,
        payload: &str,
        signature: &str,
        timestamp_header: &str,
        nonce: &str,
        secret: &str,
        now: i64,
    ) -> SignatureValidation {
        if signature.is_empty() || timestamp_header.is_empty() || nonce.is_empty() {
            return SignatureValidation::Missing;
        }

        let Some(supplied) = signature.strip_prefix(&format!("{SIGNATURE_V2}=")) else {
            // v1 has no freshness data; accepting it reopens the replay hole.
            return SignatureValidation::Invalid;
        };

        let Ok(timestamp) = timestamp_header.parse::<i64>() else {
            return SignatureValidation::Invalid;
        };

        // i128 avoids overflowing on hostile values such as i64::MIN/MAX.
        let skew = (i128::from(now) - i128::from(timestamp)).abs();
        if skew > i128::from(CLOCK_SKEW_TOLERANCE_SECONDS) {
            return SignatureValidation::Expired;
        }

        let expected = sign_payload_at(payload, secret, timestamp, nonce);
        let Some(expected_hex) = expected.strip_prefix(&format!("{SIGNATURE_V2}=")) else {
            return SignatureValidation::Invalid;
        };
        if !eq_str(supplied, expected_hex) {
            return SignatureValidation::Invalid;
        }

        let cutoff = now.saturating_sub(CLOCK_SKEW_TOLERANCE_SECONDS);
        self.seen_nonces.retain(|_, seen_at| *seen_at >= cutoff);
        if self.seen_nonces.contains_key(nonce) {
            return SignatureValidation::Replayed;
        }
        self.seen_nonces.insert(nonce.to_string(), now);

        SignatureValidation::Valid
    }

    /// Number of nonces currently held in the replay cache.
    pub fn cached_nonce_count(&self) -> usize {
        self.seen_nonces.len()
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "…"
    }
}

/// Maximum number of bytes of an outbound webhook's HTTP response that we read
/// into memory and persist to `webhook_deliveries.response_body`. A hostile or
/// misbehaving receiver could otherwise return an unbounded body, exhausting
/// both process memory and the database column (#2781).
const MAX_RESPONSE_BODY_BYTES: usize = 4 * 1024;

/// Returns a prefix of `s` no longer than `max_bytes`, trimmed back to a UTF-8
/// character boundary. Never panics: the boundary walk stops at zero.
fn cap_body(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Reads at most [`MAX_RESPONSE_BODY_BYTES`] of the response body, stopping as
/// soon as the budget is exhausted so the rest is never buffered. The returned
/// text is safe to persist: it is bounded in size and valid UTF-8 (#2781).
async fn read_capped_body(resp: &mut reqwest::Response) -> String {
    let mut buf: Vec<u8> = Vec::with_capacity(MAX_RESPONSE_BODY_BYTES);
    while buf.len() < MAX_RESPONSE_BODY_BYTES {
        match resp.chunk().await {
            Ok(Some(chunk)) if !chunk.is_empty() => {
                let remaining = MAX_RESPONSE_BODY_BYTES - buf.len();
                let take = chunk.len().min(remaining);
                buf.extend_from_slice(&chunk[..take]);
                if take < chunk.len() {
                    // Budget reached mid-chunk; do not pull any more data.
                    break;
                }
            }
            Ok(Some(_)) | Ok(None) => break,
            Err(e) => {
                log::warn!("webhook deliver: could not read response body: {e}");
                break;
            }
        }
    }
    let text = String::from_utf8_lossy(&buf);
    cap_body(text.as_ref(), MAX_RESPONSE_BODY_BYTES).to_string()
}

/// Delivers a payload with exponential backoff retries, recording every
/// attempt in `webhook_deliveries`. Runs in a background task.
///
/// The caller supplies the exact v2 triple that goes on the wire and into the
/// audit column, so the record is re-verifiable (#2645, #2829). Retries reuse
/// it: the backoff finishes inside the 300 s window and the nonce cache makes
/// a repeat harmless.
pub async fn deliver_with_retries(
    state: Arc<AppState>,
    delivery_id: Uuid,
    url: String,
    payload: &str,
    signed: DeliverySignature,
) {
    let max_attempts = 5;
    let mut attempt = 0;
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("webhook deliver: client build failed: {e}");
            mark_delivery(
                &state,
                delivery_id,
                "failed",
                None,
                None,
                attempt,
                Some(&e.to_string()),
            );
            return;
        }
    };

    loop {
        attempt += 1;
        let result = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header(SIGNATURE_HEADER, &signed.signature)
            .header(TIMESTAMP_HEADER, &signed.timestamp)
            .header(NONCE_HEADER, &signed.nonce)
            .header("X-Webhook-Event", "test")
            .body(payload.to_string())
            .send()
            .await;

        match result {
            Ok(mut resp) => {
                let code = resp.status().as_u16();
                let ok = (200..300).contains(&code);
                // Bound the read and the persisted value (#2781): a remote
                // receiver must never decide our memory or DB consumption.
                let body = read_capped_body(&mut resp).await;
                let stored_body = cap_body(&body, MAX_RESPONSE_BODY_BYTES);
                let status = if ok { "success" } else { "failed" };
                let err = if ok {
                    None
                } else {
                    Some(format!("HTTP {code}: {}", truncate(stored_body, 200)))
                };
                let should_retry = !ok && attempt < max_attempts;
                if should_retry {
                    mark_delivery(
                        &state,
                        delivery_id,
                        "retrying",
                        Some(code),
                        Some(stored_body),
                        attempt,
                        err.as_deref(),
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(
                        2u64.pow(attempt as u32) * 5,
                    ))
                    .await;
                    continue;
                }
                mark_delivery(
                    &state,
                    delivery_id,
                    status,
                    Some(code),
                    Some(stored_body),
                    attempt,
                    err.as_deref(),
                );
                return;
            }
            Err(e) => {
                let msg = format!("request error: {e}");
                if attempt < max_attempts {
                    mark_delivery(
                        &state,
                        delivery_id,
                        "retrying",
                        None,
                        None,
                        attempt,
                        Some(&msg),
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(
                        2u64.pow(attempt as u32) * 5,
                    ))
                    .await;
                    continue;
                }
                mark_delivery(
                    &state,
                    delivery_id,
                    "failed",
                    None,
                    None,
                    attempt,
                    Some(&msg),
                );
                return;
            }
        }
    }
}

fn mark_delivery(
    state: &Arc<AppState>,
    delivery_id: Uuid,
    status: &str,
    response_code: Option<u16>,
    response_body: Option<&str>,
    attempt: i32,
    error: Option<&str>,
) {
    let Some(mut conn) = state.conn.get().ok() else {
        log::warn!("webhook deliver: DB unavailable, delivery {delivery_id} not recorded");
        return;
    };
    let completed_at: Option<chrono::DateTime<Utc>> = match status {
        "success" | "failed" => Some(Utc::now()),
        _ => None,
    };
    let _ = diesel::sql_query(
        "UPDATE webhook_deliveries SET status = $1, response_code = $2, response_body = $3, \
         attempt = $4, error = $5, next_retry_at = $6, completed_at = $7 WHERE id = $8",
    )
    .bind::<diesel::sql_types::Text, _>(status)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(response_code.map(i32::from))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(response_body)
    .bind::<diesel::sql_types::Int4, _>(attempt)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(error)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(
        if status == "retrying" {
            Some(Utc::now() + chrono::Duration::seconds(60))
        } else {
            None
        },
    )
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(completed_at)
    .bind::<diesel::sql_types::Uuid, _>(delivery_id)
    .execute(&mut conn);
}

#[cfg(test)]
#[path = "webhook_delivery_tests.rs"]
mod webhook_delivery_tests;
