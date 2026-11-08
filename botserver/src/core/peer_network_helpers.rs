//  Identity, validation and expiry helpers for the peer network.
// 
//  Split out of `peer_network.rs` to keep that file inside the project's
//  size limit; `include!` keeps everything in the same module, so the
//  helpers use the imports declared there.

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
