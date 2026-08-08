use crate::agent_loop::AgentLoop;
use crate::prompt_manager::VibePromptManager;
use crate::telemetry::VibeTelemetry;
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeRun, VibeRunConfig, VibeRunState, VibeState, VibeUseCase};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeSession {
    pub session_id: Uuid,
    pub parent_session_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub intent: String,
    pub use_case: VibeUseCase,
    pub budget_cents: u64,
    pub run: Option<VibeRun>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct SessionStore {
    sessions: RwLock<Vec<VibeSession>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self { sessions: RwLock::new(Vec::new()) }
    }

    pub async fn create(
        &self,
        bot_id: Uuid,
        user_id: Uuid,
        intent: String,
        use_case: VibeUseCase,
        budget_cents: u64,
        parent_session_id: Option<Uuid>,
    ) -> VibeSession {
        let session = VibeSession {
            session_id: Uuid::new_v4(),
            parent_session_id,
            bot_id,
            user_id,
            intent,
            use_case,
            budget_cents,
            run: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let mut sessions = self.sessions.write().await;
        sessions.push(session.clone());
        session
    }

    pub async fn get(&self, session_id: Uuid) -> Option<VibeSession> {
        let sessions = self.sessions.read().await;
        sessions.iter().find(|s| s.session_id == session_id).cloned()
    }

    pub async fn list(&self) -> Vec<VibeSession> {
        let mut sessions = self.sessions.read().await.clone();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }

    pub async fn fork(&self, session_id: Uuid) -> Option<VibeSession> {
        let original = self.get(session_id).await?;
        let mut fork = self
            .create(
                original.bot_id,
                original.user_id,
                original.intent.clone(),
                original.use_case,
                original.budget_cents,
                Some(original.session_id),
            )
            .await;
        fork.run = original.run.clone();
        let mut sessions = self.sessions.write().await;
        if let Some(existing) = sessions.iter_mut().find(|s| s.session_id == fork.session_id) {
            *existing = fork.clone();
        }
        Some(fork)
    }

    pub async fn rewind(&self, session_id: Uuid, to_tool_calls: usize) -> Option<VibeSession> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.iter_mut().find(|s| s.session_id == session_id)?;
        if let Some(run) = &mut session.run {
            run.tool_calls.truncate(to_tool_calls);
            run.state = VibeRunState::Pending;
        }
        session.updated_at = chrono::Utc::now();
        Some(session.clone())
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct SessionRoutes {
    pub sessions: Arc<SessionStore>,
    pub state: Arc<dyn VibeState>,
    pub prompt_manager: Arc<VibePromptManager>,
    pub tool_executor: Arc<VibeToolExecutor>,
    pub telemetry: Arc<VibeTelemetry>,
    pub permissions: crate::permissions::PermissionEngineRef,
    pub skills: Arc<crate::skills::SkillStore>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub intent: String,
    pub use_case: Option<String>,
    pub budget_cents: Option<u64>,
    pub parent_session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RewindRequest {
    pub to_tool_calls: usize,
}

#[derive(Debug, Deserialize)]
pub struct ResumeRequest {
    pub auto_approve: Option<bool>,
    pub max_tool_calls: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    success: bool,
    session: Option<VibeSession>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionsResponse {
    success: bool,
    sessions: Vec<VibeSession>,
}

#[derive(Debug, Serialize)]
struct ResumeResponse {
    success: bool,
    session_id: Uuid,
    run_id: Uuid,
    state: String,
    error: Option<String>,
}

fn parse_use_case(s: &str) -> Option<VibeUseCase> {
    match s {
        "software_development" => Some(VibeUseCase::SoftwareDevelopment),
        "customer_support" => Some(VibeUseCase::CustomerSupport),
        "financial_analysis" => Some(VibeUseCase::FinancialAnalysis),
        _ => None,
    }
}

pub fn sessions_router(routes: SessionRoutes) -> Router {
    Router::new()
        .route("/api/vibe/sessions", axum::routing::get(list_sessions))
        .route("/api/vibe/sessions", axum::routing::post(create_session))
        .route("/api/vibe/sessions/:session_id", axum::routing::get(get_session))
        .route("/api/vibe/sessions/:session_id/fork", axum::routing::post(fork_session))
        .route("/api/vibe/sessions/:session_id/rewind", axum::routing::post(rewind_session))
        .route("/api/vibe/sessions/:session_id/resume", axum::routing::post(resume_session))
        .layer(Extension(routes))
}

async fn list_sessions(Extension(routes): Extension<SessionRoutes>) -> Json<SessionsResponse> {
    Json(SessionsResponse {
        success: true,
        sessions: routes.sessions.list().await,
    })
}

async fn create_session(
    Extension(routes): Extension<SessionRoutes>,
    Json(req): Json<CreateSessionRequest>,
) -> Json<SessionResponse> {
    let use_case = req.use_case.as_deref().and_then(parse_use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let session = routes
        .sessions
        .create(Uuid::nil(), Uuid::nil(), req.intent, use_case, req.budget_cents.unwrap_or(0), req.parent_session_id)
        .await;
    Json(SessionResponse { success: true, session: Some(session), error: None })
}

async fn get_session(
    Extension(routes): Extension<SessionRoutes>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Json<SessionResponse> {
    match routes.sessions.get(session_id).await {
        Some(session) => Json(SessionResponse { success: true, session: Some(session), error: None }),
        None => Json(SessionResponse { success: false, session: None, error: Some("Session not found".into()) }),
    }
}

async fn fork_session(
    Extension(routes): Extension<SessionRoutes>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Json<SessionResponse> {
    match routes.sessions.fork(session_id).await {
        Some(session) => Json(SessionResponse { success: true, session: Some(session), error: None }),
        None => Json(SessionResponse { success: false, session: None, error: Some("Session not found".into()) }),
    }
}

async fn rewind_session(
    Extension(routes): Extension<SessionRoutes>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Json(req): Json<RewindRequest>,
) -> Json<SessionResponse> {
    match routes.sessions.rewind(session_id, req.to_tool_calls).await {
        Some(session) => Json(SessionResponse { success: true, session: Some(session), error: None }),
        None => Json(SessionResponse { success: false, session: None, error: Some("Session not found".into()) }),
    }
}

async fn resume_session(
    Extension(routes): Extension<SessionRoutes>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    Json(req): Json<ResumeRequest>,
) -> Json<ResumeResponse> {
    let Some(session) = routes.sessions.get(session_id).await else {
        return Json(ResumeResponse { success: false, session_id, run_id: Uuid::nil(), state: "not_found".into(), error: Some("Session not found".into()) });
    };

    let config = VibeRunConfig {
        use_case: session.use_case,
        auto_approve: req.auto_approve.unwrap_or(false),
        max_tool_calls: req.max_tool_calls.unwrap_or(50),
        timeout_seconds: req.timeout_seconds.unwrap_or(300),
        model: req.model,
        budget_cents: session.budget_cents,
    };

    let mut run = VibeRun::new(session.bot_id, session.session_id, session.user_id, session.intent.clone(), config);
    let run_id = run.run_id;

    {
        let mut runs = routes.state.active_runs().write().await;
        runs.insert(run_id, run.clone());
    }

    let agent_loop = Arc::new(
        AgentLoop::new(
            routes.prompt_manager.clone(),
            routes.tool_executor.clone(),
            routes.telemetry.clone(),
            routes.state.clone(),
        )
        .with_security(routes.permissions.clone(), routes.skills.clone()),
    );

    let sessions = routes.sessions.clone();
    let state_runs = routes.state.active_runs().clone();

    tokio::spawn(async move {
        agent_loop.execute_run(&mut run).await;
        {
            let mut runs = state_runs.write().await;
            runs.insert(run_id, run.clone());
        }
        let mut sessions = sessions.sessions.write().await;
        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == session_id) {
            session.run = Some(run);
            session.updated_at = chrono::Utc::now();
        }
    });

    Json(ResumeResponse {
        success: true,
        session_id,
        run_id,
        state: "running".into(),
        error: None,
    })
}
