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
        info!("Launching LXC container: {}", container_name);

        let launch_status = tokio::process::Command::new("lxc")
            .args(["launch", "ubuntu:22.04", &container_name])
            .output()
            .await
            .map_err(|e| format!("Failed to execute lxc launch: {}", e))?;

        if !launch_status.status.success() {
            let stderr = String::from_utf8_lossy(&launch_status.stderr);
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

        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_out.send(TerminalOutput::Stdout(line)).await.is_err() {
                    break;
                }
            }
        });

        let tx_err = tx;
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_err.send(TerminalOutput::Stderr(line)).await.is_err() {
                    break;
                }
            }
        });

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

        let status = tokio::process::Command::new("lxc")
            .args(["delete", &self.container_name, "--force"])
            .output()
            .await
            .map_err(|e| format!("Failed to delete container: {}", e))?;

        if !status.status.success() {
            warn!("Failed to delete container {}: {}", self.container_name, String::from_utf8_lossy(&status.stderr));
        }

        Ok(())
    }
}

impl Drop for ContainerSession {
    fn drop(&mut self) {}
}
