use super::*;

pub(super) fn safe_sh_command(script: &str) -> Option<std::process::Output> {
    let abs = crate::os_abstraction::get_abstraction();
    let (shell, flag) = abs.shell_command();
    #[cfg(target_os = "windows")]
    let script = &format!("\"{script}\"");
    SafeCommand::new(shell)
        .and_then(|c| c.arg(flag))
        .and_then(|c| c.raw_shell_script_arg(script))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}
