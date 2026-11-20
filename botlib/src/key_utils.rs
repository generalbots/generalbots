/// Cache/Redis key building utilities.
///
/// All Redis keys should include org prefix to avoid collisions between
/// organizations. Use `build_key()` everywhere a cache key is constructed.
///
/// When `org` is empty, joins parts with ":" (backward-compatible).
/// When `org` is set, prepends "org:" then joins parts.
///
/// # Examples
///
/// ```
/// use botlib::key_utils::build_key;
///
/// // Without org (same as format!("suggestions:{}:{}", bot_id, session_id))
/// assert_eq!(
///     build_key("", &["suggestions", "bot123", "session456"]),
///     "suggestions:bot123:session456"
/// );
///
/// // With org
/// assert_eq!(
///     build_key("org_acme", &["suggestions", "bot123", "session456"]),
///     "org_acme:suggestions:bot123:session456"
/// );
/// ```
pub fn build_key(org: &str, parts: &[&str]) -> String {
    if org.is_empty() {
        parts.join(":")
    } else {
        let mut key = org.to_string();
        for part in parts {
            key.push(':');
            key.push_str(part);
        }
        key
    }
}

/// #2547 — build a cache key that carries the owning organisation when it is
/// known. `None` yields the historical, unprefixed key so pre-existing
/// sessions (created before `UserSession.org_id`) keep working. Callers must
/// dual-write the legacy key when `org` is `Some` until every read path can
/// supply the org; see `tenant_isolation` in `bottest`.
pub fn scoped_key(org: Option<uuid::Uuid>, parts: &[&str]) -> String {
    build_key(&org.map_or_else(String::new, |id| id.to_string()), parts)
}

/// The historical key, always safe to write: session ids are UUID v4 and
/// therefore globally unique, so two orgs can never collide on them.
pub fn legacy_key(parts: &[&str]) -> String {
    build_key("", parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_key_with_org() {
        let key = build_key("org_acme", &["suggestions", "bot123", "session456"]);
        assert_eq!(key, "org_acme:suggestions:bot123:session456");
    }

    #[test]
    fn test_build_key_without_org() {
        let key = build_key("", &["suggestions", "bot123", "session456"]);
        assert_eq!(key, "suggestions:bot123:session456");
    }

    #[test]
    fn test_build_key_single_part() {
        let key = build_key("org_acme", &["start_bas_executed"]);
        assert_eq!(key, "org_acme:start_bas_executed");
    }

    #[test]
    fn test_scoped_key_is_org_unique_and_legacy_key_is_stable() {
        let bot = uuid::Uuid::from_u128(7);
        let session = uuid::Uuid::from_u128(8);
        let parts = ["suggestions", &bot.to_string(), &session.to_string()];
        let org_a = uuid::Uuid::from_u128(1);
        let org_b = uuid::Uuid::from_u128(2);
        assert_ne!(scoped_key(Some(org_a), &parts), scoped_key(Some(org_b), &parts));
        assert_eq!(scoped_key(None, &parts), legacy_key(&parts));
    }
}
