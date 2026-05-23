/// Cache/Redis key building utilities.
///
/// All Redis keys should include org prefix to avoid collisions between
/// organizations. Use `build_key()` everywhere a cache key is constructed.
///
/// # Migration pattern (Issue #477)
///
/// Before:
///   format!("suggestions:{}:{}", bot_id, session_id)
///
/// After:
///   build_key(&org, &[&bot_id.to_string(), &session_id.to_string()])
///
/// Before:
///   format!("start_bas_executed:{}:{}", bot_uuid, session_id)
///
/// After:
///   build_key(&org, &["start_bas_executed", &bot_uuid.to_string(), &session_id.to_string()])
///
/// Before:
///   format!("context:{}:{}:{}", user_id, session_id, context_name)
///
/// After:
///   build_key(&org, &["context", &user_id.to_string(), &session_id.to_string(), context_name])
///
/// Before:
///   format!("hear:{}:{}", session_id, variable_name)
///
/// After:
///   build_key(&org, &["hear", &session_id.to_string(), variable_name])
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
