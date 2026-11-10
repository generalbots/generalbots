//! Office document ownership (#2137).
//!
//! Slides and Paper persist every object under `users/{owner}/...`, so the
//! owner id has to be the verified identity of the request. Both applications
//! resolve it here instead of inventing a fallback: a request whose identity
//! cannot be verified answers `401`, never a shared namespace.
//!
//! Desktop-local installs (`no_security=true` in the product file,
//! `require_auth = false`) have no directory account. They get one random id
//! generated once per install and persisted in the install's
//! `conf/system/directory_config.json` (the file that also holds the per-install
//! SaaS JWT secret), so documents stay reachable across restarts while two
//! installs never share a prefix. That branch only compiles with the
//! `desktop-local` feature.

use crate::security::auth::AuthenticatedUser;

/// Owner id for Slides and Paper storage.
///
/// Returns `None` when the request has no verified identity — the caller must
/// answer `401` rather than pick a default owner.
pub fn verified_owner_id(user: &AuthenticatedUser) -> Option<String> {
    if !user.user_id.is_nil() {
        return Some(user.user_id.to_string());
    }
    desktop_local_owner(user)
}

/// Display name for the same request, for logs only.
pub fn owner_display_name(user: &AuthenticatedUser) -> String {
    match user.email.as_deref() {
        Some(email) if !email.is_empty() => email.to_string(),
        _ => user.username.clone(),
    }
}

#[cfg(feature = "desktop-local")]
fn desktop_local_owner(user: &AuthenticatedUser) -> Option<String> {
    // Only the principal the auth middleware creates for a single-user
    // install carries this marker; a plain anonymous request never does.
    if !user.metadata.contains_key("desktop_local") {
        return None;
    }
    Some(office_install_owner())
}

#[cfg(not(feature = "desktop-local"))]
fn desktop_local_owner(_user: &AuthenticatedUser) -> Option<String> {
    None
}

/// Stable owner id of this desktop-local install.
///
/// Read from (or written to) `office_local_owner` in the install configuration,
/// cached for the process. A stored value that cannot own a namespace is
/// replaced by a fresh id, which is the safe direction: the install gets a new
/// namespace instead of sharing the legacy one.
#[cfg(feature = "desktop-local")]
pub fn office_install_owner() -> String {
    static OWNER: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    OWNER
        .get_or_init(|| {
            let stack_path = botcore::shared::utils::get_stack_path();
            let config_path = std::path::PathBuf::from(stack_path)
                .join("conf/system/directory_config.json");
            resolve_from_config(&config_path)
        })
        .clone()
}

#[cfg(feature = "desktop-local")]
const CONFIG_KEY: &str = "office_local_owner";

/// Whether `value` may be used as an office owner segment.
#[cfg(feature = "desktop-local")]
pub fn is_usable_owner(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "default-user" && trimmed != "anonymous@local"
}

#[cfg(feature = "desktop-local")]
fn resolve_from_config(config_path: &std::path::Path) -> String {
    if let Some(owner) = read_persisted(config_path) {
        return owner;
    }

    let owner = format!("local-{}", uuid::Uuid::new_v4());
    match persist(config_path, &owner) {
        Ok(()) => log::info!(
            "office: desktop-local owner {owner} stored in {}",
            config_path.display()
        ),
        Err(error) => log::warn!(
            "office: could not persist the desktop-local owner in {}: {error}",
            config_path.display()
        ),
    }
    owner
}

#[cfg(feature = "desktop-local")]
fn read_persisted(config_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&content).ok()?;
    let owner = config.get(CONFIG_KEY)?.as_str()?;
    if is_usable_owner(owner) {
        Some(owner.trim().to_string())
    } else {
        None
    }
}

#[cfg(feature = "desktop-local")]
fn persist(config_path: &std::path::Path, owner: &str) -> std::io::Result<()> {
    let mut config: serde_json::Value = std::fs::read_to_string(config_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let Some(object) = config.as_object_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "directory configuration is not a JSON object",
        ));
    };
    object.insert(
        CONFIG_KEY.to_string(),
        serde_json::Value::String(owner.to_string()),
    );
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(config_path, serde_json::to_vec_pretty(&config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anonymous_is_never_an_owner() {
        let anonymous = AuthenticatedUser::anonymous();
        assert!(verified_owner_id(&anonymous).is_none());
    }

    #[test]
    fn service_principal_is_never_an_owner() {
        let service = AuthenticatedUser::service("internal");
        assert!(verified_owner_id(&service).is_none());
    }

    #[test]
    fn verified_user_id_is_the_owner() {
        let id = uuid::Uuid::new_v4();
        let user = AuthenticatedUser::new(id, "alice".to_string());
        assert_eq!(verified_owner_id(&user), Some(id.to_string()));
    }

    #[cfg(feature = "desktop-local")]
    #[test]
    fn legacy_owners_are_not_usable() {
        assert!(!is_usable_owner(""));
        assert!(!is_usable_owner("default-user"));
        assert!(!is_usable_owner("anonymous@local"));
        assert!(is_usable_owner("local-8f14e45f-ceea-467f-a1a5-c4d1d2f4a9d6"));
    }

    #[cfg(feature = "desktop-local")]
    #[test]
    fn install_owner_is_persisted_and_reused() {
        let dir = std::env::temp_dir().join(format!(
            "office-owner-{}.json",
            uuid::Uuid::new_v4()
        ));
        let first = resolve_from_config(&dir);
        assert!(first.starts_with("local-"));
        assert_eq!(resolve_from_config(&dir), first);
        let _ = std::fs::remove_file(&dir);
    }
}
