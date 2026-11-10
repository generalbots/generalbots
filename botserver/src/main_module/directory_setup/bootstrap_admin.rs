//! First-run (desktop wizard) administrator identity and reconciliation.

use botcore::shared::utils::get_stack_path;
use log::{info, warn};

/// The desktop first-run wizard writes this file before the suite opens. It is
/// the only credential for local/single-user installs, where the directory API
/// may not be reachable at all.
#[derive(Debug, Clone)]
pub(crate) struct BootstrapAdminIdentity {
    pub email: String,
    pub user_id: Option<uuid::Uuid>,
}

fn bootstrap_admin_credentials_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for var in ["BOTSERVER_STACK_PATH", "GBO_STACK_PATH"] {
        if let Ok(stack) = std::env::var(var) {
            if !stack.trim().is_empty() {
                paths.push(
                    std::path::PathBuf::from(stack).join("conf/directory/admin-credentials.json"),
                );
            }
        }
    }
    let stack = get_stack_path();
    paths.push(std::path::PathBuf::from(&stack).join("conf/directory/admin-credentials.json"));
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("botserver-stack/conf/directory/admin-credentials.json"));
        paths.push(cwd.join("../botserver-stack/conf/directory/admin-credentials.json"));
    }
    paths
}

/// Reads the wizard administrator identity from disk. Returns `None` for
/// Zitadel/cloud deployments, which have no `admin-credentials.json`.
pub(crate) fn bootstrap_admin_identity() -> Option<BootstrapAdminIdentity> {
    let mut seen = std::collections::HashSet::new();
    for path in bootstrap_admin_credentials_paths() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let email = json
            .get("email")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if email.is_empty() {
            continue;
        }
        let user_id = json
            .get("user_id")
            .and_then(|value| value.as_str())
            .and_then(|value| uuid::Uuid::parse_str(value.trim()).ok());
        return Some(BootstrapAdminIdentity {
            email: email.to_string(),
            user_id,
        });
    }
    None
}

/// Whether a session identity is the first-run administrator. Used when a
/// desktop/login path has to create a `users` row on demand.
pub(crate) fn is_bootstrap_admin_identity(user_id: uuid::Uuid, email: Option<&str>) -> bool {
    let Some(identity) = bootstrap_admin_identity() else {
        return false;
    };
    if identity.user_id == Some(user_id) {
        return true;
    }
    email
        .map(|value| identity.email.eq_ignore_ascii_case(value.trim()))
        .unwrap_or(false)
}

/// #1839 — persist the wizard administrator as an admin in the local `users`
/// table. Desktop logins do not go through directory provisioning, so without
/// this the account only appeared after an AI-account connect inserted a bare
/// row with `is_admin = false`.
///
/// No-op when `admin-credentials.json` is absent, so Zitadel/cloud installs
/// are untouched. Idempotent: it upgrades by id or email, otherwise inserts.
pub(crate) fn ensure_bootstrap_admin_user(pool: &botcore::shared::utils::DbPool) {
    use diesel::RunQueryDsl;

    let Some(identity) = bootstrap_admin_identity() else {
        return;
    };
    let Ok(mut conn) = pool.get() else {
        warn!("bootstrap admin: database unavailable; users row not reconciled");
        return;
    };

    let user_id = identity.user_id.unwrap_or_else(|| {
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("zitadel:{}", identity.email).as_bytes(),
        )
    });
    let username = identity
        .email
        .split('@')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("admin");

    let updated = diesel::sql_query(
        "UPDATE users SET is_admin = TRUE, is_active = TRUE, updated_at = NOW() \
         WHERE id = $1 OR lower(email) = lower($2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&identity.email)
    .execute(&mut conn);

    match updated {
        Ok(count) if count > 0 => {
            info!(
                "bootstrap admin: users row for {} reconciled with is_admin=true",
                identity.email
            );
        }
        Ok(_) => {
            let inserted = diesel::sql_query(
                "INSERT INTO users \
                   (id, username, email, password_hash, is_active, is_admin, created_at, updated_at) \
                 VALUES ($1, $2, $3, '', TRUE, TRUE, NOW(), NOW()) \
                 ON CONFLICT DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Text, _>(username)
            .bind::<diesel::sql_types::Text, _>(&identity.email)
            .execute(&mut conn);

            match inserted {
                Ok(1) => info!(
                    "bootstrap admin: created users row for {} with is_admin=true",
                    identity.email
                ),
                Ok(_) => warn!(
                    "bootstrap admin: {} already has a conflicting users row",
                    identity.email
                ),
                Err(e) => warn!(
                    "bootstrap admin: could not create users row for {}: {e}",
                    identity.email
                ),
            }
        }
        Err(e) => warn!(
            "bootstrap admin: could not reconcile users row for {}: {e}",
            identity.email
        ),
    }
}
