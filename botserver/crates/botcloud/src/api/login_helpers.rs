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

/// #2635 — hash a password as an Argon2id PHC string with a random salt.
fn argon2_phc(password: &str) -> Option<String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .ok()
        .map(|hash| hash.to_string())
}

/// #2635 — verify the desktop-local administrator credential. New records are
/// Argon2id PHC strings; a legacy `SHA256(salt || password)` record verifies
/// once and is flagged for a one-time rewrite. Returns `(authorized,
/// needs_migration)`; a plaintext `password` field is never consulted.
fn verify_admin_hash(password: &str, salt: &str, hash: &str) -> (bool, bool) {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    if hash.starts_with("$argon2") {
        let parsed = match PasswordHash::new(hash) {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("malformed Argon2 admin credential hash: {error}");
                return (false, false);
            }
        };
        return (
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
            false,
        );
    }
    if salt.is_empty() || hash.is_empty() {
        return (false, false);
    }
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    (format!("{:x}", hasher.finalize()) == hash, true)
}

/// Logs a non-fatal filesystem error instead of discarding its `Result` (#2543).
fn log_fs_error(context: &str, result: std::io::Result<()>) {
    if let Err(error) = result {
        tracing::error!("admin credential {context}: {error}");
    }
}

/// Replaces `path` with `temp` atomically where the OS allows it. Windows
/// cannot rename over an existing file, so it removes the target first.
fn replace_file(temp: &std::path::Path, path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    if std::fs::rename(temp, path).is_err() {
        std::fs::remove_file(path)?;
        std::fs::rename(temp, path)?;
    }
    #[cfg(not(windows))]
    std::fs::rename(temp, path)?;
    Ok(())
}

/// #2635 — atomically rewrite a legacy SHA-256 record as Argon2id so the next
/// login uses the strong hash. Operator fields are kept, `password_salt` is
/// dropped (the PHC embeds its salt) and the file stays owner-only on Unix.
fn migrate_admin_credential(path: &std::path::Path, json: &serde_json::Value, password: &str) {
    let Some(phc) = argon2_phc(password) else {
        tracing::error!("could not migrate the legacy desktop admin credential to Argon2id");
        return;
    };
    let mut updated = json.clone();
    if let Some(object) = updated.as_object_mut() {
        object.remove("password_salt");
        object.insert("password_hash".to_string(), serde_json::json!(phc));
    }
    let Ok(raw) = serde_json::to_string_pretty(&updated) else {
        return;
    };
    let (Some(parent), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str()))
    else {
        tracing::error!("could not resolve the admin credential path for migration");
        return;
    };
    let temp = parent.join(format!(".{name}.argon2-migrate.tmp"));
    if let Err(error) = std::fs::write(&temp, raw) {
        tracing::error!("could not write the migrated admin credential: {error}");
        log_fs_error("temp cleanup", std::fs::remove_file(&temp));
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&temp) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            log_fs_error("chmod", std::fs::set_permissions(&temp, permissions));
        }
    }
    if let Err(error) = replace_file(&temp, path) {
        tracing::error!("could not atomically replace the migrated admin credential: {error}");
        log_fs_error("temp cleanup", std::fs::remove_file(&temp));
        return;
    }
    tracing::warn!("SECURITY: admin credential migrated from SHA-256 to Argon2id");
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
    .unwrap_or_else(|_| Vec::new());
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
/// Returns `(authorized, needs_migration)`. The caller is authorized only when
/// the file's email matches (ignoring ASCII case) and the password verifies as
/// an Argon2id PHC string or, for legacy records, a salted SHA-256 digest. A
/// missing/empty hash, a plaintext `password` field, environment flags and
/// hardcoded credentials never count (#2231, #2635, #2759).
pub(crate) fn credentials_authorize(
    credentials: &serde_json::Value,
    email: &str,
    password: &str,
) -> (bool, bool) {
    let Some(cred_email) = credentials.get("email").and_then(|v| v.as_str()) else {
        return (false, false);
    };
    if !cred_email.eq_ignore_ascii_case(email) {
        return (false, false);
    }
    let Some(hash) = credentials.get("password_hash").and_then(|v| v.as_str()) else {
        return (false, false);
    };
    if hash.is_empty() {
        return (false, false);
    }
    let salt = credentials
        .get("password_salt")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    verify_admin_hash(password, salt, hash)
}

/// Resolves the Zitadel user_id for the bootstrap admin from
/// `admin-credentials.json` when the Zitadel sessions API is unavailable.
/// Authorization is decided by `credentials_authorize` alone: a case-insensitive
/// email match plus an Argon2id PHC (or legacy salted SHA-256) password check.
/// A legacy match is rewritten atomically to Argon2id with a loud warning. No
/// environment flag, hardcoded email/password or plaintext field is ever
/// consulted (#2231, #2635, #2759).
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
            let (authorized, needs_migration) = credentials_authorize(&json, email, password);
            if !authorized {
                continue;
            }
            if needs_migration {
                migrate_admin_credential(&candidate, &json, password);
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

    fn legacy(salt: &str, password: &str) -> serde_json::Value {
        json!({
            "email": "admin@example.com",
            "password_salt": salt,
            "password_hash": digest(salt, password),
            "user_id": "12345",
        })
    }

    #[test]
    fn argon2id_phc_verifies_without_migration() {
        let phc = argon2_phc("correct horse").expect("hash");
        assert!(phc.starts_with("$argon2id$"));
        assert_eq!(verify_admin_hash("correct horse", "", &phc), (true, false));
        assert_eq!(verify_admin_hash("wrong", "", &phc), (false, false));
        assert_eq!(
            verify_admin_hash("correct horse", "", "$argon2id$broken"),
            (false, false)
        );
        // The wizard still writes password_salt alongside the PHC; it must be
        // ignored in favour of the salt embedded in the PHC string.
        let credentials = json!({
            "email": "Admin@Example.com",
            "password_salt": "stale-salt",
            "password_hash": phc,
            "user_id": "12345",
        });
        assert_eq!(
            credentials_authorize(&credentials, "admin@example.com", "correct horse"),
            (true, false)
        );
        assert_eq!(
            credentials_authorize(&credentials, "admin@example.com", "wrong"),
            (false, false)
        );
    }

    #[test]
    fn legacy_sha256_verifies_once_and_flags_migration() {
        let credentials = legacy("wizard-salt", "pw");
        assert_eq!(
            credentials_authorize(&credentials, "ADMIN@EXAMPLE.COM", "pw"),
            (true, true)
        );
        assert_eq!(
            credentials_authorize(&credentials, "admin@example.com", "wrong"),
            (false, true)
        );
        assert_eq!(
            credentials_authorize(&credentials, "other@example.com", "pw"),
            (false, false)
        );
    }

    #[test]
    fn plaintext_password_field_is_never_accepted() {
        let credentials = json!({
            "email": "admin@example.com",
            "password": "dev",
            "user_id": "12345",
        });
        assert_eq!(
            credentials_authorize(&credentials, "admin@example.com", "dev"),
            (false, false)
        );
        // #2759 — no hardcoded admin@localhost/dev bootstrap survives.
        let dev_bootstrap = json!({
            "email": "admin@localhost",
            "password": "dev",
            "user_id": "12345",
        });
        assert_eq!(
            credentials_authorize(&dev_bootstrap, "admin@localhost", "dev"),
            (false, false)
        );
    }

    #[test]
    fn migration_rewrites_legacy_record_as_argon2id_phc() {
        let dir = std::env::temp_dir().join(format!("gb-admin-migrate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("admin-credentials.json");
        let credentials = legacy("wizard-salt", "legacy-password");
        let raw = serde_json::to_string_pretty(&credentials).expect("json");
        std::fs::write(&path, &raw).expect("write");
        migrate_admin_credential(&path, &credentials, "legacy-password");
        let migrated: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("parse");
        assert!(migrated
            .get("password_hash")
            .and_then(|v| v.as_str())
            .expect("hash")
            .starts_with("$argon2id$"));
        assert!(migrated.get("password_salt").is_none());
        assert_eq!(
            migrated.get("user_id").and_then(|v| v.as_str()),
            Some("12345")
        );
        assert!(credentials_authorize(&migrated, "admin@example.com", "legacy-password").0);
        assert!(!credentials_authorize(&migrated, "admin@example.com", "wrong").0);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
