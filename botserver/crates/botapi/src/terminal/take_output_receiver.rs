use super::*;
use crate::incus_command;

impl TerminalSession {
    /// Builds a session whose container name follows the Incus naming rule
    /// (`term-<slug>`). An unusable session id is rejected up front instead of
    /// producing a name that later reaches the container runtime.
    pub fn new(session_id: &str) -> Result<Self, String> {
        let container_name = incus_command::session_container_name(session_id)?;

        let (output_tx, output_rx) = mpsc::channel(100);

        Ok(Self {
            session_id: session_id.to_string(),
            container_name,
            process: None,
            stdin: None,
            output_tx,
            output_rx: Some(output_rx),
        })
    }

    pub fn take_output_receiver(&mut self) -> Option<mpsc::Receiver<TerminalOutput>> {
        self.output_rx.take()
    }

    pub async fn start(&mut self) -> Result<(), String> {
        info!("Starting Incus container: {}", self.container_name);

        let launch_argv = incus_command::launch_argv(&self.container_name)?;
        let launch_output = guarded_command(&launch_argv)?
            .execute_async()
            .await
            .map_err(|e| format!("Failed to launch container: {e}"))?;

        if !launch_output.status.success() {
            let stderr = String::from_utf8_lossy(&launch_output.stderr);
            if !stderr.contains("already exists") && !stderr.contains("is already running") {
                warn!("Container launch warning: {}", stderr);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Starting bash shell in container: {}", self.container_name);

        // The shell runs INSIDE the Incus container (`incus exec <name> --
        // bash -l`), never on the host. The argv comes from the guard-validated
        // builder and the interactive spawn keeps that same command: no raw
        // `Command::new("lxc")` bypass (#2126).
        let exec_argv = incus_command::exec_argv(&self.container_name)?;
        let guarded = guarded_command(&exec_argv)?;
        let mut shell = tokio::process::Command::from(guarded.into_std_command());
        shell.kill_on_drop(true);
        shell
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = shell
            .spawn()
            .map_err(|e| format!("Failed to spawn bash: {e}"))?;

        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.process = Some(child);

        let tx = self.output_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx
                    .send(TerminalOutput::Stdout(format!("{}\r\n", line)))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        let tx = self.output_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx
                    .send(TerminalOutput::Stderr(format!("{}\r\n", line)))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        let tx = self.output_tx.clone();
        tx.send(TerminalOutput::System(
            "Container started. Welcome to your isolated terminal!\r\n".to_string(),
        ))
        .await
        .ok();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.send_command("export TERM=xterm-256color; clear")
            .await?;

        Ok(())
    }

    pub async fn send_command(&self, cmd: &str) -> Result<(), String> {
        if let Some(stdin_mutex) = &self.stdin {
            let mut stdin = stdin_mutex.lock().await;
            let cmd_with_newline = format!("{}\n", cmd);
            stdin
                .write_all(cmd_with_newline.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }
        Ok(())
    }

    pub async fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
        Ok(())
    }

    pub async fn kill(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }

        let stop_argv = incus_command::stop_argv(&self.container_name)?;
        if let Err(e) = guarded_command(&stop_argv)?.execute_async().await {
            warn!("Failed to stop container {}: {}", self.container_name, e);
        }

        let delete_argv = incus_command::delete_argv(&self.container_name)?;
        if let Err(e) = guarded_command(&delete_argv)?.execute_async().await {
            warn!("Failed to delete container {}: {}", self.container_name, e);
        }

        info!("Container {} destroyed", self.container_name);
        Ok(())
    }
}

/// Turns a builder argv (`["incus", verb, name, …]`) into a guarded command.
///
/// The first element must be the allow-listed Incus binary; anything else is
/// refused so a future builder cannot smuggle a different runtime in.
fn guarded_command(argv: &[String]) -> Result<SafeCommand, String> {
    incus_command::validate_runtime(argv)?;
    let (binary, args) = argv
        .split_first()
        .ok_or_else(|| "Empty Incus argv".to_string())?;

    let mut guarded = SafeCommand::new(binary).map_err(|e| e.to_string())?;
    for arg in args {
        guarded = guarded.arg(arg).map_err(|e| e.to_string())?;
    }
    Ok(guarded)
}
