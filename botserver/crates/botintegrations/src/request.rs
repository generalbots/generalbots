//! Pure request parsing and credential-routing rules for the integration
//! connection control plane (#939).
//!
//! These helpers contain no database or Vault access so the secret-splitting
//! behavior stays independently auditable: anything that looks secret-ish by
//! key name is routed out of configuration and toward Vault.

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::models;

/// Authentication kinds accepted by the control plane in this slice.
pub const ALLOWED_AUTH_KINDS: [&str; 6] = [
    "api_key",
    "basic",
    "token",
    "access_key",
    "oauth2",
    "protocol",
];

/// Structural members consumed by the handler itself; every other member of
/// the incoming JSON body is treated as configuration.
const STRUCTURAL_KEYS: [&str; 6] = [
    "provider_slug",
    "display_name",
    "auth_kind",
    "configuration",
    "granted_scopes",
    "expires_at",
];

pub fn key_is_secretish(key: &str) -> bool {
    let lower = key.to_lowercase();
    ["key", "token", "secret", "password", "credential"]
        .iter()
        .any(|fragment| lower.contains(fragment))
}

pub fn parse_bot_id(bot_id: &str) -> Result<Uuid, IntegrationError> {
    Uuid::parse_str(bot_id)
        .map_err(|_| IntegrationError::Validation("bot_id must be a UUID".to_string()))
}

pub fn parse_connection_id(connection_id: &str) -> Result<Uuid, IntegrationError> {
    Uuid::parse_str(connection_id)
        .map_err(|_| IntegrationError::Validation("connection_id must be a UUID".to_string()))
}

fn required_text(
    body: &serde_json::Map<String, Value>,
    field: &str,
    max_len: usize,
) -> Result<String, IntegrationError> {
    let raw = body
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| IntegrationError::Validation(format!("{field} is required")))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(IntegrationError::Validation(format!(
            "{field} must not be empty"
        )));
    }
    if trimmed.len() > max_len {
        return Err(IntegrationError::Validation(format!(
            "{field} must be at most {max_len} characters"
        )));
    }
    Ok(trimmed.to_string())
}

fn redact_secret_values(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut cleaned = serde_json::Map::new();
            for (key, item) in map {
                if key_is_secretish(key) {
                    continue;
                }
                cleaned.insert(key.clone(), redact_secret_values(item));
            }
            Value::Object(cleaned)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_secret_values).collect()),
        other => other.clone(),
    }
}

fn parse_expires_at(raw: &str) -> Result<DateTime<Utc>, IntegrationError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|_| {
            IntegrationError::Validation("expires_at must be an RFC 3339 timestamp".to_string())
        })
}

/// Splits an incoming request body into validated structural fields, the
/// sanitized configuration object and the secret envelope destined for Vault.
///
/// Secret-ish top-level keys - including an explicit `secrets` object - are
/// routed to Vault and removed from the configuration; any nested value whose
/// key looks secret-ish inside the remaining configuration is dropped.
pub fn split_request(body: &Value) -> Result<models::NewConnection, IntegrationError> {
    let object = match body {
        Value::Object(map) => map,
        _ => {
            return Err(IntegrationError::Validation(
                "request body must be a JSON object".to_string(),
            ))
        }
    };

    let provider_slug = required_text(object, "provider_slug", 100)?;
    let display_name = required_text(object, "display_name", 255)?;
    let auth_kind = required_text(object, "auth_kind", 32)?;
    if !ALLOWED_AUTH_KINDS.contains(&auth_kind.as_str()) {
        return Err(IntegrationError::Validation(format!(
            "auth_kind must be one of {}",
            ALLOWED_AUTH_KINDS.join(", ")
        )));
    }

    let expires_at = match object.get("expires_at") {
        None | Some(Value::Null) => None,
        Some(Value::String(raw)) => Some(parse_expires_at(raw)?),
        Some(_) => {
            return Err(IntegrationError::Validation(
                "expires_at must be an RFC 3339 timestamp".to_string(),
            ))
        }
    };

    let granted_scopes = match object.get("granted_scopes") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str().map(str::to_string).ok_or_else(|| {
                    IntegrationError::Validation(
                        "granted_scopes entries must be strings".to_string(),
                    )
                })
            })
            .collect::<Result<Vec<String>, IntegrationError>>()?,
        Some(_) => {
            return Err(IntegrationError::Validation(
                "granted_scopes must be an array of strings".to_string(),
            ))
        }
    };

    let visibility = match object.get("visibility") {
        None | Some(Value::Null) => "private".to_string(),
        Some(Value::String(raw)) => {
            if raw != "private" && raw != "branch" {
                return Err(IntegrationError::Validation(
                    "visibility must be private or branch".to_string(),
                ));
            }
            raw.clone()
        }
        Some(_) => {
            return Err(IntegrationError::Validation(
                "visibility must be a string".to_string(),
            ))
        }
    };

    let mut secrets = serde_json::Map::new();
    let mut configuration = serde_json::Map::new();
    collect_members(object, &mut secrets, &mut configuration)?;
    if let Some(Value::Object(explicit)) = object.get("configuration") {
        for (key, value) in explicit {
            if !key_is_secretish(key) {
                configuration.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(models::NewConnection {
        provider_slug,
        display_name,
        auth_kind,
        secrets: Value::Object(secrets),
        configuration: redact_secret_values(&Value::Object(configuration)),
        granted_scopes,
        visibility,
        expires_at,
    })
}

fn collect_members(
    object: &serde_json::Map<String, Value>,
    secrets: &mut serde_json::Map<String, Value>,
    configuration: &mut serde_json::Map<String, Value>,
) -> Result<(), IntegrationError> {
    for (key, value) in object {
        if STRUCTURAL_KEYS.contains(&key.as_str()) {
            continue;
        }
        if key == "secrets" {
            match value {
                Value::Object(envelope) => {
                    for (secret_key, secret_value) in envelope {
                        secrets.insert(secret_key.clone(), secret_value.clone());
                    }
                }
                Value::Null => {}
                _ => {
                    return Err(IntegrationError::Validation(
                        "secrets must be a JSON object".to_string(),
                    ))
                }
            }
            continue;
        }
        if key_is_secretish(key) {
            secrets.insert(key.clone(), value.clone());
        } else {
            configuration.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}

/// Extracts only the secret-ish members of a rotation payload.
pub fn split_rotation_secrets(body: &Value) -> Result<Value, IntegrationError> {
    let object = match body {
        Value::Object(map) => map,
        _ => {
            return Err(IntegrationError::Validation(
                "request body must be a JSON object".to_string(),
            ))
        }
    };
    let mut secrets = serde_json::Map::new();
    for (key, value) in object {
        if key == "secrets" {
            if let Value::Object(envelope) = value {
                for (secret_key, secret_value) in envelope {
                    secrets.insert(secret_key.clone(), secret_value.clone());
                }
            }
            continue;
        }
        if key_is_secretish(key) {
            secrets.insert(key.clone(), value.clone());
        }
    }
    if secrets.is_empty() {
        return Err(IntegrationError::Validation(
            "rotation requires credential material in the request body".to_string(),
        ));
    }
    Ok(Value::Object(secrets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn secretish_keys_route_to_vault_envelope() {
        let parsed =
            split_request(&json!({ "provider_slug": "github", "display_name": "GitHub", "auth_kind": "api_key", "api_token": "abc" }))
                .expect("valid payload");
        assert_eq!(parsed.secrets, json!({ "api_token": "abc" }));
        assert!(parsed.configuration.get("api_token").is_none());
    }

    #[test]
    fn explicit_secrets_object_routes_to_vault_envelope() {
        let parsed = split_request(&json!({
            "provider_slug": "aws",
            "display_name": "AWS",
            "auth_kind": "access_key",
            "secrets": { "access_key_id": "AKIA", "secret_access_key": "top" }
        }))
        .expect("valid payload");
        assert_eq!(
            parsed.secrets,
            json!({ "access_key_id": "AKIA", "secret_access_key": "top" })
        );
    }

    #[test]
    fn nested_configuration_secret_keys_are_redacted() {
        let parsed = split_request(&json!({
            "provider_slug": "aws",
            "display_name": "AWS",
            "auth_kind": "protocol",
            "configuration": { "region": "us-east-1", "nested": { "password": "x" } }
        }))
        .expect("valid payload");
        assert_eq!(
            parsed.configuration,
            json!({ "region": "us-east-1", "nested": {} })
        );
    }

    #[test]
    fn rejects_unknown_auth_kind_and_oversized_names() {
        let error = split_request(
            &json!({ "provider_slug": "p", "display_name": "d", "auth_kind": "magic" }),
        )
        .expect_err("unknown kind rejected");
        assert!(matches!(error, IntegrationError::Validation(_)));
        let long_name = "n".repeat(300);
        let error = split_request(
            &json!({ "provider_slug": "p", "display_name": long_name, "auth_kind": "basic" }),
        )
        .expect_err("oversized name rejected");
        assert!(matches!(error, IntegrationError::Validation(_)));
    }

    #[test]
    fn rotation_requires_credential_material() {
        let error = split_rotation_secrets(&json!({ "reason": "no creds here" }))
            .expect_err("rotation without secrets rejected");
        assert!(matches!(error, IntegrationError::Validation(_)));
        let ok = split_rotation_secrets(&json!({ "refresh_token": "t" })).expect("valid rotation");
        assert_eq!(ok, json!({ "refresh_token": "t" }));
    }
}
