use anyhow::{Context, Result};
use tracing::info;

use crate::security::command_guard::SafeCommand;

/// Security headers and rate_limit snippet to inject into every vhost
/// that uses `import tls_config` in the Caddyfile.
const RATE_LIMIT_SNIPPET: &str = r#"
(rate_limit_config) {
    rate_limit {
        zone dynamic {
            key    {remote_host}
            events 100
            window 1s
        }
    }
}
"#;

const SECURITY_HEADERS_SNIPPET: &str = r#"
(security_headers) {
    header {
        Content-Security-Policy "default-src 'self'; script-src 'self'; object-src 'none';"
        Strict-Transport-Security "max-age=63072000; includeSubDomains; preload"
        X-Frame-Options "DENY"
        X-Content-Type-Options "nosniff"
        Referrer-Policy "strict-origin-when-cross-origin"
        Permissions-Policy "geolocation=(), microphone=(), camera=()"
        -Server
    }
}
"#;

const CADDY_CONTAINER: &str = "pragmatismo-proxy";
const CADDY_CONFIG_PATH: &str = "/opt/gbo/conf/config";

pub struct CaddyHardener;

impl CaddyHardener {
    /// Patch the Caddyfile inside the proxy container:
    /// 1. Add security_headers and rate_limit snippets if missing
    /// 2. Import them in every vhost that uses tls_config
    /// 3. Reload Caddy
    pub async fn apply() -> Result<String> {
        let mut log = String::new();

        let original = Self::read_config().await?;
        let patched = Self::patch_config(&original);

        if patched == original {
            log.push_str("Caddyfile already up to date\n");
            return Ok(log);
        }

        Self::write_config(&patched).await?;
        Self::reload_caddy(&mut log).await?;

        log.push_str("Caddyfile patched and Caddy reloaded\n");
        info!("Caddy hardening applied");
        Ok(log)
    }

    pub async fn status() -> Result<String> {
        let out = Self::lxc_exec(&["caddy", "version"]).await?;
        Ok(out)
    }

    fn patch_config(original: &str) -> String {
        let mut result = original.to_string();

        // Add snippets after the global block (first `}`) if not already present
        if !result.contains("(security_headers)") {
            if let Some(pos) = result.find("\n\n") {
                result.insert_str(pos + 2, SECURITY_HEADERS_SNIPPET);
            }
        }

        if !result.contains("(rate_limit_config)") {
            if let Some(pos) = result.find("\n\n") {
                result.insert_str(pos + 2, RATE_LIMIT_SNIPPET);
            }
        }

        // Add `import security_headers` and `import rate_limit_config` inside
        // every vhost block that already has `import tls_config`
        let mut out = String::with_capacity(result.len() + 512);
        for line in result.lines() {
            out.push_str(line);
            out.push('\n');
            if line.trim() == "import tls_config" {
                if !result.contains("import security_headers") {
                    out.push_str("    import security_headers\n");
                }
                if !result.contains("import rate_limit_config") {
                    out.push_str("    import rate_limit_config\n");
                }
            }
        }
        out
    }

    async fn read_config() -> Result<String> {
        let out = Self::lxc_exec(&["cat", CADDY_CONFIG_PATH]).await?;
        Ok(out)
    }

    async fn write_config(content: &str) -> Result<()> {
        // Write to /tmp inside container then move to final path
        let tmp = "/tmp/gb-caddy-config";
        std::fs::write("/tmp/gb-caddy-host", content)
            .context("failed to write caddy config to host /tmp")?;

        // Push file into container
        SafeCommand::new("lxc")?
            .arg("file")?
            .arg("push")?
            .arg("/tmp/gb-caddy-host")?
            .arg(&format!("{CADDY_CONTAINER}{tmp}"))?
            .execute()
            .context("lxc file push failed")?;

        // Move to final location inside container
        Self::lxc_exec(&["mv", tmp, CADDY_CONFIG_PATH]).await?;
        Ok(())
    }

    async fn reload_caddy(log: &mut String) -> Result<()> {
        Self::lxc_exec(&[
            "caddy",
            "reload",
            "--config",
            CADDY_CONFIG_PATH,
            "--adapter",
            "caddyfile",
        ])
        .await
        .context("caddy reload failed")?;

        log.push_str("Caddy reloaded\n");
        Ok(())
    }

    async fn lxc_exec(cmd: &[&str]) -> Result<String> {
        let mut c = SafeCommand::new("lxc")?
            .arg("exec")?
            .arg(CADDY_CONTAINER)?
            .arg("--")?;

        for arg in cmd {
            c = c.arg(arg)?;
        }

        let out = c.execute().context("lxc exec failed")?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
