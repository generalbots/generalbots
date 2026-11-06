//! Process-wide registry of terminal sessions.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::TerminalSession;

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
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_session(
        &self,
        shell: String,
        cwd: String,
    ) -> Result<Arc<TerminalSession>, String> {
        self.create_session_with_container(shell, cwd, None)
    }

    pub fn create_session_with_container(
        &self,
        shell: String,
        cwd: String,
        container: Option<String>,
    ) -> Result<Arc<TerminalSession>, String> {
        let id = Uuid::new_v4().to_string();
        let session = Arc::new(TerminalSession::new_with_container(
            id.clone(),
            shell,
            cwd,
            container,
        ));
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
        let session = self
            .get_session(id)
            .ok_or_else(|| format!("session {id} not found"))?;
        {
            let mut guard = session
                .child
                .lock()
                .map_err(|_| "child lock poisoned".to_string())?;
            if let Some(child) = guard.as_mut() {
                let _ = child.kill();
            }
        }
        if let Ok(mut exit) = session.exit_code.lock() {
            *exit = Some(137);
        }
        self.sessions
            .lock()
            .map_err(|_| "sessions lock poisoned".to_string())?
            .remove(id);
        Ok(())
    }

    pub fn reap(&self) {
        let mut guard = self.sessions.lock().ok();
        if let Some(m) = guard.as_mut() {
            let dead: Vec<String> = m
                .iter()
                .filter(|(_, s)| !s.is_running())
                .map(|(id, _)| id.clone())
                .collect();
            for id in dead {
                m.remove(&id);
            }
        }
    }
}
