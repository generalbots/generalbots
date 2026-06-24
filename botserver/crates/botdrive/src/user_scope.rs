// Per-user file scoping for Drive — Issue #589
// Provides user-scoped and bot-scoped bucket path resolution.

use crate::drive_types::FileScope;

const USER_BUCKET_PREFIX: &str = "user-";
const GBAI_SUFFIX: &str = ".gbai";

/// Returns the bucket name for a user's personal files.
///
/// `get_user_bucket("abc-123")` → `"user-abc-123"`
pub fn get_user_bucket(user_id: &str) -> String {
    format!("{USER_BUCKET_PREFIX}{user_id}")
}

/// Returns the bucket name for a bot's shared files.
///
/// `get_bot_bucket("sales")` → `"sales.gbai"`
pub fn get_bot_bucket(bot_name: &str) -> String {
    format!("{bot_name}{GBAI_SUFFIX}")
}

/// Resolves which bucket to target given a scope and identifiers.
///
/// - `FileScope::User` with a `user_id` → user-scoped bucket
/// - `FileScope::Bot` with a `bot_name` → bot bucket
/// - Fallback to the provided `default_bucket`
pub fn resolve_bucket_name(
    scope: &FileScope,
    user_id: Option<&str>,
    bot_name: Option<&str>,
    default_bucket: &str,
) -> String {
    match scope {
        FileScope::User => user_id
            .map(get_user_bucket)
            .unwrap_or_else(|| default_bucket.to_string()),
        FileScope::Bot => bot_name
            .map(get_bot_bucket)
            .unwrap_or_else(|| default_bucket.to_string()),
    }
}

/// Returns the key prefix for scoping objects inside a bucket.
///
/// - `FileScope::User` → `"users/{user_id}/"`
/// - `FileScope::Bot` → empty (bot files sit at bucket root)
pub fn resolve_key_prefix(scope: &FileScope, user_id: &str) -> String {
    match scope {
        FileScope::User => format!("users/{user_id}/"),
        FileScope::Bot => String::new(),
    }
}

/// Builds the full object key by combining scope prefix and relative path.
pub fn build_object_key(scope: &FileScope, user_id: &str, path: &str) -> String {
    let prefix = resolve_key_prefix(scope, user_id);
    let clean = path.trim_matches('/');
    if clean.is_empty() {
        prefix
    } else if prefix.is_empty() {
        clean.to_string()
    } else {
        format!("{prefix}{clean}")
    }
}

/// Filters a list of object keys to only those belonging to the user scope.
pub fn filter_keys_for_user(keys: &[String], user_id: &str) -> Vec<String> {
    let prefix = format!("users/{user_id}/");
    keys.iter()
        .filter(|k| k.starts_with(&prefix))
        .cloned()
        .collect()
}

/// Strips the user scope prefix from a key, returning the relative path.
pub fn strip_user_prefix(key: &str, user_id: &str) -> String {
    let prefix = format!("users/{user_id}/");
    key.strip_prefix(&prefix).unwrap_or(key).to_string()
}

/// Detects whether a bucket name is a user-scoped bucket.
pub fn is_user_bucket(bucket: &str) -> bool {
    bucket.starts_with(USER_BUCKET_PREFIX)
}

/// Extracts the user id from a user-scoped bucket name, if any.
pub fn extract_user_from_bucket(bucket: &str) -> Option<&str> {
    bucket.strip_prefix(USER_BUCKET_PREFIX)
}

/// Detects whether a bucket name is a bot bucket (ends with .gbai).
pub fn is_bot_bucket(bucket: &str) -> bool {
    bucket.ends_with(GBAI_SUFFIX)
}

/// Extracts the bot name from a bot bucket, if any.
pub fn extract_bot_from_bucket(bucket: &str) -> Option<&str> {
    bucket.strip_suffix(GBAI_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_bucket_format() {
        assert_eq!(get_user_bucket("u1"), "user-u1");
        assert_eq!(get_user_bucket("550e8400-e29b"), "user-550e8400-e29b");
    }

    #[test]
    fn bot_bucket_format() {
        assert_eq!(get_bot_bucket("sales"), "sales.gbai");
    }

    #[test]
    fn resolve_bucket_name_user() {
        let r = resolve_bucket_name(&FileScope::User, Some("u1"), None, "fallback");
        assert_eq!(r, "user-u1");
    }

    #[test]
    fn resolve_bucket_name_bot() {
        let r = resolve_bucket_name(&FileScope::Bot, None, Some("sales"), "fallback");
        assert_eq!(r, "sales.gbai");
    }

    #[test]
    fn resolve_bucket_name_fallback() {
        let r = resolve_bucket_name(&FileScope::User, None, None, "fallback");
        assert_eq!(r, "fallback");
    }

    #[test]
    fn key_prefix_user() {
        assert_eq!(resolve_key_prefix(&FileScope::User, "u1"), "users/u1/");
    }

    #[test]
    fn key_prefix_bot_is_empty() {
        assert_eq!(resolve_key_prefix(&FileScope::Bot, "u1"), "");
    }

    #[test]
    fn build_object_key_user() {
        let k = build_object_key(&FileScope::User, "u1", "docs/report.pdf");
        assert_eq!(k, "users/u1/docs/report.pdf");
    }

    #[test]
    fn build_object_key_bot() {
        let k = build_object_key(&FileScope::Bot, "u1", "docs/report.pdf");
        assert_eq!(k, "docs/report.pdf");
    }

    #[test]
    fn filter_keys_for_user_works() {
        let keys = vec![
            "users/u1/a.txt".into(),
            "users/u2/b.txt".into(),
            "users/u1/c.txt".into(),
        ];
        let filtered = filter_keys_for_user(&keys, "u1");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn strip_user_prefix_works() {
        assert_eq!(strip_user_prefix("users/u1/docs/a.txt", "u1"), "docs/a.txt");
        assert_eq!(strip_user_prefix("other/path", "u1"), "other/path");
    }

    #[test]
    fn is_user_bucket_detects() {
        assert!(is_user_bucket("user-abc"));
        assert!(!is_bot_bucket("user-abc"));
    }

    #[test]
    fn extract_user_from_bucket_works() {
        assert_eq!(extract_user_from_bucket("user-abc"), Some("abc"));
        assert_eq!(extract_user_from_bucket("other"), None);
    }

    #[test]
    fn is_bot_bucket_detects() {
        assert!(is_bot_bucket("sales.gbai"));
        assert!(!is_bot_bucket("user-abc"));
    }

    #[test]
    fn extract_bot_from_bucket_works() {
        assert_eq!(extract_bot_from_bucket("sales.gbai"), Some("sales"));
        assert_eq!(extract_bot_from_bucket("other"), None);
    }
}
