//! Plan identity and authorization (issue #2136).
//!
//! Every plan route used to trust a client-supplied `plan_id` and nothing else:
//! `handle_get_plan` even *created* a plan owned by `"anonymous"` for any id it
//! was handed, and the collaboration socket invented a guest identity when no
//! token was present. Any signed-in caller (or a visitor) who learned an id
//! could read, edit and watch somebody else's plan.
//!
//! This module is the single place that answers "who is calling, and may they
//! touch this plan":
//!
//! * identity comes only from [`botsecurity_core::verified_jwt`] (HS256
//!   pinned, constant-time HMAC, `exp`/`nbf` enforced) or, for the opaque
//!   `gb_*` suite token that is not a JWT, from the session cache the auth
//!   middleware populated at login — never from a decoded payload;
//! * authorization is owner-or-member, plus a branch check so a plan minted in
//!   one tenant is invisible to another;
//! * a plan the caller cannot see is reported as **404**, so the response never
//!   confirms that a foreign id exists.

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt, JwtVerifyError, VerifiedJwt};
use uuid::Uuid;

use crate::Plan;

/// Owner recorded by the pre-#2136 code for implicitly created plans. Such a
/// plan has no real owner, so nobody inherits it.
pub const LEGACY_ANONYMOUS_OWNER: &str = "anonymous";

/// Close codes used when a socket is refused after the upgrade (the plan
/// disappeared, or the membership was revoked, between the handshake and the
/// subscription). A refusal *before* the upgrade is an HTTP `401`/`404`.
pub const PLAN_WS_CLOSE_UNAUTHORIZED: u16 = 4401;
pub const PLAN_WS_CLOSE_FORBIDDEN: u16 = 4403;

/// The verified identity behind a plan request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    /// Verified subject — the plan `owner_id` / `members` entry.
    pub user_id: String,
    /// Verified email, used only as a display label for collaborators.
    pub email: Option<String>,
    pub org_id: Option<Uuid>,
    /// Server-minted tenant branch claim; `None` when the token carries no
    /// (or a nil) branch, which is the case for suite session tokens.
    pub branch_id: Option<Uuid>,
}

/// What the caller intends to do with the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
    /// Mutating the plan itself (membership), not only its tasks.
    Owner,
}

/// Why a plan request was refused. Every variant maps to a status code and a
/// stable machine-readable code; nothing here echoes caller input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// No token, an unverifiable token, or an unknown session.
    Unauthenticated(&'static str),
    /// The plan does not exist **or** the caller may not see it.
    NotFound,
    /// The plan is visible but this action needs the owner.
    Forbidden(&'static str),
}

impl PlanError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
        }
    }
}

impl IntoResponse for PlanError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthenticated(reason) => (
                StatusCode::UNAUTHORIZED,
                [("x-auth-error", "invalid_token"), ("x-auth-reason", reason)],
                Json(serde_json::json!({
                    "ok": false,
                    "error": "authentication_required",
                    "reason": reason,
                })),
            )
                .into_response(),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "ok": false, "error": "plan_not_found" })),
            )
                .into_response(),
            Self::Forbidden(reason) => (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "ok": false,
                    "error": "forbidden",
                    "reason": reason,
                })),
            )
                .into_response(),
        }
    }
}

/// Stable code for a refused token, safe to log and to return.
pub fn reason_code(reason: &JwtVerifyError) -> &'static str {
    match reason {
        JwtVerifyError::Malformed => "malformed_token",
        JwtVerifyError::InvalidHeader => "invalid_header",
        JwtVerifyError::UnsupportedAlgorithm => "unsupported_algorithm",
        JwtVerifyError::InvalidSignature => "invalid_signature",
        JwtVerifyError::MissingSecret => "verifier_unavailable",
        JwtVerifyError::InvalidPayload => "invalid_payload",
        JwtVerifyError::MissingSubject => "missing_subject",
        JwtVerifyError::MissingExpiry => "missing_expiry",
        JwtVerifyError::Expired => "token_expired",
        JwtVerifyError::NotYetValid => "token_not_yet_valid",
    }
}

/// A nil branch claim means "no tenant scope", not a tenant of its own.
fn normalize_branch(branch: Option<Uuid>) -> Option<Uuid> {
    branch.filter(|value| !value.is_nil())
}

impl Caller {
    /// Resolves the caller from the `Authorization: Bearer` header.
    pub fn from_headers(headers: &HeaderMap) -> Result<Self, PlanError> {
        let token = bearer_token(headers)
            .filter(|value| !value.is_empty())
            .ok_or(PlanError::Unauthenticated("missing_token"))?;
        Self::from_token(token)
    }

    /// Resolves the caller from a bearer token (REST header or WS query).
    ///
    /// A JWT is verified first; an opaque suite session token (`gb_*`) is
    /// resolved through the session cache. A malformed *JWT* (three segments)
    /// never falls back to the cache, so a forged token cannot be traded for a
    /// cached identity.
    pub fn from_token(token: &str) -> Result<Self, PlanError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(PlanError::Unauthenticated("missing_token"));
        }
        match verify_saas_jwt(token) {
            Ok(verified) => Ok(Self::from_verified(verified)),
            Err(reason) => Self::from_session_cache(token)
                .ok_or(PlanError::Unauthenticated(reason_code(&reason))),
        }
    }

    fn from_verified(verified: VerifiedJwt) -> Self {
        Self {
            user_id: verified.sub,
            email: verified.email,
            org_id: verified.org_id,
            branch_id: normalize_branch(verified.branch_id),
        }
    }

    /// Opaque suite tokens are not JWTs: the auth middleware resolves them
    /// against the session cache at login. The cache carries no branch claim,
    /// so such a caller is branchless.
    fn from_session_cache(token: &str) -> Option<Self> {
        if token.contains('.') {
            return None;
        }
        let entry = botsecurity_core::lookup_session_cache(token)?;
        let user_id = entry.user_id.trim();
        if user_id.is_empty() {
            return None;
        }
        Some(Self {
            user_id: user_id.to_string(),
            email: Some(entry.email).filter(|value| !value.trim().is_empty()),
            org_id: entry.organization_id,
            branch_id: None,
        })
    }

    /// The display name a collaborator is shown under. The verified email
    /// wins; the subject is the fallback, so nobody chooses their label.
    pub fn display_name(&self) -> String {
        self.email.clone().unwrap_or_else(|| self.user_id.clone())
    }
}

/// Owner-or-member authorization plus tenant isolation.
///
/// A caller who cannot see the plan gets [`PlanError::NotFound`] (never 403),
/// so a guessed id is indistinguishable from an unused one.
pub fn authorize(plan: &Plan, caller: &Caller, access: Access) -> Result<(), PlanError> {
    if !branch_compatible(plan, caller) {
        return Err(PlanError::NotFound);
    }
    let is_owner =
        plan.owner_id != LEGACY_ANONYMOUS_OWNER && plan.owner_id == caller.user_id;
    let is_member = plan.members.iter().any(|member| member == &caller.user_id);
    if !is_owner && !is_member {
        return Err(PlanError::NotFound);
    }
    match access {
        Access::Owner if !is_owner => Err(PlanError::Forbidden("owner_only")),
        _ => Ok(()),
    }
}

/// A plan is only reachable inside the tenant that minted it.
///
/// Plans created before #2136 (and plans created by a branchless suite
/// session) carry no branch and are therefore only reachable by a branchless
/// caller — ownership still applies on top of this.
fn branch_compatible(plan: &Plan, caller: &Caller) -> bool {
    match (plan.branch_id.as_deref(), caller.branch_id) {
        (Some(recorded), Some(branch)) => recorded == branch.to_string(),
        (None, None) => true,
        _ => false,
    }
}

/// Resolves the collaborator credential of `/ws/plan/:plan_id`.
///
/// The query token is the only credential a browser WebSocket can send; the
/// bearer header is accepted as well so non-browser clients can use the same
/// session they use for REST. Both are verified exactly like REST tokens.
pub fn caller_from_ws(
    query_token: Option<&str>,
    headers: &HeaderMap,
) -> Result<Caller, PlanError> {
    match query_token.map(str::trim).filter(|token| !token.is_empty()) {
        Some(token) => Caller::from_token(token),
        None => Caller::from_headers(headers),
    }
}

/// Resolves and authorizes the collaborator of `/ws/plan/:plan_id`.
pub fn authorize_ws(
    plan: &Plan,
    query_token: Option<&str>,
    headers: &HeaderMap,
) -> Result<Caller, PlanError> {
    let caller = caller_from_ws(query_token, headers)?;
    authorize(plan, &caller, Access::Read)?;
    Ok(caller)
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod auth_tests;
