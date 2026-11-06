//! A PTY-backed shell and its output ring.

use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::TERMINAL_BUFFER_LINES;

#[derive(Clone)]
pub struct TerminalLine {
    pub data: String,
    pub at: DateTime<Utc>,
}

/// A shell running inside a real pseudo-terminal (PTY).
///
/// Running the shell behind a PTY (rather than raw pipes) is what makes the
/// terminal behave like a terminal: the kernel line discipline echoes typed
/// characters, supports line editing, and emits the prompt. Raw output is
/// forwarded to clients as-is (echo + newlines included), so xterm.js renders
/// it correctly without any client-side emulation.
pub struct TerminalSession {
    pub id: String,
    pub shell: String,
    pub cwd: String,
    /// Optional project VM container to attach to (`incus exec`). When set,
    /// the shell runs INSIDE that container instead of on the botserver host,
    /// so the Vibe terminal shows the project's environment, not the server.
    pub container: Option<String>,
    pub created_at: DateTime<Utc>,
    pub child: Mutex<Option<Box<dyn portable_pty::Child + Send + Sync>>>,
    pub writer: Mutex<Option<Box<dyn Write + Send>>>,
    pub master: Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>,
    pub exit_code: Mutex<Option<i32>>,
    pub buffer: Arc<Mutex<VecDeque<TerminalLine>>>,
    pub events: broadcast::Sender<String>,
}

impl TerminalSession {
    pub fn new(id: String, shell: String, cwd: String) -> Self {
        Self::new_with_container(id, shell, cwd, None)
    }

    pub fn new_with_container(
        id: String,
        shell: String,
        cwd: String,
        container: Option<String>,
    ) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            id,
            shell,
            cwd,
            container,
            created_at: Utc::now(),
            child: Mutex::new(None),
            writer: Mutex::new(None),
            master: Mutex::new(None),
            exit_code: Mutex::new(None),
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(TERMINAL_BUFFER_LINES))),
            events: tx,
        }
    }

    pub fn is_running(&self) -> bool {
        self.exit_code.lock().map(|c| c.is_none()).unwrap_or(false)
    }

    pub fn history(&self) -> Vec<TerminalLine> {
        self.buffer
            .lock()
            .map(|b| b.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn append_output(&self, data: String) {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() >= TERMINAL_BUFFER_LINES {
                buf.pop_front();
            }
            buf.push_back(TerminalLine {
                data: data.clone(),
                at: Utc::now(),
            });
        }
        let _ = self.events.send(data);
    }

    /// Allocate a PTY pair and spawn the shell on the slave side.
    pub fn spawn(self: &Arc<Self>) -> Result<(), String> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(portable_pty::PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("openpty: {e}"))?;

        // Attached mode: run the shell inside the project's VM container
        // (`incus exec <container> -- bash -l`) so the terminal reflects the
        // project environment, not the botserver host filesystem. The PTY
        // wraps the incus exec client; the container's own pty handles the
        // remote side.
        let mut cmd = if let Some(container) = &self.container {
            if !container
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err("Invalid container name".to_string());
            }
            let shell_arg = if self.shell.is_empty() {
                "/bin/bash".to_string()
            } else {
                self.shell.clone()
            };
            #[cfg(target_os = "windows")]
            let c = {
                let distro = std::env::var("GBO_WSL_DISTRO")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "Debian".to_string());
                let mut command = portable_pty::CommandBuilder::new("wsl.exe");
                command.args([
                    "-d",
                    &distro,
                    "-u",
                    "root",
                    "--",
                    "incus",
                    "exec",
                    container,
                    "--cwd",
                    "/opt/vibe/app",
                    "--",
                    "env",
                    "TERM=xterm-256color",
                    &shell_arg,
                    "-l",
                ]);
                command
            };
            #[cfg(not(target_os = "windows"))]
            let c = {
                // The caller may point the shell at the conversation folder
                // (that is where output/ and data/ live); /opt/vibe/app is the
                // fallback for a plain project VM.
                let container_cwd = if self.cwd.starts_with('/') && !self.cwd.contains("..") {
                    self.cwd.clone()
                } else {
                    "/opt/vibe/app".to_string()
                };
                let mut command = portable_pty::CommandBuilder::new("incus");
                command.args([
                    "exec",
                    container,
                    "--cwd",
                    &container_cwd,
                    "--",
                    "env",
                    "TERM=xterm-256color",
                    &shell_arg,
                    "-l",
                ]);
                command
            };
            c
        } else {
            let mut c = portable_pty::CommandBuilder::new(&self.shell);
            c.cwd(&self.cwd);
            c
        };
        cmd.env("TERM", "xterm-256color");
        cmd.env("PS1", "\\u@\\h:\\w \\$ ");
        // HOME must be explicit: the child inherits an empty HOME (the server
        // runs without a passwd entry in the container), so interactive bash
        // sources `/.cargo/env` (empty $HOME) and prints a spurious
        // "No such file or directory" before the prompt.
        if self.container.is_none() {
            let home = if cfg!(target_os = "windows") {
                std::env::var("USERPROFILE")
            } else {
                std::env::var("HOME")
            };
            if let Ok(home) = home {
                cmd.env("HOME", home);
            } else if !cfg!(target_os = "windows") {
                cmd.env("HOME", "/root");
            }
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("spawn shell: {e}"))?;
        drop(pair.slave);

        let master = pair.master;
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| format!("clone reader: {e}"))?;
        let writer = master
            .take_writer()
            .map_err(|e| format!("take writer: {e}"))?;

        *self
            .child
            .lock()
            .map_err(|_| "child lock poisoned".to_string())? = Some(child);
        *self
            .writer
            .lock()
            .map_err(|_| "writer lock poisoned".to_string())? = Some(writer);
        *self
            .master
            .lock()
            .map_err(|_| "master lock poisoned".to_string())? = Some(master);
        *self
            .exit_code
            .lock()
            .map_err(|_| "exit lock poisoned".to_string())? = None;

        let st = self.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                        st.append_output(text);
                    }
                    Err(e) => {
                        log::debug!("terminal reader ended: {e}");
                        break;
                    }
                }
            }
            // Reader closed → the shell exited (or was killed). Mark it done
            // so is_running()/reap() reflect reality.
            if let Ok(mut exit) = st.exit_code.lock() {
                if exit.is_none() {
                    *exit = Some(0);
                }
            }
        });

        Ok(())
    }

    pub fn write(&self, data: &str) -> Result<(), String> {
        let mut guard = self
            .writer
            .lock()
            .map_err(|_| "writer lock poisoned".to_string())?;
        if let Some(w) = guard.as_mut() {
            w.write_all(data.as_bytes())
                .map_err(|e| format!("pty write: {e}"))?;
            let _ = w.flush();
        }
        Ok(())
    }

    /// Resize the PTY (forwarded from the client's `resize <cols> <rows>`).
    pub fn resize(&self, cols: u16, rows: u16) {
        if let Ok(guard) = self.master.lock() {
            if let Some(m) = guard.as_ref() {
                let _ = m.resize(portable_pty::PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(child) = guard.as_mut() {
                let _ = child.kill();
            }
        }
    }
}
