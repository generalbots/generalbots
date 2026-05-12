use std::collections::HashMap;

pub fn generate_connection_info(
    tenant: &str,
    component: &str,
    ip: &str,
    ports: &[u16],
) -> (String, HashMap<String, String>) {
    let env_vars = HashMap::new();
    let connection_info = match component {
        "vault" => {
            format!(
                r"Vault Server:
URL: http://{}:8200
UI: http://{}:8200/ui

✓ Vault initialized and unsealed automatically
✓ Created .env with VAULT_ADDR, VAULT_TOKEN
✓ Created vault-unseal-keys (chmod 600)

Files created:
.env - Vault connection config
vault-unseal-keys - Unseal keys for auto-unseal

On server restart, run:
botserver vault unseal

Or manually:
lxc exec {}-vault -- /opt/gbo/bin/vault operator unseal <key>

For other auto-unseal options (TPM, HSM, Transit), see:
https://generalbots.github.io/botbook/chapter-08/secrets-management.html",
                ip, ip, tenant
            )
        }
        "vector_db" => {
            format!(
                r"Qdrant Vector Database:
REST API: http://{}:6333
gRPC: {}:6334
Dashboard: http://{}:6333/dashboard

Store credentials in Vault:
botserver vault put gbo/vectordb host={} port=6333",
                ip, ip, ip, ip
            )
        }
        "tables" => {
            format!(
                r"PostgreSQL Database:
Host: {}
Port: 5432
Database: botserver
User: gbuser

Store credentials in Vault:
botserver vault put gbo/tables host={} port=5432 database=botserver username=gbuser password=<your-password>",
                ip, ip
            )
        }
        "drive" => {
            format!(
                r"MinIO Object Storage:
API: https://{}:9100
Console: https://{}:9101

Store credentials in Vault:
botserver vault put gbo/drive server={} port=9100 accesskey=minioadmin secret=<your-secret>",
                ip, ip, ip
            )
        }
        "cache" => {
            format!(
                r"Redis/Valkey Cache:
Host: {}
Port: 6379

Store credentials in Vault:
botserver vault put gbo/cache host={} port=6379 password=<your-password>",
                ip, ip
            )
        }
        "email" => {
            format!(
                r"Email Server (Stalwart):
SMTP: {}:25
IMAP: {}:143
Web: http://{}:9000

Store credentials in Vault:
botserver vault put gbo/email server={} port=25 username=admin password=<your-password>",
                ip, ip, ip, ip
            )
        }
        "directory" => {
            format!(
                r"Zitadel Identity Provider:
URL: http://{}:8300
Console: http://{}:8300/ui/console

Store credentials in Vault:
botserver vault put gbo/directory url=http://{}:8300 client_id=<client-id> client_secret=<client-secret>",
                ip, ip, ip
            )
        }
        "llm" => {
            format!(
                r"LLM Server (llama.cpp):
API: http://{}:8081

Test:
curl http://{}:8081/v1/models

Store credentials in Vault:
botserver vault put gbo/llm url=http://{}:8081 local=true",
                ip, ip, ip
            )
        }
        "meeting" => {
            format!(
                r"LiveKit Meeting Server:
WebSocket: ws://{}:7880
API: http://{}:7880

Store credentials in Vault:
botserver vault put gbo/meet url=ws://{}:7880 api_key=<api-key> api_secret=<api-secret>",
                ip, ip, ip
            )
        }
        "proxy" => {
            format!(
                r"Caddy Reverse Proxy:
HTTP: http://{}:80
HTTPS: https://{}:443
Admin: http://{}:2019",
                ip, ip, ip
            )
        }
        "timeseries_db" => {
            format!(
                r"InfluxDB Time Series Database:
API: http://{}:8086

Store credentials in Vault:
botserver vault put gbo/observability url=http://{}:8086 token=<influx-token> org=system bucket=metrics",
                ip, ip
            )
        }
        "observability" => {
            format!(
                r"Vector Log Aggregation:
API: http://{}:8686

Store credentials in Vault:
botserver vault put gbo/observability vector_url=http://{}:8686",
                ip, ip
            )
        }
        "alm" => {
            format!(
                r"Forgejo Git Server:
Web: http://{}:3000
SSH: {}:22

Store credentials in Vault:
botserver vault put gbo/alm url=http://{}:3000 token=<api-token>",
                ip, ip, ip
            )
        }
        _ => {
            let ports_str = ports
                .iter()
                .map(|p| format!(" - {}:{}", ip, p))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                r"Component: {}
Container: {}-{}
IP: {}
Ports:
{}",
                component, tenant, component, ip, ports_str
            )
        }
    };
    (connection_info, env_vars)
}
