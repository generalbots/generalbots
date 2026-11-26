//! Starting and stopping the secrets server programs for the move. No secret
//! is ever passed on a command line: the servers get only their config file.

use crate::secrets_engine::Engine;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use std::path::Path;
use std::process::{Child, Stdio};

/// Config file of the secrets server, relative to the stack.
pub use super::layout::CONFIG_FILE;
/// Log file the servers started by the move append to.
pub const LOG_FILE: &str = "logs/vault/vault.log";

/// Starts `engine` on the stack's own config, without a window.
pub fn start(engine: Engine, stack: &Path) -> Result<Child> {
    let program = engine.path_in(stack);
    let log_path = stack.join(LOG_FILE);
    if let Some(dir) = log_path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let log = std::fs::OpenOptions::new().create(true).append(true).open(&log_path)?;
    let command = SafeCommand::new(&program.to_string_lossy())
        .and_then(|c| c.arg("server"))
        .and_then(|c| c.arg("-config"))
        .map(|c| c.allow_path(stack.to_path_buf()))
        .and_then(|c| c.path_arg(&stack.join(CONFIG_FILE)))
        .map_err(|e| anyhow::anyhow!("{} start command: {e}", engine.binary_name()))?;
    let mut std_command = command.into_std_command();
    std_command
        .current_dir(stack)
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std_command.creation_flags(CREATE_NO_WINDOW);
    }
    std_command.spawn().with_context(|| format!("starting {}", engine.binary_name()))
}

/// Stops a server started by [`start`] and waits for it to exit.
pub fn stop(child: &mut Child) {
    if matches!(child.try_wait(), Ok(None)) {
        if let Err(e) = child.kill() { log::debug!("cannot stop secrets server: {e}"); }
    }
    if let Err(e) = child.wait() { log::debug!("cannot wait for secrets server: {e}"); }
}

/// Stops a server left running by an interrupted move, by process id. The
/// id comes from the marker of an earlier run and may have been reused since,
/// so only a process whose program is this stack's `bao` or `vault` is
/// stopped, and only that process (not its tree).
pub fn stop_pid(pid: u32, stack: &Path) {
    let image = crate::port_guard::process_image(pid);
    if !is_stack_secrets_server(image.as_deref(), stack) {
        return;
    }
    let pid = pid.to_string();
    let command = if cfg!(windows) {
        SafeCommand::new("taskkill").and_then(|c| c.args(&["/PID", &pid, "/F"]))
    } else {
        SafeCommand::new("kill").and_then(|c| c.args(&["-9", &pid]))
    };
    if let Ok(command) = command {
        if let Err(e) = command.execute() { log::debug!("cannot signal secrets server: {e}"); }
    }
}

/// `image` is the `bao` or former `vault` program of `stack`.
pub fn is_stack_secrets_server(image: Option<&Path>, stack: &Path) -> bool {
    let Some(image) = image else { return false };
    let normalize = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
    let image = normalize(image);
    [Engine::OpenBao, Engine::LegacyVault]
        .iter()
        .any(|engine| image == normalize(&engine.path_in(stack)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_stack_secrets_programs_are_stopped_by_id() {
        let stack = Path::new(r"C:\Users\a\AppData\Local\generalbots\stack");
        let bao = Path::new(r"c:\users\a\appdata\local\GeneralBots\stack\bin\vault\bao.exe");
        let vault = Path::new("C:/Users/a/AppData/Local/generalbots/stack/bin/vault/vault.exe");
        let other = Path::new(r"C:\Windows\System32\notepad.exe");
        let sibling = Path::new(r"C:\Users\a\AppData\Local\generalbots\stack\bin\tables\postgres.exe");
        if cfg!(windows) {
            assert!(is_stack_secrets_server(Some(bao), stack));
            assert!(is_stack_secrets_server(Some(vault), stack));
        }
        assert!(!is_stack_secrets_server(Some(other), stack));
        assert!(!is_stack_secrets_server(Some(sibling), stack));
        assert!(!is_stack_secrets_server(None, stack));
    }
}
