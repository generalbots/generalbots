use anyhow::{Context, Result};
use tracing::info;

use crate::security::command_guard::SafeCommand;

const DNS_CONTAINER: &str = "pragmatismo-dns";
const COREFILE_PATH: &str = "/opt/gbo/conf/Corefile";

/// Corefile template with ACL (anti-amplification) + errors plugin.
/// Zones are passed in at runtime; the forward/catch-all block is always added.
const COREFILE_ZONE_TEMPLATE: &str = r#"{zone}:53 {{
    file /opt/gbo/data/{zone}.zone
    bind 0.0.0.0
    acl {{
        allow type ANY net 10.0.0.0/8 127.0.0.0/8
        allow type ANY net {server_ip}/32
        allow type A net 0.0.0.0/0
        allow type AAAA net 0.0.0.0/0
        allow type MX net 0.0.0.0/0
        allow type TXT net 0.0.0.0/0
        allow type NS net 0.0.0.0/0
        allow type SOA net 0.0.0.0/0
        allow type SRV net 0.0.0.0/0
        allow type CNAME net 0.0.0.0/0
        allow type HTTPS net 0.0.0.0/0
        block
    }}
    cache
    errors
}}

"#;

const COREFILE_FORWARD: &str = r#". {
    forward . 8.8.8.8 1.1.1.1
    cache
    errors
    log
}
"#;

pub struct DnsHardener;

impl DnsHardener {
    /// Patch the Corefile inside the DNS container:
    /// 1. Add ACL (anti-amplification) to each zone block if missing
    /// 2. Add errors plugin to all blocks
    /// 3. Reload CoreDNS (SIGHUP)
    pub async fn apply(zones: &[&str], server_ip: &str) -> Result<String> {
        let mut log = String::new();

        let original = Self::read_config().await?;

        // If already hardened, skip
        if original.contains("acl {") {
            log.push_str("Corefile already hardened\n");
            return Ok(log);
        }

        let mut patched = String::new();
        for zone in zones {
            patched.push_str(
                &COREFILE_ZONE_TEMPLATE
                    .replace("{zone}", zone)
                    .replace("{server_ip}", server_ip),
            );
        }
        patched.push_str(COREFILE_FORWARD);

        Self::write_config(&patched).await?;
        Self::reload_coredns(&mut log).await?;

        log.push_str("Corefile hardened (ACL + errors) and CoreDNS reloaded\n");
        info!("CoreDNS hardening applied");
        Ok(log)
    }

    pub async fn status() -> Result<String> {
        let out = Self::lxc_exec(&["coredns", "--version"]).await?;
        Ok(out)
    }

    async fn read_config() -> Result<String> {
        let out = Self::lxc_exec(&["cat", COREFILE_PATH]).await?;
        Ok(out)
    }

    async fn write_config(content: &str) -> Result<()> {
        let host_tmp = "/tmp/gb-corefile";
        std::fs::write(host_tmp, content).context("failed to write Corefile to /tmp")?;

        SafeCommand::new("lxc")?
            .arg("file")?
            .arg("push")?
            .arg(host_tmp)?
            .arg(&format!("{DNS_CONTAINER}{COREFILE_PATH}"))?
            .execute()
            .context("lxc file push Corefile failed")?;

        Ok(())
    }

    async fn reload_coredns(log: &mut String) -> Result<()> {
        Self::lxc_exec(&["pkill", "-HUP", "coredns"])
            .await
            .context("CoreDNS SIGHUP failed")?;
        log.push_str("CoreDNS reloaded\n");
        Ok(())
    }

    async fn lxc_exec(cmd: &[&str]) -> Result<String> {
        let mut c = SafeCommand::new("lxc")?
            .arg("exec")?
            .arg(DNS_CONTAINER)?
            .arg("--")?;
        for arg in cmd {
            c = c.arg(arg)?;
        }
        let out = c.execute().context("lxc exec failed")?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
