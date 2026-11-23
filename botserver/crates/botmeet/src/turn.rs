use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// TURN credentials are ephemeral; five minutes is enough for ICE gathering
/// and removes the old 24-hour window in which a leaked credential was valid.
pub const TURN_CREDENTIAL_TTL_SECS: i64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnCredentials {
    pub ice_servers: Vec<IceServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TurnConfig {
    pub turn_url: Option<String>,
    pub turn_secret: Option<String>,
    pub turn_realm: Option<String>,
}

impl TurnConfig {
    pub fn from_env() -> Self {
        Self {
            turn_url: std::env::var("MEET_TURN_URL").ok(),
            turn_secret: std::env::var("MEET_TURN_SECRET").ok(),
            turn_realm: std::env::var("MEET_TURN_REALM").ok(),
        }
    }
}

fn hmac_password(secret: &str, username: &str) -> Option<String> {
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes())
        .map_err(|e| log::error!("TURN HMAC error: {e}"))
        .ok()?;
    mac.update(username.as_bytes());
    Some(BASE64.encode(mac.finalize().into_bytes()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Issue credentials bound to `(room_id, user_id, expiry)`. The coturn REST
/// username is `expiry:room:user`; the password is the shared-secret HMAC over
/// that exact username. Callers MUST only invoke this after verifying that the
/// authenticated user is allowed into the room.
pub fn get_turn_credentials_for(
    config: &TurnConfig,
    room_id: &str,
    user_id: &str,
    ttl_secs: i64,
    now: i64,
) -> TurnCredentials {
    let (url, secret) = match (&config.turn_url, &config.turn_secret) {
        (Some(url), Some(secret)) if !url.is_empty() && !secret.is_empty() => (url, secret),
        _ => return fallback_stun_servers(),
    };
    if room_id.is_empty() || user_id.is_empty() {
        return fallback_stun_servers();
    }

    let expiry = now.saturating_add(ttl_secs);
    let username = format!("{expiry}:{room_id}:{user_id}");
    let credential = match hmac_password(secret, &username) {
        Some(value) => value,
        None => return fallback_stun_servers(),
    };

    let urls = if config.turn_realm.is_some() {
        vec![
            format!("turn:{url}?transport=udp"),
            format!("turn:{url}?transport=tcp"),
            format!("turns:{url}?transport=tcp"),
        ]
    } else {
        vec![
            format!("{url}?transport=udp"),
            format!("{url}?transport=tcp"),
        ]
    };

    TurnCredentials {
        ice_servers: vec![IceServer {
            urls,
            username: Some(username),
            credential: Some(credential),
        }],
    }
}

/// Validate a presented username/password pair at the TURN hook (or any other
/// credential check point). A credential minted for one room or user is
/// rejected for another, and an expired one is always rejected.
pub fn verify_turn_credential(
    config: &TurnConfig,
    username: &str,
    credential: &str,
    expected_room: &str,
    expected_user: &str,
    now: i64,
) -> bool {
    let secret = match &config.turn_secret {
        Some(value) if !value.is_empty() => value,
        _ => return false,
    };
    let mut parts = username.splitn(3, ':');
    let (expiry, room, user) = match (parts.next(), parts.next(), parts.next()) {
        (Some(expiry), Some(room), Some(user)) => (expiry, room, user),
        _ => return false,
    };
    let expiry = match expiry.parse::<i64>() {
        Ok(value) => value,
        Err(_) => return false,
    };
    if now >= expiry || room != expected_room || user != expected_user {
        return false;
    }
    match hmac_password(secret, username) {
        Some(expected) => constant_time_eq(expected.as_bytes(), credential.as_bytes()),
        None => false,
    }
}

fn fallback_stun_servers() -> TurnCredentials {
    TurnCredentials {
        ice_servers: vec![
            IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            },
            IceServer {
                urls: vec!["stun:stun1.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TurnConfig {
        TurnConfig {
            turn_url: Some("turn.example.com:3478".to_string()),
            turn_secret: Some("supersecret".to_string()),
            turn_realm: Some("example.com".to_string()),
        }
    }

    #[test]
    fn test_fallback_stun() {
        let config = TurnConfig {
            turn_url: None,
            turn_secret: None,
            turn_realm: None,
        };
        let creds = get_turn_credentials_for(&config, "room-a", "user-1", 300, 1000);
        assert_eq!(creds.ice_servers.len(), 2);
        assert!(creds.ice_servers[0].username.is_none());
    }

    #[test]
    fn test_turn_credentials_generation() {
        let creds = get_turn_credentials_for(&config(), "room-a", "user-1", 300, 1000);
        assert_eq!(creds.ice_servers.len(), 1);
        let server = &creds.ice_servers[0];
        assert_eq!(server.urls.len(), 3);
        assert_eq!(server.username.as_deref(), Some("1300:room-a:user-1"));
        assert!(server.credential.is_some());
    }

    #[test]
    fn credential_verifies_for_its_room_and_user() {
        let config = config();
        let creds = get_turn_credentials_for(&config, "room-a", "user-1", 300, 1000);
        let server = &creds.ice_servers[0];
        let username = server.username.as_deref().expect("username");
        let credential = server.credential.as_deref().expect("credential");
        assert!(verify_turn_credential(
            &config, username, credential, "room-a", "user-1", 1100
        ));
    }

    #[test]
    fn credential_cannot_be_reused_in_another_room_or_by_another_user() {
        let config = config();
        let creds = get_turn_credentials_for(&config, "room-a", "user-1", 300, 1000);
        let server = &creds.ice_servers[0];
        let username = server.username.as_deref().expect("username");
        let credential = server.credential.as_deref().expect("credential");
        assert!(!verify_turn_credential(
            &config, username, credential, "room-b", "user-1", 1100
        ));
        assert!(!verify_turn_credential(
            &config, username, credential, "room-a", "user-2", 1100
        ));
    }

    #[test]
    fn tampered_and_expired_credentials_are_rejected() {
        let config = config();
        let creds = get_turn_credentials_for(&config, "room-a", "user-1", 300, 1000);
        let server = &creds.ice_servers[0];
        let username = server.username.as_deref().expect("username");
        let credential = server.credential.as_deref().expect("credential");
        assert!(!verify_turn_credential(
            &config, username, "forged", "room-a", "user-1", 1100
        ));
        assert!(!verify_turn_credential(
            &config, username, credential, "room-a", "user-1", 1300
        ));
        assert!(!verify_turn_credential(
            &config,
            "not-a-turn-username",
            credential,
            "room-a",
            "user-1",
            1100
        ));
    }
}
