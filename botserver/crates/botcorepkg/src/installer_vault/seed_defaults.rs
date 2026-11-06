//! Seed the default `secret/gbo/*` credentials after Vault init.

use crate::generate_random_string;
use botlib::security::SafeCommand;
use log::{info, warn};

pub fn seed_vault_defaults(
    vault_addr: &str,
    root_token: &str,
    ca_cert: &std::path::Path,
    vault_bin: &std::path::Path,
) -> Result<()> {
    info!("Seeding default credentials into Vault...");

    let drive_user = generate_random_string(16);
    let drive_pass = generate_random_string(32);
    let cache_pass = generate_random_string(32);
    let db_pass = generate_random_string(32);
    let master_key = generate_random_string(64);
    let meet_app_id = generate_random_string(24);
    let meet_app_secret = generate_random_string(48);
    let alm_token = generate_random_string(40);

    info!(
        "Generated strong random credentials for: drive, cache, tables, encryption, meet, alm"
    );

    let defaults: Vec<(&str, Vec<(String, String)>)> = vec![
        (
            "secret/gbo/drive",
            vec![
                ("accesskey".to_string(), drive_user),
                ("secret".to_string(), drive_pass),
                ("host".to_string(), "127.0.0.1".to_string()),
                ("port".to_string(), "9100".to_string()),
                ("url".to_string(), "".to_string()),
                ("bucket".to_string(), "default.gborg".to_string()),
                // Backend selection (#1468). MinIO stays the runtime default;
                // `endpoint` overrides host/port when pointing at B2 or R2.
                ("backend".to_string(), "minio".to_string()),
                ("region".to_string(), "auto".to_string()),
                ("fallback_endpoint".to_string(), "".to_string()),
            ],
        ),
        (
            "secret/gbo/cache",
            vec![
                ("password".to_string(), cache_pass),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "6379".to_string()),
                ("url".to_string(), "redis://localhost:6379".to_string()),
            ],
        ),
        (
            "secret/gbo/tables",
            vec![
                ("password".to_string(), db_pass),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "5432".to_string()),
                ("database".to_string(), "botserver".to_string()),
                ("username".to_string(), "gbuser".to_string()),
                ("url".to_string(), "postgres://localhost:5432".to_string()),
            ],
        ),
        (
            "secret/gbo/directory",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "8300".to_string()),
                ("project_id".to_string(), "none".to_string()),
                ("client_id".to_string(), "none".to_string()),
                ("client_secret".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/email",
            vec![
                ("smtp_host".to_string(), "none".to_string()),
                ("smtp_port".to_string(), "587".to_string()),
                ("smtp_user".to_string(), "none".to_string()),
                ("smtp_password".to_string(), "none".to_string()),
                ("smtp_from".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/llm",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "8081".to_string()),
                ("model".to_string(), "gpt-4".to_string()),
                ("openai_key".to_string(), "none".to_string()),
                ("anthropic_key".to_string(), "none".to_string()),
                ("ollama_url".to_string(), "".to_string()),
                ("embedding_url".to_string(), "http://localhost:8082/v1/embeddings".to_string()),
                ("embedding_model".to_string(), "bge-small-en-v1.5-f32.gguf".to_string()),
                ("embedding_port".to_string(), "8082".to_string()),
                ("embedding_dimensions".to_string(), "384".to_string()),
            ],
        ),
        (
            "secret/gbo/encryption",
            vec![("master_key".to_string(), master_key)],
        ),
        (
            "secret/gbo/meet",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "7880".to_string()),
                ("app_id".to_string(), meet_app_id),
                ("app_secret".to_string(), meet_app_secret),
            ],
        ),
        (
            "secret/gbo/vectordb",
            vec![
                ("url".to_string(), "http://127.0.0.1:6333".to_string()),
                ("host".to_string(), "127.0.0.1".to_string()),
                ("port".to_string(), "6333".to_string()),
                ("grpc_port".to_string(), "6334".to_string()),
                ("api_key".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/alm",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "3000".to_string()),
                ("token".to_string(), alm_token),
                ("default_org".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/vdi",
            vec![
                ("default-host".to_string(), "".to_string()),
                ("default-port".to_string(), "".to_string()),
                ("default-name".to_string(), "".to_string()),
            ],
        ),
    ];

    info!("Provisioned secret/gbo/vdi path (default VDI connection configured at deploy time)");

    for (path, kv_pairs) in &defaults {
        let mut args = vec![
            "kv".to_string(),
            "put".to_string(),
            "-tls-skip-verify".to_string(),
            format!("-address={}", vault_addr),
            path.to_string(),
        ];
        for (k, v) in kv_pairs.iter() {
            args.push(format!("{}={}", k, v));
        }

        let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
            .and_then(|c| {
                let mut cmd = c;
                for arg in &args {
                    cmd = cmd.trusted_arg(arg)?;
                }
                Ok(cmd)
            })
            .and_then(|c| {
                c.env("VAULT_ADDR", vault_addr)
                    .and_then(|c| c.env("VAULT_TOKEN", root_token))
                    .and_then(|c| c.env("VAULT_CACERT", ca_cert.to_str().unwrap_or("")))
            })
            .and_then(|c| c.execute());

        match result {
            Ok(output) => {
                if output.status.success() {
                    info!("Seeded Vault defaults at {}", path);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to seed {} in Vault: {}", path, stderr);
                }
            }
            Err(e) => {
                warn!("Failed to execute vault put for {}: {}", path, e);
            }
        }
    }

    info!("Vault defaults seeded successfully");
    Ok(())
}
