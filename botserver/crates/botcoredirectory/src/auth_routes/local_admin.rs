//! The local administrator record the first-run wizard writes
//! (`conf/directory/admin-credentials.json`): checking it at sign-in, writing a
//! new password into it, and minting the suite session it signs in with.

use super::*;

/// Single-person desktop edition: the wizard's administrator record is the
/// account of record, so sign-in checks it first and password changes write it.
pub fn is_desktop_edition() -> bool {
    botcore::product::is_shell_mode()
}

/// Where the administrator record may live, most specific first.
fn credential_candidates() -> Vec<std::path::PathBuf> {
    let relative = "conf/directory/admin-credentials.json";
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    for key in ["BOTSERVER_STACK_PATH", "GBO_STACK_PATH"] {
        if let Ok(stack) = std::env::var(key) {
            if !stack.trim().is_empty() {
                candidates.push(std::path::PathBuf::from(stack).join(relative));
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("botserver-stack").join(relative));
        candidates.push(cwd.join("../botserver-stack").join(relative));
    }
    candidates
}

/// SHA-256 of `salt || password`, the digest format the wizard writes.
pub(super) fn salted_digest(salt: &str, password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Whether `password` matches a stored administrator digest. The desktop
/// wizard writes an Argon2id PHC string (#2635); records written before it,
/// and by a password change here, hold the SHA-256 of `salt || password`.
pub(super) fn password_hash_matches(salt: &str, hash: &str, password: &str) -> bool {
    if hash.starts_with("$argon2") {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        return match PasswordHash::new(hash) {
            Ok(parsed) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
            Err(e) => {
                warn!("Local admin: the stored Argon2 digest is malformed: {e}");
                false
            }
        };
    }
    !salt.is_empty() && !hash.is_empty() && salted_digest(salt, password) == hash
}

/// The first administrator record on disk, optionally only the one for `email`.
pub(super) fn find_admin_record(
    email: Option<&str>,
) -> Option<(std::path::PathBuf, serde_json::Value)> {
    credential_candidates().into_iter().find_map(|path| {
        let content = std::fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let file_email = json.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let wanted = email.map_or(!file_email.is_empty(), |e| {
            !file_email.is_empty() && file_email.eq_ignore_ascii_case(e.trim())
        });
        wanted.then_some((path, json))
    })
}

/// A record that keeps `email` and `user_id` but holds a fresh salt and the
/// digest of `password`; a legacy plaintext field and the directory marker go.
pub(super) fn record_with_password(record: &serde_json::Value, password: &str) -> serde_json::Value {
    let salt = uuid::Uuid::new_v4().simple().to_string();
    let hash = salted_digest(&salt, password);
    let mut next = record.clone();
    if let Some(obj) = next.as_object_mut() {
        obj.insert("password_salt".to_string(), serde_json::Value::String(salt));
        obj.insert("password_hash".to_string(), serde_json::Value::String(hash));
        obj.remove("password");
        obj.remove(DIRECTORY_SYNC_MARKER);
    }
    next
}

/// Writes the record back, readable only by its owner where the OS allows it.
pub(super) fn write_admin_record(
    path: &std::path::Path,
    record: &serde_json::Value,
) -> std::io::Result<()> {
    let raw = serde_json::to_string_pretty(record).map_err(std::io::Error::other)?;
    std::fs::write(path, raw)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Mints a suite session for `user_id` and stores it like a directory login.
pub(super) async fn mint_local_session(user_id: &str, email: &str) -> String {
    let api_token = format!(
        "gb_{}_{}",
        uuid::Uuid::new_v4(),
        chrono::Utc::now().timestamp()
    );
    let name = email.split('@').next().filter(|v| !v.is_empty()).unwrap_or("admin");
    let session_user = SessionUserData {
        user_id: user_id.to_string(),
        email: email.to_string(),
        username: name.to_string(),
        first_name: None,
        last_name: None,
        display_name: Some(name.to_string()),
        organization_id: None,
        roles: vec!["admin".to_string()],
        bucket: None,
        created_at: chrono::Utc::now().timestamp(),
    };
    {
        let mut cache = SESSION_CACHE.write().await;
        cache.insert(api_token.clone(), session_user.clone());
    }
    persist_session(&api_token, &session_user);
    api_token
}

/// The desktop shell reuses `desktop-session.json` on its next start; after a
/// password change or reset the old token there is revoked, so keep it current.
pub(super) fn remember_desktop_session(token: &str) {
    if !is_desktop_edition() {
        return;
    }
    let path = std::path::PathBuf::from(get_stack_path()).join("conf/directory/desktop-session.json");
    if path.exists() {
        let raw = serde_json::json!({ "access_token": token }).to_string();
        if let Err(e) = std::fs::write(&path, raw) {
            warn!("Local admin: could not refresh the desktop session cache: {e}");
        }
    }
}

/// Signs the local administrator in when `password` matches the record, and
/// brings the directory administrator to the same password the first time.
pub(super) async fn local_admin_login(
    state: &Arc<AppState>,
    email: &str,
    password: &str,
) -> Option<LoginResponse> {
    let user_id = local_admin_user_id(email, password)?;
    persist_local_admin(state, &user_id, email);
    let api_token = mint_local_session(&user_id, email).await;
    if is_desktop_edition() {
        if let Some((path, record)) = find_admin_record(Some(email)) {
            if needs_directory_sync(&record) {
                let state = Arc::clone(state);
                let password = password.to_string();
                tokio::spawn(async move {
                    sync_directory_admin_password(state, password, path).await;
                });
            }
        }
    }
    botlog::safe_info!(
        "Local administrator login for: {} (user_id: {})",
        email,
        user_id
    );
    Some(LoginResponse {
        success: true,
        user_id: Some(user_id),
        session_id: None,
        access_token: Some(api_token),
        refresh_token: None,
        expires_in: Some(session_ttl_secs()),
        requires_2fa: false,
        session_token: None,
        redirect: Some("/".to_string()),
        message: Some("Login successful".to_string()),
    })
}
/// Desktop/single-user installs have no reachable identity provider API for
/// password checks (Zitadel v2 sessions is not exposed by the bundled build).
/// The first-run wizard writes `conf/directory/admin-credentials.json`; when
/// the local password matches that file, this signs the administrator in with
/// the same persisted suite session a directory login would mint. The email
/// alone is never enough — the password must match.
pub(super) fn local_admin_user_id(email: &str, password: &str) -> Option<String> {
    for candidate in credential_candidates() {
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
        let file_salt = json.get("password_salt").and_then(|v| v.as_str()).unwrap_or("");
        let file_hash = json.get("password_hash").and_then(|v| v.as_str()).unwrap_or("");
        let mut legacy_plaintext_match = false;
        let matched = if !file_hash.is_empty() {
            password_hash_matches(file_salt, file_hash, password)
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
                        if let Err(e) = std::fs::write(&candidate, serialized) {
                            warn!("Local admin: could not scrub legacy plaintext password in {:?}: {e}", candidate);
                        } else {
                            info!("Local admin: upgraded legacy plaintext credentials to hashed form in {:?}", candidate);
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

#[cfg(test)]
mod tests {
    use super::{password_hash_matches, salted_digest};

    fn argon2_phc(password: &str) -> String {
        use argon2::password_hash::{rand_core::OsRng, SaltString};
        use argon2::{Argon2, PasswordHasher};
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("hash the fixture password")
            .to_string()
    }

    /// The record the desktop wizard writes since #2635: the salt string next
    /// to an Argon2id PHC string that embeds the same salt.
    #[test]
    fn a_wizard_argon2_record_verifies_its_own_password_only() {
        let phc = argon2_phc("correct horse");
        assert!(password_hash_matches("any-salt-field", &phc, "correct horse"));
        assert!(password_hash_matches("", &phc, "correct horse"));
        assert!(!password_hash_matches("any-salt-field", &phc, "wrong"));
        assert!(!password_hash_matches("", "$argon2id$broken", "correct horse"));
    }

    #[test]
    fn a_legacy_sha256_record_still_verifies() {
        let hash = salted_digest("fixed-salt", "legacy-password");
        assert!(password_hash_matches("fixed-salt", &hash, "legacy-password"));
        assert!(!password_hash_matches("fixed-salt", &hash, "wrong"));
        assert!(!password_hash_matches("", &hash, "legacy-password"));
        assert!(!password_hash_matches("fixed-salt", "", "legacy-password"));
    }
}
