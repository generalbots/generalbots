use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use uuid::Uuid;

pub const TERMINAL_BUFFER_LINES: usize = 512;

pub mod routes;

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
        let (tx, _) = broadcast::channel(1024);
        Self {
            id,
            shell,
            cwd,
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
        self.buffer.lock().map(|b| b.iter().cloned().collect()).unwrap_or_default()
    }

    fn append_output(&self, data: String) {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() >= TERMINAL_BUFFER_LINES {
                buf.pop_front();
            }
            buf.push_back(TerminalLine { data: data.clone(), at: Utc::now() });
        }
        let _ = self.events.send(data);
    }

    /// Allocate a PTY pair and spawn the shell on the slave side.
    pub fn spawn(self: &Arc<Self>) -> Result<(), String> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(portable_pty::PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| format!("openpty: {e}"))?;

        let mut cmd = portable_pty::CommandBuilder::new(&self.shell);
        cmd.cwd(&self.cwd);
        cmd.env("TERM", "xterm-256color");
        cmd.env("PS1", "\\u@\\h:\\w \\$ ");
        // HOME must be explicit: the child inherits an empty HOME (the server
        // runs without a passwd entry in the container), so interactive bash
        // sources `/.cargo/env` (empty $HOME) and prints a spurious
        // "No such file or directory" before the prompt.
        if let Ok(home) = std::env::var("HOME") {
            cmd.env("HOME", home);
        } else {
            cmd.env("HOME", "/root");
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("spawn shell: {e}"))?;
        drop(pair.slave);

        let master = pair.master;
        let mut reader = master.try_clone_reader().map_err(|e| format!("clone reader: {e}"))?;
        let writer = master.take_writer().map_err(|e| format!("take writer: {e}"))?;

        *self.child.lock().map_err(|_| "child lock poisoned".to_string())? = Some(child);
        *self.writer.lock().map_err(|_| "writer lock poisoned".to_string())? = Some(writer);
        *self.master.lock().map_err(|_| "master lock poisoned".to_string())? = Some(master);
        *self.exit_code.lock().map_err(|_| "exit lock poisoned".to_string())? = None;

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
        let mut guard = self.writer.lock().map_err(|_| "writer lock poisoned".to_string())?;
        if let Some(w) = guard.as_mut() {
            w.write_all(data.as_bytes()).map_err(|e| format!("pty write: {e}"))?;
            let _ = w.flush();
        }
        Ok(())
    }

    /// Resize the PTY (forwarded from the client's `resize <cols> <rows>`).
    pub fn resize(&self, cols: u16, rows: u16) {
        if let Ok(guard) = self.master.lock() {
            if let Some(m) = guard.as_ref() {
                let _ = m.resize(portable_pty::PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
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

pub struct TerminalManager {
    pub sessions: Mutex<HashMap<String, Arc<TerminalSession>>>,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    pub fn new() -> Self {
        Self { sessions: Mutex::new(HashMap::new()) }
    }

    pub fn create_session(&self, shell: String, cwd: String) -> Result<Arc<TerminalSession>, String> {
        let id = Uuid::new_v4().to_string();
        let session = Arc::new(TerminalSession::new(id.clone(), shell, cwd));
        session.spawn().map_err(|e| format!("spawn: {e}"))?;
        self.sessions
            .lock()
            .map_err(|_| "sessions lock poisoned".to_string())?
            .insert(id.clone(), session.clone());
        Ok(session)
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<TerminalSession>> {
        self.sessions.lock().ok()?.get(id).cloned()
    }

    pub fn list_sessions(&self) -> Vec<serde_json::Value> {
        let sessions = self
            .sessions
            .lock()
            .ok()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        let mut out = Vec::new();
        for (id, session) in sessions {
            out.push(serde_json::json!({
                "id": id,
                "shell": session.shell,
                "cwd": session.cwd,
                "running": session.is_running(),
                "created_at": session.created_at,
            }));
        }
        out
    }

    pub async fn kill_session(&self, id: &str) -> Result<(), String> {
        let session = self.get_session(id).ok_or_else(|| format!("session {id} not found"))?;
        {
            let mut guard = session.child.lock().map_err(|_| "child lock poisoned".to_string())?;
            if let Some(child) = guard.as_mut() {
                let _ = child.kill();
            }
        }
        if let Ok(mut exit) = session.exit_code.lock() {
            *exit = Some(137);
        }
        self.sessions.lock().map_err(|_| "sessions lock poisoned".to_string())?.remove(id);
        Ok(())
    }

    pub fn reap(&self) {
        let mut guard = self.sessions.lock().ok();
        if let Some(m) = guard.as_mut() {
            let dead: Vec<String> = m.iter().filter(|(_, s)| !s.is_running()).map(|(id, _)| id.clone()).collect();
            for id in dead {
                m.remove(&id);
            }
        }
    }
}

pub fn sanitize_cwd(cwd: &str) -> String {
    if cwd.is_empty() || !std::path::Path::new(cwd).is_dir() {
        std::env::current_dir()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string())
    } else {
        cwd.to_string()
    }
}

pub fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_cwd_falls_back_when_missing() {
        let cwd = sanitize_cwd("/definitely/not/a/dir/xyz");
        assert!(std::path::Path::new(&cwd).is_dir());
        let tmp = std::env::temp_dir();
        assert_eq!(sanitize_cwd(tmp.to_str().unwrap()), tmp.to_str().unwrap().to_string());
    }

    #[test]
    fn default_shell_is_non_empty() {
        assert!(!default_shell().is_empty());
    }

    #[test]
    fn history_ring_bounds_lines() {
        let session = TerminalSession::new("t1".into(), "/bin/sh".into(), "/tmp".into());
        for i in 0..(TERMINAL_BUFFER_LINES + 50) {
            session.append_output(format!("line {i}\n"));
        }
        let history = session.history();
        assert_eq!(history.len(), TERMINAL_BUFFER_LINES);
        assert_eq!(history.first().map(|l| l.data.as_str()), Some("line 50\n"));
    }

    #[test]
    fn not_running_when_no_exit_code_set() {
        let session = TerminalSession::new("t2".into(), "/bin/sh".into(), "/tmp".into());
        assert!(session.is_running());
    }
}
