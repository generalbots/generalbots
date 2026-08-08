use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

pub const TERMINAL_BUFFER_LINES: usize = 512;

pub mod routes;

#[derive(Clone)]
pub struct TerminalLine {
    pub data: String,
    pub at: DateTime<Utc>,
}

pub struct TerminalSession {
    pub id: String,
    pub shell: String,
    pub cwd: String,
    pub created_at: DateTime<Utc>,
    pub child: Mutex<Option<Child>>,
    pub exit_code: Mutex<Option<i32>>,
    pub stdin_tx: Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>,
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
            exit_code: Mutex::new(None),
            stdin_tx: Mutex::new(None),
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

    fn append_line(&self, data: String) {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() >= TERMINAL_BUFFER_LINES {
                buf.pop_front();
            }
            buf.push_back(TerminalLine { data: data.clone(), at: Utc::now() });
        }
        let _ = self.events.send(data);
    }

    pub fn spawn(self: &Arc<Self>) -> Result<(), String> {
        let parts: Vec<&str> = self.shell.split_whitespace().collect();
        let program = parts.first().copied().unwrap_or("sh");
        let args = parts.get(1..).unwrap_or(&[]).to_vec();
        let mut cmd = Command::new(program);
        cmd.args(args)
            .arg("-i")
            .arg("-c")
            .arg("export PS1='\\u@\\h:\\w \\$ '; exec /bin/sh")
            .current_dir(&self.cwd)
            .env("TERM", "xterm-256color")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| format!("spawn shell: {e}"))?;
        let stdin = child.stdin.take().ok_or_else(|| "stdin unavailable".to_string())?;
        let stdout = child.stdout.take().ok_or_else(|| "stdout unavailable".to_string())?;
        let stderr = child.stderr.take().ok_or_else(|| "stderr unavailable".to_string())?;
        *self.exit_code.lock().map_err(|_| "exit lock poisoned".to_string())? = None;
        *self.child.lock().map_err(|_| "child lock poisoned".to_string())? = Some(child);

        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        *self.stdin_tx.lock().map_err(|_| "stdin lock poisoned".to_string())? = Some(tx);

        tokio::spawn(async move {
            let mut writer = stdin;
            while let Some(bytes) = rx.recv().await {
                if writer.write_all(&bytes).await.is_err() {
                    break;
                }
            }
        });

        let st = self.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => st.append_line(line.trim_end_matches('\n').to_string()),
                }
            }
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => st.append_line(line.trim_end_matches('\n').to_string()),
                }
            }
        });
        Ok(())
    }
pub fn write(&self, data: &str) -> Result<(), String> {
        let guard = self.stdin_tx.lock().map_err(|_| "stdin lock poisoned".to_string())?;
        if let Some(tx) = guard.as_ref() {
            tx.send(data.as_bytes().to_vec()).map_err(|e| format!("stdin send: {e}"))?;
        }
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(child) = guard.as_mut() {
                let _ = child.start_kill();
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
        let mut child = {
            let mut guard = session.child.lock().map_err(|_| "child lock poisoned".to_string())?;
            guard.take()
        };
        if let Some(child_ref) = child.as_mut() {
            let _ = child_ref.start_kill();
            match child_ref.kill().await {
                Ok(()) => {}
                Err(e) => log::warn!("kill session {id}: {e}"),
            }
            if let Ok(status) = child_ref.wait().await {
                let code = status.code();
                if let Ok(mut exit) = session.exit_code.lock() {
                    *exit = code;
                }
            }
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
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
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
            session.append_line(format!("line {i}"));
        }
        let history = session.history();
        assert_eq!(history.len(), TERMINAL_BUFFER_LINES);
        assert_eq!(history.first().map(|l| l.data.as_str()), Some("line 50"));
    }

    #[test]
    fn not_running_when_no_exit_code_set() {
        let session = TerminalSession::new("t2".into(), "/bin/sh".into(), "/tmp".into());
        assert!(session.is_running());
    }
}
