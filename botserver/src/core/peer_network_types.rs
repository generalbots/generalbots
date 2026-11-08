use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
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
pub const ENV_SIGNING_KEY: &str = "GB_PEER_SIGNING_KEY";

/// Canonical Vault path for peer-node settings (#1968). A deployed node is
/// configured from here; the `GB_PEER_*` variables above override it for dev,
/// test and one-off runs.
pub const SECRET_PATH: &str = "gbo/peer";

/// Where an effective setting came from. `GET /api/peer/status` reports this so
/// an operator can confirm a node reads Vault; the value itself is never
/// reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    Env,
    Vault,
    ProdFile,
    Default,
}

impl ConfigSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Vault => "vault",
            Self::ProdFile => "prod-file",
            Self::Default => "default",
        }
    }
}

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


/// Fixed relay endpoint for peer and home-site resolution (#1928, #1969).
///
/// The desktop client never discovers or guesses a relay: it always resolves
/// through this host, which is published in the `/api/product` manifest. A node
/// may still advertise a different address to its peers with `advertise-addr`
/// (env or Vault) — that is its own reachability, not the client's lookup.
pub const RELAY_ENDPOINT: &str = "peer.generalbots.org";

/// Scheme-qualified form of [`RELAY_ENDPOINT`], for clients that need to build
/// a URL without re-deciding the scheme.
pub fn relay_url() -> String {
    format!("https://{RELAY_ENDPOINT}")
}

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
    /// Origin of each effective setting (`env`, `vault`, `prod-file`,
    /// `mode-file`, `default`) — never the value itself (#1968).
    pub config_sources: BTreeMap<String, String>,
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
    /// Home-site signing key (`GB_PEER_SIGNING_KEY` / Vault `signing-key`).
    /// `None` falls back to the node id.
    pub signing_key: Option<String>,
    /// Where each effective value came from. Never carries a value.
    pub sources: BTreeMap<String, &'static str>,
}

impl PeerConfig {
    /// Resolve the effective configuration (#1968).
    ///
    /// Precedence, highest first:
    ///
    /// 1. process environment (`GB_PEER_*`) — dev, test, one-off runs
    /// 2. Vault `secret/gbo/peer` — canonical for a deployed node
    /// 3. the legacy `~/.prod` file (token only)
    /// 4. built-in defaults — and the persisted `mode.json` for the on/off flag
    pub fn load() -> Self {
        let vault = vault_settings();
        let mut sources: BTreeMap<String, &'static str> = BTreeMap::new();

        let (node_id_raw, node_id_source) = resolve(ENV_NODE_ID, "node-id", &vault);
        sources.insert("node-id".to_string(), node_id_source.as_str());
        let node_id = node_id_raw.and_then(|raw| match Uuid::parse_str(&raw) {
            Ok(id) => Some(id),
            Err(error) => {
                log::warn!(
                    "Ignoring invalid node-id from {ENV_NODE_ID} or {SECRET_PATH}: {error}"
                );
                None
            }
        });

        let (node_name, node_name_source) = resolve(ENV_NODE_NAME, "node-name", &vault);
        sources.insert("node-name".to_string(), node_name_source.as_str());

        let (advertise, advertise_source) = resolve(ENV_ADVERTISE_ADDR, "advertise-addr", &vault);
        let (advertise_address, advertise_source) = match advertise {
            Some(value) => (Some(value), advertise_source),
            None => (Some(RELAY_ENDPOINT.to_string()), ConfigSource::Default),
        };
        sources.insert("advertise-addr".to_string(), advertise_source.as_str());

        let (configured_dir, dir_source) = resolve(ENV_DATA_DIR, "data-dir", &vault);
        let (data_dir, dir_source) = match configured_dir {
            Some(value) => (Some(PathBuf::from(value)), dir_source),
            None => (default_data_dir(), ConfigSource::Default),
        };
        sources.insert("data-dir".to_string(), dir_source.as_str());

        let (ttl_raw, ttl_source) = resolve(ENV_HEARTBEAT_TTL, "heartbeat-ttl-secs", &vault);
        let heartbeat_ttl_seconds = ttl_raw
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_HEARTBEAT_TTL_SECS)
            .max(1);
        sources.insert("heartbeat-ttl-secs".to_string(), ttl_source.as_str());

        let (token, token_source) = resolve(ENV_TOKEN, "token", &vault);
        sources.insert("token".to_string(), token_source.as_str());

        let (signing_key, signing_key_source) = resolve(ENV_SIGNING_KEY, "signing-key", &vault);
        sources.insert("signing-key".to_string(), signing_key_source.as_str());

        let (enabled_raw, enabled_source) = resolve(ENV_ENABLED, "enabled", &vault);
        let enabled = match enabled_raw.as_deref() {
            Some(raw) => parse_flag(raw),
            None => persisted_enabled(data_dir.as_deref()),
        };
        let enabled_origin = if enabled_raw.is_some() {
            enabled_source.as_str()
        } else if data_dir.is_some() {
            "mode-file"
        } else {
            "default"
        };
        sources.insert("enabled".to_string(), enabled_origin);

        Self {
            enabled,
            node_id,
            node_name,
            advertise_address,
            heartbeat_ttl_seconds,
            data_dir,
            token,
            signing_key,
            sources,
        }
    }

    /// Environment-driven constructor kept for existing callers; it now
    /// resolves Vault first (#1968).
    pub fn from_env() -> Self {
        Self::load()
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
            signing_key: None,
            sources: BTreeMap::new(),
        }
    }
}

/// `secret/gbo/peer` as read now; empty when Vault is unavailable, which leaves
/// the env variables and defaults in charge.
fn vault_settings() -> BTreeMap<String, String> {
    botcoresecrets::peer_secret_sync().unwrap_or_default()
}

/// Env → Vault → `~/.prod` for one setting.
fn resolve(
    key_env: &str,
    vault_key: &str,
    vault: &BTreeMap<String, String>,
) -> (Option<String>, ConfigSource) {
    pick(
        env_non_empty(key_env),
        vault
            .get(vault_key)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        read_prod_value(key_env),
    )
}

/// The precedence rule itself, split out so it is testable without touching
/// process-wide environment variables.
fn pick(
    env: Option<String>,
    vault: Option<String>,
    prod_file: Option<String>,
) -> (Option<String>, ConfigSource) {
    match (env, vault, prod_file) {
        (Some(value), ..) => (Some(value), ConfigSource::Env),
        (None, Some(value), _) => (Some(value), ConfigSource::Vault),
        (None, None, Some(value)) => (Some(value), ConfigSource::ProdFile),
        (None, None, None) => (None, ConfigSource::Default),
    }
}

fn parse_flag(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// The `mode.json` written by the Settings toggle, used when neither the
/// environment nor Vault states a preference.
fn persisted_enabled(data_dir: Option<&Path>) -> bool {
    data_dir
        .map(|dir| std::fs::read_to_string(dir.join("mode.json")).unwrap_or_default())
        .map(|raw| raw.trim().eq_ignore_ascii_case("p2p"))
        .unwrap_or(false)
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
