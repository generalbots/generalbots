//! PostgreSQL cluster bootstrap for a failed first install (#2021 follow-up).
//!
//! Split out of `with_base_path.rs` to keep both files under the 450-line
//! limit; the method is `pub(super)` so `PackageManager::start` still calls it.

use super::*;

impl PackageManager {
    /// Creates the PostgreSQL cluster when `initdb` never ran (the first
    /// install failed after the binaries were extracted, or `pgdata` was
    /// removed). Only used for local installs: container installs create the
    /// cluster inside the container through the install commands.
    #[cfg(unix)]
    pub(super) fn ensure_tables_cluster(
        bin_path: &std::path::Path,
        data_path: &std::path::Path,
        logs_path: &std::path::Path,
    ) -> Result<()> {
        let pgdata = data_path.join("pgdata");
        if pgdata.exists() {
            return Ok(());
        }
        let initdb = bin_path.join("bin").join("initdb");
        if !initdb.exists() {
            return Err(anyhow::anyhow!(
                "the database binaries are missing ({}); reinstall the database component",
                initdb.display()
            ));
        }
        std::fs::create_dir_all(logs_path)?;

        // theseus-rs PostgreSQL 18.x bundles libstdc++ from Debian 12, which
        // needs glibc 2.36 (`arc4random`). Ubuntu 22.04 ships glibc 2.35, so
        // the dynamic loader aborts before initdb runs. Drop the bundled copy
        // only when it cannot load, so the host libstdc++ is used instead.
        let bundled_stdcpp = bin_path.join("lib").join("libstdc++.so.6");
        if bundled_stdcpp.exists() {
            let probe = SafeCommand::new("bash")
                .and_then(|c| c.arg("-c"))
                .and_then(|c| c.trusted_shell_script_arg("./bin/initdb --version"))
                .and_then(|c| c.working_dir(bin_path))
                .map_err(|e| anyhow::anyhow!("could not probe initdb: {e}"))?;
            if let Ok(output) = probe.execute() {
                if !output.status.success() {
                    std::fs::remove_file(&bundled_stdcpp).map_err(|e| {
                        anyhow::anyhow!(
                            "could not replace the bundled libstdc++ ({}): {e}",
                            bundled_stdcpp.display()
                        )
                    })?;
                }
            }
        }

        let password = database_password();
        let pwfile = logs_path.join("pwfile");
        std::fs::write(&pwfile, format!("{password}\n"))
            .map_err(|e| anyhow::anyhow!("could not write {}: {e}", pwfile.display()))?;

        // Reuse the root-aware wrapper written by the install step: initdb
        // refuses to run as root, and server installs start as root.
        let pgwrap = bin_path.join("pgwrap.sh");
        let init = format!(
            "cd '{}' && ./bin/initdb -D '{}' -U gbuser --pwfile='{}'",
            bin_path.display(),
            pgdata.display(),
            pwfile.display()
        );
        let script = if pgwrap.exists() {
            format!("'{}' \"{init}\"", pgwrap.display())
        } else {
            init
        };
        let output = SafeCommand::new("bash")
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.trusted_shell_script_arg(&script))
            .and_then(|c| c.working_dir(bin_path))
            .map_err(|e| anyhow::anyhow!("could not run initdb: {e}"))?
            .execute()
            .map_err(|e| anyhow::anyhow!("could not run initdb: {e}"))?;
        let _ = std::fs::remove_file(&pwfile);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr: String = stderr.trim().chars().take(600).collect();
            return Err(anyhow::anyhow!(
                "the database cluster could not be initialised (initdb exit {:?}): {}",
                output.status.code(),
                if stderr.is_empty() { "<no stderr>" } else { stderr.as_str() }
            ));
        }
        Ok(())
    }
}

/// Resolves the database password for a fresh cluster exactly like the install
/// commands resolve `{{DB_PASSWORD}}`.
#[cfg(unix)]
fn database_password() -> String {
    if let Ok(password) = std::env::var("BOOTSTRAP_DB_PASSWORD") {
        if !password.is_empty() {
            return password;
        }
    }
    match crate::shared::utils::get_database_url_sync() {
        Ok(url) => crate::shared::utils::parse_database_url(&url).1,
        Err(_) => String::new(),
    }
}
