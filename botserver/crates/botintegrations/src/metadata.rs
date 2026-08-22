/// Allowlist sanitizer for integration connection event metadata (#939).
///
/// Event metadata is audit data, never credential storage. Unknown keys are
/// stripped recursively, keys whose names look secret-ish are always dropped,
/// long strings are truncated, oversized arrays are discarded and recursion
/// depth is capped so hostile payloads cannot bloat the audit trail.

/// Maximum recursion depth accepted for event metadata payloads.
pub const MAX_METADATA_DEPTH: usize = 4;
/// Maximum length of any string kept inside event metadata.
pub const MAX_METADATA_STRING_LEN: usize = 512;
/// Arrays longer than this are dropped entirely from event metadata.
pub const MAX_METADATA_ARRAY_LEN: usize = 50;

/// Keys allowed inside event metadata. Everything else is stripped.
const ALLOWED_METADATA_KEYS: &[&str] = &[
    "provider",
    "auth_kind",
    "status",
    "test_status",
    "outcome_detail",
    "reason",
    "error_code",
    "credential_version",
    "expires_at",
];

fn key_is_sensitive(key: &str) -> bool {
    let lower = key.to_lowercase();
    ["token", "secret", "password", "credential", "apikey"]
        .iter()
        .any(|fragment| lower.contains(fragment))
}

/// Recursively filters a metadata value through the allowlist rules above.
pub fn sanitize_metadata(value: &serde_json::Value) -> serde_json::Value {
    fn walk(value: &serde_json::Value, depth: usize) -> serde_json::Value {
        if depth >= MAX_METADATA_DEPTH {
            return serde_json::Value::Null;
        }
        match value {
            serde_json::Value::Object(map) => {
                let mut cleaned = serde_json::Map::new();
                for (key, item) in map {
                    let allowed = ALLOWED_METADATA_KEYS.contains(&key.as_str());
                    if !allowed || key_is_sensitive(key) {
                        continue;
                    }
                    cleaned.insert(key.clone(), walk(item, depth + 1));
                }
                serde_json::Value::Object(cleaned)
            }
            serde_json::Value::Array(items) => {
                if items.len() > MAX_METADATA_ARRAY_LEN {
                    return serde_json::Value::Null;
                }
                serde_json::Value::Array(items.iter().map(|item| walk(item, depth + 1)).collect())
            }
            serde_json::Value::String(text) => {
                match text.char_indices().nth(MAX_METADATA_STRING_LEN) {
                    Some((cut, _)) => serde_json::Value::String(text[..cut].to_string()),
                    None => value.clone(),
                }
            }
            other => other.clone(),
        }
    }
    walk(value, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn drops_sensitive_keys_at_any_depth() {
        let input = json!({
            "provider": "github",
            "api_token": "should-vanish",
            "reason": {
                "outcome_detail": "rotation",
                "client_secret": "hidden"
            },
            "status": {
                "error_code": {
                    "access_password": "gone",
                    "auth_kind": "kept"
                }
            }
        });
        let output = sanitize_metadata(&input);
        let rendered = output.to_string();
        assert!(rendered.contains("github"));
        assert!(rendered.contains("rotation"));
        assert!(rendered.contains("kept"));
        assert!(!rendered.contains("should-vanish"));
        assert!(!rendered.contains("hidden"));
        assert!(!rendered.contains("gone"));
        assert!(!rendered.contains("token") && !rendered.contains("secret"));
        assert!(!rendered.contains("password"));
    }

    #[test]
    fn strips_keys_outside_the_allowlist_recursively() {
        let input = json!({
            "provider": "slack",
            "unknown_key": "dropped",
            "reason": {
                "also_unknown": [1, 2],
                "outcome_detail": "kept"
            }
        });
        let output = sanitize_metadata(&input);
        assert_eq!(
            output,
            json!({
                "provider": "slack",
                "reason": { "outcome_detail": "kept" }
            })
        );
    }

    #[test]
    fn caps_string_length_at_512_characters() {
        let long = "a".repeat(600);
        let output = sanitize_metadata(&json!({ "reason": long }));
        let reason = output.get("reason").and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(reason.chars().count(), 512);
    }

    #[test]
    fn drops_arrays_larger_than_fifty_items() {
        let big: Vec<i32> = (0..51).collect();
        let small: Vec<i32> = (0..50).collect();
        let dropped = sanitize_metadata(&json!({ "provider": big }));
        assert!(dropped
            .get("provider")
            .map(|v| v.is_null())
            .unwrap_or(false));
        let kept = sanitize_metadata(&json!({ "provider": small }));
        assert_eq!(
            kept.get("provider")
                .and_then(|v| v.as_array())
                .map(Vec::len),
            Some(50)
        );
    }

    #[test]
    fn enforces_depth_cap() {
        let deep =
            json!({ "provider": { "status": { "reason": { "outcome_detail": "too-deep" } } } });
        let output = sanitize_metadata(&deep);
        let rendered = output.to_string();
        assert!(!rendered.contains("too-deep"));
    }
}
