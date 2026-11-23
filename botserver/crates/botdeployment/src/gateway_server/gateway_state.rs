use super::*;

impl Default for GatewayState {
    fn default() -> Self {
        // #2613 — the legacy comma-separated static `DEPLOY_KEYS` is no longer
        // read. Only a per-deployment secret from Vault (`gbo/alm` ->
        // `deploy_key`) or the narrowly-scoped `GB_DEPLOY_KEY` is accepted.
        let vault_key = botcoresecrets::secret_sync(botcoresecrets::SecretPaths::ALM)
            .and_then(|secrets| secrets.get(DEPLOY_KEY_VAULT_FIELD).cloned());
        let env_key = std::env::var(DEPLOY_KEY_ENV).ok();
        let source = if vault_key.is_some() {
            "Vault"
        } else if env_key.is_some() {
            "environment"
        } else {
            "none"
        };

        let (deploy_keys, key_from_config) =
            match resolve_deploy_key(vault_key.as_deref(), env_key.as_deref()) {
                Some(key) => {
                    // Never log the key itself — only its origin.
                    log::info!("Deploy gateway key loaded from {source}");
                    (vec![key], true)
                }
                None => {
                    // Fail closed: hand the process an unguessable ephemeral
                    // key nobody outside it possesses, so every deploy request
                    // is 401 until a real secret is provisioned. The listener
                    // also stays on loopback (see gateway_bind_addr).
                    log::error!(
                        "Deploy gateway has no valid per-deployment secret (source={source}); \
                         using an ephemeral key — all deploy requests will be rejected"
                    );
                    (vec![botcorepkg::generate_random_string(48)], false)
                }
            };

        Self {
            deploy_keys,
            key_from_config,
            incus_socket: std::env::var("INCUS_SOCKET")
                .unwrap_or_else(|_| "/var/lib/incus/unix.socket".to_string()),
            caddy_api_url: {
                let (caddy, _vibe) = botcoresecrets::app_runtime();
                if caddy.is_empty() {
                    std::env::var("CADDY_API_URL")
                        .unwrap_or_else(|_| "http://127.0.0.1:2019".to_string())
                } else {
                    caddy
                }
            },
            sites_root: std::env::var("SITES_ROOT")
                .unwrap_or_else(|_| "/var/www".to_string()),
        }
    }
}
