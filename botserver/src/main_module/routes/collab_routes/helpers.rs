use super::*;

/// Localized API error (#2094) — `code` stays machine-readable.
pub(crate) fn err(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    key: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab error ({}): {key}", status.as_u16());
    (status, Json(botcore::i18n::error_body(locale, key)))
}

/// Internal failure: the cause is logged, the client gets a generic localized
/// message — no internal error strings leak to the caller.
pub(crate) fn err_db<E: std::fmt::Display>(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    error: E,
) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab internal error ({}): {error}", status.as_u16());
    (
        status,
        Json(botcore::i18n::error_body(locale, "error-db-unavailable")),
    )
}

pub(crate) fn collab_user_id(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.clone();
        }
    }
    if user.user_id.is_nil() {
        "default".to_string()
    } else {
        user.user_id.to_string()
    }
}

pub(crate) fn collab_user_name(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.split('@').next().unwrap_or(email).to_string();
        }
    }
    if !user.username.is_empty() {
        user.username.clone()
    } else {
        "User".to_string()
    }
}

/// Extract `@mention` tokens from a comment body. Tokens are `@` followed by
/// 2+ word characters; trailing punctuation is stripped.
pub(crate) fn extract_mentions(body: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    for token in body.split_whitespace() {
        if let Some(rest) = token.strip_prefix('@') {
            let cleaned: String = rest
                .trim_end_matches(|c: char| c.is_ascii_punctuation())
                .to_string();
            if cleaned.chars().count() >= 2 {
                mentions.push(cleaned);
            }
        }
    }
    mentions
}

/// #2452 criterion 4 — the IP-aware shape is the primary INSERT; the original
/// column list is kept as a fallback so a deployment whose `collab_activity`
/// predates the migration still records the row instead of losing it.
const ACTIVITY_INSERT_WITH_IP: &str = "INSERT INTO collab_activity \
     (resource_type, resource_id, actor_id, actor_name, action, payload, ip) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)";
const ACTIVITY_INSERT_WITHOUT_IP: &str = "INSERT INTO collab_activity \
     (resource_type, resource_id, actor_id, actor_name, action, payload) \
     VALUES ($1, $2, $3, $4, $5, $6)";

/// Append an audit-trail row for a resource. This is fire-and-forget from the
/// mutating handlers — an audit-log failure is logged but never fatal to the
/// primary write it trails. Existing callers keep this exact signature and
/// write `ip = NULL` through [`record_activity_with_ip`].
pub(crate) fn record_activity(
    conn: &mut PgConnection,
    actor_id: &str,
    actor_name: &str,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    payload: &serde_json::Value,
) {
    record_activity_with_ip(
        conn,
        actor_id,
        actor_name,
        resource_type,
        resource_id,
        action,
        payload,
        None,
    );
}

/// IP-aware variant of [`record_activity`]. `ip` is the server-resolved peer
/// address supplied by the request layer; `None` is stored as SQL NULL and is
/// never fabricated here. A failed IP-aware insert falls back to the legacy
/// column shape so an audit row is never dropped on an older deployment.
pub(crate) fn record_activity_with_ip(
    conn: &mut PgConnection,
    actor_id: &str,
    actor_name: &str,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    payload: &serde_json::Value,
    ip: Option<&str>,
) {
    let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    let result = diesel::sql_query(ACTIVITY_INSERT_WITH_IP)
        .bind::<Text, _>(resource_type)
        .bind::<Text, _>(resource_id)
        .bind::<Text, _>(actor_id)
        .bind::<Text, _>(actor_name)
        .bind::<Text, _>(action)
        .bind::<Text, _>(&payload_json)
        .bind::<Nullable<Text>, _>(ip)
        .execute(conn);
    if let Err(e) = result {
        // Non-fatal by design: an older deployment without the `ip` column
        // still keeps the audit row through the legacy INSERT below.
        warn!("collab activity IP-aware insert failed, retrying legacy shape: {e}");
    } else {
        return;
    }
    let fallback = diesel::sql_query(ACTIVITY_INSERT_WITHOUT_IP)
        .bind::<Text, _>(resource_type)
        .bind::<Text, _>(resource_id)
        .bind::<Text, _>(actor_id)
        .bind::<Text, _>(actor_name)
        .bind::<Text, _>(action)
        .bind::<Text, _>(&payload_json)
        .execute(conn);
    if let Err(e) = fallback {
        warn!("collab activity insert failed: {e}");
    }
}

/// Resolve the (resource_type, resource_id) a comment belongs to, so mutating
/// handlers can write an audit trail against the parent resource.
pub(crate) fn comment_resource(
    conn: &mut PgConnection,
    id: uuid::Uuid,
) -> Option<(String, String)> {
    #[derive(QueryableByName)]
    struct ResRow {
        #[diesel(sql_type = Text)]
        pub(crate) resource_type: String,
        #[diesel(sql_type = Text)]
        pub(crate) resource_id: String,
    }
    diesel::sql_query("SELECT resource_type, resource_id FROM collab_comments WHERE id = $1")
        .bind::<SqlUuid, _>(id)
        .load::<ResRow>(conn)
        .ok()
        .and_then(|mut rows| rows.pop())
        .map(|r| (r.resource_type, r.resource_id))
}

/// Resolve the (resource_type, resource_id) a version belongs to. Handlers
/// addressed by a bare version id use this to derive the ownership scope from
/// the stored row before returning or mutating its content (#2768).
pub(crate) fn version_resource(
    conn: &mut PgConnection,
    id: uuid::Uuid,
) -> Option<(String, String)> {
    #[derive(QueryableByName)]
    struct ResRow {
        #[diesel(sql_type = Text)]
        pub(crate) resource_type: String,
        #[diesel(sql_type = Text)]
        pub(crate) resource_id: String,
    }
    diesel::sql_query("SELECT resource_type, resource_id FROM collab_versions WHERE id = $1")
        .bind::<SqlUuid, _>(id)
        .load::<ResRow>(conn)
        .ok()
        .and_then(|mut rows| rows.pop())
        .map(|r| (r.resource_type, r.resource_id))
}

/// SHA-256 hex digest of a snapshot's content, used to dedup unchanged saves
/// and shown as an integrity fingerprint in the version list.
pub(crate) fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Resolve the effective role for `user` on a resource. Order: explicit user
/// grant (owner first), then a domain grant matching the user's email domain.
/// Returns None when the user has no access.
pub(crate) fn effective_role(
    conn: &mut PgConnection,
    user: &AuthenticatedUser,
    resource_type: &str,
    resource_id: &str,
) -> Option<String> {
    let uid = collab_user_id(user);
    let uname = collab_user_name(user);

    #[derive(QueryableByName)]
    struct RoleRow {
        #[diesel(sql_type = Text)]
        pub(crate) role: String,
    }

    let rows = match diesel::sql_query(
        "SELECT role FROM resource_permissions \
         WHERE resource_type = $1 AND resource_id = $2 AND grantee_type = 'user' \
           AND (grantee_id = $3 OR grantee_id = $4) \
         ORDER BY CASE role WHEN 'owner' THEN 0 ELSE 1 END LIMIT 1",
    )
    .bind::<Text, _>(resource_type)
    .bind::<Text, _>(resource_id)
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&uname)
    .load::<RoleRow>(conn)
    {
        Ok(r) => r,
        Err(e) => {
            warn!("permission lookup failed: {e}");
            return None;
        }
    };
    if let Some(r) = rows.into_iter().next() {
        return Some(r.role);
    }

    if let Some(ref email) = user.email {
        if let Some(domain) = email.split('@').nth(1) {
            let bare = domain.to_string();
            let at = format!("@{domain}");
            let rows = match diesel::sql_query(
                "SELECT role FROM resource_permissions \
                 WHERE resource_type = $1 AND resource_id = $2 AND grantee_type = 'domain' \
                   AND (grantee_id = $3 OR grantee_id = $4) LIMIT 1",
            )
            .bind::<Text, _>(resource_type)
            .bind::<Text, _>(resource_id)
            .bind::<Text, _>(&bare)
            .bind::<Text, _>(&at)
            .load::<RoleRow>(conn)
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("domain permission lookup failed: {e}");
                    return None;
                }
            };
            if let Some(r) = rows.into_iter().next() {
                return Some(r.role);
            }
        }
    }
    None
}

/// Pure ownership predicate over a role resolved from `resource_permissions`.
/// `None` means the caller has no grant at all (a foreign resource), which is
/// never ownership. The role comes from `effective_role`, which derives the
/// caller identity from *verified* JWT claims (user id / email / email domain),
/// never from a request parameter (#2768).
pub(crate) fn is_owner_role(role: Option<&str>) -> bool {
    role == Some("owner")
}

/// Pure access predicate over a role resolved from `resource_permissions`.
/// Any explicit grant — viewer, commenter, editor or owner — is legitimate
/// collaboration; `None` (foreign resource) is denied. The admin/super-admin
/// bypass is applied in [`has_collab_access`], where the verified
/// `AuthenticatedUser` claims are available (#2768).
pub(crate) fn grants_collab_access(role: Option<&str>) -> bool {
    matches!(role, Some("viewer" | "commenter" | "editor" | "owner"))
}

/// Gate every collab REST handler on the caller's relationship to the target
/// resource. Scope is derived from verified claims only: the `AuthenticatedUser`
/// extracted by the auth middleware (admin flags, user id, email, domain). The
/// resource type/id come from the request but are only used as a lookup key into
/// `resource_permissions`. Fail closed: no grant and no admin claim → deny.
pub(crate) fn has_collab_access(
    conn: &mut PgConnection,
    user: &AuthenticatedUser,
    resource_type: &str,
    resource_id: &str,
) -> bool {
    user.is_admin()
        || user.is_super_admin()
        || grants_collab_access(effective_role(conn, user, resource_type, resource_id).as_deref())
}

pub(crate) fn is_owner(
    conn: &mut PgConnection,
    user: &AuthenticatedUser,
    resource_type: &str,
    resource_id: &str,
) -> bool {
    is_owner_role(effective_role(conn, user, resource_type, resource_id).as_deref())
}

/// Bootstrap ownership: if a resource has no owner grant yet, make the current
/// user its owner. Returns true if the user is (now) the owner. This lets the
/// first collaborator to share a resource claim ownership without any client
/// input.
pub(crate) fn ensure_owner(
    conn: &mut PgConnection,
    user: &AuthenticatedUser,
    resource_type: &str,
    resource_id: &str,
) -> bool {
    if is_owner(conn, user, resource_type, resource_id) {
        return true;
    }
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        pub(crate) count: i64,
    }
    let owned = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM resource_permissions \
         WHERE resource_type = $1 AND resource_id = $2 AND grantee_type = 'user' AND role = 'owner'",
    )
    .bind::<Text, _>(resource_type)
    .bind::<Text, _>(resource_id)
    .load::<CountRow>(conn)
    .ok()
    .and_then(|mut r| r.pop())
    .map(|r| r.count)
    .unwrap_or(0);
    if owned > 0 {
        return false;
    }
    let uid = collab_user_id(user);
    let res = diesel::sql_query(
        "INSERT INTO resource_permissions \
         (resource_type, resource_id, grantee_type, grantee_id, role) \
         VALUES ($1, $2, 'user', $3, 'owner') ON CONFLICT DO NOTHING",
    )
    .bind::<Text, _>(resource_type)
    .bind::<Text, _>(resource_id)
    .bind::<Text, _>(&uid)
    .execute(conn);
    match res {
        Ok(_) => true,
        Err(e) => {
            warn!("owner bootstrap failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two-tenant fixture for the pure predicates. Each tenant owns a distinct
    /// resource; there is deliberately no cross-grant. A lookup for a resource
    /// the caller has no grant on resolves to `None`, exactly like
    /// `effective_role` does against `resource_permissions`.
    fn granted_role<'a>(
        grants: &'a [((&str, &str), &'a str)],
        tenant: &str,
        resource: &str,
    ) -> Option<&'a str> {
        grants
            .iter()
            .find(|((t, r), _)| *t == tenant && *r == resource)
            .map(|(_, role)| *role)
    }

    #[test]
    fn only_an_explicit_owner_grant_counts_as_ownership() {
        let grants = [
            (("tenant-a", "doc-a"), "owner"),
            (("tenant-b", "doc-b"), "owner"),
        ];
        assert!(is_owner_role(granted_role(&grants, "tenant-a", "doc-a")));
        assert!(is_owner_role(granted_role(&grants, "tenant-b", "doc-b")));
        // A foreign resource has no grant for the caller — never ownership.
        assert!(!is_owner_role(granted_role(&grants, "tenant-a", "doc-b")));
        assert!(!is_owner_role(granted_role(&grants, "tenant-b", "doc-a")));
        assert!(!is_owner_role(None));
        assert!(!is_owner_role(Some("editor")));
    }

    #[test]
    fn any_explicit_grant_allows_collaboration_but_foreign_resources_do_not() {
        let grants = [
            (("tenant-a", "doc-a"), "owner"),
            (("tenant-b", "doc-b"), "viewer"),
            (("tenant-b", "doc-c"), "commenter"),
            (("tenant-b", "doc-d"), "editor"),
        ];
        assert!(grants_collab_access(granted_role(&grants, "tenant-a", "doc-a")));
        assert!(grants_collab_access(granted_role(&grants, "tenant-b", "doc-b")));
        assert!(grants_collab_access(granted_role(&grants, "tenant-b", "doc-c")));
        assert!(grants_collab_access(granted_role(&grants, "tenant-b", "doc-d")));
        // Tenant B has no grant on A's document and must be denied.
        assert!(!grants_collab_access(granted_role(&grants, "tenant-b", "doc-a")));
        assert!(!grants_collab_access(granted_role(&grants, "tenant-a", "doc-b")));
        assert!(!grants_collab_access(None));
    }

    #[test]
    fn unknown_or_malformed_roles_are_denied() {
        assert!(!grants_collab_access(Some("")));
        assert!(!grants_collab_access(Some("administrator")));
        assert!(!grants_collab_access(Some("OWNER")));
        assert!(!is_owner_role(Some("administrator")));
    }
}
