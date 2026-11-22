//! Form capture for published WebA sites (#2538).
//!
//! A visitor submits a form to the suite endpoint (the published document
//! itself stays sandboxed by [`crate::csp::WEBA_CSP`]). We validate the payload,
//! rate-limit it, store it in the durable branch snapshot with one submission
//! per e-mail, and hand it to the existing CRM lead-form service which creates
//! or updates the contact and the lead with source `site:{slug}`.

use super::persistence::FormSubmission;
use super::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_EMAIL_LEN: usize = 320;
const MAX_NAME_LEN: usize = 200;
const MAX_MESSAGE_LEN: usize = 8_000;
const MAX_FIELDS: usize = 50;
const MAX_FIELD_KEY_LEN: usize = 64;
const MAX_FIELD_VALUE_LEN: usize = 2_000;

/// Payload accepted by `POST /forms/:slug/submit`.
#[derive(Debug, Deserialize)]
pub struct FormSubmitRequest {
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    /// Honeypot: real browsers leave it empty; bots tend to fill every input.
    #[serde(default)]
    pub website: Option<String>,
}

/// Stable validation result. `None` means the payload is acceptable.
pub fn validate(request: &FormSubmitRequest) -> Option<&'static str> {
    let email = request.email.trim();
    if email.is_empty() || email.len() > MAX_EMAIL_LEN || !email.contains('@') {
        return Some("invalid_email");
    }
    if email.split('@').count() != 2 {
        return Some("invalid_email");
    }
    if request.name.as_ref().is_some_and(|name| name.len() > MAX_NAME_LEN) {
        return Some("invalid_name");
    }
    if request.message.as_ref().is_some_and(|text| text.len() > MAX_MESSAGE_LEN) {
        return Some("invalid_message");
    }
    if request.fields.len() > MAX_FIELDS {
        return Some("too_many_fields");
    }
    if request.fields.iter().any(|(key, value)| {
        key.len() > MAX_FIELD_KEY_LEN || value.len() > MAX_FIELD_VALUE_LEN
    }) {
        return Some("invalid_field");
    }
    None
}

/// Case-insensitive e-mail key used for dedupe and CRM lookup.
#[must_use]
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// Fixed-capacity token bucket keyed by slug and caller address.
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, Bucket>>,
    capacity: f64,
    refill_per_second: f64,
}

impl RateLimiter {
    #[must_use]
    pub fn new(capacity: u32, refill_per_second: f64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            capacity: f64::from(capacity),
            refill_per_second,
        }
    }

    /// Returns `false` when the caller has exhausted its budget.
    pub fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut buckets = match self.buckets.lock() {
            Ok(buckets) => buckets,
            Err(_) => return false,
        };
        let bucket = buckets
            .entry(key.to_string())
            .or_insert(Bucket { tokens: self.capacity, last_refill: now });
        let elapsed = now.saturating_duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_second).min(self.capacity);
        bucket.last_refill = now;
        if bucket.tokens < 1.0 {
            return false;
        }
        bucket.tokens -= 1.0;
        true
    }

    /// Drops stale buckets so the map cannot grow without bound.
    pub fn prune(&self, max_idle: Duration) {
        let now = Instant::now();
        if let Ok(mut buckets) = self.buckets.lock() {
            buckets.retain(|_, bucket| now.saturating_duration_since(bucket.last_refill) < max_idle);
        }
    }
}

fn default_bot(conn: &mut diesel::PgConnection) -> Uuid {
    drop(conn);
    Uuid::nil()
}

fn contact_change(conn: &mut diesel::PgConnection, id: Uuid, action: &str, user: Uuid) {
    drop((conn, id, action, user));
}

fn deal_stage_change(conn: &mut diesel::PgConnection, id: Uuid, old: &str, next: &str, user: Uuid) {
    drop((conn, id, old, next, user));
}

fn field(fields: &HashMap<String, String>, name: &str) -> Option<String> {
    fields
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.clone())
}

/// Creates or updates the CRM contact and lead through the existing CRM
/// lead-form handler. `source` is always `site:{slug}`.
pub async fn sync_crm(slug: &str, submission: &FormSubmission) -> Result<(), String> {
    let pool = botcore::shared::utils::pool().map_err(|error| {
        let (status, detail) = error;
        format!("pool status {} detail {detail}", status.as_u16())
    })?;
    let state = botcontacts::CrateState::new(
        (*pool).clone(),
        std::sync::Arc::new(default_bot),
        std::sync::Arc::new(contact_change),
        std::sync::Arc::new(deal_stage_change),
    );
    let source = format!("site:{slug}");
    let lead = botcontacts::requests::CreateLeadForm {
        title: Some(slug.to_string()),
        first_name: submission.name.clone(),
        last_name: field(&submission.fields, "last_name"),
        email: Some(submission.email.clone()),
        phone: field(&submission.fields, "phone"),
        company: field(&submission.fields, "company"),
        job_title: field(&submission.fields, "job_title"),
        source: Some(source),
        value: None,
        description: submission.message.clone(),
        currency: None,
        lost_reason: None,
    };
    match botcontacts::handlers::deals::create_lead_form(
        State(std::sync::Arc::new(state)),
        axum::http::HeaderMap::new(),
        Json(lead),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err((status, detail)) => Err(format!("CRM status {} detail {detail}", status.as_u16())),
    }
}

/// Builds a fresh submission or updates the existing one for the same e-mail.
pub fn upsert_submission(
    existing: Option<&FormSubmission>,
    site_id: Uuid,
    request: &FormSubmitRequest,
    slug: &str,
) -> FormSubmission {
    let now = chrono::Utc::now();
    let source = format!("site:{slug}");
    match existing {
        Some(previous) => FormSubmission {
            id: previous.id,
            site_id: previous.site_id,
            slug: previous.slug.clone(),
            email: previous.email.clone(),
            name: request.name.clone().or_else(|| previous.name.clone()),
            message: request.message.clone().or_else(|| previous.message.clone()),
            fields: request.fields.clone(),
            source,
            created_at: previous.created_at,
            updated_at: now,
        },
        None => FormSubmission {
            id: Uuid::new_v4(),
            site_id,
            slug: slug.to_string(),
            email: normalize_email(&request.email),
            name: request.name.clone(),
            message: request.message.clone(),
            fields: request.fields.clone(),
            source,
            created_at: now,
            updated_at: now,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(email: &str) -> FormSubmitRequest {
        FormSubmitRequest {
            email: email.to_string(),
            name: None,
            message: None,
            fields: HashMap::new(),
            website: None,
        }
    }

    #[test]
    fn validates_required_fields() {
        assert_eq!(validate(&request("")), Some("invalid_email"));
        assert_eq!(validate(&request("no-at-sign")), Some("invalid_email"));
        assert_eq!(validate(&request("a@example.com")), None);
    }

    #[test]
    fn normalizes_email_for_dedupe() {
        assert_eq!(normalize_email("  Foo@Example.COM "), "foo@example.com");
    }

    #[test]
    fn dedupes_by_email() {
        let first = upsert_submission(None, Uuid::nil(), &request("a@example.com"), "signup");
        let updated = upsert_submission(Some(&first), Uuid::nil(), &request("a@example.com"), "signup");
        assert_eq!(first.id, updated.id);
        assert_eq!(first.email, updated.email);
    }

    #[test]
    fn rate_limits_a_burst() {
        let limiter = RateLimiter::new(2, 0.0);
        assert!(limiter.allow("signup"));
        assert!(limiter.allow("signup"));
        assert!(!limiter.allow("signup"));
    }
}
