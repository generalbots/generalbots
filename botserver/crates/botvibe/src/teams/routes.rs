//! `teams::routes` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

#[derive(Debug, Serialize)]
pub(crate) struct TeamResponse {
    pub(crate) success: bool,
    pub(crate) team: Option<VibeTeam>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TeamsResponse {
    pub(crate) success: bool,
    pub(crate) teams: Vec<VibeTeam>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TeamCreateResponse {
    pub(crate) success: bool,
    pub(crate) team_id: Uuid,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
}

pub fn teams_router(routes: TeamRoutes) -> Router {
    Router::new()
        .route("/api/vibe/teams", axum::routing::get(list_teams))
        .route("/api/vibe/teams", axum::routing::post(create_team))
        .route("/api/vibe/teams/:team_id", axum::routing::get(get_team))
        .layer(Extension(routes))
}

pub(crate) async fn list_teams(
    Extension(routes): Extension<TeamRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Json<TeamsResponse> {
    if user.user_id.is_nil() {
        return Json(TeamsResponse { success: false, teams: Vec::new() });
    }
    Json(TeamsResponse { success: true, teams: routes.teams.list().await })
}

pub(crate) async fn get_team(
    Extension(routes): Extension<TeamRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    axum::extract::Path(team_id): axum::extract::Path<Uuid>,
) -> Json<TeamResponse> {
    if user.user_id.is_nil() {
        return Json(TeamResponse { success: false, team: None, error: Some("forbidden: anonymous".into()) });
    }
    match routes.teams.get(team_id).await {
        Some(team) => Json(TeamResponse { success: true, team: Some(team), error: None }),
        None => Json(TeamResponse { success: false, team: None, error: Some("Team not found".into()) }),
    }
}
