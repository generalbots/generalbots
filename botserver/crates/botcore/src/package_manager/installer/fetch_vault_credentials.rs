use super::*;

impl PackageManager {
    pub(super) fn fetch_vault_credentials() -> HashMap<String, String> {
        let mut credentials = HashMap::new();

        if let Ok(stack) = std::env::var("BOTSERVER_STACK_PATH") {
        dotenvy::from_path(std::path::PathBuf::from(stack).join(".env")).ok();
    } else {
        dotenvy::dotenv().ok();
    }

        let base_path = std::env::var("BOTSERVER_STACK_PATH")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("botserver-stack")
            });

        // #2498 — the scoped sidecar token lives in the private credential
        // store; `.env` only holds a VAULT_TOKEN_FILE reference. Resolve it
        // from the store instead of reading a token value out of `.env`.
        if let Err(e) = botcorepkg::installer_vault::secret_store::load_token_into_env() {
            log::warn!("Could not load scoped Vault token into the environment: {e}");
        }
        let vault_addr = match std::env::var("VAULT_ADDR") {
            Ok(addr) => addr,
            Err(_) => String::new(),
        };
        let vault_token = match std::env::var("VAULT_TOKEN")
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| {
                std::env::var("VAULT_TOKEN_FILE")
                    .ok()
                    .filter(|p| !p.is_empty())
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            }) {
            Some(token) => token,
            // Not configured yet during first boot; the empty check below
            // reports this through the normal bootstrap-defaults path.
            None => String::new(),
        };

        if vault_token.is_empty() || vault_addr.is_empty() {
            info!("VAULT_ADDR or VAULT_TOKEN not set yet, using bootstrap defaults");
            return credentials;
        }

        let client_cert = base_path.join("conf/system/certificates/botserver/client.crt");
        let client_key = base_path.join("conf/system/certificates/botserver/client.key");
        let vault_check = SafeCommand::new("curl")
            .and_then(|c| {
                c.args(&[
                    "-sfk",
                    "--connect-timeout", "2", "--max-time", "5",
                    "--cert",
                    &client_cert.to_string_lossy(),
                    "--key",
                    &client_key.to_string_lossy(),
                    &format!("{}/v1/sys/health", vault_addr),
                ])
            })
            .and_then(|c| c.execute())
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !vault_check {
            trace!(
                "Vault not reachable at {}, skipping credential fetch",
                vault_addr
            );
            return credentials;
        }

        let vault_bin = botcorepkg::secrets_engine::cli_path(&base_path);
        let ca_cert_path = std::env::var("VAULT_CACERT").unwrap_or_else(|_| {
            base_path
                .join("conf/system/certificates/ca/ca.crt")
                .to_string_lossy()
                .to_string()
        });

        let services = [
            ("drive", "secret/gbo/drive"),
            ("cache", "secret/gbo/cache"),
            ("tables", "secret/gbo/tables"),
            ("vectordb", "secret/gbo/vectordb"),
            ("directory", "secret/gbo/directory"),
            ("llm", "secret/gbo/llm"),
            ("meet", "secret/gbo/meet"),
            ("alm", "secret/gbo/alm"),
            ("encryption", "secret/gbo/encryption"),
        ];

        // Every service start used to read the nine paths one after another
        // through the Vault CLI (about 5 s per call on Windows), which made
        // each start take ~45 s. The reads now run together, and services
        // starting at the same moment share one result.
        let cache_key = credentials_cache_key(&vault_addr, &vault_token);
        let mut cached = CREDENTIALS_CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = cached.as_ref() {
            if entry.key == cache_key && entry.fetched.elapsed() < CREDENTIALS_TTL {
                return entry.values.clone();
            }
        }

        let ca_cert = std::path::Path::new(&ca_cert_path)
            .is_file()
            .then_some(ca_cert_path.as_str());
        let vault_bin = vault_bin.to_string_lossy().to_string();
        std::thread::scope(|scope| {
            let reads: Vec<_> = services
                .iter()
                .map(|(service_name, vault_path)| {
                    let vault_bin = vault_bin.as_str();
                    let vault_addr = vault_addr.as_str();
                    let vault_token = vault_token.as_str();
                    scope.spawn(move || {
                        read_kv_json(vault_bin, vault_addr, vault_token, ca_cert, vault_path)
                            .map(|json| credentials_from_kv(service_name, &json))
                            .unwrap_or_default()
                    })
                })
                .collect();
            for read in reads {
                if let Ok(pairs) = read.join() {
                    credentials.extend(pairs);
                }
            }
        });

        trace!("Fetched {} credentials from Vault", credentials.len());
        if !credentials.is_empty() {
            *cached = Some(CachedCredentials {
                key: cache_key,
                fetched: std::time::Instant::now(),
                values: credentials.clone(),
            });
        }
        credentials
    }
}

/// How long one credential read is reused by the services that start after it.
const CREDENTIALS_TTL: std::time::Duration = std::time::Duration::from_secs(60);

struct CachedCredentials {
    key: u64,
    fetched: std::time::Instant,
    values: HashMap<String, String>,
}

static CREDENTIALS_CACHE: std::sync::Mutex<Option<CachedCredentials>> =
    std::sync::Mutex::new(None);

/// Identifies the Vault the cached credentials came from without keeping a
/// second copy of the token.
fn credentials_cache_key(vault_addr: &str, vault_token: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    vault_addr.hash(&mut hasher);
    vault_token.hash(&mut hasher);
    hasher.finish()
}

fn read_kv_json(
    vault_bin: &str,
    vault_addr: &str,
    vault_token: &str,
    ca_cert: Option<&str>,
    vault_path: &str,
) -> Option<serde_json::Value> {
    let output = SafeCommand::new(vault_bin)
        .and_then(|c| {
            let mut command = c.env("VAULT_ADDR", vault_addr)?.env("VAULT_TOKEN", vault_token)?;
            if let Some(ca) = ca_cert {
                command = command.env("VAULT_CACERT", ca)?;
            }
            Ok(command)
        })
        .and_then(|c| c.args(&["kv", "get", "-format=json", "-tls-skip-verify", vault_path]))
        .and_then(|c| c.execute())
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

/// `{"data":{"data":{"key":"v"}}}` from service `drive` becomes `DRIVE_KEY=v`.
fn credentials_from_kv(service_name: &str, json: &serde_json::Value) -> Vec<(String, String)> {
    let prefix = service_name.to_uppercase();
    json.get("data")
        .and_then(|d| d.get("data"))
        .and_then(|d| d.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(key, value)| {
                    value
                        .as_str()
                        .map(|v| (format!("{}_{}", prefix, key.to_uppercase()), v.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kv_json_becomes_prefixed_env_names() {
        let json = serde_json::json!({"data": {"data": {"accesskey": "a", "port": 9000, "secret": "s"}}});
        let mut pairs = credentials_from_kv("drive", &json);
        pairs.sort();
        assert_eq!(
            pairs,
            vec![
                ("DRIVE_ACCESSKEY".to_string(), "a".to_string()),
                ("DRIVE_SECRET".to_string(), "s".to_string()),
            ]
        );
    }

    #[test]
    fn kv_json_without_data_gives_nothing() {
        assert!(credentials_from_kv("cache", &serde_json::json!({"errors": []})).is_empty());
    }

    #[test]
    fn cache_key_changes_with_vault_or_token() {
        let base = credentials_cache_key("https://127.0.0.1:8200", "t1");
        assert_eq!(base, credentials_cache_key("https://127.0.0.1:8200", "t1"));
        assert_ne!(base, credentials_cache_key("https://127.0.0.1:8200", "t2"));
        assert_ne!(base, credentials_cache_key("https://127.0.0.1:8201", "t1"));
    }
}
