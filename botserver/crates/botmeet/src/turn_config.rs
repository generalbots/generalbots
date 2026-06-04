use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct TurnConfig {
    pub turn_url: Option<String>,
    pub turn_username: Option<String>,
    pub turn_credential: Option<String>,
    pub stun_url: Option<String>,
}

impl TurnConfig {
    pub fn from_env() -> Self {
        Self {
            turn_url: std::env::var("TURN_URL").ok(),
            turn_username: std::env::var("TURN_USERNAME").ok(),
            turn_credential: std::env::var("TURN_CREDENTIAL").ok(),
            stun_url: std::env::var("STUN_URL").ok(),
        }
    }
}

pub fn generate_ice_servers(config: &TurnConfig) -> Value {
    let mut servers: Vec<Value> = Vec::new();

    if let Some(ref turn_url) = config.turn_url {
        if let (Some(ref user), Some(ref cred)) = (&config.turn_username, &config.turn_credential) {
            servers.push(json!({
                "urls": [
                    format!("{turn_url}?transport=udp"),
                    format!("{turn_url}?transport=tcp"),
                ],
                "username": user,
                "credential": cred,
            }));
        }
    }

    let stun = config.stun_url.as_deref().unwrap_or("stun:stun.l.google.com:19302");
    servers.push(json!({
        "urls": [stun],
    }));

    json!({ "iceServers": servers })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_stun_only() {
        let config = TurnConfig::from_env();
        let value = generate_ice_servers(&config);
        let servers = value["iceServers"].as_array().unwrap();
        assert!(!servers.is_empty());
        assert!(servers.last().unwrap()["urls"][0].as_str().unwrap().contains("stun"));
    }

    #[test]
    fn test_full_turn_config() {
        let config = TurnConfig {
            turn_url: Some("turn.example.com:3478".into()),
            turn_username: Some("user".into()),
            turn_credential: Some("pass".into()),
            stun_url: Some("stun:custom.stun.server:3478".into()),
        };
        let value = generate_ice_servers(&config);
        let servers = value["iceServers"].as_array().unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers[0]["username"].as_str().is_some());
    }
}
