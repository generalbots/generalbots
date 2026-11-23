//! Process-wide registry of terminal sessions.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::routes::authz::{verify_session_access, SessionAccessDenied, SessionScope};
use super::TerminalSession;
use crate::security::auth_api::types::AuthenticatedUser;

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
        self.create_scoped_session(shell, cwd, container, SessionScope::new(Uuid::nil(), None))
    }

    /// Create a PTY bound to the authenticated owner. Both the scope and the
    /// capability are stored on the session so later list/kill/WS operations
    /// can be checked against them.
    pub fn create_scoped_session(
        &self,
        shell: String,
        cwd: String,
        container: Option<String>,
        scope: SessionScope,
    ) -> Result<Arc<TerminalSession>, String> {
        let id = Uuid::new_v4().to_string();
        let session = Arc::new(TerminalSession::new_scoped(
            id.clone(),
            shell,
            cwd,
            container,
            scope.user_id,
            scope.tenant_id,
        ));
        session.spawn().map_err(|e| format!("spawn: {e}"))?;
        self.sessions
            .lock()
            .map_err(|_| "sessions lock poisoned".to_string())?
            .insert(id, session.clone());
        Ok(session)
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<TerminalSession>> {
        self.sessions.lock().ok()?.get(id).cloned()
    }

    /// WS handshake gate: resolve a session only if the caller presents its
    /// unguessable capability and (when the auth middleware supplied one)
    /// matches the exact user + tenant that created it.
    pub fn get_session_scoped(
        &self,
        id: &str,
        user: Option<&AuthenticatedUser>,
        capability: &str,
    ) -> Result<Arc<TerminalSession>, SessionAccessDenied> {
        let session = self.get_session(id).ok_or(SessionAccessDenied)?;
        let scope = SessionScope::new(session.owner_user_id, session.owner_tenant_id);
        verify_session_access(&scope, &session.capability, user, capability)?;
        Ok(session)
    }

    /// List ONLY the caller's sessions. There is deliberately no unscoped
    /// variant: process-wide enumeration is what exposed every user's PTY id.
    pub fn list_sessions_for(&self, scope: &SessionScope) -> Vec<serde_json::Value> {
        let sessions = self
            .sessions
            .lock()
            .ok()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        let mut out = Vec::new();
        for (id, session) in sessions {
            if session.owner_user_id != scope.user_id
                || session.owner_tenant_id != scope.tenant_id
            {
                continue;
            }
            out.push(serde_json::json!({
                "id": id,
                "shell": session.shell,
                "cwd": session.cwd,
                "container": session.container,
                "running": session.is_running(),
                "created_at": session.created_at,
                "capability": session.capability,
            }));
        }
        out
    }

    /// Kill a session only for its exact owner. Cross-user / cross-tenant
    /// requests (and unknown ids) all fail with the same error so the endpoint
    /// cannot be used to probe which sessions exist.
    pub async fn kill_session_scoped(
        &self,
        id: &str,
        user: &AuthenticatedUser,
    ) -> Result<(), SessionAccessDenied> {
        let session = self.get_session(id).ok_or(SessionAccessDenied)?;
        let scope = SessionScope::new(session.owner_user_id, session.owner_tenant_id);
        if !scope.matches(user) {
            return Err(SessionAccessDenied);
        }
        self.kill_session(id).await.map_err(|_| SessionAccessDenied)
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
