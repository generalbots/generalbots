use super::*;

pub(crate) fn base64_url_encode(input: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD
        .encode(input)
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

/// Picks the JWT subject for a password-verified Zitadel session. Normally the
/// Zitadel user id is used (RBAC and org checks derive a stable UUIDv5 from
/// it). When that identity has no users-table row but the verified email does,
/// keep the existing account — its org memberships and RBAC groups survive
/// Zitadel account re-creation (new numeric id, same email). The SaaS JWT
/// provider parses UUID subjects directly, so a users-row id works unchanged.
pub(crate) fn resolve_login_subject(
    conn: &mut diesel::PgConnection,
    zitadel_user_id: &str,
    email: &str,
) -> String {
    let derived = Uuid::new_v5(
        &Uuid::NAMESPACE_DNS,
        format!("zitadel:{zitadel_user_id}").as_bytes(),
    );
    #[derive(diesel::QueryableByName)]
    struct UuidRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let email_row = diesel::sql_query(
        "SELECT id FROM users WHERE email = $1 AND is_active = true LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<UuidRow>(conn)
    .optional()
    .ok()
    .flatten()
    .map(|r| r.id);
    let derived_row = diesel::sql_query("SELECT id FROM users WHERE id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(derived)
        .get_result::<UuidRow>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.id);
    match (derived_row, email_row) {
        (Some(_), _) => zitadel_user_id.to_string(),
        (None, Some(existing)) => existing.to_string(),
        (None, None) => zitadel_user_id.to_string(),
    }
}

/// Resolves the caller's workspace branch from their user→org membership
/// binding (users → user_organizations → branches). Returns `None` when the
/// user has no verified org binding — callers then omit the branch claim
/// rather than minting an unverified scope.
pub(crate) fn resolve_branch_from_user_binding(conn: &mut diesel::PgConnection, email: &str) -> Option<Uuid> {
    #[derive(diesel::QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let user_id = diesel::sql_query("SELECT id FROM users WHERE email = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(email)
        .get_result::<UserRow>(conn)
        .optional()
        .ok()
        .flatten()?
        .id;

    #[derive(diesel::QueryableByName)]
    struct BindingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    let org_id = diesel::sql_query(
        "SELECT org_id FROM user_organizations WHERE user_id = $1 ORDER BY is_default DESC, joined_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<BindingRow>(conn)
    .optional()
    .ok()
    .flatten()?
    .org_id;

    #[derive(diesel::QueryableByName)]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    diesel::sql_query(
        "SELECT id FROM branches WHERE org_id = $1 AND is_active = true ORDER BY created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .get_result::<BranchRow>(conn)
    .optional()
    .ok()
    .flatten()
    .map(|r| r.id)
}

/// Resolves the effective role vector from RBAC group membership (fix #843).
/// Maps non-UUID identity ids (Zitadel numeric) through the same stable UUID
/// derivation used by `resolve_user_role` in the main crate. Falls back to
/// the plain "user" role when the user has no admin group.
pub(crate) fn resolve_rbac_roles(conn: &mut diesel::PgConnection, user_id: &str) -> Vec<String> {
    let stable = match Uuid::parse_str(user_id) {
        Ok(u) => u,
        Err(_) => Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{user_id}").as_bytes()),
    };
    #[derive(diesel::QueryableByName)]
    struct GroupName {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let names: Vec<GroupName> = diesel::sql_query(
        "SELECT g.name FROM rbac_groups g \
         JOIN rbac_user_groups ug ON ug.group_id = g.id \
         WHERE ug.user_id = $1 AND g.is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(stable)
    .load(conn)
    .unwrap_or_default();
    if names.iter().any(|g| g.name.to_lowercase().contains("admin")) {
        vec!["admin".to_string()]
    } else {
        vec!["user".to_string()]
    }
}

/// Fallback for local dev: resolves the Zitadel user_id for the bootstrap admin
/// from `admin-credentials.json` when the Zitadel sessions API is unavailable.
/// Verifies the password against the salted digest persisted by the desktop
/// wizard (never against a plaintext value read from disk). The legacy
/// `admin@localhost / "dev"` bootstrap is accepted only when
/// `BOTSERVER_DEV_MODE=1` is set. Never an email-only match.
pub(crate) fn lookup_admin_credentials_user_id(email: &str, password: &str) -> Option<String> {
    let base = std::env::current_dir().ok()?;
    let mut candidates = vec![
        base.join("botserver-stack/conf/directory/admin-credentials.json"),
        base.join("../botserver-stack/conf/directory/admin-credentials.json"),
    ];
    // Desktop edition: the stack lives outside the process working directory.
    if let Ok(stack) = std::env::var("BOTSERVER_STACK_PATH") {
        if !stack.trim().is_empty() {
            candidates.push(
                std::path::PathBuf::from(stack).join("conf/directory/admin-credentials.json"),
            );
        }
    }
    if let Ok(stack) = std::env::var("GBO_STACK_PATH") {
        if !stack.trim().is_empty() {
            candidates.push(
                std::path::PathBuf::from(stack).join("conf/directory/admin-credentials.json"),
            );
        }
    }
    for candidate in candidates {
        // Missing/unreadable file: try the NEXT candidate — an early `?` here
        // silently disabled the dev fallback whenever botserver's cwd was not
        // the repo root (first candidate simply did not exist).
        let Ok(content) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let cred_email = json.get("email").and_then(|v| v.as_str()).unwrap_or("");
            if !cred_email.eq_ignore_ascii_case(email) {
                continue;
            }
            // #2231 — verify against the salted digest persisted by the
            // desktop wizard (never against a plaintext password read from
            // disk). Legacy files that still carry a plaintext `password`
            // field are accepted only to upgrade them in place, then scrubbed.
            let file_salt = json.get("password_salt").and_then(|v| v.as_str()).unwrap_or("");
            let file_hash = json.get("password_hash").and_then(|v| v.as_str()).unwrap_or("");
            let mut legacy_plaintext_match = false;
            let matched = if !file_salt.is_empty() && !file_hash.is_empty() {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(file_salt.as_bytes());
                hasher.update(password.as_bytes());
                format!("{:x}", hasher.finalize()) == file_hash
            } else {
                let file_password = json.get("password").and_then(|v| v.as_str()).unwrap_or("");
                legacy_plaintext_match = !file_password.is_empty() && file_password == password;
                legacy_plaintext_match
            };
            // #2231 — the hardcoded "dev" bootstrap is gated behind an explicit
            // dev-mode flag so production builds never accept it.
            if !matched {
                let dev_mode = std::env::var("BOTSERVER_DEV_MODE")
                    .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                if dev_mode
                    && email.eq_ignore_ascii_case("admin@localhost")
                    && password == "dev"
                {
                    // fall through to user_id resolution below
                } else {
                    continue;
                }
            }
            // Upgrade legacy plaintext files to the hashed form (best effort).
            if legacy_plaintext_match {
                use sha2::{Digest, Sha256};
                let salt = uuid::Uuid::new_v4().to_string();
                let mut hasher = Sha256::new();
                hasher.update(salt.as_bytes());
                hasher.update(password.as_bytes());
                let hash = format!("{:x}", hasher.finalize());
                let mut upgraded = json.clone();
                if let Some(obj) = upgraded.as_object_mut() {
                    obj.insert("password_salt".to_string(), serde_json::Value::String(salt));
                    obj.insert("password_hash".to_string(), serde_json::Value::String(hash));
                    obj.remove("password");
                    if let Ok(serialized) = serde_json::to_string_pretty(&upgraded) {
                        let _ = std::fs::write(&candidate, serialized);
                    }
                }
            }
            if let Some(user_id) = json.get("user_id").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                return Some(user_id.to_string());
            }
        }
    }
    None
}

