use super::*;
use botlib::security::SafeCommand;

impl DriveConfig {
    /// Carrega credenciais exclusivamente do Vault.
    /// Se o Vault não estiver acessível ou o token for inválido, retorna `Err`.
    pub fn from_vault() -> Result<Self, String> {
        let vault_addr = std::env::var("VAULT_ADDR").map_err(|_| "VAULT_ADDR not set".to_string())?;
        let vault_token = botcoresecrets::vault_token()
            .ok_or_else(|| "VAULT_TOKEN not set".to_string())?;
        let ca_cert = match std::env::var("VAULT_CACERT") {
            Ok(cert) => cert,
            Err(_) => String::new(),
        };
        let url = format!("{}/v1/secret/data/gbo/drive", vault_addr);

        let mut curl_args = vec!["-sf"];
        let skip_verify = std::env::var("VAULT_SKIP_VERIFY").map(|v| v == "true" || v == "1").unwrap_or(false);
        if skip_verify || ca_cert.is_empty() || !std::path::Path::new(&ca_cert).exists() {
            curl_args.push("-k");
        } else {
            curl_args.push("--cacert");
            curl_args.push(&ca_cert);
        }
        let auth_header = format!("X-Vault-Token: {}", vault_token);
        curl_args.push("-H");
        curl_args.push(&auth_header);
        curl_args.push(&url);
        // Vault may still be bootstrapping when the process starts (fresh
        // install or container restart); a single curl attempt then returns
        // empty stdout and the stack boots without Drive. Retry with backoff
        // for a few seconds before giving up (#1200).
        let mut last_err = "no attempt made".to_string();
        let mut curl = SafeCommand::new("curl")
            .map_err(|e| format!("Failed to build curl command for Vault: {}", e))?;
        for arg in &curl_args {
            curl = curl
                .trusted_arg(*arg)
                .map_err(|e| format!("Failed to build curl command for Vault: {}", e))?;
        }
        for attempt in 0..10 {
            let output = curl
                .execute()
                .map_err(|e| format!("Failed to execute curl to Vault: {}", e))?;
            match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                Ok(data) => {
                    let secret_data = data
                        .get("data")
                        .and_then(|d| d.get("data"))
                        .ok_or_else(|| "Vault response missing 'data.data' path".to_string())?;
                    let host = secret_data.get("host").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'host' not set".to_string())?;
                    let port = secret_data.get("port").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'port' not set".to_string())?;
                    let accesskey = secret_data.get("accesskey").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'accesskey' not set".to_string())?;
                    let secret = secret_data.get("secret").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'secret' not set".to_string())?;
                    let bucket = secret_data.get("bucket").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'bucket' not set".to_string())?;
                    let server = format!("{}:{}", host, port);
                    // Optional backend selection (#1468). `endpoint` wins when set,
                    // so an operator can point at B2 or R2 by URL alone; `host`
                    // and `port` remain the OtterIO-shaped default.
                    let endpoint_override = secret_data.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
                    let endpoint = if endpoint_override.trim().is_empty() {
                        format!("http://{}:{}", host, port)
                    } else {
                        endpoint_override.trim_end_matches('/').to_string()
                    };
                    let backend = secret_data.get("backend").and_then(|v| v.as_str()).unwrap_or("otterio");
                    let region = secret_data.get("region").and_then(|v| v.as_str()).unwrap_or("");
                    let region = if region.trim().is_empty() { "auto".to_string() } else { region.to_string() };
                    let fallback_endpoint = secret_data.get("fallback_endpoint").and_then(|v| v.as_str()).unwrap_or("");
                    return Ok(Self {
                        endpoint,
                        bucket: bucket.to_string(),
                        region,
                        access_key: accesskey.to_string(),
                        secret_key: secret.to_string(),
                        server,
                        backend: backend.to_string(),
                        fallback_endpoint: fallback_endpoint.trim_end_matches('/').to_string(),
                    });
                }
                Err(e) => {
                    last_err = format!("Failed to parse Vault response: {}", e);
                    log::warn!("DriveConfig::from_vault() attempt {}: {last_err}", attempt + 1);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
        Err(format!("DriveConfig::from_vault() failed after retries: {last_err}"))
    }

    /// Carrega credenciais das variáveis de ambiente.
    /// NÃO faz fallback para credenciais hardcoded — se as env vars não existirem,
    /// retorna `Err`.
    pub fn from_env() -> Result<Self, String> {
        let access_key = std::env::var("OTTERIO_ACCESS_KEY")
            .map_err(|_| "OTTERIO_ACCESS_KEY not set".to_string())?;
        let secret_key = std::env::var("OTTERIO_SECRET_KEY")
            .map_err(|_| "OTTERIO_SECRET_KEY not set".to_string())?;
        let endpoint = std::env::var("OTTERIO_ENDPOINT")
            .map_err(|_| "OTTERIO_ENDPOINT not set".to_string())?;
        let bucket = std::env::var("OTTERIO_BUCKET")
            .map_err(|_| "OTTERIO_BUCKET not set".to_string())?;
        let server = std::env::var("OTTERIO_SERVER")
            .map_err(|_| "OTTERIO_SERVER not set".to_string())?;

        // OTTERIO_REGION / OTTERIO_BACKEND / OTTERIO_FALLBACK_ENDPOINT let an operator
        // select an external backend (#1468) without a Vault round trip; all are
        // optional so the OtterIO-only deployment is unchanged.
        let region = std::env::var("OTTERIO_REGION").unwrap_or_else(|_| "auto".to_string());
        let backend = std::env::var("OTTERIO_BACKEND").unwrap_or_else(|_| "otterio".to_string());
        let fallback_endpoint = match std::env::var("OTTERIO_FALLBACK_ENDPOINT") {
            Ok(endpoint) => endpoint,
            Err(_) => String::new(),
        };

        Ok(Self {
            endpoint,
            bucket,
            region,
            access_key,
            secret_key,
            server,
            backend,
            fallback_endpoint,
        })
    }
}
