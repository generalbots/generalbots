//! `incus.exe` for the agent workspace on Windows.
//!
//! botagent runs a single program for Incus (`AGENT_INCUS_BIN`). This one
//! forwards every argument, unchanged and without a shell, to Incus inside
//! a WSL2 distro: `wsl.exe -d <distro> -u root --exec incus <args...>`.
//! The distro comes from `GBO_WSL_DISTRO` (default `GeneralBotsLite`, the
//! product's own distro); a name outside `[A-Za-z0-9._-]` is refused.
//! Standard input, output and error pass through; the exit code is kept.
//!
//! `__gb-service state` and `__gb-service restart` run the fixed commands
//! `systemctl show incus.service --property=ActiveState,SubState,MainPID`
//! and `systemctl restart incus.service` instead, so botagent can tell a
//! stuck daemon unit from a slow one and restart it.

mod args;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let configured = std::env::var("GBO_WSL_DISTRO").ok();
    let forwarded: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let wsl_args = match args::resolve_distro(configured.as_deref())
        .and_then(|distro| args::wsl_args(&distro, &forwarded))
    {
        Ok(wsl_args) => wsl_args,
        Err(error) => {
            eprintln!("incus wrapper: {error}");
            return ExitCode::from(2);
        }
    };
    let mut cmd = Command::new("wsl.exe");
    cmd.args(wsl_args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    match cmd.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("incus wrapper: cannot run wsl.exe: {error}");
            ExitCode::from(127)
        }
    }
}
