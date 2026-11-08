#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn enabled_config(dir: &Path, ttl_seconds: u64) -> PeerConfig {
        PeerConfig {
            enabled: true,
            node_id: None,
            node_name: Some("test-node".to_string()),
            advertise_address: None,
            heartbeat_ttl_seconds: ttl_seconds,
            data_dir: Some(dir.to_path_buf()),
            token: None,
            signing_key: None,
            sources: BTreeMap::new(),
        }
    }

    #[test]
    fn environment_beats_vault_and_vault_beats_the_prod_file() {
        assert_eq!(
            pick(Some("env".into()), Some("vault".into()), Some("prod".into())),
            (Some("env".to_string()), ConfigSource::Env)
        );
        assert_eq!(
            pick(None, Some("vault".into()), Some("prod".into())),
            (Some("vault".to_string()), ConfigSource::Vault)
        );
        assert_eq!(
            pick(None, None, Some("prod".into())),
            (Some("prod".to_string()), ConfigSource::ProdFile)
        );
        assert_eq!(pick(None, None, None), (None, ConfigSource::Default));
    }

    #[test]
    fn flags_accept_the_documented_spellings() {
        for raw in ["1", "true", "YES", " on "] {
            assert!(parse_flag(raw), "{raw} should enable the node");
        }
        for raw in ["0", "false", "off", ""] {
            assert!(!parse_flag(raw), "{raw} should not enable the node");
        }
    }

    #[test]
    fn disabled_config_carries_no_secret_and_no_sources() {
        let config = PeerConfig::disabled();

        assert!(config.token.is_none());
        assert!(config.signing_key.is_none());
        assert!(config.sources.is_empty());
    }

    #[test]
    fn signing_key_falls_back_to_the_node_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let network = PeerNetwork::new(enabled_config(dir.path(), 30));
        let node_id = network.identity().expect("identity").node_id;

        assert_eq!(network.signing_key(), node_id.to_string());
    }

    #[test]
    fn configured_signing_key_wins_over_the_node_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = enabled_config(dir.path(), 30);
        config.signing_key = Some("from-vault".to_string());

        assert_eq!(PeerNetwork::new(config).signing_key(), "from-vault");
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
        let first_identity = first.identity().expect("identity");
        assert_eq!(first_identity.name, "test-node");
        drop(first);

        let second = PeerNetwork::new(enabled_config(dir.path(), 30));
        let second_identity = second.identity().expect("identity");

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
