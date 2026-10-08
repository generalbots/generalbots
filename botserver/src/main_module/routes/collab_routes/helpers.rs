use super::*;

pub(crate) fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab error ({}): {}", status.as_u16(), msg);
    (status, Json(serde_json::json!({ "error": msg })))
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

/// Append an audit-trail row for a resource. This is fire-and-forget from the
/// mutating handlers — an audit-log failure is logged but never fatal to the
/// primary write it trails.
pub(crate) fn record_activity(
    conn: &mut PgConnection,
    actor_id: &str,
    actor_name: &str,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    payload: &serde_json::Value,
) {
    let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    let res = diesel::sql_query(
        "INSERT INTO collab_activity \
         (resource_type, resource_id, actor_id, actor_name, action, payload) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<Text, _>(resource_type)
    .bind::<Text, _>(resource_id)
    .bind::<Text, _>(actor_id)
    .bind::<Text, _>(actor_name)
    .bind::<Text, _>(action)
    .bind::<Text, _>(&payload_json)
    .execute(conn);
    if let Err(e) = res {
        warn!("collab activity insert failed: {e}");
    }
}

/// Resolve the (resource_type, resource_id) a comment belongs to, so mutating
/// handlers can write an audit trail against the parent resource.
pub(crate) fn comment_resource(conn: &mut PgConnection, id: uuid::Uuid) -> Option<(String, String)> {
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

pub(crate) fn is_owner(
    conn: &mut PgConnection,
    user: &AuthenticatedUser,
    resource_type: &str,
    resource_id: &str,
) -> bool {
    effective_role(conn, user, resource_type, resource_id).as_deref() == Some("owner")
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
