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
//! | `GB_PEER_SIGNING_KEY` | Home-site signing key; falls back to the node id |
//!
//! #1968 — Vault is the canonical source for a deployed node:
//! `secret/gbo/peer` carries `enabled`, `node-id`, `node-name`,
//! `advertise-addr`, `data-dir`, `heartbeat-ttl-secs`, `token` and
//! `signing-key`. Precedence, highest first: process environment →
//! `secret/gbo/peer` → legacy `~/.prod` → built-in default. Only the origin of
//! each value is ever logged, never the value.
//!
//! The registration table is intentionally in memory: a peer that stops
//! heartbeating expires, and it re-registers on its next boot. Node identity is
//! the only durable piece, written to `node.json` under the data/stack
//! directory so the same install keeps the same id across restarts.

include!("peer_network_types.rs");

/// Node identity plus the registration/heartbeat table.
pub struct PeerNetwork {
    config: PeerConfig,
    identity: RwLock<Option<NodeIdentity>>,
    peers: RwLock<HashMap<Uuid, PeerRecord>>,
    /// Runtime override for the opt-in flag. `None` follows `config.enabled`.
    runtime_enabled: RwLock<Option<bool>>,
    mode_file: Option<PathBuf>,
}

impl PeerNetwork {
    pub fn new(config: PeerConfig) -> Self {
        let mode_file = config
            .data_dir
            .as_ref()
            .map(|dir| dir.join("mode.json"));
        let identity = if config.enabled {
            Some(load_or_create_identity(&config))
        } else {
            None
        };
        Self {
            config,
            identity: RwLock::new(identity),
            peers: RwLock::new(HashMap::new()),
            runtime_enabled: RwLock::new(None),
            mode_file,
        }
    }

    pub fn from_env() -> Self {
        Self::new(PeerConfig::from_env())
    }

    pub fn enabled(&self) -> bool {
        if let Some(value) = *self.runtime_enabled.read().unwrap_or_else(|p| p.into_inner()) {
            return value;
        }
        self.config.enabled
    }

    pub fn identity(&self) -> Option<NodeIdentity> {
        self.identity.read().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Turn the peer network on or off at runtime. When turning on, the node
    /// identity is created lazily if it does not exist yet.
    pub fn set_enabled(&self, value: bool) {
        *self.runtime_enabled.write().unwrap_or_else(|p| p.into_inner()) = Some(value);
        if value {
            let mut identity = self.identity.write().unwrap_or_else(|p| p.into_inner());
            if identity.is_none() {
                *identity = Some(load_or_create_identity(&self.config));
            }
        }
        self.persist_mode(value);
    }

    pub fn enable(&self) {
        self.set_enabled(true);
    }

    pub fn disable(&self) {
        self.set_enabled(false);
    }

    pub fn advertise_address(&self) -> Option<&str> {
        self.config.advertise_address.as_deref()
    }

    fn persist_mode(&self, value: bool) {
        if let Some(path) = &self.mode_file {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let payload = if value { "p2p" } else { "offline" };
            let _ = std::fs::write(path, payload);
        }
    }

    pub fn heartbeat_ttl_seconds(&self) -> u64 {
        self.config.heartbeat_ttl_seconds
    }

    pub fn token(&self) -> Option<&str> {
        self.config.token.as_deref()
    }

    /// Home-site signing key: `GB_PEER_SIGNING_KEY` → Vault
    /// `secret/gbo/peer` `signing-key` → the node id (#1968).
    pub fn signing_key(&self) -> String {
        self.config.signing_key.clone().unwrap_or_else(|| {
            self.identity()
                .map(|identity| identity.node_id.to_string())
                .unwrap_or_default()
        })
    }

    /// Which source supplied each setting. Never includes a value.
    pub fn config_sources(&self) -> BTreeMap<String, String> {
        self.config
            .sources
            .iter()
            .map(|(key, source)| (key.clone(), (*source).to_string()))
            .collect()
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
        if self.identity().map(|identity| identity.node_id) == Some(request.node_id) {
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
                config_sources: self.config_sources(),
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
            node: self.identity(),
            advertise_address: self.advertise_address().map(str::to_string),
            heartbeat_ttl_seconds: self.heartbeat_ttl_seconds(),
            peer_count: list.len(),
            peers: list,
            config_sources: self.config_sources(),
        }
    }
}

static GLOBAL: OnceLock<PeerNetwork> = OnceLock::new();

/// Process-wide instance used by the HTTP handlers.
pub fn global() -> &'static PeerNetwork {
    GLOBAL.get_or_init(PeerNetwork::from_env)
}

/// Resolve the peer configuration at boot (#1968), after the secrets manager is
/// ready, so Vault `secret/gbo/peer` is read before the first request and the
/// global instance is fixed once instead of on first traffic.
///
/// Logs the origin of each setting; values (token, signing key) are reported as
/// `configured` / `absent` only.
pub fn log_startup_configuration() {
    let network = global();
    let token_source = network
        .config_sources()
        .get("token")
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    log::info!(
        "peer network: enabled={} node={} advertise={} token={} (token source: {token_source})",
        network.enabled(),
        network
            .identity()
            .map(|identity| identity.name)
            .unwrap_or_else(|| "none".to_string()),
        network.advertise_address().unwrap_or("none"),
        if network.token().is_some() {
            "configured"
        } else {
            "absent"
        },
    );
    log::debug!("peer network config sources: {:?}", network.config_sources());
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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


include!("peer_network_tests.rs");
