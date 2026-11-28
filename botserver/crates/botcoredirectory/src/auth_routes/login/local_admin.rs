use super::super::*;
use sha2::{Digest, Sha256};

/// Desktop/single-user installs have no reachable identity provider API for
/// password checks (Zitadel v2 sessions is not exposed by the bundled build).
/// The first-run wizard writes `conf/directory/admin-credentials.json`; when
/// the local password matches that file, this signs the administrator in with
/// the same persisted suite session a directory login would mint. The email
/// alone is never enough — the password must match.
pub(super) fn local_admin_user_id(email: &str, password: &str) -> Option<String> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
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
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("botserver-stack/conf/directory/admin-credentials.json"));
        candidates.push(cwd.join("../botserver-stack/conf/directory/admin-credentials.json"));
    }
    for candidate in candidates {
        let Ok(content) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let file_email = json.get("email").and_then(|v| v.as_str()).unwrap_or("");
        if file_email.is_empty() || !file_email.eq_ignore_ascii_case(email) {
            continue;
        }
        // #2231 — new installs persist a salted SHA-256 digest, never the
        // password itself. Legacy files with a plaintext `password` keep
        // working until the desktop wizard scrubs them after the first login.
        let file_salt = json
            .get("password_salt")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let file_hash = json
            .get("password_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut legacy_plaintext_match = false;
        let matched = if !file_hash.is_empty() {
            super::super::local_admin::password_hash_matches(file_salt, file_hash, password)
        } else {
            let file_password = json.get("password").and_then(|v| v.as_str()).unwrap_or("");
            legacy_plaintext_match = !file_password.is_empty() && file_password == password;
            legacy_plaintext_match
        };
        if matched {
            // #2235 F10 — legacy installs stored the password in plaintext.
            // Upgrade the file to a salted digest on first successful use so
            // credentials stop living on disk in cleartext (best effort).
            if legacy_plaintext_match {
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
                        if let Err(e) = std::fs::write(&candidate, serialized) {
                            warn!(
                                "Local admin: could not scrub legacy plaintext password in {:?}: {e}",
                                candidate
                            );
                        } else {
                            info!(
                                "Local admin: upgraded legacy plaintext credentials to hashed form in {:?}",
                                candidate
                            );
                        }
                    }
                }
            }
            let user_id = json
                .get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if !user_id.is_empty() {
                return Some(user_id.to_string());
            }
        }
    }
    None
}

/// #1839 — desktop logins mint a session from `admin-credentials.json` without
/// going through directory provisioning. Persist the same identity into the
/// local `users` table as an administrator so Settings → Users can list it
/// immediately after the first-run wizard, without waiting for a restart.
pub(super) fn persist_local_admin(state: &Arc<AppState>, user_id: &str, email: &str) {
    use diesel::RunQueryDsl;

    let Ok(mut conn) = state.conn.get() else {
        warn!("Local admin: database unavailable; users row not reconciled");
        return;
    };
    let uuid = uuid::Uuid::parse_str(user_id.trim()).unwrap_or_else(|_| {
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("zitadel:{}", user_id.trim()).as_bytes(),
        )
    });
    let username = email
        .split('@')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("admin");

    let updated = diesel::sql_query(
        "UPDATE users SET is_admin = TRUE, is_active = TRUE, updated_at = NOW() \
         WHERE id = $1 OR lower(email) = lower($2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid)
    .bind::<diesel::sql_types::Text, _>(email)
    .execute(&mut conn);

    match updated {
        Ok(count) if count > 0 => {
            info!(
                "Local admin: users row for {} reconciled with is_admin=true",
                email
            );
        }
        Ok(_) => {
            let inserted = diesel::sql_query(
                "INSERT INTO users \
                   (id, username, email, password_hash, is_active, is_admin, created_at, updated_at) \
                 VALUES ($1, $2, $3, '', TRUE, TRUE, NOW(), NOW()) \
                 ON CONFLICT DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(uuid)
            .bind::<diesel::sql_types::Text, _>(username)
            .bind::<diesel::sql_types::Text, _>(email)
            .execute(&mut conn);

            match inserted {
                Ok(1) => info!(
                    "Local admin: created users row for {} with is_admin=true",
                    email
                ),
                Ok(_) => warn!("Local admin: {} already has a conflicting users row", email),
                Err(e) => warn!("Local admin: could not create users row for {}: {e}", email),
            }
        }
        Err(e) => warn!(
            "Local admin: could not reconcile users row for {}: {e}",
            email
        ),
    }
}
