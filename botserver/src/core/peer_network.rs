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
//! | `GB_NETWORK_MODE` | desktop first-run answer: `offline` disables every peer surface, `p2p` starts the node on boot (#1865) |
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

/// #1865 — persists the master switch into the stack `.env` that botserver
/// loads on boot, so a choice made in Settings survives a restart. The write is
/// line-based: an existing `GB_NETWORK_MODE=` is replaced, otherwise one line
/// is appended. Callers log the error; a failed write must never break the
/// runtime toggle.
pub fn persist_network_mode(mode: &str) -> Result<(), String> {
    let stack = botcore::shared::utils::get_stack_path();
    let dir = PathBuf::from(&stack);
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {stack}: {e}"))?;
    let env_path = dir.join(".env");
    let new_line = format!("{ENV_NETWORK_MODE}={mode}");
    let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
    let updated = if existing
        .lines()
        .any(|line| line.starts_with(&format!("{ENV_NETWORK_MODE}=")))
    {
        let lines: Vec<&str> = existing
            .lines()
            .map(|line| {
                if line.starts_with(&format!("{ENV_NETWORK_MODE}=")) {
                    new_line.as_str()
                } else {
                    line
                }
            })
            .collect();
        lines.join("\n") + "\n"
    } else if existing.is_empty() {
        format!("{new_line}\n")
    } else if existing.ends_with('\n') {
        format!("{existing}{new_line}\n")
    } else {
        format!("{existing}\n{new_line}\n")
    };
    let fresh = !env_path.exists();
    std::fs::write(&env_path, updated)
        .map_err(|e| format!("write {}: {e}", env_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fresh {
            if let Ok(metadata) = std::fs::metadata(&env_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o600);
                let _ = std::fs::set_permissions(&env_path, permissions);
            }
        }
    }
    log::info!(
        "peer network mode persisted as {mode} in {}",
        env_path.display()
    );
    Ok(())
}

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

    /// Public relay the desktop client resolves peers and home sites through.
    /// Fixed on purpose (#1969): the client never discovers a relay.
    pub fn relay_endpoint(&self) -> &'static str {
        RELAY_ENDPOINT
    }

    /// What the desktop client needs to resolve peers and home sites (#1969).
    ///
    /// Published in `GET /api/product` so the suite has one authoritative
    /// endpoint. It carries no token, signing key or peer address.
    pub fn client_manifest(&self) -> serde_json::Value {
        serde_json::json!({
            "relay": self.relay_endpoint(),
            "relay_url": relay_url(),
            "enabled": self.enabled(),
            "node_id": self.identity().map(|identity| identity.node_id.to_string()),
        })
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

include!("peer_network_helpers.rs");

include!("peer_network_tests.rs");
