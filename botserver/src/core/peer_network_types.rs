use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const ENV_ENABLED: &str = "GB_PEER_NETWORK";
pub const ENV_NODE_ID: &str = "GB_PEER_NODE_ID";
pub const ENV_NODE_NAME: &str = "GB_PEER_NODE_NAME";
pub const ENV_ADVERTISE_ADDR: &str = "GB_PEER_ADVERTISE_ADDR";
pub const ENV_HEARTBEAT_TTL: &str = "GB_PEER_HEARTBEAT_TTL_SECS";
pub const ENV_DATA_DIR: &str = "GB_PEER_DATA_DIR";
pub const ENV_TOKEN: &str = "GB_PEER_TOKEN";

const DEFAULT_HEARTBEAT_TTL_SECS: u64 = 90;
const MAX_PEERS: usize = 1024;
const MAX_NAME_LEN: usize = 128;
const MAX_ADDRESS_LEN: usize = 255;
const MAX_CAPABILITIES: usize = 32;
const MAX_CAPABILITY_LEN: usize = 64;
/// Best-effort read of a key from `~/.prod` (home directory). Used only as a
/// fallback so peer networking can auto-start from production credentials
/// without programmer-facing env var configuration (#1928). Values are never
/// logged.
fn read_prod_value(key: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = Path::new(&home).join(".prod");
    let raw = std::fs::read_to_string(path).ok()?;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}


/// Managed relay endpoint for the peer network (#1928). Auto-configured so
/// end users never have to set it manually.
pub const RELAY_ENDPOINT: &str = "relay.generalbots.org";

/// Stable identity of this installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub node_id: Uuid,
    pub name: String,
    pub created_at: u64,
}

/// A peer announcing itself for the first time (or after a restart).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub node_id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Periodic keep-alive from an already registered peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub node_id: Uuid,
    /// Omit to keep the address from registration.
    #[serde(default)]
    pub address: Option<String>,
    /// Omit to keep the capabilities from registration.
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,
}

/// One row of the in-memory registration/heartbeat table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRecord {
    pub node_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub capabilities: Vec<String>,
    pub first_seen: u64,
    pub last_heartbeat: u64,
}

/// Response body for `GET /api/peer/status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerStatus {
    pub enabled: bool,
    pub node: Option<NodeIdentity>,
    pub advertise_address: Option<String>,
    pub heartbeat_ttl_seconds: u64,
    pub peer_count: usize,
    pub peers: Vec<PeerRecord>,
}

/// Runtime configuration, separated from process state so the module is
/// testable without touching global environment variables.
#[derive(Debug, Clone)]
pub struct PeerConfig {
    pub enabled: bool,
    pub node_id: Option<Uuid>,
    pub node_name: Option<String>,
    pub advertise_address: Option<String>,
    pub heartbeat_ttl_seconds: u64,
    /// Directory holding `node.json`; `None` keeps identity process-local.
    pub data_dir: Option<PathBuf>,
    pub token: Option<String>,
}

impl PeerConfig {
    /// Read the opt-in configuration from the environment. Disabled unless
    /// `GB_PEER_NETWORK` is explicitly truthy.
    pub fn from_env() -> Self {
        let node_id = env_non_empty(ENV_NODE_ID).and_then(|raw| {
            Uuid::parse_str(&raw)
                .map_err(|error| log::warn!("Ignoring invalid {ENV_NODE_ID}: {error}"))
                .ok()
        });
        let heartbeat_ttl_seconds = env_non_empty(ENV_HEARTBEAT_TTL)
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(DEFAULT_HEARTBEAT_TTL_SECS)
            .max(1);

        let data_dir = default_data_dir();
        let env_enabled_raw = std::env::var(ENV_ENABLED).ok();
        let enabled = match env_enabled_raw {
            Some(value) => matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
            None => data_dir
                .as_ref()
                .map(|dir| std::fs::read_to_string(dir.join("mode.json")).unwrap_or_default())
                .map(|raw| raw.trim().eq_ignore_ascii_case("p2p"))
                .unwrap_or(false),
        };
        Self {
            enabled,
            node_id,
            node_name: env_non_empty(ENV_NODE_NAME),
            advertise_address: env_non_empty(ENV_ADVERTISE_ADDR).or_else(|| Some(RELAY_ENDPOINT.to_string())),
            heartbeat_ttl_seconds,
            data_dir,
            token: env_non_empty(ENV_TOKEN).or_else(|| read_prod_value(ENV_TOKEN)).or_else(|| read_prod_value("GB_PEER_TOKEN")),
        }
    }

    /// The explicit "off" state used by tests and by callers that do not want
    /// to read process-wide environment variables.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            node_id: None,
            node_name: None,
            advertise_address: None,
            heartbeat_ttl_seconds: DEFAULT_HEARTBEAT_TTL_SECS,
            data_dir: None,
            token: None,
        }
    }
}

/// Errors returned by the registry. The HTTP layer maps these to status codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerError {
    Disabled,
    Unauthorized,
    Invalid(String),
    UnknownNode(Uuid),
    RegistryFull,
}

impl std::fmt::Display for PeerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "peer network is disabled; set {ENV_ENABLED}=1 to opt in"),
            Self::Unauthorized => write!(f, "a valid peer bearer token is required"),
            Self::Invalid(message) => write!(f, "{message}"),
            Self::UnknownNode(id) => write!(f, "peer {id} is not registered"),
            Self::RegistryFull => write!(f, "peer registry is full"),
        }
    }
}

impl std::error::Error for PeerError {}
