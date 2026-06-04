use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

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

pub fn get_turn_credentials(config: &TurnConfig) -> TurnCredentials {
    if let (Some(ref url), Some(ref secret)) = (&config.turn_url, &config.turn_secret) {
        let timestamp = Utc::now().timestamp();
        let expiry = timestamp + 86400;

        let username = format!("{expiry}");

        let mut mac = HmacSha1::new_from_slice(secret.as_bytes())
            .map_err(|e| log::error!("HMAC error: {e}"))
            .ok();

        let credential = if let Some(ref mut m) = mac {
            m.update(username.as_bytes());
            let result = m.clone().finalize();
            BASE64.encode(result.into_bytes())
        } else {
            return fallback_stun_servers();
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
    } else {
        fallback_stun_servers()
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

    #[test]
    fn test_fallback_stun() {
        let config = TurnConfig {
            turn_url: None,
            turn_secret: None,
            turn_realm: None,
        };
        let creds = get_turn_credentials(&config);
        assert_eq!(creds.ice_servers.len(), 2);
        assert!(creds.ice_servers[0].username.is_none());
    }

    #[test]
    fn test_turn_credentials_generation() {
        let config = TurnConfig {
            turn_url: Some("turn.example.com:3478".to_string()),
            turn_secret: Some("supersecret".to_string()),
            turn_realm: Some("example.com".to_string()),
        };
        let creds = get_turn_credentials(&config);
        assert_eq!(creds.ice_servers.len(), 1);
        assert!(creds.ice_servers[0].username.is_some());
        assert!(creds.ice_servers[0].credential.is_some());
        assert_eq!(creds.ice_servers[0].urls.len(), 3);
    }
}
