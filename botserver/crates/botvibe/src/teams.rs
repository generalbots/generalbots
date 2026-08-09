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
        teams.sort_by_key(|t| std::cmp::Reverse(t.created_at));
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
            team.status = team_status(&team.members).to_string();
            if team.members
                .iter()
                .all(|m| m.state == "completed" || m.state == "failed")
            {
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

/// Aggregates member states into a team-level status: "completed" only when
/// every member completed, "failed" when all members are terminal and at
/// least one failed, "running" otherwise.
fn team_status(members: &[TeamMember]) -> &'static str {
    let all_terminal = members
        .iter()
        .all(|m| m.state == "completed" || m.state == "failed");
    if all_terminal {
        if members.iter().all(|m| m.state == "completed") {
            "completed"
        } else {
            "failed"
        }
    } else {
        "running"
    }
}

/// Coordinates a team run: executes one member as its own agent run and
/// records the outcome on the team row. The execution itself stays async so
/// callers decide whether members run concurrently or in waves.
#[derive(Clone, Copy)]
pub struct TeamCoordinator;

impl TeamCoordinator {
    /// Runs one member task through the agent loop and returns the member
    /// with its run id, state and error attached.
    pub async fn execute_member(
        &self,
        state: Arc<dyn VibeState>,
        prompt_manager: Arc<VibePromptManager>,
        tool_executor: Arc<VibeToolExecutor>,
        telemetry: Arc<VibeTelemetry>,
        permissions: crate::permissions::PermissionEngineRef,
        skills: Arc<crate::skills::SkillStore>,
        member: &TeamMember,
    ) -> TeamMember {
        let config = VibeRunConfig {
            use_case: VibeUseCase::SoftwareDevelopment,
            lang: "en".to_string(),
            auto_approve: true,
            max_tool_calls: 50,
            timeout_seconds: 600,
            model: None,
            budget_cents: 0,
        };
        let mut run = VibeRun::new(
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
            member.task.clone(),
            config,
        );
        let run_id = run.run_id;
        {
            let mut runs = state.active_runs().write().await;
            runs.insert(run_id, run.clone());
        }
        let agent_loop = Arc::new(
            AgentLoop::new(prompt_manager, tool_executor, telemetry, state.clone())
                .with_security(permissions, skills),
        );
        agent_loop.execute_run(&mut run).await;
        {
            let mut runs = state.active_runs().write().await;
            runs.insert(run_id, run.clone());
        }
        TeamMember {
            name: member.name.clone(),
            task: member.task.clone(),
            run_id: Some(run_id),
            state: run.state.to_string(),
            error: run.error.clone(),
        }
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
        let coordinator = TeamCoordinator;
        let mut handles = Vec::new();
        for (index, member) in members_copy.iter().enumerate() {
            let state = state.clone();
            let prompt = prompt.clone();
            let executor = executor.clone();
            let telemetry = telemetry.clone();
            let permissions = permissions.clone();
            let skills = skills.clone();
            let teams = teams.clone();
            let member = member.clone();
            handles.push(tokio::spawn(async move {
                let updated = coordinator
                    .execute_member(
                        state,
                        prompt,
                        executor,
                        telemetry,
                        permissions,
                        skills,
                        &member,
                    )
                    .await;
                teams.update_member(team_id, index, updated).await;
            }));
        }
        for handle in handles {
            let _ = handle.await;
        }
    });

    Json(TeamCreateResponse { success: true, team_id, status: "running".into(), error: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_team(name: &str) -> VibeTeam {
        VibeTeam {
            team_id: Uuid::new_v4(),
            name: name.to_string(),
            objective: "ship".to_string(),
            members: vec![
                TeamMember { name: "alice".into(), task: "t1".into(), run_id: None, state: "pending".into(), error: None },
                TeamMember { name: "bob".into(), task: "t2".into(), run_id: None, state: "pending".into(), error: None },
            ],
            shared_tasks: vec!["t1".into(), "t2".into()],
            status: "running".into(),
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    #[tokio::test]
    async fn insert_get_list_order() {
        let store = TeamStore::new();
        let older = sample_team("older");
        store.insert(older.clone()).await;
        let newer = sample_team("newer");
        store.insert(newer.clone()).await;
        assert_eq!(store.get(newer.team_id).await.unwrap().name, "newer");
        let list = store.list().await;
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].team_id, newer.team_id);
        assert!(store.get(Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn update_member_marks_team_failed_when_any_member_fails() {
        let store = TeamStore::new();
        let team = sample_team("t");
        store.insert(team.clone()).await;
        store.update_member(team.team_id, 0, TeamMember { name: "alice".into(), task: "t1".into(), run_id: Some(Uuid::new_v4()), state: "completed".into(), error: None }).await;
        let mid = store.get(team.team_id).await.unwrap();
        assert_eq!(mid.status, "running");
        store.update_member(team.team_id, 1, TeamMember { name: "bob".into(), task: "t2".into(), run_id: None, state: "failed".into(), error: Some("boom".into()) }).await;
        let done = store.get(team.team_id).await.unwrap();
        assert_eq!(done.status, "failed");
        assert!(done.completed_at.is_some());
    }

    #[tokio::test]
    async fn update_member_marks_team_completed_when_all_members_complete() {
        let store = TeamStore::new();
        let team = sample_team("t");
        store.insert(team.clone()).await;
        store.update_member(team.team_id, 0, TeamMember { name: "alice".into(), task: "t1".into(), run_id: Some(Uuid::new_v4()), state: "completed".into(), error: None }).await;
        store.update_member(team.team_id, 1, TeamMember { name: "bob".into(), task: "t2".into(), run_id: Some(Uuid::new_v4()), state: "completed".into(), error: None }).await;
        let done = store.get(team.team_id).await.unwrap();
        assert_eq!(done.status, "completed");
        assert!(done.completed_at.is_some());
    }

    #[test]
    fn team_status_aggregates_member_states() {
        fn member(state: &str) -> TeamMember {
            TeamMember { name: "x".into(), task: "t".into(), run_id: None, state: state.into(), error: None }
        }
        assert_eq!(team_status(&[member("completed"), member("completed")]), "completed");
        assert_eq!(team_status(&[member("completed"), member("failed")]), "failed");
        assert_eq!(team_status(&[member("running"), member("pending")]), "running");
        assert_eq!(team_status(&[]), "completed", "empty team is trivially done");
    }

    #[tokio::test]
    async fn update_member_ignores_unknown_team() {
        let store = TeamStore::new();
        store.update_member(Uuid::new_v4(), 0, TeamMember { name: "x".into(), task: "t".into(), run_id: None, state: "completed".into(), error: None }).await;
        assert!(store.list().await.is_empty());
    }
}
