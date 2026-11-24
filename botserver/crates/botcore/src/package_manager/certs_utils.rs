use crate::package_manager::container::SERVICE_MAP;
use crate::package_manager::generate_random_string;
use anyhow::Result;
use botlib::security::command_guard::SafeCommand;
use log::{info, warn};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// #2791 — an unpredictable, non-world-writable remote staging directory.
fn remote_cert_staging_dir(container: &str) -> String {
    format!("/tmp/gb-certs-{}-{}", container, uuid::Uuid::new_v4())
}

#[cfg(test)]
mod cert_staging_tests {
    use super::remote_cert_staging_dir;

    #[test]
    fn staging_path_is_unpredictable() {
        let first = remote_cert_staging_dir("proxy");
        let second = remote_cert_staging_dir("proxy");
        assert_ne!(first, second);
        assert!(first.starts_with("/tmp/gb-certs-proxy-"));
        assert!(!first.ends_with("/ca.crt"));
    }
}

/// Service descriptor for certificate generation
pub struct ServiceCert {
    pub name: &'static str,
    pub ip: &'static str,
    pub mtls_port: u16,
}

pub fn get_service_list() -> Vec<ServiceCert> {
    SERVICE_MAP
        .iter()
        .map(|(name, _container, ip, _port, mtls_port)| ServiceCert {
            name,
            ip,
            mtls_port: *mtls_port,
        })
        .collect()
}

/// Generate CA if not exists, then generate certs for all services
pub fn generate_all_certs(base_path: &Path) -> Result<()> {
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

/// Fresh, unpredictable `mktemp` template for a private staging directory on
/// the remote host. The alphanumeric prefix comes from the process CSPRNG;
/// the trailing X's make `mktemp` create the directory with O_EXCL semantics,
/// so two concurrent pushes can never share a staging path.
fn staging_dir_template() -> String {
    format!("/tmp/gbo-certs.{}.XXXXXXXXXX", generate_random_string(16))
}

/// Wrap a value in single quotes for the local shell so arbitrary path/host
/// text stays one argument and cannot break out of its quoting context.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// SHA-256 of a local file, used to verify the staged and pushed bytes.
fn file_sha256(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// Build one SSH command that stages a single cert file in a private,
/// random remote directory and pushes it into the container. The local file
/// bytes travel over stdin (never in argv), the staging directory is created
/// with `mktemp -d` (0700, O_EXCL), the staged file with `mktemp` (0600,
/// O_EXCL) and an atomic rename, and an EXIT trap removes everything on all
/// paths. Bytes are verified both before and after the push.
fn build_remote_push_script(
    ssh_host: &str,
    container: &str,
    name: &str,
    fname: &str,
    expected_sha256: &str,
    local_path: &str,
) -> String {
    let target_dir = format!("/opt/gbo/conf/system/certificates/{name}");
    let template = staging_dir_template();
    let remote_script = format!(
        concat!(
            "set -eu\n",
            "stage_dir=$(mktemp -d {template})\n",
            "cleanup() {{ rm -rf \"$stage_dir\"; }}\n",
            "trap cleanup EXIT\n",
            "umask 077\n",
            "tmp=$(mktemp \"$stage_dir/staged.XXXXXXXX\")\n",
            "cat > \"$tmp\"\n",
            "expected={expected}\n",
            "actual=$(sha256sum \"$tmp\" | cut -d \" \" -f1)\n",
            "if [ \"$actual\" != \"$expected\" ]; then echo 'staged byte mismatch' >&2; exit 1; fi\n",
            "stage=\"$stage_dir/staged\"\n",
            "mv -f \"$tmp\" \"$stage\"\n",
            "sudo incus exec {container} -- mkdir -p {target}\n",
            "sudo incus file push \"$stage\" {dest}\n",
            "pushed=$(sudo incus exec {container} -- sha256sum {target_file} | cut -d \" \" -f1)\n",
            "if [ \"$pushed\" != \"$expected\" ]; then echo 'pushed byte mismatch' >&2; exit 1; fi\n",
        ),
        template = shell_quote(&template),
        expected = shell_quote(expected_sha256),
        container = shell_quote(container),
        target = shell_quote(&target_dir),
        dest = shell_quote(&format!("{container}:{target_dir}/{fname}")),
        target_file = shell_quote(&format!("{target_dir}/{fname}")),
    );
    format!(
        "ssh -o StrictHostKeyChecking=accept-new -o ConnectTimeout=10 {host} {script} < {path}",
        host = shell_quote(ssh_host),
        script = shell_quote(&remote_script),
        path = shell_quote(local_path),
    )
}

/// Push all certs to their respective containers via SSH to incus host.
/// Staging happens in a fresh private directory per file — never at a
/// predictable `/tmp/{container}-{fname}` path.
pub fn push_all_certs_ssh(base_path: &Path, ssh_host: &str) -> Result<()> {
    let cert_base = base_path.join("conf/system/certificates");
    let ca_crt = cert_base.join("ca/ca.crt");
    if !ca_crt.exists() {
        return Err(anyhow::anyhow!(
            "CA not found at {:?}. Run generate first.",
            ca_crt
        ));
    }

    for (name, container, _ip, _port, _mtls_port) in SERVICE_MAP {
        info!("Pushing certs for {} to container {}", name, container);

        let svc_dir = cert_base.join(name);
        let files: Vec<(&str, PathBuf)> = vec![
            ("ca.crt", ca_crt.clone()),
            ("server.crt", svc_dir.join("server.crt")),
            ("server.key", svc_dir.join("server.key")),
        ];

        // #2791/#2711 — build_remote_push_script stages each file in a
        // private mktemp directory and verifies the SHA-256 both before and
        // after the incus push. This supersedes the old scp+incus path.
        for (fname, fpath) in &files {
            if !fpath.exists() {
                warn!("{} missing for {}, skipping", fname, name);
                continue;
            }
            let expected = file_sha256(fpath)?;
            let script = build_remote_push_script(
                ssh_host,
                container,
                name,
                fname,
                &expected,
                &fpath.to_string_lossy(),
            );
            safe_sh(&script)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staging_dir_template_is_unique_and_unpredictable() {
        let a = staging_dir_template();
        let b = staging_dir_template();
        assert_ne!(a, b, "each staging dir must get a fresh random prefix");
        for template in [&a, &b] {
            assert!(template.starts_with("/tmp/gbo-certs."));
            assert!(template.ends_with(".XXXXXXXXXX"));
            let random = &template["/tmp/gbo-certs.".len()..template.len() - ".XXXXXXXXXX".len()];
            assert_eq!(random.len(), 16);
            assert!(
                random.chars().all(|c| c.is_ascii_alphanumeric()),
                "unsafe char in staging path"
            );
        }
    }

    #[test]
    fn test_remote_push_script_never_uses_predictable_tmp_path() {
        let script = build_remote_push_script(
            "prod-host",
            "cache",
            "cache",
            "server.key",
            "abc123",
            "/local/server.key",
        );
        assert!(
            !script.contains("/tmp/cache-server.key"),
            "old predictable path must be gone"
        );
        assert!(!script.contains("scp "), "scp staging must be gone");
        assert!(
            script.contains("mktemp -d "),
            "staging dir must be random and O_EXCL"
        );
        assert!(script.contains("XXXXXXXXXX"));
        assert!(script.contains("umask 077"));
        assert!(script.contains("trap cleanup EXIT"), "cleanup on all paths");
        assert!(
            script.contains("mktemp \"$stage_dir/staged.XXXXXXXX\""),
            "0600 O_EXCL staged file"
        );
        assert!(
            script.contains("mv -f \"$tmp\" \"$stage\""),
            "atomic rename"
        );
        assert!(script.contains("sha256sum"), "verify bytes");
        assert!(script.contains("staged byte mismatch"));
        assert!(script.contains("pushed byte mismatch"));
        assert!(script.contains("incus file push"));
        assert!(
            script.contains("< '/local/server.key'"),
            "secret bytes travel on stdin"
        );
    }

    #[test]
    fn test_shell_quote_escapes_embedded_single_quotes() {
        assert_eq!(shell_quote("plain"), "'plain'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }
}
