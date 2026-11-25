use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;
use sha2::Digest;
use std::sync::Arc;

use crate::click_sign::{self, ClickError, ClickOutcome, ClickQuery};
use crate::schema::email_tracking;
use crate::state::AppState;

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let chars: Vec<u8> = input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' {
                Some(c as u8)
            } else {
                None
            }
        })
        .collect();

    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63, 52, 53,
        54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0,
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1, -1, 26,
        27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42,
        43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1, -1, -1,
    ];

    let mut output = Vec::with_capacity(chars.len() * 3 / 4);
    let mut buf = [0u8; 4];
    let mut count = 0;

    for &byte in chars.iter() {
        if byte >= 128 {
            return None;
        }
        let val = DECODE_TABLE[byte as usize];
        if val < 0 {
            continue;
        }
        buf[count] = val as u8;
        count += 1;
        if count == 4 {
            output.push((buf[0] << 2) | (buf[1] >> 4));
            output.push((buf[1] << 4) | (buf[2] >> 2));
            output.push((buf[2] << 6) | buf[3]);
            count = 0;
        }
    }

    if count >= 2 {
        output.push((buf[0] << 2) | (buf[1] >> 4));
        if count > 2 {
            output.push((buf[1] << 4) | (buf[2] >> 2));
        }
    }

    Some(output)
}

/// Short, deterministic fingerprint of a secret token for safe logging.
///
/// An open-tracking token is a bearer capability: anyone who can read logs and
/// sees the raw token can forge an open event for that recipient. The
/// fingerprint keeps log lines correlatable (the same token always produces the
/// same value) without disclosing the secret itself.
fn redact_token(token: &str) -> String {
    let digest = sha2::Sha256::digest(token.as_bytes());
    hex::encode(&digest[..4])
}

pub async fn track_email_open_pixel(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Response {
    let pixel = base64_decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==")
        .unwrap_or_else(|| vec![0u8; 1]);

    if let Ok(mut conn) = state.conn.get() {
        if let Ok(token_uuid) = uuid::Uuid::parse_str(&token) {
            let now = Utc::now();
            if let Err(e) = diesel::update(
                email_tracking::table.filter(email_tracking::open_token.eq(Some(token_uuid))),
            )
            .set((
                email_tracking::opened.eq(true),
                email_tracking::opened_at.eq(Some(now)),
            ))
            .execute(&mut conn)
            {
                log::warn!("Failed to record email open pixel: {e}");
            }

            log::info!(
                "Email open tracked via pixel: token_fp={}",
                redact_token(&token)
            );
        }
    }

    let mut response = Response::new(Body::from(pixel));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/png"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    response
}

/// Rejects a click with a plain-text reason and no `Location` header.
fn reject(status: StatusCode, reason: &'static str) -> Response {
    let mut response = Response::new(Body::from(reason));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

/// Loads the tracking record a click link belongs to.
///
/// Returns `None` for a malformed id and for an id that has no row, so the
/// caller answers `404` instead of redirecting to anything.
fn load_tracking_record(conn: &mut PgConnection, raw_id: &str) -> Option<click_sign::TrackingRecord> {
    let id = uuid::Uuid::parse_str(raw_id).ok()?;
    email_tracking::table
        .filter(email_tracking::id.eq(id))
        .select((
            email_tracking::id,
            email_tracking::campaign_id,
            email_tracking::recipient_id,
        ))
        .first::<(uuid::Uuid, Option<uuid::Uuid>, Option<uuid::Uuid>)>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|(id, campaign_id, recipient_id)| click_sign::TrackingRecord {
            id,
            campaign_id,
            recipient_id,
        })
}

/// Records the click on the tracking row (best effort, never fails the request).
fn record_click(conn: &mut PgConnection, tracking_id: uuid::Uuid) {
    let now = Utc::now();
    if let Err(e) = diesel::update(
        email_tracking::table.filter(email_tracking::id.eq(tracking_id)),
    )
    .set((
        email_tracking::clicked.eq(true),
        email_tracking::clicked_at.eq(Some(now)),
    ))
    .execute(conn)
    {
        log::error!("Email click not recorded for tracking_id={tracking_id}: {e}");
    } else {
        log::info!("Email click tracked: tracking_id={tracking_id}");
    }
}

/// Pure ordering rule for #2800: a click is persisted only for the single
/// outcome that a verified, unexpired signature produces. Kept free of I/O so
/// the record/no-record decision can be tested without a database.
fn should_record_click(outcome: &ClickOutcome) -> bool {
    matches!(outcome, ClickOutcome::Redirect(_))
}

/// `GET /api/marketing/track/click/:id?u=<url>&e=<expiry>&s=<signature>`
///
/// #2139 — the destination is only honoured when it was signed into the link
/// (HMAC over tracking record, destination and expiry). A tampered, unsigned,
/// unsafe or expired target answers `400`; an unknown tracking id answers
/// `404`. Neither ever answers `302`.
///
/// #2800 — the click is recorded only after the signature verifies. A forged
/// or unsigned link answers `4xx` and leaves no analytics row behind: knowing
/// the tracking id alone must not let anyone mark a record as clicked.
pub async fn track_email_click(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<ClickQuery>,
) -> Response {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Email click: DB pool unavailable: {e}");
            return reject(
                StatusCode::BAD_REQUEST,
                ClickError::StorageUnavailable.reason(),
            );
        }
    };

    let Some(record) = load_tracking_record(&mut conn, &id) else {
        log::warn!("Email click rejected: unknown tracking id");
        return reject(
            StatusCode::NOT_FOUND,
            ClickError::UnknownTrackingId.reason(),
        );
    };

    let now = Utc::now().timestamp();
    let outcome = click_sign::evaluate(&query, Some(&record), now, click_sign::signing_key());

    // #2800 — verify first, record second. Every rejection answer must leave
    // the tracking row untouched, so holding a tracking id cannot forge a
    // click by supplying arbitrary `u`/`e`/`s` values.
    if should_record_click(&outcome) {
        record_click(&mut conn, record.id);
    }

    match outcome {
        ClickOutcome::Redirect(destination) => match destination.parse::<HeaderValue>() {
            Ok(location) => {
                let mut response = Response::new(Body::empty());
                *response.status_mut() = StatusCode::FOUND;
                response.headers_mut().insert(header::LOCATION, location);
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("no-store"),
                );
                response
            }
            // A destination that cannot become a header value is not a
            // redirect either (#2139).
            Err(_) => reject(
                StatusCode::BAD_REQUEST,
                ClickError::UnsafeDestination.reason(),
            ),
        },
        ClickOutcome::BadRequest(reason) => {
            log::warn!("Email click rejected: {reason}");
            reject(StatusCode::BAD_REQUEST, reason)
        }
        ClickOutcome::NotFound => reject(
            StatusCode::NOT_FOUND,
            ClickError::UnknownTrackingId.reason(),
        ),
    }
}

/// Legacy shape `GET /api/marketing/track/click/:id/*destination`.
///
/// The path used to carry the destination, which is exactly the open redirect
/// #2139 removed. It is kept only to answer an explicit `400` (instead of a
/// confusing `404`) for links already sitting in inboxes; the destination is
/// never read. #2800: there is no signature to verify here, so a `400` must
/// record nothing — an old link must not be usable to forge a click.
pub async fn track_email_click_legacy_path(
    Path((id, destination)): Path<(String, String)>,
) -> Response {
    log::warn!(
        "Email click rejected: caller-supplied path destination ({} bytes, id {} bytes) is not accepted",
        destination.len(),
        id.len()
    );
    reject(
        StatusCode::BAD_REQUEST,
        "destination in the path is not accepted; use the signed link",
    )
}

pub fn configure_marketing_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/crm/campaigns", get(crate::campaigns::list_campaigns).post(crate::campaigns::create_campaign))
        .route("/api/crm/campaigns/:id", get(crate::campaigns::get_campaign).put(crate::campaigns::update_campaign).delete(crate::campaigns::delete_campaign))
        .route("/api/crm/campaigns/:id/send", post(crate::campaigns::send_campaign))
        .route("/api/crm/campaigns/:id/publish", post(crate::publish::publish_campaign))
        .route("/api/crm/campaigns/:id/control/:action", post(crate::campaign::control::control_campaign))
        .route("/api/crm/campaigns/worker/:action", post(crate::campaign::control::control_worker))
        .route("/api/crm/campaigns/:id/events", get(crate::campaign::control::campaign_events))

        .route("/api/crm/lists", get(crate::lists::list_lists).post(crate::lists::create_list))
        // #2537 — static field catalog and preview are registered before the
        // generic `:id` route so a literal path segment is never parsed as a UUID.
        .route("/api/crm/lists/segment/fields", get(crate::lists::segment_fields))
        // #2537 — recursive preview used by the shipped segment editor. Static
        // path, registered before the generic `:id` routes above/below.
        .route("/api/crm/segments/preview", post(crate::lists::segment_preview))
        .route("/api/crm/lists/:id", get(crate::lists::get_list).put(crate::lists::update_list).delete(crate::lists::delete_list))
        .route("/api/crm/lists/:id/preview", post(crate::lists::preview_list))
        .route("/api/crm/lists/:id/refresh", post(crate::lists::refresh_marketing_list))

        .route("/api/crm/templates", get(crate::templates::list_templates).post(crate::templates::create_template))
        .route("/api/crm/templates/:id", get(crate::templates::get_template).put(crate::templates::update_template).delete(crate::templates::delete_template))

        .route("/api/crm/email/track/open", post(crate::triggers::track_email_open))
        .route("/api/marketing/track/open/:token", get(track_email_open_pixel))
        // #2536 — anonymous but HMAC-signed; GET supports mail clients that
        // render the header as a link, POST is the RFC 8058 one-click shape.
        .route(
            "/api/marketing/unsubscribe/:token",
            get(crate::compliance::unsubscribe::unsubscribe)
                .post(crate::compliance::unsubscribe::unsubscribe),
        )
        // #2886 — anonymous webhook, but every state change is gated by a
        // verified HMAC capability bound to (branch, recipient, address).
        .route("/api/marketing/bounce", post(crate::compliance::bounce::record_bounce))
        // #2139 — the destination is a signed query parameter, never a path
        // segment: an anonymous route must not redirect to caller input.
        .route("/api/marketing/track/click/:id", get(track_email_click))
        .route(
            "/api/marketing/track/click/:id/*destination",
            get(track_email_click_legacy_path),
        )
        .route("/api/crm/email/send", post(crate::email::send_email_api))

        .route("/api/crm/whatsapp/send", post(crate::whatsapp::send_whatsapp_api))

        .route("/api/crm/metrics/campaign/:id", get(crate::metrics_api::get_campaign_metrics_api))
        .route("/api/crm/metrics/campaign/:id/channels", get(crate::metrics_api::get_campaign_channel_breakdown_api))
        .route("/api/crm/metrics/campaign/:id/timeseries/:interval", get(crate::metrics_api::get_campaign_timeseries_api))
        .route("/api/crm/metrics/aggregate", get(crate::metrics_api::get_aggregate_metrics_api))

        .route("/api/crm/ai/generate", post(crate::ai::generate_content_api))
        .route("/api/crm/ai/image", post(crate::ai::generate_studio_image))
        .route("/api/crm/ai/personalize", post(crate::ai::personalize_api))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::click_sign::TrackingRecord;

    const KEY: &str = "click-ordering-test-key";
    const NOW: i64 = 1_700_000_000;
    const EXPIRES: i64 = 2_000_000_000;

    fn record() -> TrackingRecord {
        TrackingRecord {
            id: uuid::Uuid::from_u128(1),
            campaign_id: None,
            recipient_id: Some(uuid::Uuid::from_u128(2)),
        }
    }

    fn query(signature: &str) -> ClickQuery {
        ClickQuery {
            u: Some("/pricing".to_string()),
            e: Some(EXPIRES.to_string()),
            s: Some(signature.to_string()),
        }
    }

    #[test]
    fn only_a_verified_redirect_is_recordable() {
        assert!(should_record_click(&ClickOutcome::Redirect(
            "/pricing".to_string()
        )));
        assert!(!should_record_click(&ClickOutcome::BadRequest(
            ClickError::BadSignature.reason()
        )));
        assert!(!should_record_click(&ClickOutcome::BadRequest(
            ClickError::Expired.reason()
        )));
        assert!(!should_record_click(&ClickOutcome::NotFound));
    }

    #[test]
    fn an_invalid_signature_never_produces_a_recordable_click() {
        // A correctly signed link is the only outcome allowed to record.
        let valid = click_sign::sign(KEY, &record(), "/pricing", EXPIRES);
        let verified = click_sign::evaluate(&query(&valid), Some(&record()), NOW, KEY);
        assert_eq!(verified, ClickOutcome::Redirect("/pricing".to_string()));
        assert!(should_record_click(&verified));

        // A forged signature answers 4xx and records nothing.
        let forged = click_sign::evaluate(&query("deadbeef"), Some(&record()), NOW, KEY);
        assert!(matches!(forged, ClickOutcome::BadRequest(_)));
        assert!(!should_record_click(&forged));

        // A signature issued for another destination cannot be replayed here.
        let other = click_sign::sign(KEY, &record(), "/other", EXPIRES);
        let replayed = click_sign::evaluate(&query(&other), Some(&record()), NOW, KEY);
        assert!(matches!(replayed, ClickOutcome::BadRequest(_)));
        assert!(!should_record_click(&replayed));

        // An expired-but-correctly-signed link records nothing either.
        let expired = click_sign::evaluate(&query(&valid), Some(&record()), EXPIRES, KEY);
        assert!(matches!(expired, ClickOutcome::BadRequest(_)));
        assert!(!should_record_click(&expired));
    }

    #[test]
    fn redact_token_omits_raw_token() {
        let token = "6f8a1e2c-3b4d-4a5e-8f90-123456789abc";
        let redacted = redact_token(token);

        assert!(
            !redacted.contains(token),
            "raw token leaked into log output: {redacted}"
        );
        assert!(
            !redacted.contains("6f8a1e2c"),
            "token fragment leaked into log output: {redacted}"
        );
        assert_eq!(redacted.len(), 8, "fingerprint should be a short digest");
        assert_eq!(redacted, redact_token(token), "fingerprint must be stable");
        assert_ne!(redacted, redact_token("a-different-token"));
    }
}
