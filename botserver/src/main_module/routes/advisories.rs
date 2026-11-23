//! Read-only, anonymous advisory API (#2723).
//!
//! Responses are the canonical `Advisory` records from `botlib`, the same
//! structs the CSAF/OSV generators consume, so there is no second source of
//! truth. A bundle can be pinned with `GB_ADVISORY_BUNDLE` for mirrored and
//! air-gapped deployments; otherwise the compiled-in bundle is used.
//!
//! Privacy: this module never logs query strings, IPs or caller headers. Only
//! fixed error identifiers are logged, and no caller-supplied text is echoed
//! into a response. Security headers and the global rate limiter are applied
//! by the server's outermost layers; the quota below is advisory-specific.

use axum::{
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use botlib::security::advisory::{AdvisoryQuery, AdvisoryStore, Severity, ADVISORY_SCHEMA_URL};
use chrono::NaiveDate;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    num::NonZeroU32,
    path::Path as FsPath,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const CACHE_TTL: Duration = Duration::from_secs(300);

struct CachedResponse {
    created_at: Instant,
    body: Value,
}

struct AdvisoryState {
    store: AdvisoryStore,
    cache: Mutex<HashMap<String, CachedResponse>>,
    limiter: GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

fn quota() -> Quota {
    let per_second = NonZeroU32::new(120).unwrap_or(NonZeroU32::MIN);
    let burst = NonZeroU32::new(240).unwrap_or(NonZeroU32::MIN);
    Quota::per_second(per_second).allow_burst(burst)
}

fn advisory_state() -> Result<&'static AdvisoryState, ApiError> {
    static STATE: OnceLock<Result<AdvisoryState, String>> = OnceLock::new();
    let initialized = STATE.get_or_init(|| {
        let store = match std::env::var("GB_ADVISORY_BUNDLE") {
            Ok(path) if !path.trim().is_empty() => {
                AdvisoryStore::from_path(FsPath::new(&path)).map_err(|error| error.to_string())?
            }
            _ => AdvisoryStore::embedded().map_err(|error| error.to_string())?,
        };
        Ok(AdvisoryState {
            store,
            cache: Mutex::new(HashMap::new()),
            limiter: GovernorRateLimiter::direct(quota()),
        })
    });
    match initialized {
        Ok(state) => Ok(state),
        Err(reason) => {
            log::error!("advisory: store unavailable ({reason})");
            Err(ApiError::StoreUnavailable)
        }
    }
}

enum ApiError {
    StoreUnavailable,
    BadFilter,
    NotFound,
    RateLimited,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::StoreUnavailable => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "advisory_store_unavailable" })),
            )
                .into_response(),
            Self::BadFilter => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid_filter" })),
            )
                .into_response(),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "advisory_not_found" })),
            )
                .into_response(),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "1")],
                Json(json!({ "error": "rate_limited" })),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ListParams {
    pub product: Option<String>,
    pub component: Option<String>,
    pub version: Option<String>,
    pub severity: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn build_query(params: &ListParams) -> Result<AdvisoryQuery, ApiError> {
    let severity = match &params.severity {
        Some(value) => match Severity::parse(value) {
            Some(parsed) => Some(parsed),
            None => return Err(ApiError::BadFilter),
        },
        None => None,
    };
    let from = match &params.from {
        Some(value) => match parse_date(value) {
            Some(parsed) => Some(parsed),
            None => return Err(ApiError::BadFilter),
        },
        None => None,
    };
    let to = match &params.to {
        Some(value) => match parse_date(value) {
            Some(parsed) => Some(parsed),
            None => return Err(ApiError::BadFilter),
        },
        None => None,
    };
    if let (Some(from), Some(to)) = (from, to) {
        if from > to {
            return Err(ApiError::BadFilter);
        }
    }
    Ok(AdvisoryQuery {
        product: params.product.clone(),
        component: params.component.clone(),
        version: params.version.clone(),
        severity,
        from,
        to,
    })
}

fn cache_key(params: &ListParams) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        params.product.as_deref().unwrap_or("-"),
        params.component.as_deref().unwrap_or("-"),
        params.version.as_deref().unwrap_or("-"),
        params.severity.as_deref().unwrap_or("-"),
        params.from.as_deref().unwrap_or("-"),
        params.to.as_deref().unwrap_or("-"),
    )
}

fn cached_body(state: &AdvisoryState, key: &str, produce: impl FnOnce() -> Value) -> Value {
    if let Ok(cache) = state.cache.lock() {
        if let Some(entry) = cache.get(key) {
            if entry.created_at.elapsed() < CACHE_TTL {
                return entry.body.clone();
            }
        }
    }
    let body = produce();
    if let Ok(mut cache) = state.cache.lock() {
        cache.insert(
            key.to_string(),
            CachedResponse {
                created_at: Instant::now(),
                body: body.clone(),
            },
        );
    }
    body
}

/// `GET /api/advisories` — canonical advisory records with filters.
pub async fn list(Query(params): Query<ListParams>) -> Response {
    let state = match advisory_state() {
        Ok(state) => state,
        Err(error) => return error.into_response(),
    };
    if state.limiter.check().is_err() {
        return ApiError::RateLimited.into_response();
    }
    let filter = match build_query(&params) {
        Ok(filter) => filter,
        Err(error) => return error.into_response(),
    };
    let body = cached_body(state, &cache_key(&params), || {
        let advisories = state.store.query(&filter);
        json!({
            "schema": ADVISORY_SCHEMA_URL,
            "count": advisories.len(),
            "advisories": advisories,
        })
    });
    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /api/advisories/{id}` — one canonical bulletin.
pub async fn get(Path(id): Path<String>) -> Response {
    let state = match advisory_state() {
        Ok(state) => state,
        Err(error) => return error.into_response(),
    };
    if state.limiter.check().is_err() {
        return ApiError::RateLimited.into_response();
    }
    let body = cached_body(state, &format!("id|{id}"), || match state.store.get(&id) {
        Some(advisory) => json!({ "schema": ADVISORY_SCHEMA_URL, "advisory": advisory }),
        None => json!({ "error": "advisory_not_found" }),
    });
    match body.get("error") {
        Some(_) => ApiError::NotFound.into_response(),
        None => (StatusCode::OK, Json(body)).into_response(),
    }
}
