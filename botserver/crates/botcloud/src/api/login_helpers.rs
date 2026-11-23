use super::*;

pub(crate) fn base64_url_encode(input: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
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
    let email_row =
        diesel::sql_query("SELECT id FROM users WHERE email = $1 AND is_active = true LIMIT 1")
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
pub(crate) fn resolve_branch_from_user_binding(
    conn: &mut diesel::PgConnection,
    email: &str,
) -> Option<Uuid> {
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
        Err(_) => Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("zitadel:{user_id}").as_bytes(),
        ),
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
    if names
        .iter()
        .any(|g| botsecurity_auth::rbac_middleware::is_admin_group_name(&g.name))
    {
        vec!["admin".to_string()]
    } else {
        vec!["user".to_string()]
    }
}

/// Pure authorization predicate for a parsed `admin-credentials.json`.
/// The caller is authorized only when the file's email matches (ignoring
/// ASCII case) and the supplied password verifies against the salted SHA-256
/// digest persisted by the wizard. A missing/empty salt or digest, a plaintext
/// `password` field, environment flags and hardcoded credentials never count
/// (#2231, #2759).
pub(crate) fn credentials_authorize(
    credentials: &serde_json::Value,
    email: &str,
    password: &str,
) -> bool {
    let Some(cred_email) = credentials.get("email").and_then(|v| v.as_str()) else {
        return false;
    };
    if !cred_email.eq_ignore_ascii_case(email) {
        return false;
    }
    let Some(salt) = credentials.get("password_salt").and_then(|v| v.as_str()) else {
        return false;
    };
    let Some(expected) = credentials.get("password_hash").and_then(|v| v.as_str()) else {
        return false;
    };
    if salt.is_empty() || expected.is_empty() {
        return false;
    }
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize()) == expected
}

/// Resolves the Zitadel user_id for the bootstrap admin from
/// `admin-credentials.json` when the Zitadel sessions API is unavailable.
/// Authorization is decided by `credentials_authorize` alone: a case-insensitive
/// email match AND a salted SHA-256 digest of the supplied password. No
/// environment flag, hardcoded email/password or plaintext field is ever
/// consulted (#2231, #2759).
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
            if !credentials_authorize(&json, email, password) {
                continue;
            }
            if let Some(user_id) = json
                .get("user_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(user_id.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn digest(salt: &str, password: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn credentials_authorize_accepts_salted_digest() {
        let credentials = json!({
            "email": "admin@example.com",
            "password_salt": "wizard-salt",
            "password_hash": digest("wizard-salt", "correct horse"),
            "user_id": "12345",
        });
        assert!(credentials_authorize(
            &credentials,
            "admin@example.com",
            "correct horse"
        ));
    }

    #[test]
    fn credentials_authorize_email_compare_is_case_insensitive() {
        let credentials = json!({
            "email": "Admin@Example.com",
            "password_salt": "wizard-salt",
            "password_hash": digest("wizard-salt", "pw"),
        });
        assert!(credentials_authorize(
            &credentials,
            "ADMIN@EXAMPLE.COM",
            "pw"
        ));
    }

    #[test]
    fn credentials_authorize_rejects_wrong_password_or_email() {
        let credentials = json!({
            "email": "admin@example.com",
            "password_salt": "wizard-salt",
            "password_hash": digest("wizard-salt", "pw"),
        });
        assert!(!credentials_authorize(
            &credentials,
            "admin@example.com",
            "wrong"
        ));
        assert!(!credentials_authorize(
            &credentials,
            "other@example.com",
            "pw"
        ));
    }

    #[test]
    fn credentials_authorize_never_uses_plaintext_field() {
        let credentials = json!({
            "email": "admin@example.com",
            "password": "dev",
            "user_id": "12345",
        });
        assert!(!credentials_authorize(
            &credentials,
            "admin@example.com",
            "dev"
        ));
    }

    #[test]
    fn legacy_dev_bootstrap_is_rejected_without_a_digest() {
        // #2759 — no environment flag or hardcoded admin@localhost/dev pair
        // remains in the resolution path; a digest must be provisioned.
        let credentials = json!({
            "email": "admin@localhost",
            "password": "dev",
            "user_id": "12345",
        });
        assert!(!credentials_authorize(
            &credentials,
            "admin@localhost",
            "dev"
        ));
    }
}
