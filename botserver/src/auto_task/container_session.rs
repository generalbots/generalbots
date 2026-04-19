use crate::security::command_guard::SafeCommand;
use log::{info, warn};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{mpsc, Mutex};

#[derive(Debug)]
pub enum TerminalOutput {
    Stdout(String),
    Stderr(String),
}

pub struct ContainerSession {
    pub session_id: String,
    pub container_name: String,
    process: Option<Child>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
}

impl ContainerSession {
    pub async fn new(session_id: &str) -> Result<Self, String> {
        let container_name = format!("agent-{}", session_id.chars().take(8).collect::<String>());
        
        // Launch the container (this might take a moment if the image isn't cached locally)
        info!("Launching LXC container: {}", container_name);
        let safe_cmd = SafeCommand::new("lxc").map_err(|e| format!("{}", e))?;
        let safe_cmd = safe_cmd.args(&["launch", "ubuntu:22.04", &container_name]).map_err(|e| format!("{}", e))?;
        let launch_status = safe_cmd.execute_async().await
            .map_err(|e| format!("Failed to execute lxc launch: {}", e))?;

        if !launch_status.status.success() {
            let stderr = String::from_utf8_lossy(&launch_status.stderr);
            // If it already exists, that's fine, we can just use it
            if !stderr.contains("already exists") {
                warn!("Warning during LXC launch (might already exist): {}", stderr);
            }
        }

        Ok(Self {
            session_id: session_id.to_string(),
            container_name,
            process: None,
            stdin: None,
        })
    }

    pub async fn start_terminal(&mut self, tx: mpsc::Sender<TerminalOutput>) -> Result<(), String> {
        info!("Starting terminal session in container: {}", self.container_name);
        
        // SafeCommand doesn't support async piped I/O, so we use tokio::process::Command directly.
        // Security: container_name is derived from session_id (not user input), and commands run
        // inside an isolated LXC container, not on the host.
        let mut child = tokio::process::Command::new("lxc")
            .args(["exec", &self.container_name, "--", "bash"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn lxc exec: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.process = Some(child);

        // Spawn stdout reader
        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_out.send(TerminalOutput::Stdout(line)).await.is_err() {
                    break;
                }
            }
        });

        // Spawn stderr reader
        let tx_err = tx;
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_err.send(TerminalOutput::Stderr(line)).await.is_err() {
                    break;
                }
            }
        });

        // Send a setup command to get things ready
        self.send_command("export TERM=xterm-256color; cd /root").await?;

        Ok(())
    }

    pub async fn send_command(&self, cmd: &str) -> Result<(), String> {
        if let Some(stdin_mutex) = &self.stdin {
            let mut stdin = stdin_mutex.lock().await;
            let cmd_with_newline = format!("{}\n", cmd);
            stdin.write_all(cmd_with_newline.as_bytes()).await
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin.flush().await
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
            Ok(())
        } else {
            Err("Terminal not started".to_string())
        }
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        info!("Stopping container session: {}", self.container_name);
        
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }

        // Clean up container
        let safe_cmd = SafeCommand::new("lxc").map_err(|e| format!("{}", e))?;
        let safe_cmd = safe_cmd.args(&["delete", &self.container_name, "--force"]).map_err(|e| format!("{}", e))?;
        let status = safe_cmd.execute_async().await
            .map_err(|e| format!("Failed to delete container: {}", e))?;

        if !status.status.success() {
            warn!("Failed to delete container {}: {}", self.container_name, String::from_utf8_lossy(&status.stderr));
        }

        Ok(())
    }
}

impl Drop for ContainerSession {
    fn drop(&mut self) {
        // We can't easily await inside drop, but the actual LXC container persists
        // unless we spawn a blocking task or fire-and-forget task to delete it.
        // For reliability, we expect the caller to call `stop().await`.
    }
}
