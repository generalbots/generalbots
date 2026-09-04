use diesel::prelude::*;
use uuid::Uuid;

use super::handlers::DbPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeployRole {
    Viewer,
    Developer,
    Admin,
    Owner,
}

impl DeployRole {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "viewer" => Some(Self::Viewer),
            "developer" => Some(Self::Developer),
            "admin" => Some(Self::Admin),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Developer => "developer",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }
}


type Conn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(diesel::QueryableByName)]
struct StringCell {
    #[diesel(sql_type = diesel::sql_types::Text)]
    value: String,
}

fn conn(pool: &DbPool) -> Result<Conn, String> {
    pool.get().map_err(|e| format!("db pool: {e}"))
}

fn user_group_names(conn: &mut Conn, user_id: Uuid) -> Result<Vec<String>, String> {
    if user_id.is_nil() {
        return Ok(Vec::new());
    }
    diesel::sql_query(
        "SELECT g.name AS value FROM rbac_user_groups ug \
         JOIN rbac_groups g ON g.id = ug.group_id \
         WHERE ug.user_id = $1 AND g.is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load::<StringCell>(conn)
    .map_err(|e| format!("resolve user groups: {e}"))
    .map(|rows| rows.into_iter().map(|r| r.value).collect())
}

pub fn resolve_project_role(
    pool: &DbPool,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<DeployRole, String> {
    let mut conn = conn(pool)?;
    let direct: Option<StringCell> = diesel::sql_query(
        "SELECT role AS value FROM project_members WHERE project_id = $1 AND user_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(project_id)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<StringCell>(&mut conn)
    .optional()
    .map_err(|e| format!("resolve project role: {e}"))?;
    if let Some(role) = direct {
        if let Some(r) = DeployRole::parse(&role.value) {
            return Ok(r);
        }
    }
    let groups = user_group_names(&mut conn, user_id)?;
    let mut best = DeployRole::Viewer;
    for name in &groups {
        let group_role: Option<StringCell> = diesel::sql_query(
            "SELECT role AS value FROM project_members WHERE project_id = $1 AND group_name = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(name)
        .get_result::<StringCell>(&mut conn)
        .optional()
        .map_err(|e| format!("resolve group role: {e}"))?;
        if let Some(role) = group_role {
            if let Some(r) = DeployRole::parse(&role.value) {
                best = best.max(r);
            }
        }
    }
    Ok(best)
}

pub fn is_org_admin(pool: &DbPool, user_id: Uuid) -> Result<bool, String> {
    if user_id.is_nil() {
        return Ok(false);
    }
    let mut conn = conn(pool)?;
    let groups = user_group_names(&mut conn, user_id)?;
    Ok(groups.iter().any(|n| n.to_lowercase().contains("admin")))
}

pub fn require_deploy_role(
    pool: &DbPool,
    user_id: Uuid,
    project_id: Option<Uuid>,
) -> Result<(), String> {
    match project_id {
        Some(pid) => {
            let role = resolve_project_role(pool, user_id, pid)?;
            // #1280 — an org-level admin administers every project in the
            // org, so a missing project_members row must not lock them out;
            // everyone below admin still gets the explicit denial.
            if role < DeployRole::Admin && !is_org_admin(pool, user_id)? {
                return Err(format!(
                    "forbidden: role '{}' requires 'admin' on project {pid}",
                    role.as_str()
                ));
            }
        }
        None => {
            if !is_org_admin(pool, user_id)? {
                return Err("forbidden: role 'viewer' requires 'admin' to deploy".to_string());
            }
        }
    }
    Ok(())
}
