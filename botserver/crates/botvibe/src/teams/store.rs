//! `teams::store` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub struct TeamStore {
    pub(crate) teams: RwLock<Vec<VibeTeam>>,
    pub(crate) pool: Option<crate::types::DbPool>,
}

impl TeamStore {
    pub fn new() -> Self {
        Self {
            teams: RwLock::new(Vec::new()),
            pool: None,
        }
    }

    /// #816 — write-through persistence so teams survive restarts.
    /// Hydrates from the database on construction (#921).
    pub fn with_persistence(pool: crate::types::DbPool) -> Self {
        let teams = crate::catalog_persistence::load_teams(&pool).unwrap_or_else(|e| {
            log::warn!("team hydrate failed: {e}");
            Vec::new()
        });
        Self {
            teams: RwLock::new(teams),
            pool: Some(pool),
        }
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

    pub(crate) async fn insert(&self, team: VibeTeam) {
        let mut teams = self.teams.write().await;
        teams.push(team.clone());
        drop(teams);
        self.persist(&team).await;
    }

    pub(crate) async fn update_member(&self, team_id: Uuid, index: usize, member: TeamMember) {
        let mut teams = self.teams.write().await;
        let updated = match teams.iter_mut().find(|t| t.team_id == team_id) {
            Some(team) => {
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
                Some(team.clone())
            }
            None => None,
        };
        drop(teams);
        if let Some(team) = updated {
            self.persist(&team).await;
        }
    }

    /// Persists a team snapshot (or logs on failure — never panics).
    pub(crate) async fn persist(&self, team: &VibeTeam) {
        let Some(pool) = &self.pool else { return };
        if let Err(e) = crate::catalog_persistence::save_team(pool, team) {
            log::error!("team persist failed for {}: {e}", team.team_id);
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
pub(crate) fn team_status(members: &[TeamMember]) -> &'static str {
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
