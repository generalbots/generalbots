use crate::package_manager::container::SERVICE_MAP;
use botlib::security::command_guard::SafeCommand;
use anyhow::Result;
use log::{info, warn};
use std::path::PathBuf;

/// Service descriptor for certificate generation
pub struct ServiceCert {
    pub name: &'static str,
    pub ip: &'static str,
    pub mtls_port: u16,
}

pub fn get_service_list() -> Vec<ServiceCert> {
    SERVICE_MAP.iter().map(|(name, _container, ip, _port, mtls_port)| {
        ServiceCert { name, ip, mtls_port: *mtls_port }
    }).collect()
}

/// Generate CA if not exists, then generate certs for all services
pub fn generate_all_certs(base_path: &PathBuf) -> Result<()> {
    let cert_base = base_path.join("conf/system/certificates");
    let ca_key = cert_base.join("ca/ca.key");
    let ca_crt = cert_base.join("ca/ca.crt");

    // Generate CA if needed
    if !ca_key.exists() || !ca_crt.exists() {
        info!("Generating Certificate Authority...");
        std::fs::create_dir_all(cert_base.join("ca"))?;

        safe_sh(&format!(
            "openssl genrsa -out {} 4096 2>/dev/null",
            ca_key.display()
        ))?;
        safe_sh(&format!(
            "openssl req -new -x509 -days 3650 -key {} -out {} -subj '/C=BR/ST=SP/L=Sao Paulo/O=General Bots Internal CA/CN=General Bots CA' 2>/dev/null",
            ca_key.display(), ca_crt.display()
        ))?;
        info!("CA generated at {:?}", cert_base.join("ca"));
    } else {
        info!("CA already exists at {:?}", cert_base.join("ca"));
    }

    // Generate cert for each service
    for svc in get_service_list() {
        let svc_dir = cert_base.join(svc.name);
        let server_key = svc_dir.join("server.key");
        let server_crt = svc_dir.join("server.crt");

        if server_key.exists() && server_crt.exists() {
            info!("Cert for {} already exists, skipping", svc.name);
            continue;
        }

        info!("Generating certificates for {} ({})", svc.name, svc.ip);
        std::fs::create_dir_all(&svc_dir)?;

        // Generate server key
        safe_sh(&format!(
            "openssl genrsa -out {} 4096 2>/dev/null",
            server_key.display()
        ))?;

        // Generate CSR
        safe_sh(&format!(
            "openssl req -new -key {} -out {}.csr -subj '/C=BR/ST=SP/L=Sao Paulo/O=General Bots/CN={}' 2>/dev/null",
            server_key.display(), svc_dir.join("server").display(), svc.name
        ))?;

        // SAN extensions with container IP
        let ext_file = svc_dir.join("server.ext");
        let ext_content = format!(
            "subjectAltName = DNS:localhost,IP:127.0.0.1,IP:{}\nkeyUsage = digitalSignature,keyEncipherment\nextendedKeyUsage = serverAuth",
            svc.ip
        );
        std::fs::write(&ext_file, ext_content)?;

        // Sign with CA
        safe_sh(&format!(
            "openssl x509 -req -days 3650 -in {}.csr -CA {} -CAkey {} -CAcreateserial -out {} -extfile {} 2>/dev/null",
            svc_dir.join("server").display(), ca_crt.display(), ca_key.display(), server_crt.display(), ext_file.display()
        ))?;

        // Generate client cert for mTLS
        let client_key = svc_dir.join("client.key");
        let client_crt = svc_dir.join("client.crt");
        safe_sh(&format!(
            "openssl genrsa -out {} 4096 2>/dev/null",
            client_key.display()
        ))?;
        safe_sh(&format!(
            "openssl req -new -key {} -out {}.csr -subj '/C=BR/ST=SP/L=Sao Paulo/O=General Bots/CN={}-client' 2>/dev/null",
            client_key.display(), svc_dir.join("client").display(), svc.name
        ))?;
        safe_sh(&format!(
            "openssl x509 -req -days 3650 -in {}.csr -CA {} -CAkey {} -CAcreateserial -out {} -extfile {} 2>/dev/null",
            svc_dir.join("client").display(), ca_crt.display(), ca_key.display(), client_crt.display(), ext_file.display()
        ))?;

        // Cleanup CSRs and ext
        std::fs::remove_file(svc_dir.join("server.csr")).ok();
        std::fs::remove_file(svc_dir.join("client.csr")).ok();
        std::fs::remove_file(&ext_file).ok();

        info!("Certificates generated for {}", svc.name);
    }

    info!("All service certificates generated successfully");
    Ok(())
}

/// Push all certs to their respective containers via SSH to incus host
pub fn push_all_certs_ssh(base_path: &PathBuf, ssh_host: &str) -> Result<()> {
    let cert_base = base_path.join("conf/system/certificates");
    let ca_crt = cert_base.join("ca/ca.crt");
    if !ca_crt.exists() {
        return Err(anyhow::anyhow!("CA not found at {:?}. Run generate first.", ca_crt));
    }

    for (name, container, _ip, _port, _mtls_port) in SERVICE_MAP {
        info!("Pushing certs for {} to container {}", name, container);

        let svc_dir = cert_base.join(name);
        let files: Vec<(&str, PathBuf)> = vec![
            ("ca.crt", ca_crt.clone()),
            ("server.crt", svc_dir.join("server.crt")),
            ("server.key", svc_dir.join("server.key")),
        ];

        for (fname, fpath) in &files {
            if !fpath.exists() {
                warn!("{} missing for {}, skipping", fname, name);
                continue;
            }

            let remote_tmp = PathBuf::from(format!("/tmp/{}-{}", container, fname));

            // SCP to host
            safe_sh(&format!(
                "scp -o StrictHostKeyChecking=no -o ConnectTimeout=10 {} {}:{}",
                fpath.display(), ssh_host, remote_tmp.display()
            ))?;

            // mkdir + incus file push on host
            let target_dir = format!("/opt/gbo/conf/system/certificates/{}", name);
            safe_sh(&format!(
                "ssh {} -o StrictHostKeyChecking=no 'sudo incus exec {} -- mkdir -p {} && sudo incus file push {} {}:{}/{}'",
                ssh_host, container, target_dir, remote_tmp.display(), container, target_dir, fname
            ))?;

            // Cleanup temp
            safe_sh(&format!(
                "ssh {} -o StrictHostKeyChecking=no 'sudo rm -f {}'",
                ssh_host, remote_tmp.display()
            ))?;
        }

        info!("Certificates pushed for {} -> {}", name, container);
    }

    Ok(())
}

fn safe_sh(script: &str) -> Result<std::process::Output> {
    SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.trusted_shell_script_arg(script))
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .execute()
        .map_err(|e| anyhow::anyhow!("Command failed: {e}\nScript: {script}"))
}
