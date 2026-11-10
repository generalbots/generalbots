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

pub async fn track_email_open_pixel(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Response {
    let pixel = base64_decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==")
        .unwrap_or_else(|| vec![0u8; 1]);

    if let Ok(mut conn) = state.conn.get() {
        if let Ok(token_uuid) = uuid::Uuid::parse_str(&token) {
            let now = Utc::now();
            let _ = diesel::update(
                email_tracking::table.filter(email_tracking::open_token.eq(Some(token_uuid))),
            )
            .set((
                email_tracking::opened.eq(true),
                email_tracking::opened_at.eq(Some(now)),
            ))
            .execute(&mut conn);

            log::info!("Email open tracked via pixel: token={}", token);
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

/// `GET /api/marketing/track/click/:id?u=<url>&e=<expiry>&s=<signature>`
///
/// #2139 — the destination is only honoured when it was signed into the link
/// (HMAC over tracking record, destination and expiry). A tampered, unsigned,
/// unsafe or expired target answers `400`; an unknown tracking id answers
/// `404`. Neither ever answers `302`. The click itself is still recorded.
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

    record_click(&mut conn, record.id);

    let now = Utc::now().timestamp();
    match click_sign::evaluate(&query, Some(&record), now, click_sign::signing_key()) {
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
/// never read, and the click is still recorded.
pub async fn track_email_click_legacy_path(
    State(state): State<Arc<AppState>>,
    Path((id, destination)): Path<(String, String)>,
) -> Response {
    if let Ok(mut conn) = state.conn.get() {
        if let Some(record) = load_tracking_record(&mut conn, &id) {
            record_click(&mut conn, record.id);
        }
    }
    log::warn!(
        "Email click rejected: caller-supplied path destination ({} bytes) is not accepted",
        destination.len()
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
        .route("/api/crm/lists/:id", get(crate::lists::get_list).put(crate::lists::update_list).delete(crate::lists::delete_list))
        .route("/api/crm/lists/:id/refresh", post(crate::lists::refresh_marketing_list))

        .route("/api/crm/templates", get(crate::templates::list_templates).post(crate::templates::create_template))
        .route("/api/crm/templates/:id", get(crate::templates::get_template).put(crate::templates::update_template).delete(crate::templates::delete_template))

        .route("/api/crm/email/track/open", post(crate::triggers::track_email_open))
        .route("/api/marketing/track/open/:token", get(track_email_open_pixel))
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
