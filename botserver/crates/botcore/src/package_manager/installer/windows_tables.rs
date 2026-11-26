//! The local PostgreSQL of the Windows desktop stack: start it, create the
//! application database, and make TCP logins require the per-install password
//! kept in Vault (`secret/gbo/tables`, field `password`).

use super::*;
use botcorepkg::desktop_hardening::{harden_pg_hba, pg_hba_is_hardened};
use std::path::Path;
use std::process::Output;

const PORT: u16 = 5432;

struct Tables<'a> {
    bin: &'a Path,
    pgdata: std::path::PathBuf,
    password: Option<String>,
}

impl Tables<'_> {
    fn sql(&self, statement: &str) -> Result<Output> {
        let mut command = SafeCommand::new(&self.bin.join("bin/psql.exe").to_string_lossy())?
            .args(&["-w", "-v", "ON_ERROR_STOP=1", "-h", "localhost", "-p", "5432", "-U", "gbuser", "-d", "postgres", "-tAc"])?
            .trusted_arg(statement)?
            .env("PGCONNECT_TIMEOUT", "5")?
            .env("PGOPTIONS", "-c statement_timeout=30000")?;
        if let Some(password) = &self.password {
            command = command.env("PGPASSWORD", password)?;
        }
        Ok(command.execute()?)
    }

    fn accepting(&self) -> bool {
        SafeCommand::new(&self.bin.join("bin/pg_isready.exe").to_string_lossy())
            .and_then(|c| c.args(&["-h", "localhost", "-p", "5432", "-t", "5"]))
            .and_then(|c| c.execute())
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn pg_ctl(&self, action: &str) -> Result<bool> {
        Ok(SafeCommand::new(&self.bin.join("bin/pg_ctl.exe").to_string_lossy())?
            .args(&["-D", &self.pgdata.to_string_lossy(), action, "-s"])?
            .execute()?
            .status
            .success())
    }

    /// Switches the TCP rules from `trust` to SCRAM: set the password while the
    /// server still trusts loopback, then rewrite `pg_hba.conf` and reload.
    /// Rolls back when a password login does not work afterwards.
    fn require_passwords(&self) -> Result<()> {
        let hba = self.pgdata.join("pg_hba.conf");
        let original = std::fs::read_to_string(&hba)?;
        let Some(hardened) = harden_pg_hba(&original) else { return Ok(()) };
        let Some(password) = self.password.as_deref() else { return Ok(()) };
        let Some(verifier) = botcorepkg::pg_scram::verifier(password) else {
            warn!("local database password has unsupported characters; logins stay as they are");
            return Ok(());
        };
        if !self.sql(&format!("ALTER ROLE gbuser WITH PASSWORD '{verifier}'"))?.status.success() {
            return Err(anyhow::anyhow!("Could not set the local database password"));
        }
        std::fs::write(&hba, hardened)?;
        let reloaded = self.pg_ctl("reload")?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        if reloaded && self.sql("SELECT 1")?.status.success() {
            info!("local database now requires a password for every connection");
            return Ok(());
        }
        std::fs::write(&hba, original)?;
        self.pg_ctl("reload")?;
        Err(anyhow::anyhow!("The local database rejected its password; the previous login rules were restored"))
    }
}

impl PackageManager {
    pub(super) fn start_windows_tables(bin: &Path, data: &Path, logs: &Path, stack: &Path) -> Result<std::process::Child> {
        let tables = Tables {
            bin,
            pgdata: data.join("pgdata"),
            password: Self::fetch_vault_credentials()
                .get("TABLES_PASSWORD")
                .cloned()
                .filter(|p| !p.is_empty()),
        };
        std::fs::create_dir_all(data)?;
        std::fs::create_dir_all(logs)?;
        if !tables.pgdata.join("PG_VERSION").is_file() {
            // A new cluster trusts loopback until `require_passwords` below
            // sets the password and switches the rules.
            let result = SafeCommand::new(&bin.join("bin/initdb.exe").to_string_lossy())?
                .args(&["--encoding=UTF8", "--no-locale", "-D", &tables.pgdata.to_string_lossy(), "-U", "gbuser", "--auth=trust"])?
                .execute()?;
            if !result.status.success() {
                return Err(anyhow::anyhow!("Local database initialization failed; check the database logs"));
            }
            botcorepkg::desktop_hardening::harden_stack_configs(stack);
        }
        if !tables.accepting() {
            botcorepkg::port_guard::ensure_ports_free("tables", &[PORT], stack)
                .map_err(|message| anyhow::anyhow!(message))?;
            let log = std::fs::OpenOptions::new().create(true).append(true).open(logs.join("pgctl.log"))?;
            let mut command = SafeCommand::new(&bin.join("bin/pg_ctl.exe").to_string_lossy())?
                .args(&["-D", &tables.pgdata.to_string_lossy(), "-l", &logs.join("postgresql.log").to_string_lossy(), "start", "-w", "-t", "30"])?
                .stdout(std::process::Stdio::from(log.try_clone()?)).stderr(std::process::Stdio::from(log));
            // Wait for pg_ctl itself, never for EOF on pipes inherited by the
            // long-lived Windows postgres/cmd descendants.
            if !command.spawn()?.wait()?.success() {
                return Err(anyhow::anyhow!("Local database startup failed; check pgctl.log"));
            }
        }
        let hba = std::fs::read_to_string(tables.pgdata.join("pg_hba.conf"))
            .unwrap_or_else(|e| { log::debug!("cannot read pg_hba.conf: {e}"); Default::default() });
        if tables.password.is_none() {
            if pg_hba_is_hardened(&hba) {
                info!("local database is up; its password is read from Vault once Vault answers");
                return SafeCommand::noop_child().map_err(Into::into);
            }
            info!("Vault is not answering yet; the local database keeps its current login rules for now");
        }
        let exists = tables.sql("SELECT 1 FROM pg_database WHERE datname = 'botserver'")?;
        if !exists.status.success() {
            return Err(anyhow::anyhow!("Local database did not accept SQL connections"));
        }
        if String::from_utf8_lossy(&exists.stdout).trim() != "1"
            && !tables.sql("CREATE DATABASE botserver WITH OWNER gbuser")?.status.success()
        {
            return Err(anyhow::anyhow!("Could not create the local application database"));
        }
        tables.require_passwords()?;
        SafeCommand::noop_child().map_err(Into::into)
    }
}
