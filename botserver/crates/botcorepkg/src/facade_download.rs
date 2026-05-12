use crate::cache::{CacheResult, DownloadCache};
use crate::component::ComponentConfig;
use crate::db_utils::{get_database_url_sync, parse_database_url};
use crate::OsType;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{error, info, trace, warn};
use reqwest::Client;
use std::path::PathBuf;

fn safe_apt_get(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("apt-get")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

fn safe_brew(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("brew")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

fn safe_tar(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("tar")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
    let url = url.to_string();
    let output_path = output_path.to_string();
    let download_handle = tokio::spawn(async move {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
            .connect_timeout(std::time::Duration::from_secs(30))
            .read_timeout(std::time::Duration::from_secs(300))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()?;
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let mut file = tokio::fs::File::create(&output_path).await?;
            use tokio::io::AsyncWriteExt;
            file.write_all(&bytes).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("HTTP {}: {}", response.status(), url))
        }
    });
    download_handle.await?
}

pub fn exec_in_container(container: &str, command: &str) -> Result<()> {
    info!("Executing in container {}: {}", container, command);
    let output = std::process::Command::new("lxc")
        .args(["exec", container, "--", "bash", "-c", command])
        .output()
        .ok()
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
            if stderr.is_empty() { stdout.to_string() } else { stderr.to_string() }
        ));
    }
    Ok(())
}

pub fn download_in_container(container: &str, url: &str, _component: &str, binary_name: Option<&str>) -> Result<()> {
    let download_cmd = format!("wget -O /tmp/download.tmp {}", url);
    exec_in_container(container, &download_cmd)?;
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
        exec_in_container(container, "tar -xzf /tmp/download.tmp -C /opt/gbo/bin")?;
    } else if is_zip {
        exec_in_container(container, "unzip -o /tmp/download.tmp -d /opt/gbo/bin")?;
    } else if let Some(name) = binary_name {
        let mv_cmd = format!(
            "mv /tmp/download.tmp /opt/gbo/bin/{} && chmod +x /opt/gbo/bin/{}",
            name, name
        );
        exec_in_container(container, &mv_cmd)?;
    }
    exec_in_container(container, "rm -f /tmp/download.tmp")?;
    Ok(())
}

pub fn create_directories(base_path: &std::path::Path, component: &str) -> Result<()> {
    for dir in &["bin", "data", "conf", "logs"] {
        let path = base_path.join(dir).join(component);
        std::fs::create_dir_all(&path)
            .context(format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

pub fn install_system_packages(component: &ComponentConfig, os_type: &OsType) -> Result<()> {    let packages = match os_type {
        OsType::Linux => &component.linux_packages,
        OsType::MacOS => &component.macos_packages,
        OsType::Windows => &component.windows_packages,
    };
    if packages.is_empty() {
        return Ok(());
    }
    trace!(
        "Installing {} system packages for component '{}'",
        packages.len(),
        component.name
    );
    match os_type {
        OsType::Linux => {
            if let Some(output) = safe_apt_get(&["update"]) {
                if !output.status.success() {
                    warn!("apt-get update had issues");
                }
            }
            let mut args = vec!["install", "-y"];
            for pkg in packages {
                args.push(pkg.as_str());
            }
            if let Some(output) = safe_apt_get(&args) {
                if !output.status.success() {
                    warn!("Some packages may have failed to install");
                }
            }
        }
        OsType::MacOS => {
            let mut args = vec!["install"];
            for pkg in packages {
                args.push(pkg.as_str());
            }
            if let Some(output) = safe_brew(&args) {
                if !output.status.success() {
                    warn!("Homebrew installation had warnings");
                }
            }
        }
        OsType::Windows => {
            warn!("Windows package installation not implemented");
        }
    }
    Ok(())
}

pub async fn download_and_install(base_path: &std::path::Path, url: &str, component: &str, binary_name: Option<&str>) -> Result<()> {
    let bin_path = base_path.join("bin").join(component);
    std::fs::create_dir_all(&bin_path)?;
    let cache_base = base_path.parent().unwrap_or(base_path);
    let cache = match DownloadCache::new(cache_base) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to initialize download cache: {}", e);
            match DownloadCache::new(base_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to create fallback cache: {}", e);
                    return Err(anyhow::anyhow!("Failed to create download cache"));
                }
            }
        }
    };
    let cache_result = cache.resolve_component_url(component, url);
    let source_file = match cache_result {
        CacheResult::Cached(cached_path) => {
            info!("Using cached file for {}: {}", component, cached_path.display());
            cached_path
        }
        CacheResult::Download { url: download_url, cache_path } => {
            info!("Downloading {} from {}", component, download_url);
            println!("Downloading {}", download_url);
            download_with_reqwest(&download_url, &cache_path, component).await?;
            info!("Cached {} to {}", component, cache_path.display());
            cache_path
        }
    };
    handle_downloaded_file(&source_file, &bin_path, binary_name)?;
    Ok(())
}

pub async fn download_with_reqwest(url: &str, target_file: &std::path::Path, component: &str) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);
    if let Some(parent) = target_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("botserver-package-manager/1.0")
        .build()?;
    let mut last_error = None;
    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            trace!("Retry attempt {}/{} for {}", attempt, MAX_RETRIES, component);
            std::thread::sleep(RETRY_DELAY * attempt);
        }
        match attempt_reqwest_download(&client, url, target_file).await {
            Ok(_size) => {
                if attempt > 0 {
                    trace!("Download succeeded on retry attempt {}", attempt);
                }
                return Ok(());
            }
            Err(e) => {
                warn!("Download attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
                let _ = std::fs::remove_file(target_file);
            }
        }
    }
    Err(anyhow::anyhow!(
        "Failed to download {} after {} attempts. Last error: {}",
        component,
        MAX_RETRIES + 1,
        last_error.unwrap_or_else(|| anyhow::anyhow!("unknown error"))
    ))
}

pub async fn attempt_reqwest_download(_client: &Client, url: &str, temp_file: &std::path::Path) -> Result<u64> {
    let output_path = temp_file.to_str().context("Invalid temp file path")?;
    download_file(url, output_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to download file using shared utility: {}", e))?;
    let metadata = std::fs::metadata(temp_file).context("Failed to get file metadata")?;
    let size = metadata.len();
    Ok(size)
}

pub fn handle_downloaded_file(temp_file: &std::path::Path, bin_path: &std::path::Path, binary_name: Option<&str>) -> Result<()> {
    let metadata = std::fs::metadata(temp_file)?;
    if metadata.len() == 0 {
        return Err(anyhow::anyhow!("Downloaded file is empty"));
    }
    let file_extension = temp_file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    match file_extension {
        "gz" | "tgz" => {
            extract_tar_gz(temp_file, bin_path)?;
        }
        "zip" => {
            extract_zip(temp_file, bin_path)?;
        }
        _ => {
            if let Some(name) = binary_name {
                install_binary(temp_file, bin_path, name)?;
            } else {
                let final_path = bin_path.join(temp_file.file_name().unwrap_or_default());
                if temp_file.to_string_lossy().contains("botserver-installers") {
                    std::fs::copy(temp_file, &final_path)?;
                } else {
                    std::fs::rename(temp_file, &final_path)?;
                }
                make_executable(&final_path)?;
            }
        }
    }
    Ok(())
}

pub fn extract_tar_gz(temp_file: &std::path::Path, bin_path: &std::path::Path) -> Result<()> {
    let temp_file_str = temp_file.to_str().unwrap_or_default();
    let list_output = safe_tar(&["-tzf", temp_file_str]);
    let list_output = match list_output {
        Some(o) => o,
        None => return Err(anyhow::anyhow!("Failed to execute tar list command")),
    };
    let has_subdir = if list_output.status.success() {
        let contents = String::from_utf8_lossy(&list_output.stdout);
        contents.lines().next().map(|l| l.contains('/')).unwrap_or(false)
    } else {
        false
    };
    let mut args = vec!["-xzf", temp_file.to_str().unwrap_or_default()];
    if has_subdir {
        args.push("--strip-components=1");
    }
    let output = SafeCommand::new("tar")
        .and_then(|c| c.args(&args))
        .and_then(|c| c.working_dir(bin_path))
        .map_err(|e| anyhow::anyhow!("Failed to build tar command: {}", e))?
        .execute()
        .map_err(|e| anyhow::anyhow!("Failed to execute tar: {}", e))?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "tar extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !temp_file.to_string_lossy().contains("botserver-installers") {
        std::fs::remove_file(temp_file)?;
    }
    Ok(())
}

pub fn extract_zip(temp_file: &std::path::Path, bin_path: &std::path::Path) -> Result<()> {
    let temp_file_str = temp_file.to_str().unwrap_or_default();
    let output = SafeCommand::new("unzip")
        .and_then(|c| c.args(&["-o", "-q", temp_file_str]))
        .and_then(|c| c.working_dir(bin_path))
        .map_err(|e| anyhow::anyhow!("Failed to build unzip command: {}", e))?
        .execute()
        .map_err(|e| anyhow::anyhow!("Failed to execute unzip: {}", e))?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "unzip extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(entries) = std::fs::read_dir(bin_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        let mut perms = metadata.permissions();
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ext.is_empty() || ext == "sh" || ext == "bash" {
                            perms.set_mode(0o755);
                            let _ = std::fs::set_permissions(&path, perms);
                            trace!("Made executable: {}", path.display());
                        }
                    }
                }
            }
        }
    }
    if !temp_file.to_string_lossy().contains("botserver-installers") {
        std::fs::remove_file(temp_file)?;
    }
    Ok(())
}

pub fn install_binary(temp_file: &std::path::Path, bin_path: &std::path::Path, name: &str) -> Result<()> {
    let final_path = bin_path.join(name);
    if temp_file.to_string_lossy().contains("botserver-installers") {
        std::fs::copy(temp_file, &final_path)?;
    } else {
        std::fs::rename(temp_file, &final_path)?;
    }
    make_executable(&final_path)?;
    Ok(())
}

pub fn make_executable(path: &std::path::Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

pub fn run_commands(commands: &[String], target: &str, component: &str, base_path: &std::path::Path, db_password_override: &str) -> Result<()> {
    run_commands_with_password(commands, target, component, base_path, db_password_override)
}

pub fn run_commands_with_password(
    commands: &[String], target: &str, component: &str,
    base_path: &std::path::Path, db_password_override: &str,
) -> Result<()> {
    let bin_path = if target == "local" {
        base_path.join("bin").join(component)
    } else {
        PathBuf::from("/opt/gbo/bin")
    };
    let data_path = if target == "local" {
        base_path.join("data").join(component)
    } else {
        PathBuf::from("/opt/gbo/data")
    };
    let conf_path = if target == "local" {
        base_path.join("conf")
    } else {
        PathBuf::from("/opt/gbo/conf")
    };
    let logs_path = if target == "local" {
        base_path.join("logs").join(component)
    } else {
        PathBuf::from("/opt/gbo/logs")
    };
    let db_password = if let Ok(env_pwd) = std::env::var("BOOTSTRAP_DB_PASSWORD") {
        env_pwd
    } else if !db_password_override.is_empty() {
        db_password_override.to_string()
    } else {
        match get_database_url_sync() {
            Ok(url) => {
                let (_, password, _, _, _) = parse_database_url(&url);
                password
            }
            Err(_) => {
                trace!("Vault not available for DB_PASSWORD, using empty string");
                String::new()
            }
        }
    };
    for cmd in commands {
        let rendered_cmd = cmd
            .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
            .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
            .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
            .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy())
            .replace("{{DB_PASSWORD}}", &db_password);
        if target == "local" {
            trace!("Executing command: {}", rendered_cmd);
            let cmd = SafeCommand::new("bash")
                .and_then(|c| c.arg("-c"))
                .and_then(|c| c.trusted_shell_script_arg(&rendered_cmd))
                .and_then(|c| c.working_dir(&bin_path))
                .map_err(|e| anyhow::anyhow!("Failed to build bash command: {}", e))?;
            let output = cmd.execute().with_context(|| {
                format!("Failed to execute command for component '{}'", component)
            })?;
            if !output.status.success() {
                error!("Command had non-zero exit: {}", String::from_utf8_lossy(&output.stderr));
            }
        } else {
            exec_in_container(target, &rendered_cmd)?;
        }
    }
    Ok(())
}
