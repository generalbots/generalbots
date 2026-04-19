use crate::core::package_manager::PackageManager;
use crate::security::command_guard::SafeCommand;
use anyhow::{Context, Result};
use log::info;
use std::path::Path;

/// NAT rule configuration for container port forwarding
#[derive(Debug, Clone)]
pub struct NatRule {
    pub port: u16,
    pub protocol: String,
}

impl NatRule {
    pub fn new(port: u16, protocol: &str) -> Self {
        Self {
            port,
            protocol: protocol.to_string(),
        }
    }
}

/// Container-specific settings for component deployment
#[derive(Debug, Clone)]
pub struct ContainerSettings {
    pub container_name: String,
    pub ip: String,
    pub user: String,
    pub group: Option<String>,
    pub working_dir: Option<String>,
    pub service_template: String,
    pub nat_rules: Vec<NatRule>,
    pub binary_path: String,
    pub config_path: String,
    pub data_path: Option<String>,
    pub exec_cmd_args: Vec<String>,
    pub internal_ports: Vec<u16>,
    pub external_port: Option<u16>,
}

impl ContainerSettings {
    pub fn new(
        container_name: &str,
        ip: &str,
        user: &str,
        binary_path: &str,
        config_path: &str,
    ) -> Self {
        Self {
            container_name: container_name.to_string(),
            ip: ip.to_string(),
            user: user.to_string(),
            group: None,
            working_dir: None,
            service_template: String::new(),
            nat_rules: Vec::new(),
            binary_path: binary_path.to_string(),
            config_path: config_path.to_string(),
            data_path: None,
            exec_cmd_args: Vec::new(),
            internal_ports: Vec::new(),
            external_port: None,
        }
    }

    pub fn with_group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }

    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    pub fn with_service_template(mut self, template: &str) -> Self {
        self.service_template = template.to_string();
        self
    }

    pub fn with_nat_rules(mut self, rules: Vec<NatRule>) -> Self {
        self.nat_rules = rules;
        self
    }

    pub fn with_data_path(mut self, path: &str) -> Self {
        self.data_path = Some(path.to_string());
        self
    }

    pub fn with_exec_args(mut self, args: Vec<String>) -> Self {
        self.exec_cmd_args = args;
        self
    }

    pub fn with_internal_ports(mut self, ports: Vec<u16>) -> Self {
        self.internal_ports = ports;
        self
    }

    pub fn with_external_port(mut self, port: u16) -> Self {
        self.external_port = Some(port);
        self
    }
}

/// Extension trait for PackageManager to handle container operations
pub trait ContainerOperations {
    /// Bootstrap a container with all its services and NAT rules
    fn bootstrap_container(
        &self,
        container_name: &str,
        source_lxd: Option<&str>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Cleanup existing socat and proxy devices
    fn cleanup_existing(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Copy container from LXD source
    fn copy_container(
        &self,
        source_remote: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Add eth0 network to container
    fn ensure_network(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Sync data from host to container
    fn sync_data_to_container(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Fix file permissions based on container user
    fn fix_permissions(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Install systemd service file and start
    fn install_systemd_service(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Configure iptables NAT rules on host
    fn configure_iptables_nat(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Start CoreDNS (special case)
    fn start_coredns(&self, container: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Reload DNS zones with new IPs
    fn reload_dns_zones(&self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get container settings for a component
    fn get_container_settings(&self, container: &str) -> Result<&ContainerSettings>;

    /// Install binary to container from URL
    fn install_binary_to_container(
        &self,
        container: &str,
        component: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Download and push binary to container
    fn download_and_push_binary(
        &self,
        container: &str,
        url: &str,
        binary_name: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl ContainerOperations for PackageManager {
    async fn bootstrap_container(
        &self,
        container_name: &str,
        source_lxd: Option<&str>,
    ) -> Result<()> {
        info!("Bootstrapping container: {container_name}");

        // 0. CLEANUP - Remove any existing socat or proxy devices
        self.cleanup_existing(container_name).await?;

        // 1. Copy from source LXD if migrating
        if let Some(source_remote) = source_lxd {
            self.copy_container(source_remote, container_name).await?;
        }

        // 2. Ensure network is configured
        self.ensure_network(container_name).await?;

        // 3. Sync data from host to container
        self.sync_data_to_container(container_name).await?;

        // 4. Fix permissions
        self.fix_permissions(container_name).await?;

        // 5. Install and start service
        self.install_systemd_service(container_name).await?;

        // 6. Configure NAT rules on host (ONLY iptables, never socat)
        self.configure_iptables_nat(container_name).await?;

        // 7. Reload DNS if dns container
        if container_name == "dns" {
            self.reload_dns_zones().await?;
        }

        info!("Container {container_name} bootstrapped successfully");
        Ok(())
    }

    async fn cleanup_existing(&self, container: &str) -> Result<()> {
        // Remove socat processes
        let _ = SafeCommand::new("pkill")
            .and_then(|c| c.arg("-9"))
            .and_then(|c| c.arg("-f"))
            .and_then(|c| c.arg("socat"))
            .and_then(|cmd| cmd.execute());

        // Remove proxy devices from container
        let output = SafeCommand::new("incus")
            .and_then(|c| c.arg("config"))
            .and_then(|c| c.arg("device"))
            .and_then(|c| c.arg("list"))
            .and_then(|c| c.arg(container))
            .and_then(|cmd| cmd.execute())?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("proxy") || line.contains("port") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    let _ = SafeCommand::new("incus")
                        .and_then(|c| c.arg("config"))
                        .and_then(|c| c.arg("device"))
                        .and_then(|c| c.arg("remove"))
                        .and_then(|c| c.arg(container))
                        .and_then(|c| c.arg(name))
                        .and_then(|cmd| cmd.execute());
                }
            }
        }

        Ok(())
    }

    async fn copy_container(&self, source_remote: &str, name: &str) -> Result<()> {
        info!("Copying container {name} from {source_remote}");

        let remote_path = format!("{source_remote}:{name}");
        SafeCommand::new("incus")
            .and_then(|c| c.arg("copy"))
            .and_then(|c| c.arg("--instance-only"))
            .and_then(|c| c.arg(remote_path.as_str()))
            .and_then(|c| c.arg(name))
            .and_then(|cmd| cmd.execute())
            .context("Failed to copy container")?;

        SafeCommand::new("incus")
            .and_then(|c| c.arg("start"))
            .and_then(|c| c.arg(name))
            .and_then(|cmd| cmd.execute())
            .context("Failed to start container")?;

        Ok(())
    }

    async fn ensure_network(&self, container: &str) -> Result<()> {
        let output = SafeCommand::new("incus")
            .and_then(|c| c.arg("config"))
            .and_then(|c| c.arg("device"))
            .and_then(|c| c.arg("list"))
            .and_then(|c| c.arg(container))
            .and_then(|cmd| cmd.execute())?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.contains("eth0") {
            SafeCommand::new("incus")
                .and_then(|c| c.arg("config"))
                .and_then(|c| c.arg("device"))
                .and_then(|c| c.arg("add"))
                .and_then(|c| c.arg(container))
                .and_then(|c| c.arg("eth0"))
                .and_then(|c| c.arg("nic"))
                .and_then(|c| c.arg("name=eth0"))
                .and_then(|c| c.arg("network=PROD-GBO"))
                .and_then(|cmd| cmd.execute())?;
        }
        Ok(())
    }

    async fn sync_data_to_container(&self, container: &str) -> Result<()> {
        let source_path = format!("/opt/gbo/tenants/{}/{}/", self.tenant, container);

        if Path::new(&source_path).exists() {
            info!("Syncing data for {container}");

            SafeCommand::new("incus")
                .and_then(|c| c.arg("exec"))
                .and_then(|c| c.arg(container))
                .and_then(|c| c.arg("--"))
                .and_then(|c| c.arg("mkdir"))
                .and_then(|c| c.arg("-p"))
                .and_then(|c| c.arg("/opt/gbo"))
                .and_then(|cmd| cmd.execute())?;

            let source_path_dot = format!("{source_path}.");
            let target_path = format!("{container}:/opt/gbo/");
            SafeCommand::new("incus")
                .and_then(|c| c.arg("file"))
                .and_then(|c| c.arg("push"))
                .and_then(|c| c.arg("--recursive"))
                .and_then(|c| c.arg(source_path_dot.as_str()))
                .and_then(|c| c.arg(target_path.as_str()))
                .and_then(|cmd| cmd.execute())?;
        }
        Ok(())
    }

    async fn fix_permissions(&self, container: &str) -> Result<()> {
        let settings = self.get_container_settings(container)?;

        let chown_cmd = if let Some(group) = &settings.group {
            format!("chown -R {}:{} /opt/gbo/", settings.user, group)
        } else {
            format!("chown -R {}:{} /opt/gbo/", settings.user, settings.user)
        };

        SafeCommand::new("incus")
            .and_then(|c| c.arg("exec"))
            .and_then(|c| c.arg(container))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg("sh"))
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.arg(&chown_cmd))
            .and_then(|cmd| cmd.execute())?;

        // Make binaries executable
        let bin_path = format!("{}/bin/*", self.base_path.display());
        SafeCommand::new("incus")
            .and_then(|c| c.arg("exec"))
            .and_then(|c| c.arg(container))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg("chmod"))
            .and_then(|c| c.arg("+x"))
            .and_then(|c| c.arg(bin_path.as_str()))
            .and_then(|cmd| cmd.execute())?;

        Ok(())
    }

    async fn install_systemd_service(&self, container: &str) -> Result<()> {
        let settings = self.get_container_settings(container)?;

        let service_name = format!("{container}.service");
        let temp_path = format!("/tmp/{service_name}");

        std::fs::write(&temp_path, &settings.service_template)
            .context("Failed to write service template")?;

    let target_service_path = format!("{container}:/etc/systemd/system/{service_name}");
        SafeCommand::new("incus")
            .and_then(|c| c.arg("file"))
            .and_then(|c| c.arg("push"))
            .and_then(|c| c.arg(temp_path.as_str()))
            .and_then(|c| c.arg(target_service_path.as_str()))
            .and_then(|cmd| cmd.execute())?;

        let commands: Vec<Vec<&str>> = vec![
            vec!["daemon-reload"],
            vec!["enable", service_name.as_str()],
            vec!["start", service_name.as_str()],
        ];
        for cmd_args in commands {
            let mut cmd_builder: Result<SafeCommand, crate::security::command_guard::CommandGuardError> = SafeCommand::new("incus")
                .and_then(|c| c.arg("exec"))
                .and_then(|c| c.arg(container))
                .and_then(|c| c.arg("--"))
                .and_then(|c| c.arg("systemctl"));

            for arg in cmd_args {
                cmd_builder = cmd_builder.and_then(|c| c.arg(arg));
            }
            cmd_builder?.execute()?;
        }

        std::fs::remove_file(&temp_path).ok();
        Ok(())
    }

    async fn configure_iptables_nat(&self, container: &str) -> Result<()> {
        let settings = self.get_container_settings(container)?;

        // Set route_localnet if not already set
        let _ = SafeCommand::new("sudo")
            .and_then(|c| c.arg("sysctl"))
            .and_then(|c| c.arg("-w"))
            .and_then(|c| c.arg("net.ipv4.conf.all.route_localnet=1"))
            .and_then(|cmd| cmd.execute());

        for rule in &settings.nat_rules {
            // Pre-allocate strings to satisfy lifetime requirements
            let port_str = rule.port.to_string();
            let dest = format!("{}:{}", settings.ip, rule.port);
            let dest_ref = dest.as_str();
            let port_ref = port_str.as_str();
            let protocol_ref = rule.protocol.as_str();
            let ip_ref = settings.ip.as_str();

            // PREROUTING rule - for external traffic
            SafeCommand::new("sudo")
                .and_then(|c| c.arg("iptables"))
                .and_then(|c| c.arg("-t"))
                .and_then(|c| c.arg("nat"))
                .and_then(|c| c.arg("-A"))
                .and_then(|c| c.arg("PREROUTING"))
                .and_then(|c| c.arg("-p"))
                .and_then(|c| c.arg(protocol_ref))
                .and_then(|c| c.arg("--dport"))
                .and_then(|c| c.arg(port_ref))
                .and_then(|c| c.arg("-j"))
                .and_then(|c| c.arg("DNAT"))
                .and_then(|c| c.arg("--to-destination"))
                .and_then(|c| c.arg(dest_ref))
                .and_then(|cmd| cmd.execute())?;

            // OUTPUT rule - for local traffic
            SafeCommand::new("sudo")
                .and_then(|c| c.arg("iptables"))
                .and_then(|c| c.arg("-t"))
                .and_then(|c| c.arg("nat"))
                .and_then(|c| c.arg("-A"))
                .and_then(|c| c.arg("OUTPUT"))
                .and_then(|c| c.arg("-p"))
                .and_then(|c| c.arg(protocol_ref))
                .and_then(|c| c.arg("--dport"))
                .and_then(|c| c.arg(port_ref))
                .and_then(|c| c.arg("-j"))
                .and_then(|c| c.arg("DNAT"))
                .and_then(|c| c.arg("--to-destination"))
                .and_then(|c| c.arg(dest_ref))
                .and_then(|cmd| cmd.execute())?;

            // FORWARD rules
            SafeCommand::new("sudo")
                .and_then(|c| c.arg("iptables"))
                .and_then(|c| c.arg("-A"))
                .and_then(|c| c.arg("FORWARD"))
                .and_then(|c| c.arg("-p"))
                .and_then(|c| c.arg(protocol_ref))
                .and_then(|c| c.arg("-d"))
                .and_then(|c| c.arg(ip_ref))
                .and_then(|c| c.arg("--dport"))
                .and_then(|c| c.arg(port_ref))
                .and_then(|c| c.arg("-j"))
                .and_then(|c| c.arg("ACCEPT"))
                .and_then(|cmd| cmd.execute())?;
        }

        let settings_ip_ref = settings.ip.as_str();

        // POSTROUTING MASQUERADE for return traffic
        SafeCommand::new("sudo")
            .and_then(|c| c.arg("iptables"))
            .and_then(|c| c.arg("-t"))
            .and_then(|c| c.arg("nat"))
            .and_then(|c| c.arg("-A"))
            .and_then(|c| c.arg("POSTROUTING"))
            .and_then(|c| c.arg("-p"))
            .and_then(|c| c.arg("tcp"))
            .and_then(|c| c.arg("-d"))
            .and_then(|c| c.arg(settings_ip_ref))
            .and_then(|c| c.arg("-j"))
            .and_then(|c| c.arg("MASQUERADE"))
            .and_then(|cmd| cmd.execute())?;

        // Save rules
        let _ = SafeCommand::new("sudo")
            .and_then(|c| c.arg("sh"))
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.arg("iptables-save > /etc/iptables/rules.v4"))
            .and_then(|cmd| cmd.execute());

        Ok(())
    }

    async fn start_coredns(&self, container: &str) -> Result<()> {
        SafeCommand::new("incus")
            .and_then(|c| c.arg("exec"))
            .and_then(|c| c.arg(container))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg("bash"))
            .and_then(|c| c.arg("-c"))
            .and_then(|c| {
                c.arg("mkdir -p /opt/gbo/logs && nohup /opt/gbo/bin/coredns -conf /opt/gbo/conf/Corefile > /opt/gbo/logs/coredns.log 2>&1 &")
            })
            .and_then(|cmd| cmd.execute())?;

        Ok(())
    }

    async fn reload_dns_zones(&self) -> Result<()> {
        // Update zone files to point to new IP
        let _ = SafeCommand::new("incus")
            .and_then(|c| c.arg("exec"))
            .and_then(|c| c.arg("dns"))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg("sh"))
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.arg("sed -i 's/OLD_IP/NEW_IP/g' /opt/gbo/data/*.zone"))
            .and_then(|cmd| cmd.execute());

        // Restart coredns
        self.start_coredns("dns").await?;

        Ok(())
    }

    fn get_container_settings(&self, container: &str) -> Result<&ContainerSettings> {
        self.components
            .get(container)
            .and_then(|c| c.container.as_ref())
            .context("Container settings not found")
    }

    async fn install_binary_to_container(
        &self,
        container: &str,
        component: &str,
    ) -> Result<()> {
        let config = self
            .components
            .get(component)
            .context("Component not found")?;

        let binary_name = config
            .binary_name
            .as_ref()
            .context("No binary name")?;

        let settings = config
            .container
            .as_ref()
            .context("No container settings")?;

        // Check if already exists
        let check = SafeCommand::new("incus")
            .and_then(|c| c.arg("exec"))
            .and_then(|c| c.arg(container))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg("test"))
            .and_then(|c| c.arg("-f"))
            .and_then(|c| c.arg(&settings.binary_path))
            .and_then(|cmd| cmd.execute());

        if check.is_ok() {
            info!("Binary {binary_name} already exists in {container}");
            return Ok(());
        }

        // Download if URL available
        if let Some(url) = &config.download_url {
            self.download_and_push_binary(container, url, binary_name)
                .await?;
        }

        // Make executable
        SafeCommand::new("incus")
            .and_then(|c| c.arg("exec"))
            .and_then(|c| c.arg(container))
            .and_then(|c| c.arg("--"))
            .and_then(|c| c.arg("chmod"))
            .and_then(|c| c.arg("+x"))
            .and_then(|c| c.arg(&settings.binary_path))
            .and_then(|cmd| cmd.execute())?;

        Ok(())
    }

    async fn download_and_push_binary(
        &self,
        container: &str,
        url: &str,
        binary_name: &str,
    ) -> Result<()> {
        let temp_path = format!("/tmp/{binary_name}");

        // Download to temp
        let output = SafeCommand::new("curl")
            .and_then(|c| c.arg("-fsSL"))
            .and_then(|c| c.arg(url))
            .and_then(|cmd| cmd.execute())?;

        std::fs::write(&temp_path, output.stdout)?;

        // Push to container
        let target_path = format!("{container}:/opt/gbo/bin/{binary_name}");
        SafeCommand::new("incus")
            .and_then(|c| c.arg("file"))
            .and_then(|c| c.arg("push"))
            .and_then(|c| c.arg(temp_path.as_str()))
            .and_then(|c| c.arg(target_path.as_str()))
            .and_then(|cmd| cmd.execute())?;

        std::fs::remove_file(&temp_path).ok();
        Ok(())
    }
}

/// Bootstrap an entire tenant
pub async fn bootstrap_tenant(
    pm: &PackageManager,
    tenant: &str,
    containers: &[&str],
    source_remote: Option<&str>,
) -> Result<()> {
    info!("Bootstrapping tenant: {tenant}");

    for container in containers {
        pm.bootstrap_container(container, source_remote).await?;
    }

    info!("Tenant {tenant} bootstrapped successfully");
    Ok(())
}

/// Bootstrap all pragmatismo containers
pub async fn bootstrap_pragmatismo(pm: &PackageManager) -> Result<()> {
    let containers = [
        "dns", "email", "webmail", "alm", "drive", "tables", "system", "proxy", "alm-ci",
        "table-editor",
    ];

    bootstrap_tenant(pm, "pragmatismo", &containers, Some("lxd-source")).await
}

/// Service file templates for various containers
pub mod service_templates {
    /// CoreDNS service template
    pub fn dns_service() -> &'static str {
        r#"[Unit]
Description=CoreDNS
After=network.target

[Service]
User=root
WorkingDirectory=/opt/gbo
ExecStart=/opt/gbo/bin/coredns -conf /opt/gbo/conf/Corefile
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#
    }

    /// Stalwart email service template
    pub fn email_service() -> &'static str {
        r#"[Unit]
Description=Stalwart Mail Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/gbo
ExecStart=/opt/gbo/bin/stalwart --config /opt/gbo/conf/config.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#
    }

    /// Caddy proxy service template
    pub fn proxy_service() -> &'static str {
        r#"[Unit]
Description=Caddy Reverse Proxy
After=network.target

[Service]
User=gbuser
Group=gbuser
WorkingDirectory=/opt/gbo
ExecStart=/usr/bin/caddy run --config /opt/gbo/conf/config --adapter caddyfile
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#
    }

    /// Forgejo ALM service template
    pub fn alm_service() -> &'static str {
        r#"[Unit]
Description=Forgejo Git Server
After=network.target

[Service]
User=gbuser
Group=gbuser
WorkingDirectory=/opt/gbo
ExecStart=/opt/gbo/bin/forgejo web --config /opt/gbo/conf/app.ini
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#
    }

    /// MinIO drive service template
    pub fn minio_service() -> &'static str {
        r#"[Unit]
Description=MinIO Object Storage
After=network-online.target
Wants=network-online.target

[Service]
User=gbuser
Group=gbuser
WorkingDirectory=/opt/gbo
ExecStart=/opt/gbo/bin/minio server --console-address :4646 /opt/gbo/data
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"#
    }

    /// PostgreSQL tables service template
    pub fn tables_service() -> &'static str {
        r#"[Unit]
Description=PostgreSQL
After=network.target

[Service]
User=gbuser
Group=gbuser
WorkingDirectory=/opt/gbo
ExecStart=/opt/gbo/bin/postgres -D /opt/gbo/data -c config_file=/opt/gbo/conf/postgresql.conf
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#
    }

    /// Apache webmail service template
    pub fn webmail_service() -> &'static str {
        r#"[Unit]
Description=Apache Webmail
After=network.target

[Service]
User=www-data
Group=www-data
WorkingDirectory=/var/www/html
ExecStart=/usr/sbin/apache2 -D FOREGROUND
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#
    }
}
// CI trigger
// CI trigger Fri Apr 17 17:34:16 -03 2026
