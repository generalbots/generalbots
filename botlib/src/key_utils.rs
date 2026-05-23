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
}
