use super::*;

/// Build the in-container `chown` argv used by `fix_permissions_inner`.
///
/// Every value stays its own argv element and no shell is involved, so a
/// settings-derived value containing shell metacharacters is handed to `chown`
/// verbatim instead of being interpreted by `sh -c` (issue #2623).
fn chown_argv(owner: &str, path: &str) -> Vec<String> {
    vec![
        "chown".to_string(),
        "-R".to_string(),
        owner.to_string(),
        path.to_string(),
    ]
}

impl PackageManager {

    pub(crate) async fn bootstrap_container_inner(
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

        // 5. Provision TLS certificates
        self.provision_certs_to_container(container_name).await?;

        // 6. Install and start service
        self.install_systemd_service(container_name).await?;

        // 7. Configure NAT rules on host (ONLY iptables, never socat)
        self.configure_iptables_nat(container_name).await?;

        // 8. Reload DNS if dns container
        if container_name == "dns" {
            self.reload_dns_zones().await?;
        }

        info!("Container {container_name} bootstrapped successfully");
        Ok(())
    }

    pub(crate) async fn cleanup_existing_inner(&self, container: &str) -> Result<()> {
        // Remove socat processes
        if let Err(e) = SafeCommand::new("pkill")
            .and_then(|c| c.arg("-9"))
            .and_then(|c| c.arg("-f"))
            .and_then(|c| c.arg("socat"))
            .and_then(|cmd| cmd.execute())
        {
            log::warn!("Could not kill leftover socat processes: {e}");
        }

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
                    if let Err(e) = SafeCommand::new("incus")
                        .and_then(|c| c.arg("config"))
                        .and_then(|c| c.arg("device"))
                        .and_then(|c| c.arg("remove"))
                        .and_then(|c| c.arg(container))
                        .and_then(|c| c.arg(name))
                        .and_then(|cmd| cmd.execute())
                    {
                        log::warn!("Could not remove stale Incus device {name}: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn copy_container_inner(&self, source_remote: &str, name: &str) -> Result<()> {
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

    pub(crate) async fn ensure_network_inner(&self, container: &str) -> Result<()> {
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

    pub(crate) async fn sync_data_to_container_inner(&self, container: &str) -> Result<()> {
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

    pub(crate) async fn fix_permissions_inner(&self, container: &str) -> Result<()> {
        let settings = self.get_container_settings(container)?;

        let owner = match &settings.group {
            Some(group) => format!("{}:{}", settings.user, group),
            None => format!("{}:{}", settings.user, settings.user),
        };

        // #2623 — no `sh -c`: each value is a distinct argv element, so a
        // settings-derived owner/path can never be interpreted by a shell.
        let mut cmd_builder: Result<SafeCommand, botlib::security::command_guard::CommandGuardError> =
            SafeCommand::new("incus")
                .and_then(|c| c.arg("exec"))
                .and_then(|c| c.arg(container))
                .and_then(|c| c.arg("--"));
        for arg in chown_argv(&owner, "/opt/gbo/") {
            cmd_builder = cmd_builder.and_then(|c| c.arg(arg));
        }
        cmd_builder?.execute()?;

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

    pub(crate) async fn install_systemd_service_inner(&self, container: &str) -> Result<()> {
        let settings = self.get_container_settings(container)?;

        let service_name = format!("{container}.service");
        // #2616 — private unpredictable staging; create_new + 0600 refuses
        // pre-created files/symlinks, and the guard cleans up after the push.
        let staged = crate::package_manager::safe_staging::StagedFile::create(
            &service_name,
            settings.service_template.as_bytes(),
        )
        .context("Failed to stage service template")?;

    let target_service_path = format!("{container}:/etc/systemd/system/{service_name}");
        SafeCommand::new("incus")
            .and_then(|c| c.arg("file"))
            .and_then(|c| c.arg("push"))
            .and_then(|c| c.arg(staged.as_path().to_string_lossy().as_ref()))
            .and_then(|c| c.arg(target_service_path.as_str()))
            .and_then(|cmd| cmd.execute())?;

        let commands: Vec<Vec<&str>> = vec![
            vec!["daemon-reload"],
            vec!["enable", service_name.as_str()],
            vec!["start", service_name.as_str()],
        ];
        for cmd_args in commands {
            let mut cmd_builder: Result<SafeCommand, botlib::security::command_guard::CommandGuardError> = SafeCommand::new("incus")
                .and_then(|c| c.arg("exec"))
                .and_then(|c| c.arg(container))
                .and_then(|c| c.arg("--"))
                .and_then(|c| c.arg("systemctl"));

            for arg in cmd_args {
                cmd_builder = cmd_builder.and_then(|c| c.arg(arg));
            }
            cmd_builder?.execute()?;
        }

        // #2616 — StagedFile Drop removes the private staging directory.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::chown_argv;

    #[test]
    fn chown_argv_passes_shell_metacharacters_verbatim() {
        // A settings-derived value that would run arbitrary commands if a
        // shell were involved. It must reach `chown` as one literal argument.
        let evil_path = "/opt/gbo/; touch /tmp/pwned $(id) `whoami`";
        let owner = "gbuser:gbgroup; touch /tmp/pwned";

        let args = chown_argv(owner, evil_path);

        // The program after `--` is chown itself — there is no `sh -c`.
        assert_eq!(args.first().map(String::as_str), Some("chown"));
        assert!(args.iter().all(|a| a != "sh" && a != "-c"));

        // The owner and the path survive as single, distinct argv elements,
        // byte-for-byte, with no expansion or splitting by a shell.
        assert_eq!(args.get(2).map(String::as_str), Some(owner));
        assert_eq!(args.get(3).map(String::as_str), Some(evil_path));
        assert_eq!(args.len(), 4);

        // No element merged the two values into one shell command string.
        assert!(!args.iter().any(|a| a.contains(evil_path) && a.contains(owner)));
    }
}
