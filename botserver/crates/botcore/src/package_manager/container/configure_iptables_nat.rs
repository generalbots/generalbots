use super::*;
use super::nat_rule_guard::{is_valid_interface, prerouting_dnat_args, public_dnat_verdict};

/// Fail-closed verdict for an unpinned container binary (#2610, #2709). Pure
/// so the GB_REQUIRE_CHECKSUMS policy is testable without mutating the env.
fn unpinned_checksum_error(url: &str, strict: bool) -> Option<String> {
    if strict {
        Some(format!(
            "no SHA-256 pin for '{url}' and GB_REQUIRE_CHECKSUMS is on; refusing to push an unverified binary"
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod unpinned_checksum_tests {
    use super::unpinned_checksum_error;

    #[test]
    fn unpinned_is_fatal_in_strict_mode() {
        assert!(unpinned_checksum_error("https://example.invalid/bin", true).is_some());
        assert!(unpinned_checksum_error("https://example.invalid/bin", false).is_none());
    }

    #[test]
    fn refusal_names_the_url_and_the_policy() {
        let message = unpinned_checksum_error("https://example.invalid/pinned.bin", true)
            .expect("strict mode must refuse an unpinned binary");
        assert!(message.contains("https://example.invalid/pinned.bin"));
        assert!(message.contains("GB_REQUIRE_CHECKSUMS"));
    }

    #[test]
    fn lenient_mode_only_warns_for_any_unpinned_url() {
        for url in ["https://example.invalid/bin", "", "not-a-url"] {
            assert!(unpinned_checksum_error(url, false).is_none(), "{url}");
        }
    }
}

impl ContainerOperations for PackageManager {
    async fn configure_iptables_nat(&self, container: &str) -> Result<()> {
        let settings = self.get_container_settings(container)?;

        // #2617 — never create an unscoped DNAT rule. An explicit external
        // interface is required; without it the host keeps its existing rules.
        let iface = match std::env::var("GB_EXTERNAL_IFACE") {
            Ok(iface) => iface,
            Err(_) => String::new(),
        };
        if !is_valid_interface(&iface) {
            return Err(anyhow::anyhow!(
                "GB_EXTERNAL_IFACE must be a valid interface name; refusing to create unscoped DNAT rules"
            ));
        }

        // Scope route_localnet to that interface, never to the global namespace.
        let _ = SafeCommand::new("sudo")
            .and_then(|c| c.arg("sysctl"))
            .and_then(|c| c.arg("-w"))
            .and_then(|c| c.arg(format!("net.ipv4.conf.{iface}.route_localnet=1")))
            .and_then(|cmd| cmd.execute());

        for rule in &settings.nat_rules {
            // Pre-allocate strings to satisfy lifetime requirements
            let port_str = rule.port.to_string();
            let dest = format!("{}:{}", settings.ip, rule.port);
            let dest_ref = dest.as_str();
            let port_ref = port_str.as_str();
            let protocol_ref = rule.protocol.as_str();
            let ip_ref = settings.ip.as_str();

            // #2617 — never DNAT an internal/admin port from the public
            // interface. Local OUTPUT/FORWARD rules below still let the host
            // and Incus network reach those services.
            if public_dnat_verdict(rule.port) {
                let args = prerouting_dnat_args(iface.as_str(), protocol_ref, rule.port, dest_ref);
                let mut cmd = SafeCommand::new("sudo")?.arg("iptables")?;
                for a in &args {
                    cmd = cmd.arg(a)?;
                }
                cmd.execute()?;
            } else {
                log::warn!("Skipping public DNAT for internal-only port {}", rule.port);
            }

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
        // #2616 — private unpredictable staging for the downloaded binary.
        let output = SafeCommand::new("curl")
            .and_then(|c| c.arg("-fsSL"))
            .and_then(|c| c.arg(url))
            .and_then(|cmd| cmd.execute())?;
        let staged = crate::package_manager::safe_staging::StagedFile::create(binary_name, &output.stdout)?;

        // #2610 — verify the pinned SHA-256 before a downloaded binary becomes
        // root-owned code inside a container. An unpinned download fails closed
        // under GB_REQUIRE_CHECKSUMS and is loudly warned about otherwise,
        // matching the other download paths in package_manager.
        match botcorepkg::installer::checksum_for_url(url) {
            Some(expected) if !expected.trim().is_empty() => {
                botcorepkg::checksum::verify_download(staged.as_path(), &expected, binary_name)?;
            }
            _ => {
                if let Some(message) = unpinned_checksum_error(url, botcorepkg::checksum::require_checksums()) {
                    return Err(anyhow::anyhow!(message));
                }
                log::warn!("No SHA-256 pin for '{url}'; container binary is NOT verified");
            }
        }

        // Push to container
        let target_path = format!("{container}:/opt/gbo/bin/{binary_name}");
        SafeCommand::new("incus")
            .and_then(|c| c.arg("file"))
            .and_then(|c| c.arg("push"))
            .and_then(|c| c.arg(staged.as_path().to_string_lossy().as_ref()))
            .and_then(|c| c.arg(target_path.as_str()))
            .and_then(|cmd| cmd.execute())?;

        // #2616 — StagedFile Drop removes the private staging directory.
        Ok(())
    }

    async fn provision_certs_to_container(&self, container: &str) -> Result<()> {
        let cert_base = self.base_path.join("conf/system/certificates");
        let ca_cert = cert_base.join("ca/ca.crt");
        if !ca_cert.exists() {
            botlog::safe_info!("No CA found at {:?}, skipping cert provisioning for {container}", ca_cert);
            return Ok(());
        }

        let settings = match self.components.get(container).and_then(|c| c.container.as_ref()) {
            Some(s) => s,
            None => {
                trace!("No container settings for {container}, skipping certs");
                return Ok(());
            }
        };

        let cert_dir = cert_base.join(&settings.cert_name);

        for (src, dest) in [
            (&ca_cert, "/opt/gbo/conf/system/certificates/ca/ca.crt".to_string()),
            (&cert_dir.join("server.crt"), format!("/opt/gbo/conf/system/certificates/{}/server.crt", settings.cert_name)),
            (&cert_dir.join("server.key"), format!("/opt/gbo/conf/system/certificates/{}/server.key", settings.cert_name)),
        ] {
            if !src.exists() {
                trace!("Cert file {:?} not found, skipping", src);
                continue;
            }
            let src_arg = match src.to_str() {
                Some(path) => path,
                None => {
                    log::warn!("Certificate path is not valid UTF-8: {}", src.display());
                    ""
                }
            };
            SafeCommand::new("incus")
                .and_then(|c| c.arg("exec"))
                .and_then(|c| c.arg(container))
                .and_then(|c| c.arg("--"))
                .and_then(|c| c.arg("mkdir"))
                .and_then(|c| c.arg("-p"))
                .and_then(|c| c.arg(
                    std::path::Path::new(&dest)
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or(".")
                ))
                .and_then(|cmd| cmd.execute())?;

            SafeCommand::new("incus")
                .and_then(|c| c.arg("file"))
                .and_then(|c| c.arg("push"))
                .and_then(|c| c.arg(src_arg))
                .and_then(|c| c.arg(format!("{container}:{dest}").as_str()))
                .and_then(|cmd| cmd.execute())?;
        }

        info!("Provisioned certificates for {container}");
        Ok(())
    }

    async fn bootstrap_container(
        &self,
        container_name: &str,
        source_lxd: Option<&str>,
    ) -> Result<()> {
        self.bootstrap_container_inner(container_name, source_lxd).await
    }
    async fn cleanup_existing(&self, container: &str) -> Result<()> {
        self.cleanup_existing_inner(container).await
    }
    async fn copy_container(&self, source_remote: &str, name: &str) -> Result<()> {
        self.copy_container_inner(source_remote, name).await
    }
    async fn ensure_network(&self, container: &str) -> Result<()> {
        self.ensure_network_inner(container).await
    }
    async fn sync_data_to_container(&self, container: &str) -> Result<()> {
        self.sync_data_to_container_inner(container).await
    }
    async fn fix_permissions(&self, container: &str) -> Result<()> {
        self.fix_permissions_inner(container).await
    }
    async fn install_systemd_service(&self, container: &str) -> Result<()> {
        self.install_systemd_service_inner(container).await
    }
}
#[cfg(test)]
mod ssh_verification_tests {
    /// #2620 — no provisioning helper may disable SSH host-key verification.
    /// The unreachable SSH/SCP helper that did so was removed; a regression
    /// guard keeps it out of the tree.
    #[test]
    fn no_disabled_host_key_checking_in_package_manager() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/package_manager");
        fn walk(dir: &std::path::Path, bad: &mut Vec<String>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() { walk(&p, bad); }
                    else if let Ok(text) = std::fs::read_to_string(&p) {
                        let needle = concat!("StrictHostKeyChecking=", "no");
                        if text.contains(needle) {
                            bad.push(p.display().to_string());
                        }
                    }
                }
            }
        }
        let mut bad = Vec::new();
        walk(&root, &mut bad);
        assert!(bad.is_empty(), "host-key verification disabled in: {bad:?}");
    }
}

#[cfg(test)]
mod staged_push_guard_tests {
    const SOURCE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/package_manager/container/configure_iptables_nat.rs"
    );
    /// #2610/#2709 — a tampered pin must refuse the staged bytes *and* leave
    /// nothing behind, because the privileged `incus file push` is never
    /// reached on this path. Using the real staging helper proves cleanup on
    /// the failure path without a container or a network download.
    #[test]
    fn mismatched_pin_refuses_the_staged_binary_and_cleans_up() {
        let staged = crate::package_manager::safe_staging::StagedFile::create(
            "container-binary",
            b"downloaded payload",
        )
        .expect("stage downloaded bytes");
        let staging_dir = staged
            .as_path()
            .parent()
            .expect("staging directory")
            .to_path_buf();
        let error = botcorepkg::checksum::verify_with_policy(
            staged.as_path(),
            &"0".repeat(64),
            "container-binary",
            true,
        )
        .expect_err("a mismatched pin must refuse the staged binary");
        assert!(error.to_string().contains("checksum mismatch"));
        assert!(staging_dir.exists(), "staging survives until the guard returns");
        drop(staged);
        assert!(
            !staging_dir.exists(),
            "no staged binary may survive a refused push"
        );
    }

    /// #2709 — the privileged push must resolve the pin and verify first.
    #[test]
    fn download_and_push_verifies_before_privileged_push() {
        let text = std::fs::read_to_string(SOURCE).expect("read container source");
        let start = text.find("async fn download_and_push_binary").expect("download helper");
        let end = text.find("async fn provision_certs_to_container").expect("next helper");
        let body = &text[start..end];
        assert!(body.contains("checksum_for_url(url)"), "component pin must be resolved");
        assert!(body.contains("unpinned_checksum_error"), "unpinned downloads must fail closed");
        let verify = body.find("verify_download").expect("verify_download call");
        let push = body.find("arg(\"push\")").expect("incus file push call");
        assert!(verify < push, "checksum must be verified before the privileged push");
    }
}
