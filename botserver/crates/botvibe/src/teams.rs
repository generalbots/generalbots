use crate::agent_loop::AgentLoop;
use crate::prompt_manager::VibePromptManager;
use crate::telemetry::VibeTelemetry;
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeRun, VibeRunConfig, VibeState, VibeUseCase};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub task: String,
    pub run_id: Option<Uuid>,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeTeam {
    pub team_id: Uuid,
    pub name: String,
    pub objective: String,
    pub members: Vec<TeamMember>,
    pub shared_tasks: Vec<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct TeamStore {
    teams: RwLock<Vec<VibeTeam>>,
}

impl TeamStore {
    pub fn new() -> Self {
        Self { teams: RwLock::new(Vec::new()) }
    }

    pub async fn get(&self, team_id: Uuid) -> Option<VibeTeam> {
        let teams = self.teams.read().await;
        teams.iter().find(|t| t.team_id == team_id).cloned()
    }

    pub async fn list(&self) -> Vec<VibeTeam> {
        let mut teams = self.teams.read().await.clone();
        teams.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        teams
    }

    async fn insert(&self, team: VibeTeam) {
        let mut teams = self.teams.write().await;
        teams.push(team);
    }

    async fn update_member(&self, team_id: Uuid, index: usize, member: TeamMember) {
        let mut teams = self.teams.write().await;
        if let Some(team) = teams.iter_mut().find(|t| t.team_id == team_id) {
            if let Some(m) = team.members.get_mut(index) {
                *m = member;
            }
            if team.members.iter().all(|m| m.state == "completed" || m.state == "failed") {
                team.status = "completed".into();
                team.completed_at = Some(chrono::Utc::now());
            }
        }
    }
}

impl Default for TeamStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct TeamRoutes {
    pub teams: Arc<TeamStore>,
    pub state: Arc<dyn VibeState>,
    pub prompt_manager: Arc<VibePromptManager>,
    pub tool_executor: Arc<VibeToolExecutor>,
    pub telemetry: Arc<VibeTelemetry>,
    pub permissions: crate::permissions::PermissionEngineRef,
    pub skills: Arc<crate::skills::SkillStore>,
}

#[derive(Debug, Deserialize)]
pub struct TeamMemberRequest {
    pub name: String,
    pub task: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub objective: String,
    pub members: Vec<TeamMemberRequest>,
}

#[derive(Debug, Serialize)]
struct TeamResponse {
    success: bool,
    team: Option<VibeTeam>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct TeamsResponse {
    success: bool,
    teams: Vec<VibeTeam>,
}

#[derive(Debug, Serialize)]
struct TeamCreateResponse {
    success: bool,
    team_id: Uuid,
    status: String,
    error: Option<String>,
}

pub fn teams_router(routes: TeamRoutes) -> Router {
    Router::new()
        .route("/api/vibe/teams", axum::routing::get(list_teams))
        .route("/api/vibe/teams", axum::routing::post(create_team))
        .route("/api/vibe/teams/:team_id", axum::routing::get(get_team))
        .layer(Extension(routes))
}

async fn list_teams(Extension(routes): Extension<TeamRoutes>) -> Json<TeamsResponse> {
    Json(TeamsResponse { success: true, teams: routes.teams.list().await })
}

async fn get_team(
    Extension(routes): Extension<TeamRoutes>,
    axum::extract::Path(team_id): axum::extract::Path<Uuid>,
) -> Json<TeamResponse> {
    match routes.teams.get(team_id).await {
        Some(team) => Json(TeamResponse { success: true, team: Some(team), error: None }),
        None => Json(TeamResponse { success: false, team: None, error: Some("Team not found".into()) }),
    }
}

async fn create_team(
    Extension(routes): Extension<TeamRoutes>,
    Json(req): Json<CreateTeamRequest>,
) -> Json<TeamCreateResponse> {
    if req.members.is_empty() {
        return Json(TeamCreateResponse { success: false, team_id: Uuid::nil(), status: "failed".into(), error: Some("At least one member is required".into()) });
    }

    let team_id = Uuid::new_v4();
    let members: Vec<TeamMember> = req
        .members
        .iter()
        .map(|m| TeamMember {
            name: m.name.clone(),
            task: m.task.clone(),
            run_id: None,
            state: "pending".into(),
            error: None,
        })
        .collect();

    let shared_tasks = req
        .members
        .iter()
        .map(|m| m.task.clone())
        .collect::<Vec<_>>();

    let team = VibeTeam {
        team_id,
        name: req.name.clone(),
        objective: req.objective.clone(),
        members: members.clone(),
        shared_tasks,
        status: "running".into(),
        created_at: chrono::Utc::now(),
        completed_at: None,
    };
    routes.teams.insert(team).await;

    let state = routes.state.clone();
    let prompt = routes.prompt_manager.clone();
    let executor = routes.tool_executor.clone();
    let telemetry = routes.telemetry.clone();
    let permissions = routes.permissions.clone();
    let skills = routes.skills.clone();
    let teams = routes.teams.clone();
    let members_copy = members.clone();

    tokio::spawn(async move {
        let mut handles = Vec::new();
        for (index, member) in members_copy.iter().enumerate() {
            let state = state.clone();
            let prompt = prompt.clone();
            let executor = executor.clone();
            let telemetry = telemetry.clone();
            let permissions = permissions.clone();
            let skills = skills.clone();
            let teams = teams.clone();
            let member_name = member.name.clone();
            let task = member.task.clone();
            handles.push(tokio::spawn(async move {
                let config = VibeRunConfig {
                    use_case: VibeUseCase::SoftwareDevelopment,
                    auto_approve: true,
                    max_tool_calls: 50,
                    timeout_seconds: 600,
                    model: None,
                    budget_cents: 0,
                };
                let mut run = VibeRun::new(Uuid::nil(), Uuid::nil(), Uuid::nil(), task.clone(), config);
                let run_id = run.run_id;
                {
                    let mut runs = state.active_runs().write().await;
                    runs.insert(run_id, run.clone());
                }
                let agent_loop = Arc::new(
                    AgentLoop::new(prompt, executor, telemetry, state.clone())
                        .with_security(permissions, skills),
                );
                agent_loop.execute_run(&mut run).await;
                {
                    let mut runs = state.active_runs().write().await;
                    runs.insert(run_id, run.clone());
                }
                let member = TeamMember {
                    name: member_name,
                    task,
                    run_id: Some(run_id),
                    state: run.state.to_string(),
                    error: run.error.clone(),
                };
                teams.update_member(team_id, index, member).await;
            }));
        }
        for handle in handles {
            let _ = handle.await;
        }
    });

    Json(TeamCreateResponse { success: true, team_id, status: "running".into(), error: None })
}
