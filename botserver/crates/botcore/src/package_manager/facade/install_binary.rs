use super::*;

impl PackageManager {
    /// Installs a single binary from `temp_file` into `bin_path/name`.
    ///
    /// When `preserve_source` is true the source file is copied instead of
    /// moved, so a read-only bundled-installers directory is never modified.
    pub fn install_binary(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
        name: &str,
        preserve_source: bool,
    ) -> Result<()> {
        #[cfg(target_os = "windows")]
        let name = if !name.ends_with(".exe") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        #[cfg(not(target_os = "windows"))]
        let name = name.to_string();
        let final_path = bin_path.join(name);

        if preserve_source {
            std::fs::copy(temp_file, &final_path)?;
        } else {
            std::fs::rename(temp_file, &final_path)?;
        }
        self.make_executable(&final_path)?;
        Ok(())
    }
    pub fn make_executable(&self, path: &std::path::Path) -> Result<()> {
        #[cfg(unix)]
        {
            if let Err(e) = botlib::os::fs::get_permissions_manager().set_executable(path) {
                log::warn!("Could not make {} executable: {e}", path.display());
            }
        }
        #[cfg(windows)]
        {
            trace!("Skipping chmod on Windows for: {}", path.display());
        }
        Ok(())
    }
    pub fn run_commands(&self, commands: &[String], target: &str, component: &str) -> Result<()> {
        self.run_commands_with_password(commands, target, component, "")
    }

    pub fn run_commands_with_password(&self, commands: &[String], target: &str, component: &str, db_password_override: &str) -> Result<()> {
        let bin_path = if target == "local" {
            self.base_path.join("bin").join(component)
        } else {
            PathBuf::from("/opt/gbo/bin")
        };
        let data_path = if target == "local" {
            self.base_path.join("data").join(component)
        } else {
            PathBuf::from("/opt/gbo/data")
        };

        let conf_path = if target == "local" {
            self.base_path.join("conf")
        } else {
            PathBuf::from("/opt/gbo/conf")
        };
        let logs_path = if target == "local" {
            self.base_path.join("logs").join(component)
        } else {
            PathBuf::from("/opt/gbo/logs")
        };

        let db_password = if !commands.iter().any(|cmd| cmd.contains("{{DB_PASSWORD}}")) {
            String::new()
        } else if let Ok(env_pwd) = std::env::var("BOOTSTRAP_DB_PASSWORD") {
            env_pwd
        } else if !db_password_override.is_empty() {
            db_password_override.to_string()
        } else {
            let from_url = match get_database_url_sync() {
                Ok(url) => parse_database_url(&url).1,
                Err(error) => {
                    warn!("cached database URL unavailable for {component}: {error}");
                    String::new()
                }
            };
            if from_url.is_empty() {
                // The cached URL can lack a password during first boot; read
                // the strict Vault field instead so installer SQL never runs
                // with PGPASSWORD=''.
                get_secret_value_strict_sync("secret/gbo/tables", "password")
                    .unwrap_or_else(|error| {
                        warn!("Vault DB password unavailable for {component}: {error}");
                        String::new()
                    })
            } else {
                from_url
            }
        };
        // The directory installer's SQL also carries the Zitadel role
        // password. It must be substituted here too; leaving the placeholder
        // in the rendered command creates a role with a literal placeholder
        // password and Zitadel can never authenticate.
        let zitadel_password = if commands
            .iter()
            .any(|cmd| cmd.contains("{{ZITADEL_PASSWORD}}"))
        {
            get_secret_value_strict_sync("secret/gbo/directory", "password")
                .map_err(|error| anyhow::anyhow!("Cannot resolve Zitadel DB password: {error}"))?
        } else {
            String::new()
        };

        for cmd in commands {
            let rendered_cmd = cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy())
                .replace("{{STACK_PATH}}", &self.base_path.to_string_lossy())
                .replace("{{DATA_PATH_FWD}}", &data_path.to_string_lossy().replace('\\', "/"))
                .replace("{{DB_PASSWORD}}", &db_password)
                .replace("{{ZITADEL_PASSWORD}}", &zitadel_password);
            if target == "local" {
                trace!("Executing command: {}", rendered_cmd);
                #[cfg(target_os = "windows")]
                let cmd_result = {
                    let wrapped = format!("\"{rendered_cmd}\"");
                    let cmd = SafeCommand::new("cmd")
                        .and_then(|c| c.arg("/C"))
                        .and_then(|c| c.raw_shell_script_arg(&wrapped))
                        .and_then(|c| c.working_dir(&bin_path))
                        .map_err(|e| anyhow::anyhow!("Failed to build cmd command: {}", e))?;
                    let output = cmd.execute().with_context(|| {
                        format!("Failed to execute command for component '{}'", component)
                    })?;
                    if !output.status.success() {
                        error!(
                            "Command '{}' failed. Stderr: {}",
                            rendered_cmd,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    output
                };
                #[cfg(not(target_os = "windows"))]
                let cmd_result = {
                    let cmd = SafeCommand::new("bash")
                        .and_then(|c| c.arg("-c"))
                        .and_then(|c| c.trusted_shell_script_arg(&rendered_cmd))
                        .and_then(|c| c.working_dir(&bin_path))
                        .map_err(|e| anyhow::anyhow!("Failed to build bash command: {}", e))?;
                    cmd.execute().with_context(|| {
                        format!("Failed to execute command for component '{}'", component)
                    })?
                };
                if !cmd_result.status.success() {
                    // A post-install step that fails used to be only logged,
                    // so the component was still reported as installed and the
                    // wizard waited for a service that could never start (the
                    // PostgreSQL cluster stayed without pgdata and pg_ctl only
                    // printed "directory ... does not exist"). Echo to stderr
                    // so the provisioning log carries the cause, and fail the
                    // install unless the registry explicitly tolerates failure
                    // with a trailing `|| true`.
                    let stderr = String::from_utf8_lossy(&cmd_result.stderr);
                    let stderr: String = stderr.trim().chars().take(600).collect();
                    let display_cmd = redact_secret(&rendered_cmd, &db_password);
                    eprintln!(
                        "installer: command failed for component '{}' (exit {:?})\n  command: {}\n  stderr: {}",
                        component,
                        cmd_result.status.code(),
                        display_cmd,
                        if stderr.is_empty() { "<empty>" } else { stderr.as_str() }
                    );
                    error!(
                        "Command failed for component '{}' (exit {:?}): {}\nCommand: {}",
                        component,
                        cmd_result.status.code(),
                        if stderr.is_empty() { "<empty>" } else { stderr.as_str() },
                        display_cmd
                    );
                    // A failed prerequisite must stop setup on every platform.
                    if !rendered_cmd.trim_end().ends_with("|| true") {
                        return Err(anyhow::anyhow!(
                            "'{}' failed while installing {} (exit {:?}): {}",
                            display_cmd,
                            component,
                            cmd_result.status.code(),
                            if stderr.is_empty() { "<no stderr>" } else { stderr.as_str() }
                        ));
                    }
                }
            } else {
                self.exec_in_container(target, &rendered_cmd)?;
            }
        }
        Ok(())
    }
    pub fn exec_in_container(&self, container: &str, command: &str) -> Result<()> {
        info!("Executing in container {}: {}", container, command);
        let output = safe_incus_exec_in_container(container, command)
            .ok_or_else(|| anyhow::anyhow!("Failed to execute lxc command"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!(
                "Container command failed.\nCommand: {}\nStderr: {}\nStdout: {}",
                command, stderr, stdout
            );
            return Err(anyhow::anyhow!(
                "Container command failed: {}",
                if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                }
            ));
        }
        Ok(())
    }
    /// Fail-closed verdict for an unpinned component (#2622). Pure so the
    /// policy is unit-testable without mutating process environment.
    fn unpinned_checksum_error(component: &str, required: bool) -> Option<String> {
        if required {
            Some(format!(
                "no SHA-256 configured for '{component}' and GB_REQUIRE_CHECKSUMS is on; \
                 refusing to install an unverified download"
            ))
        } else {
            None
        }
    }

    pub fn download_in_container(
        &self,
        container: &str,
        url: &str,
        component: &str,
        binary_name: Option<&str>,
    ) -> Result<()> {
        let download_cmd = format!("wget -O /tmp/download.tmp {}", url);
        self.exec_in_container(container, &download_cmd)?;
        // #2001 — verify the fetched asset before unpacking it into
        // /opt/gbo/bin (the container path had no checksum at all).
        // #2622 — handle the missing pin explicitly. A default fallback
        // silently turned "no pin configured" into an empty expected hash,
        // blurring the policy boundary with the cache path which refuses to
        // treat an empty sha256 as verified. Now the unpinned case is an
        // explicit branch with the same fail-closed verdict under
        // GB_REQUIRE_CHECKSUMS as the cache installer path.
        let expected = botcorepkg::installer::checksum_for_url(url);
        match expected.as_deref() {
            Some(hash) if !hash.is_empty() => {
                let verify_cmd = format!("echo '{hash}  /tmp/download.tmp' | sha256sum -c -");
                self.exec_in_container(container, &verify_cmd).map_err(|error| {
                    anyhow::anyhow!("checksum verification failed for '{component}': {error}")
                })?;
                info!("Checksum verified for '{}' in {}", component, container);
            }
            _ => {
                if let Some(message) = Self::unpinned_checksum_error(
                    component,
                    botcorepkg::checksum::require_checksums(),
                ) {
                    return Err(anyhow::anyhow!(message));
                }
                warn!(
                    "No SHA-256 checksum configured for '{}'; the container download is NOT verified",
                    component
                );
            }
        }
        let path = std::path::Path::new(url);
        let is_tar_gz = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("tgz"))
            || (path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
                && path
                    .file_stem()
                    .and_then(|s| std::path::Path::new(s).extension())
                    .is_some_and(|e| e.eq_ignore_ascii_case("tar")));
        let is_zip = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));
        if is_tar_gz {
            // Try tar, fallback to trying to find it in common paths
            self.exec_in_container(container, "/usr/bin/tar -xzf /tmp/download.tmp -C /opt/gbo/bin || tar -xzf /tmp/download.tmp -C /opt/gbo/bin")?;
        } else if is_zip {
            self.exec_in_container(container, "unzip -o /tmp/download.tmp -d /opt/gbo/bin")?;
        } else if let Some(name) = binary_name {
            let mv_cmd = format!(
                "mv /tmp/download.tmp /opt/gbo/bin/{} && chmod +x /opt/gbo/bin/{}",
                name, name
            );
            self.exec_in_container(container, &mv_cmd)?;
        }
        self.exec_in_container(container, "rm -f /tmp/download.tmp")?;
        Ok(())
    }
    pub fn mount_container_directories(&self, container: &str, component: &str) -> Result<()> {
        let host_base = format!("/opt/gbo/tenants/{}/{}", self.tenant, component);
        for dir in &["data", "conf", "logs"] {
            let host_path = format!("{}/{}", host_base, dir);
            std::fs::create_dir_all(&host_path)?;
            let device_name = format!("{}-{}", component, dir);
            let container_path = format!("/opt/gbo/{}", dir);
            drop(safe_incus(&["config", "device", "remove", container, &device_name]));
            let source_arg = format!("source={}", host_path);
            let path_arg = format!("path={}", container_path);
            let output = safe_incus(&[
                "config",
                "device",
                "add",
                container,
                &device_name,
                "disk",
                &source_arg,
                &path_arg,
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
            trace!(
                "Mounted {} to {} in container {}",
                host_path,
                container_path,
                container
            );
        }
        Ok(())
    }
    pub fn create_container_service(
        &self,
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
                trace!(
                    "Vault not available for DB_PASSWORD in container service, using empty string"
                );
                String::new()
            }
        };

        let rendered_cmd = exec_cmd
            .replace("{{DB_PASSWORD}}", &db_password)
            .replace(botcorepkg::secrets_engine::CLI_PLACEHOLDER, botcorepkg::secrets_engine::Engine::OpenBao.binary_name())
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
            if let Err(e) = writeln!(env_section, "Environment={key}={rendered_value}") { log::debug!("cannot write environment section: {e}"); }
        }
        let service_content = format!(
            "[Unit]\nDescription={} Service\nAfter=network.target\n\n[Service]\nType=simple\n{}ExecStart={}\nWorkingDirectory=/opt/gbo/data\nRestart=always\nRestartSec=10\nUser=root\n\n[Install]\nWantedBy=multi-user.target\n",
            component, env_section, rendered_cmd
        );
        // #2616 — private unpredictable staging, no predictable /tmp name.
        let service_name = format!("{component}.service");
        let staged = crate::package_manager::safe_staging::StagedFile::create(
            &service_name,
            service_content.as_bytes(),
        )?;
        let dest = format!("{container}/etc/systemd/system/{component}.service");
        let service_file = staged.as_path().to_string_lossy().to_string();
        let output = safe_incus(&["file", "push", &service_file, &dest])
            .ok_or_else(|| anyhow::anyhow!("Failed to execute lxc file push command"))?;
        if !output.status.success() {
            warn!("Failed to push service file to container");
        }
        self.exec_in_container(container, "systemctl daemon-reload")?;
        self.exec_in_container(container, &format!("systemctl enable {}", component))?;
        self.exec_in_container(container, &format!("systemctl start {}", component))?;
        drop(staged); // #2616 — clean up the private staging directory
        trace!(
            "Created and started service in container {}: {}",
            container,
            component
        );
        Ok(())
    }
}

/// Replaces the resolved database password in a command before it is printed
/// to the provisioning log or returned in an error message.
fn redact_secret(command: &str, secret: &str) -> String {
    if secret.is_empty() {
        command.to_string()
    } else {
        command.replace(secret, "***")
    }
}

#[cfg(test)]
mod unpinned_checksum_tests {
    use super::PackageManager;

    #[test]
    fn unpinned_checksum_fails_closed_when_required() {
        let err = PackageManager::unpinned_checksum_error("demo", true)
            .expect("required checksums must refuse an unpinned component");
        assert!(err.contains("demo"));
        assert!(err.contains("GB_REQUIRE_CHECKSUMS"));
        assert!(err.contains("refusing to install an unverified download"));
    }

    #[test]
    fn unpinned_checksum_is_acknowledged_when_not_required() {
        assert!(PackageManager::unpinned_checksum_error("demo", false).is_none());
    }
}

#[cfg(test)]
mod preserve_source_tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper: construct a minimal `PackageManager` for unit tests.
    fn test_pm(base: &std::path::Path) -> PackageManager {
        PackageManager::with_base_path(
            InstallMode::Local,
            Some("test".to_string()),
            base.to_path_buf(),
        )
        .expect("test PackageManager")
    }

    // ---- install_binary ----

    #[test]
    fn install_binary_preserve_copies_and_keeps_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let installers = dir.path().join("installers");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&installers).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        let src = installers.join("otterio.exe");
        std::fs::write(&src, b"OTTERIO_BINARY").unwrap();
        let original_len = std::fs::metadata(&src).unwrap().len();

        let pm = test_pm(dir.path());
        pm.install_binary(&src, &bin, "otterio.exe", true).unwrap();

        // Source must still exist with the same size.
        assert!(src.exists(), "bundled source was deleted");
        assert_eq!(std::fs::metadata(&src).unwrap().len(), original_len);
        // Destination must exist.
        assert!(bin.join("otterio.exe").exists(), "binary not installed");
    }

    #[test]
    fn install_binary_no_preserve_moves_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let downloads = dir.path().join("downloads");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&downloads).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        let src = downloads.join("otterio.exe");
        std::fs::write(&src, b"OTTERIO_BINARY").unwrap();

        let pm = test_pm(dir.path());
        pm.install_binary(&src, &bin, "otterio.exe", false).unwrap();

        // Source should be gone (moved).
        assert!(!src.exists(), "downloaded source was not cleaned up");
        assert!(bin.join("otterio.exe").exists(), "binary not installed");
    }

    // ---- handle_downloaded_file: unrecognised extension, no binary_name ----

    #[test]
    fn handle_downloaded_file_preserve_copies_unknown_ext() {
        let dir = tempfile::tempdir().expect("tempdir");
        let installers = dir.path().join("installers");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&installers).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        let src = installers.join("somefile");
        std::fs::write(&src, b"DATA").unwrap();
        let original_len = std::fs::metadata(&src).unwrap().len();

        let pm = test_pm(dir.path());
        pm.handle_downloaded_file(&src, &bin, None, true).unwrap();

        assert!(src.exists(), "bundled source was deleted");
        assert_eq!(std::fs::metadata(&src).unwrap().len(), original_len);
    }

    // ---- extract_zip: bundled archive preserved, downloaded archive cleaned ----

    #[test]
    fn extract_zip_preserve_keeps_archive() {
        let dir = tempfile::tempdir().expect("tempdir");
        let installers = dir.path().join("installers");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&installers).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        // Create a minimal valid zip in the installers folder.
        let zip_path = installers.join("component.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("hello.txt", options).unwrap();
            std::io::Write::write_all(&mut writer, b"hello").unwrap();
            writer.finish().unwrap();
        }
        let original_len = std::fs::metadata(&zip_path).unwrap().len();

        let pm = test_pm(dir.path());
        pm.extract_zip(&zip_path, &bin, true).unwrap();

        assert!(zip_path.exists(), "bundled zip was deleted");
        assert_eq!(std::fs::metadata(&zip_path).unwrap().len(), original_len);
        assert!(bin.join("hello.txt").exists(), "zip not extracted");
    }

    #[test]
    fn extract_zip_no_preserve_removes_archive() {
        let dir = tempfile::tempdir().expect("tempdir");
        let downloads = dir.path().join("downloads");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&downloads).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        let zip_path = downloads.join("component.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("hello.txt", options).unwrap();
            std::io::Write::write_all(&mut writer, b"hello").unwrap();
            writer.finish().unwrap();
        }

        let pm = test_pm(dir.path());
        pm.extract_zip(&zip_path, &bin, false).unwrap();

        assert!(!zip_path.exists(), "downloaded zip was not cleaned up");
        assert!(bin.join("hello.txt").exists(), "zip not extracted");
    }

    // ---- read-only source directory (Windows: read-only attribute on files) ----

    #[cfg(target_os = "windows")]
    #[test]
    fn install_binary_works_when_source_is_read_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let installers = dir.path().join("installers");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&installers).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        let src = installers.join("otterio.exe");
        std::fs::write(&src, b"OTTERIO_BINARY").unwrap();

        // Set read-only attribute on the source file.
        let mut perms = std::fs::metadata(&src).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&src, perms).unwrap();

        let pm = test_pm(dir.path());
        let result = pm.install_binary(&src, &bin, "otterio.exe", true);
        assert!(result.is_ok(), "install failed on read-only source: {result:?}");
        assert!(src.exists(), "read-only source was deleted");
        assert!(bin.join("otterio.exe").exists(), "binary not installed");

        // Clean up: remove read-only so tempdir can be deleted.
        let mut perms = std::fs::metadata(&src).unwrap().permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(&src, perms).unwrap();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn extract_zip_works_when_source_is_read_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let installers = dir.path().join("installers");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&installers).unwrap();
        std::fs::create_dir_all(&bin).unwrap();

        let zip_path = installers.join("component.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("data.bin", options).unwrap();
            std::io::Write::write_all(&mut writer, b"payload").unwrap();
            writer.finish().unwrap();
        }

        let mut perms = std::fs::metadata(&zip_path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&zip_path, perms).unwrap();

        let pm = test_pm(dir.path());
        let result = pm.extract_zip(&zip_path, &bin, true);
        assert!(result.is_ok(), "extract failed on read-only source: {result:?}");
        assert!(zip_path.exists(), "read-only zip was deleted");
        assert!(bin.join("data.bin").exists(), "zip not extracted");

        let mut perms = std::fs::metadata(&zip_path).unwrap().permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(&zip_path, perms).unwrap();
    }
}
