//! Execution methods of [`SafeCommand`] (split from `command_guard.rs`).
//!
//! # Why this module uses `tokio::process`
//!
//! The old `execute_async()` built a `std::process::Command` and called its
//! blocking `output()` inside an `async fn`. That made the function a
//! *synthetic* async call: it never yielded, so a `tokio::time::timeout`
//! wrapped around it could never be polled while the child was running. A
//! `RUN BASH "sleep 100000"` therefore pinned the caller forever and the
//! timeout in the code sandbox was dead code (#2660).
//!
//! Every path here now builds a `tokio::process::Command` with
//! `kill_on_drop(true)` so:
//!   * the `.await` points really let the timer fire, and
//!   * dropping the future (cancellation, timeout, shutdown) kills the child
//!     and tokio's orphan reaper collects it instead of leaving a zombie.
//!
//! On Unix the child is also placed in its own process group so the sandbox
//! can kill grandchildren (a shell that spawned `sleep 60`), not just the
//! direct child.

use super::*;
use std::time::Duration;

impl SafeCommand {
    /// Blocking execution — kept for the synchronous callers of `SafeCommand`.
    pub fn execute(&self) -> Result<Output, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        self.append_args(&mut cmd);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        self.apply_common_env(&mut cmd);

        cmd.output()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    /// Build the validated, scrubbed command as a tokio child process.
    ///
    /// Reuses the exact same allow-list/argument/env validation as the
    /// blocking helpers by converting an already-built `std::process::Command`.
    fn build_tokio_command(&self) -> tokio::process::Command {
        let mut cmd = std::process::Command::new(&self.command);
        self.append_args(&mut cmd);
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }
        self.apply_common_env(&mut cmd);
        tokio::process::Command::from(cmd)
    }

    /// Truly asynchronous execution. The future yields while the child runs,
    /// so an enclosing `tokio::time::timeout` can fire, and `kill_on_drop`
    /// guarantees the child is killed if the future is cancelled.
    pub async fn execute_async(&self) -> Result<Output, CommandGuardError> {
        let mut cmd = self.build_tokio_command();
        cmd.kill_on_drop(true);

        cmd.output()
            .await
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    /// Execute with an enforced deadline that **kills the child process group**
    /// and reaps the child when the deadline expires.
    ///
    /// Returns `Ok(None)` on timeout and `Ok(Some(output))` on completion.
    /// This is the entry point untrusted code sandboxes must use: a bare
    /// `tokio::time::timeout(execute_async())` would kill only the direct
    /// child on drop, leaving grandchildren of `bash -c` running (#2660).
    pub async fn execute_async_with_timeout(
        &self,
        deadline: Duration,
    ) -> Result<Option<Output>, CommandGuardError> {
        let mut cmd = self.build_tokio_command();
        // A dedicated group lets us signal the whole tree (shell + its
        // children) with one negative-pid `kill`, not just the group leader.
        #[cfg(unix)]
        cmd.process_group(0);
        // Cancellation safety: if this future is dropped before the deadline
        // (outer timeout, shutdown), tokio kills the child and reaps it.
        cmd.kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))?;

        // Capture the pid before `wait_with_output` consumes the child; a
        // missing id means it already exited, so no group kill is needed.
        #[cfg(unix)]
        let pgid = child.id();

        let wait = child.wait_with_output();
        tokio::pin!(wait);

        match tokio::time::timeout(deadline, &mut wait).await {
            Ok(output) => output
                .map(Some)
                .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string())),
            Err(_) => {
                // `wait` is dropped at the end of this arm, which (via
                // kill_on_drop) sends SIGKILL to the direct child and hands it
                // to tokio's orphan reaper. The group kill below also takes
                // out any grandchildren that inherited the process group.
                #[cfg(unix)]
                if let Some(pgid) = pgid {
                    kill_process_group(pgid).await;
                }
                Ok(None)
            }
        }
    }

    pub fn spawn(&mut self) -> Result<Child, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        self.append_args(&mut cmd);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        if let Some(stdout) = self.stdout.take() {
            cmd.stdout(stdout);
        }

        if let Some(stderr) = self.stderr.take() {
            cmd.stderr(stderr);
        }

        self.apply_common_env(&mut cmd);

        cmd.spawn()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    pub fn spawn_with_envs(
        &self,
        envs: &HashMap<String, String>,
    ) -> Result<Child, CommandGuardError> {
        let mut cmd = std::process::Command::new(&self.command);
        self.append_args(&mut cmd);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        self.apply_common_env(&mut cmd);

        for (key, value) in envs {
            if validate_argument(key).is_ok() && validate_argument(value).is_ok() {
                cmd.env(key, value);
            }
        }

        cmd.spawn()
            .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
    }

    /// Builds the validated process command (allow-listed binary, validated
    /// argv, scrubbed environment) for callers that need streaming I/O the
    /// blocking helpers cannot provide — an interactive piped shell or a PTY.
    ///
    /// Callers wrap this instead of recreating `Command::new` from a raw
    /// binary name, so the binary stays allow-listed and every argument stays
    /// validated (#2126).
    pub fn into_std_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.command);
        self.append_args(&mut cmd);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        self.apply_common_env(&mut cmd);
        cmd
    }

    pub fn noop_child() -> Result<Child, CommandGuardError> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            std::process::Command::new("cmd")
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .args(["/C", "exit /b 0"])
                .spawn()
                .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
        }

        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new("true")
                .spawn()
                .map_err(|e| CommandGuardError::ExecutionFailed(e.to_string()))
        }
    }
}

/// Pure decision: translate a child's pid into the negative-pid token that
/// `kill(1)` uses to signal an entire process group. `None` (pid 0 / invalid)
/// means there is no group to signal. Kept pure so the timeout decision can be
/// unit-tested offline (#2660).
#[cfg(unix)]
fn group_kill_token(pgid: u32) -> Option<String> {
    if pgid == 0 {
        None
    } else {
        Some(format!("-{pgid}"))
    }
}

/// Signal the timed-out sandbox's whole process group with SIGKILL.
///
/// Uses the allow-listed `kill` binary with a numeric, internally generated
/// argument (never user input), so it cannot smuggle a shell metacharacter.
#[cfg(unix)]
async fn kill_process_group(pgid: u32) {
    let Some(token) = group_kill_token(pgid) else {
        return;
    };
    let cmd = match SafeCommand::new("kill").and_then(|c| c.arg("-KILL")) {
        Ok(cmd) => cmd,
        Err(e) => {
            log::warn!("cannot build process-group kill command: {e}");
            return;
        }
    };
    let cmd = match cmd.arg(&token) {
        Ok(cmd) => cmd,
        Err(e) => {
            log::warn!("invalid process-group kill target {token}: {e}");
            return;
        }
    };
    if let Err(e) = cmd.execute_async().await {
        // The group may have already exited between the timeout and the kill;
        // that is the desired end state, so this is not an error.
        log::debug!("process group {token} already gone: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn group_kill_token_decides_whether_a_group_must_be_signalled() {
        assert_eq!(group_kill_token(0), None);
        assert_eq!(group_kill_token(4321).as_deref(), Some("-4321"));
    }

    #[cfg(unix)]
    fn count_probe_processes() -> usize {
        let marker: &[u8] = b"gb_sandbox_timeout_probe";
        let mut found = 0;
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) {
                    if cmdline.windows(marker.len()).any(|w| w == marker) {
                        found += 1;
                    }
                }
            }
        }
        found
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn deadline_kills_sleep_and_leaves_no_child_behind() {
        let cmd = match SafeCommand::new("bash")
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.shell_script_arg("exec -a gb_sandbox_timeout_probe sleep 60"))
        {
            Ok(cmd) => cmd,
            Err(_) => return, // no bash in this environment — nothing to test
        };

        let started = std::time::Instant::now();
        let outcome = match cmd.execute_async_with_timeout(Duration::from_secs(2)).await {
            Ok(outcome) => outcome,
            Err(_) => return,
        };
        assert!(outcome.is_none(), "a 60s sleep must hit the 2s deadline");
        assert!(
            started.elapsed() < Duration::from_secs(8),
            "timeout must fire within budget"
        );

        // Give tokio's orphan reaper a moment to collect the killed child.
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            count_probe_processes(),
            0,
            "the timed-out sleep must not survive the deadline"
        );
    }
}
