//! Opt-in peer-network foundation (#1864).
//!
//! This is deliberately the smallest useful slice of the `botpublic` epic: a
//! stable node identity, an in-memory registration/heartbeat table and a status
//! view. Nothing is sent to another node, and nothing is exposed unless the
//! operator opts in with `GB_PEER_NETWORK=1`.
//!
//! Environment:
//!
//! | Variable | Meaning |
//! |---|---|
//! | `GB_PEER_NETWORK` | `1`/`true`/`yes`/`on` enables the node endpoints |
//! | `GB_PEER_NODE_ID` | Stable UUID; otherwise generated and persisted |
//! | `GB_PEER_NODE_NAME` | Display name; otherwise `general-bots-<id8>` |
//! | `GB_PEER_ADVERTISE_ADDR` | Optional address relayed to other peers |
//! | `GB_PEER_HEARTBEAT_TTL_SECS` | Peer expiry, default 90 seconds |
//! | `GB_PEER_DATA_DIR` | Directory for `node.json` |
//! | `GB_PEER_TOKEN` | Optional shared bearer token for machine callers |
//!
//! The registration table is intentionally in memory: a peer that stops
//! heartbeating expires, and it re-registers on its next boot. Node identity is
//! the only durable piece, written to `node.json` under the data/stack
//! directory so the same install keeps the same id across restarts.

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

        Self {
            enabled: env_truthy(ENV_ENABLED),
            node_id,
            node_name: env_non_empty(ENV_NODE_NAME),
            advertise_address: env_non_empty(ENV_ADVERTISE_ADDR),
            heartbeat_ttl_seconds,
            data_dir: default_data_dir(),
            token: env_non_empty(ENV_TOKEN),
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

/// Node identity plus the registration/heartbeat table.
pub struct PeerNetwork {
    config: PeerConfig,
    identity: Option<NodeIdentity>,
    peers: RwLock<HashMap<Uuid, PeerRecord>>,
}

impl PeerNetwork {
    pub fn new(config: PeerConfig) -> Self {
        let identity = if config.enabled {
            Some(load_or_create_identity(&config))
        } else {
            None
        };
        Self {
            config,
            identity,
            peers: RwLock::new(HashMap::new()),
        }
    }

    pub fn from_env() -> Self {
        Self::new(PeerConfig::from_env())
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn identity(&self) -> Option<&NodeIdentity> {
        self.identity.as_ref()
    }

    pub fn advertise_address(&self) -> Option<&str> {
        self.config.advertise_address.as_deref()
    }

    pub fn heartbeat_ttl_seconds(&self) -> u64 {
        self.config.heartbeat_ttl_seconds
    }

    pub fn token(&self) -> Option<&str> {
        self.config.token.as_deref()
    }

    /// Add or replace a peer registration. Heartbeats refresh `last_heartbeat`;
    /// `first_seen` is preserved across re-registration.
    pub fn register(&self, request: RegisterRequest) -> Result<PeerRecord, PeerError> {
        self.register_at(request, unix_now())
    }

    /// Refresh an existing registration.
    pub fn heartbeat(&self, request: HeartbeatRequest) -> Result<PeerRecord, PeerError> {
        self.heartbeat_at(request, unix_now())
    }

    /// Snapshot of this node and the peers that are still alive.
    pub fn status(&self) -> PeerStatus {
        self.status_at(unix_now())
    }

    fn register_at(&self, request: RegisterRequest, now: u64) -> Result<PeerRecord, PeerError> {
        if !self.enabled() {
            return Err(PeerError::Disabled);
        }
        if self.identity.as_ref().map(|identity| identity.node_id) == Some(request.node_id) {
            return Err(PeerError::Invalid(
                "a node cannot register itself as a peer".to_string(),
            ));
        }

        let name = normalize_name(&request.name)?;
        let address = normalize_address(request.address)?;
        let capabilities = normalize_capabilities(request.capabilities)?;

        let mut peers = self
            .peers
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_expired(&mut peers, now, self.heartbeat_ttl_seconds());

        if !peers.contains_key(&request.node_id) && peers.len() >= MAX_PEERS {
            return Err(PeerError::RegistryFull);
        }

        let first_seen = peers
            .get(&request.node_id)
            .map(|peer| peer.first_seen)
            .unwrap_or(now);
        let record = PeerRecord {
            node_id: request.node_id,
            name,
            address,
            capabilities,
            first_seen,
            last_heartbeat: now,
        };
        peers.insert(request.node_id, record.clone());
        Ok(record)
    }

    fn heartbeat_at(&self, request: HeartbeatRequest, now: u64) -> Result<PeerRecord, PeerError> {
        if !self.enabled() {
            return Err(PeerError::Disabled);
        }

        let mut peers = self
            .peers
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_expired(&mut peers, now, self.heartbeat_ttl_seconds());

        let record = peers
            .get_mut(&request.node_id)
            .ok_or(PeerError::UnknownNode(request.node_id))?;
        if let Some(address) = request.address {
            record.address = normalize_address(Some(address))?;
        }
        if let Some(capabilities) = request.capabilities {
            record.capabilities = normalize_capabilities(capabilities)?;
        }
        record.last_heartbeat = now;
        Ok(record.clone())
    }

    fn status_at(&self, now: u64) -> PeerStatus {
        if !self.enabled() {
            return PeerStatus {
                enabled: false,
                node: None,
                advertise_address: None,
                heartbeat_ttl_seconds: self.heartbeat_ttl_seconds(),
                peer_count: 0,
                peers: Vec::new(),
            };
        }

        let mut peers = self
            .peers
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_expired(&mut peers, now, self.heartbeat_ttl_seconds());
        let mut list: Vec<PeerRecord> = peers.values().cloned().collect();
        list.sort_by_key(|peer| peer.node_id);

        PeerStatus {
            enabled: true,
            node: self.identity.clone(),
            advertise_address: self.advertise_address().map(str::to_string),
            heartbeat_ttl_seconds: self.heartbeat_ttl_seconds(),
            peer_count: list.len(),
            peers: list,
        }
    }
}

static GLOBAL: OnceLock<PeerNetwork> = OnceLock::new();

/// Process-wide instance used by the HTTP handlers.
pub fn global() -> &'static PeerNetwork {
    GLOBAL.get_or_init(PeerNetwork::from_env)
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_truthy(key: &str) -> bool {
    env_non_empty(key)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn default_data_dir() -> Option<PathBuf> {
    env_non_empty(ENV_DATA_DIR)
        .map(PathBuf::from)
        .or_else(|| env_non_empty("DATA_DIR").map(|dir| PathBuf::from(dir).join("peer")))
        .or_else(|| {
            env_non_empty("BOTSERVER_STACK_PATH")
                .map(|dir| PathBuf::from(dir).join("conf").join("peer"))
        })
        .or_else(|| {
            env_non_empty("GBO_STACK_PATH").map(|dir| PathBuf::from(dir).join("conf").join("peer"))
        })
        .or_else(|| dirs::data_dir().map(|dir| dir.join("general-bots").join("peer")))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn load_or_create_identity(config: &PeerConfig) -> NodeIdentity {
    let now = unix_now();

    // An explicitly configured id wins and is never rewritten on disk.
    if let Some(node_id) = config.node_id {
        return NodeIdentity {
            node_id,
            name: config
                .node_name
                .clone()
                .unwrap_or_else(|| default_node_name(node_id)),
            created_at: now,
        };
    }

    let path = identity_path(config);
    if let Some(path) = path.as_deref() {
        match std::fs::read_to_string(path) {
            Ok(raw) => match serde_json::from_str::<NodeIdentity>(&raw) {
                Ok(mut identity) => {
                    if let Some(name) = &config.node_name {
                        identity.name = name.clone();
                    }
                    return identity;
                }
                Err(error) => log::warn!(
                    "Ignoring malformed peer identity at {}: {error}",
                    path.display()
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => log::warn!(
                "Could not read peer identity at {}: {error}",
                path.display()
            ),
        }
    }

    let node_id = Uuid::new_v4();
    let identity = NodeIdentity {
        node_id,
        name: config
            .node_name
            .clone()
            .unwrap_or_else(|| default_node_name(node_id)),
        created_at: now,
    };
    if let Some(path) = path.as_deref() {
        persist_identity(path, &identity);
    }
    identity
}

fn identity_path(config: &PeerConfig) -> Option<PathBuf> {
    config.data_dir.as_ref().map(|dir| dir.join("node.json"))
}

fn persist_identity(path: &Path, identity: &NodeIdentity) {
    let Some(parent) = path.parent() else {
        return;
    };
    if let Err(error) = std::fs::create_dir_all(parent) {
        log::warn!(
            "Could not create peer identity directory {}: {error}",
            parent.display()
        );
        return;
    }
    let Ok(payload) = serde_json::to_string_pretty(identity) else {
        return;
    };
    let temporary = path.with_extension("json.tmp");
    if let Err(error) = std::fs::write(&temporary, payload) {
        log::warn!(
            "Could not write peer identity at {}: {error}",
            temporary.display()
        );
        return;
    }
    if let Err(error) = std::fs::rename(&temporary, path) {
        log::warn!(
            "Could not publish peer identity at {}: {error}",
            path.display()
        );
    }
}

fn default_node_name(node_id: Uuid) -> String {
    format!("general-bots-{}", &node_id.simple().to_string()[..8])
}

fn normalize_name(raw: &str) -> Result<String, PeerError> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(PeerError::Invalid("peer name is required".to_string()));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(PeerError::Invalid(format!(
            "peer name must be at most {MAX_NAME_LEN} characters"
        )));
    }
    if name.chars().any(char::is_control) {
        return Err(PeerError::Invalid(
            "peer name must not contain control characters".to_string(),
        ));
    }
    Ok(name.to_string())
}

fn normalize_address(raw: Option<String>) -> Result<Option<String>, PeerError> {
    let address = raw
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(address) = &address {
        if address.chars().count() > MAX_ADDRESS_LEN {
            return Err(PeerError::Invalid(format!(
                "peer address must be at most {MAX_ADDRESS_LEN} characters"
            )));
        }
        if address.chars().any(char::is_control) {
            return Err(PeerError::Invalid(
                "peer address must not contain control characters".to_string(),
            ));
        }
    }
    Ok(address)
}

fn normalize_capabilities(raw: Vec<String>) -> Result<Vec<String>, PeerError> {
    let mut capabilities: Vec<String> = raw
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    if capabilities.len() > MAX_CAPABILITIES {
        return Err(PeerError::Invalid(format!(
            "a peer may advertise at most {MAX_CAPABILITIES} capabilities"
        )));
    }
    if capabilities.iter().any(|value| {
        value.chars().count() > MAX_CAPABILITY_LEN || value.chars().any(char::is_control)
    }) {
        return Err(PeerError::Invalid(format!(
            "capabilities must be at most {MAX_CAPABILITY_LEN} characters and contain no control characters"
        )));
    }
    capabilities.sort();
    capabilities.dedup();
    Ok(capabilities)
}

fn prune_expired(peers: &mut HashMap<Uuid, PeerRecord>, now: u64, ttl_seconds: u64) {
    peers.retain(|_, peer| now.saturating_sub(peer.last_heartbeat) <= ttl_seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_config(dir: &Path, ttl_seconds: u64) -> PeerConfig {
        PeerConfig {
            enabled: true,
            node_id: None,
            node_name: Some("test-node".to_string()),
            advertise_address: None,
            heartbeat_ttl_seconds: ttl_seconds,
            data_dir: Some(dir.to_path_buf()),
            token: None,
        }
    }

    fn registration(name: &str) -> RegisterRequest {
        RegisterRequest {
            node_id: Uuid::new_v4(),
            name: name.to_string(),
            address: Some("tcp://peer.example:44364".to_string()),
            capabilities: vec!["inference".to_string()],
        }
    }

    #[test]
    fn disabled_network_rejects_writes_and_hides_identity() {
        let network = PeerNetwork::new(PeerConfig::disabled());

        assert!(!network.enabled());
        assert!(network.identity().is_none());
        assert_eq!(
            network.register(registration("peer-a")),
            Err(PeerError::Disabled)
        );
        assert_eq!(
            network.heartbeat(HeartbeatRequest {
                node_id: Uuid::new_v4(),
                address: None,
                capabilities: None,
            }),
            Err(PeerError::Disabled)
        );

        let status = network.status();
        assert!(!status.enabled);
        assert!(status.node.is_none());
        assert_eq!(status.peer_count, 0);
        assert!(status.peers.is_empty());
    }

    #[test]
    fn registration_and_heartbeat_populate_the_table() {
        let dir = tempfile::tempdir().expect("tempdir");
        let network = PeerNetwork::new(enabled_config(dir.path(), 30));
        let request = registration("peer-a");
        let node_id = request.node_id;

        let first = network.register_at(request, 100).expect("register");
        assert_eq!(first.first_seen, 100);
        assert_eq!(first.last_heartbeat, 100);
        assert_eq!(network.status_at(100).peer_count, 1);

        let heartbeat = network
            .heartbeat_at(
                HeartbeatRequest {
                    node_id,
                    address: None,
                    capabilities: Some(vec!["chat".to_string(), "chat".to_string()]),
                },
                120,
            )
            .expect("heartbeat");
        assert_eq!(heartbeat.first_seen, 100);
        assert_eq!(heartbeat.last_heartbeat, 120);
        // Omitted fields are preserved.
        assert_eq!(
            heartbeat.address.as_deref(),
            Some("tcp://peer.example:44364")
        );
        assert_eq!(heartbeat.capabilities, vec!["chat".to_string()]);

        let status = network.status_at(120);
        assert_eq!(status.peer_count, 1);
        assert_eq!(status.peers[0].node_id, node_id);
    }

    #[test]
    fn heartbeat_for_unknown_node_is_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let network = PeerNetwork::new(enabled_config(dir.path(), 30));
        let unknown = Uuid::new_v4();

        assert_eq!(
            network.heartbeat_at(
                HeartbeatRequest {
                    node_id: unknown,
                    address: None,
                    capabilities: None,
                },
                100,
            ),
            Err(PeerError::UnknownNode(unknown))
        );
    }

    #[test]
    fn peers_expire_after_the_heartbeat_ttl() {
        let dir = tempfile::tempdir().expect("tempdir");
        let network = PeerNetwork::new(enabled_config(dir.path(), 10));

        network
            .register_at(registration("peer-a"), 100)
            .expect("register");
        assert_eq!(network.status_at(110).peer_count, 1);
        assert_eq!(network.status_at(111).peer_count, 0);
    }

    #[test]
    fn identity_is_persisted_across_restarts() {
        let dir = tempfile::tempdir().expect("tempdir");

        let first = PeerNetwork::new(enabled_config(dir.path(), 30));
        let first_identity = first.identity().cloned().expect("identity");
        assert_eq!(first_identity.name, "test-node");
        drop(first);

        let second = PeerNetwork::new(enabled_config(dir.path(), 30));
        let second_identity = second.identity().cloned().expect("identity");

        assert_eq!(second_identity.node_id, first_identity.node_id);
        assert_eq!(second_identity.created_at, first_identity.created_at);
        assert_eq!(second_identity.name, first_identity.name);
    }

    #[test]
    fn registration_validates_name_and_self_registration() {
        let dir = tempfile::tempdir().expect("tempdir");
        let network = PeerNetwork::new(enabled_config(dir.path(), 30));

        let mut unnamed = registration("");
        unnamed.name = "   ".to_string();
        assert!(matches!(
            network.register_at(unnamed, 100),
            Err(PeerError::Invalid(_))
        ));

        let mut itself = registration("itself");
        itself.node_id = network.identity().expect("identity").node_id;
        assert!(matches!(
            network.register_at(itself, 100),
            Err(PeerError::Invalid(_))
        ));
    }
}
