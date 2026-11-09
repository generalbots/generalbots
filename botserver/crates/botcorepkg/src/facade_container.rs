use crate::component::{ComponentConfig, InstallResult};
use crate::db_utils::{get_database_url_sync, parse_database_url};
use crate::OsType;
use anyhow::Result;
use botlib::security::SafeCommand;
use log::{info, trace, warn};
use std::collections::HashMap;
// #2030 — `create_container_service` builds the systemd unit with
// `writeln!(env_section, …)`, which needs the `fmt::Write` trait in scope. The
// Vault split (#1933) moved the other `writeln!` user out of this file and took
// the import with it; `as _` imports the trait for method resolution only.
use std::fmt::Write as _;
use std::path::PathBuf;

pub(crate) fn safe_lxc(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("lxc")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub(crate) fn safe_lxd(args: &[&str]) -> Option<std::process::Output> {
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
        // #2001 — the pinned digest travels with the URL, so the container
        // installer can verify the asset it just fetched.
        let expected = crate::installer::checksum_for_url(url).unwrap_or_default();
        crate::facade_download::download_in_container(
            &container_name,
            url,
            &component.name,
            component.binary_name.as_deref(),
            &expected,
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
    let bridge_info = SafeCommand::new("ip")
        .and_then(|c| c.args(&["-4", "addr", "show", "lxdbr0"]))
        .ok()
        .and_then(|c| c.execute().ok())
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

// #1933 — the container-scoped Vault bootstrap (init, unseal, `.env`
// generation) lives in `installer_vault/container_env.rs`: this file had
// crossed the 450-line hard limit. Re-exported so callers keep using
// `facade_container::{initialize_vault, generate_env_from_vault}`.
pub use crate::installer_vault::container_env::{generate_env_from_vault, initialize_vault};

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
