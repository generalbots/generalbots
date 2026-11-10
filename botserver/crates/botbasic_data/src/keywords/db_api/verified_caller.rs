//! Verified caller identity for the generic `/api/db/*` CRUD routes (#2122).
//!
//! These routes used to build [`UserRoles`] straight out of the `X-User-Id`
//! and `X-User-Roles` request headers. Both are caller-supplied, so anyone who
//! could reach the API port could name any user — or grant themselves `admin` —
//! and then read, create, update or delete rows in every table the generic CRUD
//! surface exposes. Table-level and field-level access control is only as good
//! as the identity it is evaluated against, so the header path is gone.
//!
//! Identity now comes from server-verified sources only:
//!
//! * the [`AuthenticatedUser`] request extension the global auth middleware
//!   inserts after it verified a bearer JWT, API key or session cookie, or
//! * the `Authorization: Bearer` credential itself, verified against the SaaS
//!   JWT secret by `botsecurity_core::verified_jwt` — the same verifier cloud
//!   login uses and the same one that accepts the signed `api.exec` loopback
//!   assertion (`gb_aud`/`scope`, HS256 with the SaaS secret), so an internal
//!   call is authenticated by a signature instead of by headers.
//!
//! Roles are the union of the verified principal's roles and the caller's
//! active `rbac_groups` memberships — both server-side data. A request without
//! a usable credential is rejected with `401`; it never receives a default
//! identity and never a default role.

use super::*;
use axum::Extension;
use botsecurity::auth::{AuthenticatedUser, Role};
use botsecurity::botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt};
use diesel::sql_types::{Text, Uuid as SqlUuid};
use diesel::QueryableByName;

/// Why a caller could not be verified.
///
/// Deliberately a single variant: the caller learns that it is unauthenticated,
/// never which check failed (that detail goes to the server log only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbApiAuthError {
    Unauthenticated,
}

impl DbApiAuthError {
    /// Renders the rejection. Every variant is a `401` — fail closed.
    pub(crate) fn into_response(self) -> axum::response::Response {
        match self {
            DbApiAuthError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "authentication required" })),
            )
                .into_response(),
        }
    }
}

/// A caller whose credentials were verified by the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerifiedCaller {
    user_id: Uuid,
    email: Option<String>,
    /// Roles the verified credential already established.
    roles: Vec<String>,
}

/// Resolves the caller from the auth extension, else from the Bearer credential.
///
/// `user` is the optional [`AuthenticatedUser`] extension; handlers pass it
/// through so a single call site covers both credential sources.
pub(crate) fn caller_from(
    user: Option<Extension<AuthenticatedUser>>,
    headers: &HeaderMap,
) -> Result<VerifiedCaller, DbApiAuthError> {
    VerifiedCaller::resolve(user.as_ref().map(|Extension(user)| user), headers)
}

impl VerifiedCaller {
    pub(crate) fn resolve(
        user: Option<&AuthenticatedUser>,
        headers: &HeaderMap,
    ) -> Result<Self, DbApiAuthError> {
        if let Some(user) = user {
            if let Some(caller) = Self::from_principal(user) {
                return Ok(caller);
            }
        }
        Self::from_bearer_token(headers)
    }

    /// The extension is authoritative when it names a real principal.
    ///
    /// The middleware always inserts an extension — including the anonymous
    /// fallback for public paths — so an anonymous one must not be accepted
    /// here. The single-user desktop principal is the documented exception: it
    /// is only ever inserted when the profile sets `require_auth=false`, and it
    /// carries the `desktop_local` metadata marker.
    fn from_principal(user: &AuthenticatedUser) -> Option<Self> {
        let local_desktop = user.metadata.contains_key("desktop_local");
        if !user.is_authenticated() && !local_desktop {
            return None;
        }
        Some(Self {
            user_id: user.user_id,
            email: user.email.clone().filter(|email| !email.trim().is_empty()),
            roles: user.roles.iter().map(role_name).map(str::to_string).collect(),
        })
    }

    /// Verifies the Bearer credential the way every other handler does.
    ///
    /// A forged, tampered, `alg=none` or expired token yields `401`; the
    /// payload is never read on its own.
    fn from_bearer_token(headers: &HeaderMap) -> Result<Self, DbApiAuthError> {
        let token = bearer_token(headers).ok_or(DbApiAuthError::Unauthenticated)?;
        let claims = verify_saas_jwt(token).map_err(|reason| {
            botlog::safe_warn!("db_api: bearer credential rejected: {reason}");
            DbApiAuthError::Unauthenticated
        })?;
        // Cloud/suite tokens carry a UUID `sub`; a Zitadel numeric subject is
        // mapped to the same deterministic UUID the SaaS auth provider derives,
        // so RBAC lookups agree with the rest of the server.
        let user_id = match Uuid::parse_str(claims.sub.trim()) {
            Ok(user_id) => user_id,
            Err(_) => Uuid::new_v5(
                &Uuid::NAMESPACE_DNS,
                format!("zitadel:{}", claims.sub).as_bytes(),
            ),
        };
        Ok(Self {
            user_id,
            email: claims.email,
            roles: Vec::new(),
        })
    }

    /// Table-access roles for this caller.
    ///
    /// The verified principal's roles are combined with the names of the
    /// caller's active `rbac_groups` rows — the same membership rows
    /// `resolve_user_role` reads. The lookup is server-side data, so a failure
    /// yields *fewer* roles (deny), never a default one.
    pub(crate) fn user_roles(&self, conn: &mut diesel::PgConnection) -> UserRoles {
        let mut roles = self.roles.clone();
        merge_roles(&mut roles, active_group_roles(conn, self.user_id));

        // Identity-id derivation drift (#808): older deployments stored the
        // memberships under the canonical `users` row for the same email, so a
        // derived id finds none. Mirrors `resolve_user_role_with_email` — the
        // email comes from the verified credential, never from a header.
        if roles.len() == self.roles.len() {
            if let Some(canonical) = self.canonical_user_id(conn) {
                if canonical != self.user_id {
                    merge_roles(&mut roles, active_group_roles(conn, canonical));
                }
            }
        }

        if self.user_id.is_nil() {
            UserRoles::new(roles)
        } else {
            UserRoles::with_user_id(roles, self.user_id)
        }
    }

    /// The `users` row that owns the verified email, when it differs from the
    /// derived id.
    fn canonical_user_id(&self, conn: &mut diesel::PgConnection) -> Option<Uuid> {
        let email = self.email.as_deref()?;

        #[derive(QueryableByName)]
        struct IdRow {
            #[diesel(sql_type = SqlUuid)]
            id: Uuid,
        }

        let row: Result<Option<IdRow>, _> = diesel::sql_query(
            "SELECT id FROM users WHERE lower(email) = lower($1) AND is_active = true LIMIT 1",
        )
        .bind::<Text, _>(email)
        .get_result(conn)
        .optional();

        match row {
            Ok(row) => row.map(|row| row.id),
            Err(error) => {
                botlog::safe_warn!("db_api: canonical user lookup failed: {error}");
                None
            }
        }
    }
}

/// Adds role names that are not already present, preserving the verified order.
fn merge_roles(target: &mut Vec<String>, names: Vec<String>) {
    for name in names {
        if !target.contains(&name) {
            target.push(name);
        }
    }
}

/// Role name used by the `table_access` rules (`read_roles`/`write_roles`).
fn role_name(role: &Role) -> &'static str {
    match role {
        Role::Anonymous => "anonymous",
        Role::User => "user",
        Role::Moderator => "moderator",
        Role::Admin => "admin",
        Role::SuperAdmin => "superadmin",
        Role::Service => "service",
        Role::Bot => "bot",
        Role::BotOwner => "bot_owner",
        Role::BotOperator => "bot_operator",
        Role::BotViewer => "bot_viewer",
    }
}

#[derive(QueryableByName)]
struct GroupRoleRow {
    #[diesel(sql_type = Text)]
    name: String,
}

/// Names of the caller's active RBAC groups, lowercased.
fn active_group_roles(conn: &mut diesel::PgConnection, user_id: Uuid) -> Vec<String> {
    if user_id.is_nil() {
        return Vec::new();
    }
    let rows: Result<Vec<GroupRoleRow>, _> = diesel::sql_query(
        "SELECT g.name AS name FROM rbac_user_groups ug \
         JOIN rbac_groups g ON g.id = ug.group_id \
         WHERE ug.user_id = $1 AND g.is_active = TRUE",
    )
    .bind::<SqlUuid, _>(user_id)
    .load(conn);

    match rows {
        Ok(rows) => rows
            .into_iter()
            .map(|row| row.name.trim().to_lowercase())
            .filter(|name| !name.is_empty())
            .collect(),
        Err(error) => {
            botlog::safe_warn!("db_api: RBAC group roles unavailable for user {user_id}: {error}");
            Vec::new()
        }
    }
}

#[cfg(test)]
#[path = "verified_caller_tests.rs"]
mod tests;
