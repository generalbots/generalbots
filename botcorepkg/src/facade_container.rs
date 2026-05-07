use crate::component::{ComponentConfig, InstallResult};
use crate::db_utils::{get_database_url_sync, parse_database_url};
use crate::OsType;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{info, trace, warn};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

fn safe_lxc(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("lxc")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

fn safe_lxd(args: &[&str]) -> Option<std::process::Output> {
    let cmd_res = SafeCommand::new("lxd").and_then(|c| c.args(args));
    cmd_res.ok().and_then(|cmd| cmd.execute().ok())
}

pub fn install_container(
    tenant: &str,
    component: &ComponentConfig,
    os_type: &OsType,
    _base_path: &std::path::Path,
) -> Result<InstallResult> {
    let container_name = format!("{}-{}", tenant, component.name);
    let _ = safe_lxd(&["init", "--auto"]);
    let images = [
        "ubuntu:24.04",
        "ubuntu:22.04",
        "images:debian/12",
        "images:debian/11",
    ];
    let mut last_error = String::new();
    let mut success = false;
    for image in &images {
        info!("Attempting to create container with image: {}", image);
        let output = safe_lxc(&[
            "launch", image, &container_name, "-c", "security.privileged=true",
        ]);
        let output = match output {
            Some(o) => o,
            None => continue,
        };
        if output.status.success() {
            info!("Successfully created container with image: {}", image);
            success = true;
            break;
        }
        last_error = String::from_utf8_lossy(&output.stderr).to_string();
        warn!("Failed to create container with {}: {}", image, last_error);
        let _ = safe_lxc(&["delete", &container_name, "--force"]);
    }
    if !success {
        return Err(anyhow::anyhow!(
            "LXC container creation failed with all images. Last error: {}", last_error
        ));
    }
    std::thread::sleep(std::time::Duration::from_secs(15));
    let exec = crate::facade_download::exec_in_container;
    exec(&container_name, "mkdir -p /opt/gbo/bin /opt/gbo/data /opt/gbo/conf /opt/gbo/logs")?;
    exec(&container_name, "echo 'nameserver 8.8.8.8' > /etc/resolv.conf")?;
    exec(&container_name, "echo 'nameserver 8.8.4.4' >> /etc/resolv.conf")?;
    exec(&container_name, "apt-get update -qq")?;
    exec(&container_name,
        "DEBIAN_FRONTEND=noninteractive apt-get install -y -qq wget unzip curl ca-certificates")?;
    let (pre_cmds, post_cmds) = match os_type {
        OsType::Linux => (&component.pre_install_cmds_linux, &component.post_install_cmds_linux),
        OsType::MacOS => (&component.pre_install_cmds_macos, &component.post_install_cmds_macos),
        OsType::Windows => (&component.pre_install_cmds_windows, &component.post_install_cmds_windows),
    };
    crate::facade_download::run_commands(
        pre_cmds, &container_name, &component.name, &PathBuf::from("/opt/gbo"), "",
    )?;
    let packages = match os_type {
        OsType::Linux => &component.linux_packages,
        OsType::MacOS => &component.macos_packages,
        OsType::Windows => &component.windows_packages,
    };
    if !packages.is_empty() {
        let pkg_list = packages.join(" ");
        exec(&container_name, "apt-get update -qq")?;
        exec(&container_name,
            &format!("DEBIAN_FRONTEND=noninteractive apt-get install -y -qq {}", pkg_list))?;
    }
    if let Some(url) = &component.download_url {
        crate::facade_download::download_in_container(
            &container_name, url, &component.name, component.binary_name.as_deref(),
        )?;
    }
    crate::facade_download::run_commands(
        post_cmds, &container_name, &component.name, &PathBuf::from("/opt/gbo"), "",
    )?;
    exec(&container_name,
        "useradd --system --no-create-home --shell /bin/false gbuser")?;
    mount_container_directories(&container_name, &component.name, tenant)?;
    if !component.exec_cmd.is_empty() {
        create_container_service(&container_name, &component.name, &component.exec_cmd, &component.env_vars)?;
    }
    setup_port_forwarding(&container_name, &component.ports)?;
    let container_ip = get_container_ip(&container_name)?;
    if component.name == "vault" {
        initialize_vault(&container_name, &container_ip)?;
    }
    let (connection_info, env_vars) =
        crate::facade_connection::generate_connection_info(tenant, &component.name, &container_ip, &component.ports);
    trace!("Container installation of '{}' completed in {}", component.name, container_name);
    Ok(InstallResult {
        component: component.name.clone(),
        container_name,
        container_ip,
        ports: component.ports.clone(),
        env_vars,
        connection_info,
    })
}

pub fn get_container_ip(container_name: &str) -> Result<String> {
    for _ in 0..15 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        if let Some(o) = safe_lxc(&["list", container_name, "-c", "4", "--format", "csv"]) {
            if o.status.success() {
                let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let ip = out.split([' ', '(']).next().unwrap_or("").trim();
                if !ip.is_empty() && ip.contains('.') {
                    return Ok(ip.to_string());
                }
            }
        }
        if let Some(o) = safe_lxc(&["exec", container_name, "--", "hostname", "-I"]) {
            if o.status.success() {
                let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if let Some(ip) = out.split_whitespace().find(|s| s.contains('.')) {
                    return Ok(ip.to_string());
                }
            }
        }
    }
    warn!("DHCP timeout for '{}', assigning static IP via lxdbr0", container_name);
    let static_ip = assign_static_ip(container_name)?;
    Ok(static_ip)
}

pub fn assign_static_ip(container_name: &str) -> Result<String> {
    let bridge_info = std::process::Command::new("ip")
        .args(["-4", "addr", "show", "lxdbr0"])
        .output().ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            out.lines().find(|l| l.contains("inet "))
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|s| s.to_string())
        });
    let (gateway, prefix) = match bridge_info.as_deref().and_then(|cidr| {
        let (ip, pfx_str) = cidr.split_once('/')?;
        let parts: Vec<&str> = ip.split('.').collect();
        let base = format!("{}.{}.{}.", parts[0], parts[1], parts[2]);
        let pfx = pfx_str.parse::<u8>().ok()?;
        Some((ip.to_string(), format!("{base}{{octet}}/{pfx}")))
    }) {
        Some(pair) => pair,
        None => return Err(anyhow::anyhow!("Cannot determine lxdbr0 subnet")),
    };
    let mac_out = safe_lxc(&["exec", container_name, "--", "cat", "/sys/class/net/eth0/address"]);
    let octet = mac_out
        .and_then(|o| o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string()))
        .and_then(|mac| {
            let parts: Vec<&str> = mac.split(':').collect();
            let a = u8::from_str_radix(parts.get(4)?, 16).ok()?;
            let b = u8::from_str_radix(parts.get(5)?, 16).ok()?;
            Some(100u16 + (u16::from(a) * 256 + u16::from(b)) % 150)
        })
        .unwrap_or(100);
    let cidr = prefix.replace("{octet}", &octet.to_string());
    let ip = cidr.split('/').next().unwrap_or("").to_string();
    safe_lxc(&["exec", container_name, "--", "ip", "addr", "add", &cidr, "dev", "eth0"]);
    safe_lxc(&["exec", container_name, "--", "ip", "route", "add", "default", "via", &gateway]);
    safe_lxc(&["exec", container_name, "--", "bash", "-c",
        &format!("printf 'nameserver 8.8.8.8\\nnameserver 8.8.4.4\\n' > /etc/resolv.conf && \
printf '#!/bin/sh\\nip addr add {cidr} dev eth0 2>/dev/null||true\\nip route add default via {gateway} 2>/dev/null||true\\nexit 0\\n' > /etc/rc.local && \
chmod +x /etc/rc.local && systemctl enable rc-local 2>/dev/null||true")]);
    info!("Assigned static IP {} to container '{}'", ip, container_name);
    Ok(ip)
}

pub fn initialize_vault(container_name: &str, ip: &str) -> Result<()> {
    info!("Initializing Vault...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    let output = safe_lxc(&[
        "exec", container_name, "--", "bash", "-c",
        "VAULT_ADDR=http://127.0.0.1:8200 /opt/gbo/bin/vault operator init -key-shares=5 -key-threshold=3 -format=json",
    ]);
    let output = match output {
        Some(o) => o,
        None => return Err(anyhow::anyhow!("Failed to execute vault init command")),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already initialized") {
            warn!("Vault already initialized, skipping file generation");
            return Ok(());
        }
        return Err(anyhow::anyhow!("Failed to initialize Vault: {}", stderr));
    }
    let init_output = String::from_utf8_lossy(&output.stdout);
    let init_json: serde_json::Value =
        serde_json::from_str(&init_output).context("Failed to parse Vault init output")?;
    let unseal_keys = init_json["unseal_keys_b64"]
        .as_array().context("No unseal keys in output")?;
    let root_token = init_json["root_token"]
        .as_str().context("No root token in output")?;
    let unseal_keys_file = PathBuf::from("vault-unseal-keys");
    let mut unseal_content = String::new();
    for (i, key) in unseal_keys.iter().enumerate() {
        if i < 3 {
            let _ = writeln!(unseal_content, "VAULT_UNSEAL_KEY_{}={}", i + 1, key.as_str().unwrap_or(""));
        }
    }
    std::fs::write(&unseal_keys_file, &unseal_content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&unseal_keys_file, std::fs::Permissions::from_mode(0o600))?;
    }
    info!("Created {}", unseal_keys_file.display());
    let env_file = PathBuf::from(".env");
    let env_content = format!(
        "\n# Vault Configuration (auto-generated)\nVAULT_ADDR=http://{}:8200\nVAULT_TOKEN={}\n",
        ip, root_token
    );
    if env_file.exists() {
        let existing = std::fs::read_to_string(&env_file)?;
        if existing.contains("VAULT_ADDR=") {
            warn!(".env already contains VAULT_ADDR, not overwriting");
        } else {
            let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
            use std::io::Write;
            file.write_all(env_content.as_bytes())?;
            info!("Appended Vault config to .env");
        }
    } else {
        std::fs::write(&env_file, env_content.trim_start())?;
        info!("Created .env with Vault config");
    }
    for i in 0..3 {
        if let Some(key) = unseal_keys.get(i) {
            let key_str = key.as_str().unwrap_or("");
            let unseal_cmd = format!(
                "VAULT_ADDR=http://127.0.0.1:8200 /opt/gbo/bin/vault operator unseal {}", key_str
            );
            let unseal_output = safe_lxc(&["exec", container_name, "--", "bash", "-c", &unseal_cmd]);
            if let Some(output) = unseal_output {
                if !output.status.success() {
                    warn!("Unseal step {} may have failed", i + 1);
                }
            } else {
                warn!("Unseal step {} command failed to execute", i + 1);
            }
        }
    }
    info!("Vault initialized and unsealed successfully");
    Ok(())
}

pub fn generate_env_from_vault(container_name: &str) -> Result<String> {
    let container_ip = get_container_ip(container_name)?;
    info!("Generating .env from vault at {}", container_ip);
    let output = safe_lxc(&[
        "exec", container_name, "--", "bash", "-c",
        "cat /opt/gbo/data/core/raft/raft_state 2>/dev/null || echo 'not_initialized'",
    ]);
    let initialized = match output {
        Some(o) => o.status.success(),
        None => false,
    };
    if !initialized {
        return Err(anyhow::anyhow!(
            "Vault in container {} is not initialized. Please initialize it first with: \
lxc exec {} -- /opt/gbo/bin/vault operator init -key-shares=1 -key-threshold=1",
            container_name, container_name
        ));
    }
    let unseal_output = safe_lxc(&[
        "exec", container_name, "--", "bash", "-c",
        "VAULT_ADDR=http://127.0.0.1:8200 vault status -format=json",
    ]);
    let sealed = match unseal_output {
        Some(o) if o.status.success() => {
            let status: serde_json::Value = serde_json::from_str(
                &String::from_utf8_lossy(&o.stdout)
            ).unwrap_or_default();
            status["sealed"].as_bool().unwrap_or(true)
        }
        _ => true,
    };
    if sealed {
        return Err(anyhow::anyhow!(
            "Vault in container {} is sealed. Please unseal it first with: \
lxc exec {} -- /opt/gbo/bin/vault operator unseal <key>",
            container_name, container_name
        ));
    }
    let token_output = safe_lxc(&[
        "exec", container_name, "--", "bash", "-c",
        "cat /root/.vault-token 2>/dev/null || echo ''",
    ]);
    let root_token = match token_output {
        Some(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        None => String::new(),
    };
    if root_token.is_empty() {
        return Err(anyhow::anyhow!(
            "Could not find root token in vault container. Please ensure vault is properly initialized."
        ));
    }
    let env_content = format!(
        "# Vault Configuration (auto-generated)\nVAULT_ADDR=http://{}:8200\nVAULT_TOKEN={}\n",
        container_ip, root_token
    );
    let env_file = PathBuf::from(".env");
    if env_file.exists() {
        let existing = std::fs::read_to_string(&env_file)?;
        if existing.contains("VAULT_ADDR=") {
            info!(".env already contains VAULT_ADDR, updating...");
            let updated: String = existing
                .lines()
                .filter(|line| !line.starts_with("VAULT_ADDR=") && !line.starts_with("VAULT_TOKEN="))
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(&env_file, format!("{}\n{}", updated.trim(), env_content))?;
        } else {
            let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
            use std::io::Write;
            file.write_all(env_content.as_bytes())?;
        }
    } else {
        std::fs::write(&env_file, env_content.trim_start())?;
    }
    info!("Generated .env with Vault config from container {}", container_name);
    Ok(format!(
        "VAULT_ADDR=http://{}:8200\nVAULT_TOKEN={}",
        container_ip, root_token
    ))
}

pub fn mount_container_directories(container: &str, component: &str, tenant: &str) -> Result<()> {
    let host_base = format!("/opt/gbo/tenants/{}/{}", tenant, component);
    for dir in &["data", "conf", "logs"] {
        let host_path = format!("{}/{}", host_base, dir);
        std::fs::create_dir_all(&host_path)?;
        let device_name = format!("{}-{}", component, dir);
        let container_path = format!("/opt/gbo/{}", dir);
        let _ = safe_lxc(&["config", "device", "remove", container, &device_name]);
        let source_arg = format!("source={}", host_path);
        let path_arg = format!("path={}", container_path);
        let output = safe_lxc(&[
            "config", "device", "add", container, &device_name, "disk",
            &source_arg, &path_arg,
        ]);
        let output = match output {
            Some(o) => o,
            None => {
                warn!("Failed to execute lxc mount command for {}", dir);
                continue;
            }
        };
        if !output.status.success() {
            warn!("Failed to mount {} in container {}", dir, container);
        }
        trace!("Mounted {} to {} in container {}", host_path, container_path, container);
    }
    Ok(())
}

pub fn create_container_service(
    container: &str,
    component: &str,
    exec_cmd: &str,
    env_vars: &HashMap<String, String>,
) -> Result<()> {
    let db_password = match get_database_url_sync() {
        Ok(url) => {
            let (_, password, _, _, _) = parse_database_url(&url);
            password
        }
        Err(_) => {
            trace!("Vault not available for DB_PASSWORD in container service, using empty string");
            String::new()
        }
    };
    let rendered_cmd = exec_cmd
        .replace("{{DB_PASSWORD}}", &db_password)
        .replace("{{BIN_PATH}}", "/opt/gbo/bin")
        .replace("{{DATA_PATH}}", "/opt/gbo/data")
        .replace("{{CONF_PATH}}", "/opt/gbo/conf")
        .replace("{{LOGS_PATH}}", "/opt/gbo/logs");
    let mut env_section = String::new();
    for (key, value) in env_vars {
        let rendered_value = value
            .replace("{{DATA_PATH}}", "/opt/gbo/data")
            .replace("{{BIN_PATH}}", "/opt/gbo/bin")
            .replace("{{CONF_PATH}}", "/opt/gbo/conf")
            .replace("{{LOGS_PATH}}", "/opt/gbo/logs");
        let _ = writeln!(env_section, "Environment={key}={rendered_value}");
    }
    let service_content = format!(
        "[Unit]\nDescription={} Service\nAfter=network.target\n\n[Service]\nType=simple\n{}ExecStart={}\nWorkingDirectory=/opt/gbo/data\nRestart=always\nRestartSec=10\nUser=root\n\n[Install]\nWantedBy=multi-user.target\n",
        component, env_section, rendered_cmd
    );
    let service_file = format!("/tmp/{}.service", component);
    std::fs::write(&service_file, &service_content)?;
    let dest = format!("{container}/etc/systemd/system/{component}.service");
    let output = safe_lxc(&["file", "push", &service_file, &dest])
        .ok_or_else(|| anyhow::anyhow!("Failed to execute lxc file push command"))?;
    if !output.status.success() {
        warn!("Failed to push service file to container");
    }
    let exec = crate::facade_download::exec_in_container;
    exec(container, "systemctl daemon-reload")?;
    exec(container, &format!("systemctl enable {}", component))?;
    exec(container, &format!("systemctl start {}", component))?;
    std::fs::remove_file(&service_file)?;
    trace!("Created and started service in container {}: {}", container, component);
    Ok(())
}

pub fn setup_port_forwarding(container: &str, ports: &[u16]) -> Result<()> {
    for port in ports {
        let device_name = format!("port-{}", port);
        let _ = safe_lxc(&["config", "device", "remove", container, &device_name]);
        let listen_arg = format!("listen=tcp:0.0.0.0:{port}");
        let connect_arg = format!("connect=tcp:127.0.0.1:{port}");
        let output = safe_lxc(&[
            "config", "device", "add", container, &device_name,
            "proxy", &listen_arg, &connect_arg,
        ])
        .ok_or_else(|| anyhow::anyhow!("Failed to execute lxc port forward command"))?;
        if !output.status.success() {
            warn!("Failed to setup port forwarding for port {}", port);
        }
        trace!("Port forwarding configured: {} -> container {}", port, container);
    }
    Ok(())
}
