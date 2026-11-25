//! #2452 — deterministic ownership transfer for collaboration resources.
//!
//! The old transfer handler wrote whatever string the caller sent straight
//! into `resource_permissions`. That made UAT impossible locally: the only
//! second "user" the browser could name was a Zitadel identity, and the local
//! Zitadel exposes no `/v2/sessions`, so no second legitimate user could sign
//! in. This slice keeps the authorization rule unchanged (verified owner or
//! admin) but resolves the target **deterministically from the local `users`
//! table**, addressed by user id or email. No directory round trip, no
//! fabricated identity and no weakening of `owner_or_admin`.
//!
//! The target key is validated with the shared `botsecurity_core` validators;
//! every SQL statement is parameterized, so a caller-supplied identifier can
//! never become SQL. Comments, versions and audit rows are untouched — only
//! the `resource_permissions` owner grant is replaced, atomically.

use super::*;
use botsecurity_core::validation::{validate_email, validate_uuid};

/// Maximum accepted length of an email address (RFC 5321 practical limit),
/// also the shared validator's own bound.
const MAX_TARGET_LEN: usize = 254;

/// Caller-supplied target identifier, normalized once at the HTTP boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TransferTargetKey {
    /// A canonical lowercase hyphenated UUID.
    UserId(String),
    /// A lowercase, syntactically valid email address.
    Email(String),
}

/// A target that really exists in the local `users` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransferTarget {
    /// Canonical `users.id` as a lowercase hyphenated UUID.
    pub(crate) id: String,
    /// Stored `users.email`, used as the grantee id: `effective_role` resolves
    /// an authenticated collaborator by email, never by a request parameter.
    pub(crate) email: String,
}

/// Caller identity derived only from verified `AuthenticatedUser` claims,
/// mirrored from `collab_user_id` so a self-transfer cannot be disguised by a
/// different spelling of the same account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallerIdentity {
    pub(crate) user_id: Option<String>,
    pub(crate) email: Option<String>,
}

/// Pure reasons a transfer may be refused. The HTTP layer maps each to a
/// generic, already-translated error body; none of them echoes the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransferRejection {
    InvalidTarget,
    SelfTransfer,
    NotAllowed,
}

/// Normalize and validate the caller-supplied identifier with the shared
/// security validators. Anything else — SQL fragments, whitespace, a bare
/// username, an over-long string — fails closed before the database is hit.
pub(crate) fn normalize_target_key(raw: &str) -> Result<TransferTargetKey, TransferRejection> {
    let value = raw.trim();
    if value.is_empty() || value.len() > MAX_TARGET_LEN {
        return Err(TransferRejection::InvalidTarget);
    }
    if validate_uuid(value).is_ok() {
        return Ok(TransferTargetKey::UserId(value.to_ascii_lowercase()));
    }
    if validate_email(value).is_ok() {
        return Ok(TransferTargetKey::Email(value.to_ascii_lowercase()));
    }
    Err(TransferRejection::InvalidTarget)
}

/// Build the caller identity from verified claims only. The `"session-user"`
/// placeholder used by some local sessions is deliberately excluded, as in
/// `collab_user_id`.
pub(crate) fn caller_identity(user: &AuthenticatedUser) -> CallerIdentity {
    let email = user
        .email
        .as_deref()
        .filter(|value| !value.trim().is_empty() && *value != "session-user")
        .map(|value| value.trim().to_string());
    let user_id = (!user.user_id.is_nil()).then(|| user.user_id.to_string());
    CallerIdentity { user_id, email }
}

/// Pure self-transfer predicate. A target is the caller when either their
/// canonical user id or their verified email matches, case-insensitively.
pub(crate) fn is_self_transfer(caller: &CallerIdentity, target: &TransferTarget) -> bool {
    let same_id = caller
        .user_id
        .as_deref()
        .is_some_and(|id| id.eq_ignore_ascii_case(&target.id));
    let same_email = caller
        .email
        .as_deref()
        .is_some_and(|email| email.eq_ignore_ascii_case(&target.email));
    same_id || same_email
}

/// Pure authorization core. `privileged` is the result of the unchanged
/// `owner_or_admin` lookup, so this function cannot be satisfied by a role
/// smuggled in the request body.
pub(crate) fn authorize_transfer(
    privileged: bool,
    caller: &CallerIdentity,
    target: &TransferTarget,
) -> Result<(), TransferRejection> {
    if !privileged {
        return Err(TransferRejection::NotAllowed);
    }
    if is_self_transfer(caller, target) {
        return Err(TransferRejection::SelfTransfer);
    }
    Ok(())
}

/// Resolve the normalized key against the local `users` table. Both branches
/// are bound parameters; no caller text is ever concatenated into SQL. An
/// unknown target is indistinguishable from "no such user" — no hint about
/// the directory, Zitadel or other accounts is returned.
pub(crate) fn find_user(
    conn: &mut PgConnection,
    key: &TransferTargetKey,
) -> Result<Option<TransferTarget>, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = SqlUuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = Text)]
        email: String,
    }

    let (by_id, by_email) = match key {
        TransferTargetKey::UserId(id) => (id.as_str(), ""),
        TransferTargetKey::Email(email) => ("", email.as_str()),
    };
    let rows = diesel::sql_query(
        "SELECT id, email FROM users \
         WHERE ($1::text <> '' AND id::text = $1) \
            OR ($2::text <> '' AND lower(email) = $2) \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind::<Text, _>(by_id)
    .bind::<Text, _>(by_email)
    .load::<UserRow>(conn)?;

    Ok(rows.into_iter().next().map(|row| TransferTarget {
        id: row.id.to_string(),
        email: row.email.trim().to_string(),
    }))
}

/// Reject with a sanitized server-side cause and a generic, localized body.
/// Existing catalog keys are reused so no new user-facing string is added to
/// any locale (#2094); the caller never sees the target id, email or the
/// reason string.
fn rejection(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    reason: &'static str,
) -> (StatusCode, Json<serde_json::Value>) {
    let safe = botsecurity_core::error_sanitizer::log_and_sanitize_str(
        reason,
        "collab_routes::transfer_ownership",
        None,
    );
    warn!("collab transfer rejected ({status}): {}", safe.message);
    err(locale, status, "error-collab-invalid-new-owner")
}

/// `POST /api/collab/permissions/transfer` — transfer ownership to a user
/// that exists in the local `users` table. Owner (or admin) only; records a
/// `transfer` audit event. Comments, versions and earlier audit rows are left
/// exactly as they were — only the owner grant changes.
pub async fn transfer_ownership(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<TransferBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ));
    }
    let key = normalize_target_key(&req.new_owner_id)
        .map_err(|_| rejection(locale, StatusCode::BAD_REQUEST, "invalid transfer target"))?;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // Unchanged authorization rule: verified owner or admin, resolved from
    // claims by the existing RBAC-aware helper.
    if !owner_or_admin(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
    }

    let target = find_user(&mut conn, &key)
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| {
            rejection(
                locale,
                StatusCode::NOT_FOUND,
                "transfer target not found in local users",
            )
        })?;
    let caller = caller_identity(&user);
    // `true` because `owner_or_admin` above already proved the privilege; the
    // pure core still takes it explicitly so the rule is testable in isolation.
    authorize_transfer(true, &caller, &target).map_err(|reason| match reason {
        TransferRejection::SelfTransfer => {
            rejection(locale, StatusCode::BAD_REQUEST, "self-transfer rejected")
        }
        TransferRejection::NotAllowed => {
            rejection(locale, StatusCode::FORBIDDEN, "caller is not owner or admin")
        }
        TransferRejection::InvalidTarget => {
            rejection(locale, StatusCode::BAD_REQUEST, "invalid transfer target")
        }
    })?;

    // Atomic swap: between the DELETE and the INSERT no reader can observe a
    // resource with no owner, and a failure rolls both statements back.
    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::sql_query(
            "DELETE FROM resource_permissions \
             WHERE resource_type = $1 AND resource_id = $2 \
               AND grantee_type = 'user' AND role = 'owner'",
        )
        .bind::<Text, _>(&req.resource_type)
        .bind::<Text, _>(&req.resource_id)
        .execute(conn)?;

        diesel::sql_query(
            "INSERT INTO resource_permissions \
             (resource_type, resource_id, grantee_type, grantee_id, role) \
             VALUES ($1, $2, 'user', $3, 'owner') \
             ON CONFLICT (resource_type, resource_id, grantee_type, grantee_id) \
             DO UPDATE SET role = 'owner', updated_at = NOW()",
        )
        .bind::<Text, _>(&req.resource_type)
        .bind::<Text, _>(&req.resource_id)
        .bind::<Text, _>(&target.email)
        .execute(conn)?;
        Ok(())
    })
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    record_activity(
        &mut conn,
        &collab_user_id(&user),
        &collab_user_name(&user),
        &req.resource_type,
        &req.resource_id,
        "transfer",
        &serde_json::json!({ "new_owner_id": target.id }),
    );
    Ok(Json(serde_json::json!({
        "success": true,
        "new_owner_id": target.id,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER_ID: &str = "69a32dda-0000-4000-8000-000000000001";
    const TARGET_ID: &str = "b2a87ec8-0000-4000-8000-000000000002";

    fn target() -> TransferTarget {
        TransferTarget {
            id: TARGET_ID.to_string(),
            email: "reviewer@example.com".to_string(),
        }
    }

    fn caller(id: Option<&str>, email: Option<&str>) -> CallerIdentity {
        CallerIdentity {
            user_id: id.map(str::to_string),
            email: email.map(str::to_string),
        }
    }

    #[test]
    fn a_user_id_is_normalized_to_a_canonical_uuid() {
        let upper = format!("{}", TARGET_ID.to_ascii_uppercase());
        assert_eq!(
            normalize_target_key(&upper),
            Ok(TransferTargetKey::UserId(TARGET_ID.to_string()))
        );
        assert_eq!(
            normalize_target_key(&format!("  {TARGET_ID}  ")),
            Ok(TransferTargetKey::UserId(TARGET_ID.to_string()))
        );
    }

    #[test]
    fn an_email_is_normalized_with_the_shared_validator() {
        assert_eq!(
            normalize_target_key(" Reviewer@Example.COM "),
            Ok(TransferTargetKey::Email("reviewer@example.com".to_string()))
        );
        assert_eq!(
            normalize_target_key("first.last+tag@corp.example.org"),
            Ok(TransferTargetKey::Email(
                "first.last+tag@corp.example.org".to_string()
            ))
        );
    }

    #[test]
    fn malformed_and_injection_targets_fail_closed() {
        let overlong = "a".repeat(MAX_TARGET_LEN + 1);
        for raw in [
            "",
            "   ",
            "reviewer",
            "reviewer@",
            "@example.com",
            "a@@example.com",
            "two words@example.com",
            "x'; DROP TABLE users; --",
            overlong.as_str(),
        ] {
            assert_eq!(normalize_target_key(raw), Err(TransferRejection::InvalidTarget), "{raw}");
        }
    }

    #[test]
    fn self_transfer_matches_email_or_uuid_case_insensitively() {
        let target = target();
        assert!(is_self_transfer(&caller(Some(TARGET_ID), None), &target));
        assert!(is_self_transfer(
            &caller(None, Some("REVIEWER@example.com")),
            &target
        ));
        assert!(is_self_transfer(
            &caller(Some(OWNER_ID), Some("Reviewer@Example.com")),
            &target
        ));
    }

    #[test]
    fn a_foreign_caller_is_not_a_self_transfer() {
        let target = target();
        assert!(!is_self_transfer(&caller(Some(OWNER_ID), Some("admin@localhost")), &target));
        // A session without claims is never the target merely by omission.
        assert!(!is_self_transfer(&caller(None, None), &target));
    }

    #[test]
    fn privileged_non_self_transfer_is_authorized() {
        let caller = caller(Some(OWNER_ID), Some("admin@localhost"));
        assert_eq!(authorize_transfer(true, &caller, &target()), Ok(()));
    }

    #[test]
    fn authorization_never_accepts_a_non_owner_or_a_self_transfer() {
        let target = target();
        let stranger = caller(Some(OWNER_ID), Some("admin@localhost"));
        assert_eq!(
            authorize_transfer(false, &stranger, &target),
            Err(TransferRejection::NotAllowed)
        );
        // Even an admin must not transfer a resource to itself.
        let self_caller = caller(Some(TARGET_ID), Some("reviewer@example.com"));
        assert_eq!(
            authorize_transfer(true, &self_caller, &target),
            Err(TransferRejection::SelfTransfer)
        );
        // No privilege and self-transfer: privilege is checked first.
        assert_eq!(
            authorize_transfer(false, &self_caller, &target),
            Err(TransferRejection::NotAllowed)
        );
    }
}
