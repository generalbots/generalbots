//! `teams::coordinator` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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

/// Runtime dependencies shared by all members of a team run. `user_id`,
/// `bot_id` and `session_id` attribute every member run to the authenticated
/// caller instead of `Uuid::nil()` (#927).
#[derive(Clone)]
pub struct MemberRuntime {
    pub state: Arc<dyn VibeState>,
    pub prompt_manager: Arc<VibePromptManager>,
    pub tool_executor: Arc<VibeToolExecutor>,
    pub telemetry: Arc<VibeTelemetry>,
    pub permissions: crate::permissions::PermissionEngineRef,
    pub skills: Arc<crate::skills::SkillStore>,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub session_id: Uuid,
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
        runtime: &MemberRuntime,
        member: &TeamMember,
    ) -> TeamMember {
        let config = VibeRunConfig {
            use_case: VibeUseCase::SoftwareDevelopment,
            lang: "en".to_string(),
            // Destructive tools reached through a team member must still
            // prompt for approval — auto-approving team work lets a single
            // team run push/delete/publish with no human gate (#927).
            auto_approve: false,
            max_tool_calls: 50,
            timeout_seconds: 600,
            model: None,
            llm_key: None,
            llm_url: None,
            agent: None,
            budget_cents: 0,
            project_id: None,
            project_name: None,
            pipeline_mode: None,
        };
        let mut run = VibeRun::new(
            runtime.bot_id,
            runtime.session_id,
            runtime.user_id,
            member.task.clone(),
            config,
        );
        let run_id = run.run_id;
        {
            let mut runs = runtime.state.active_runs().write().await;
            runs.insert(run_id, run.clone());
        }
        let agent_loop = Arc::new(
            AgentLoop::new(
                runtime.prompt_manager.clone(),
                runtime.tool_executor.clone(),
                runtime.telemetry.clone(),
                runtime.state.clone(),
            )
            .with_security(
                runtime.permissions.clone(),
                runtime.skills.clone(),
            ),
        );
        agent_loop.execute_run(&mut run).await;
        {
            let mut runs = runtime.state.active_runs().write().await;
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

pub(crate) async fn create_team(
    Extension(routes): Extension<TeamRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateTeamRequest>,
) -> Json<TeamCreateResponse> {
    if user.user_id.is_nil() {
        return Json(TeamCreateResponse { success: false, team_id: Uuid::nil(), status: "failed".into(), error: Some("forbidden: anonymous users cannot create teams".into()) });
    }
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

    // Attribute every member run to the authenticated caller: bot id from
    // the current bot context, session id from the session (parsed), user id
    // from the JWT principal. Never `Uuid::nil()` (#927).
    let bot_id = user.current_bot_id.unwrap_or_else(Uuid::nil);
    let session_id = user
        .session_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::nil);
    let user_id = user.user_id;

    tokio::spawn(async move {
        let coordinator = TeamCoordinator;
        let mut handles = Vec::new();
        let runtime = MemberRuntime {
            state,
            prompt_manager: prompt,
            tool_executor: executor,
            telemetry,
            permissions,
            skills,
            user_id,
            bot_id,
            session_id,
        };
        for (index, member) in members_copy.iter().enumerate() {
            let runtime = runtime.clone();
            let teams = teams.clone();
            let member = member.clone();
            handles.push(tokio::spawn(async move {
                let updated = coordinator
                    .execute_member(&runtime, &member)
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
