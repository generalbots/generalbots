//! Cancels the provisioning child process.

#[cfg(unix)]
pub(super) fn terminate(pid: u32) {
    // SAFETY: sending SIGTERM to a known child pid; failure is reported by the
    // caller through the process exit status.
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
}

#[cfg(windows)]
pub(super) fn terminate(pid: u32) {
    let _ = botlib::security::command_guard::SafeCommand::new("taskkill")
        .and_then(|command| command.args(&["/PID", &pid.to_string(), "/T", "/F"]))
        .and_then(|command| command.execute());
}
