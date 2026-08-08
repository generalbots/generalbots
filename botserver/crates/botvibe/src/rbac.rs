use diesel::prelude::*;
use std::fmt;
use uuid::Uuid;

use crate::types::DbPool;

pub const PROJECT_MEMBERS_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS project_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES vibe_projects(id) ON DELETE CASCADE,
    user_id UUID,
    group_name VARCHAR(255),
    role VARCHAR(32) NOT NULL DEFAULT 'viewer',
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT project_members_user_xor_group
        CHECK ((user_id IS NOT NULL)::int + (group_name IS NOT NULL)::int = 1),
    CONSTRAINT project_members_role_valid
        CHECK (role IN ('owner', 'admin', 'developer', 'viewer'))
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_project_members_user
    ON project_members(project_id, user_id) WHERE user_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_project_members_group
    ON project_members(project_id, group_name) WHERE group_name IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_project_members_project_role
    ON project_members(project_id, role);
CREATE INDEX IF NOT EXISTS idx_project_members_user
    ON project_members(user_id) WHERE user_id IS NOT NULL;
";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectRole {
    Viewer,
    Developer,
    Admin,
    Owner,
}

impl ProjectRole {
    pub fn parse(s: &str) -> Option<Self> {
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

impl fmt::Display for ProjectRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for ProjectRole {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectMember {
    pub project_id: Uuid,
    pub user_id: Option<Uuid>,
    pub group_name: Option<String>,
    pub role: ProjectRole,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct ProjectRbac {
    pool: DbPool,
}

impl ProjectRbac {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, String> {
        self.pool.get().map_err(|e| format!("db pool: {e}"))
    }

    pub fn ensure_schema(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(PROJECT_MEMBERS_SCHEMA)
            .execute(&mut conn)
            .map_err(|e| format!("project_members schema: {e}"))?;
        Ok(())
    }

    fn user_group_names(&self, conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, user_id: Uuid) -> Result<Vec<String>, String> {
        if user_id == Uuid::nil() {
            return Ok(Vec::new());
        }
        diesel::sql_query(
            "SELECT g.name FROM rbac_user_groups ug \
             JOIN rbac_groups g ON g.id = ug.group_id \
             WHERE ug.user_id = $1 AND g.is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<String>(conn)
        .map_err(|e| format!("resolve user groups: {e}"))
    }

    pub fn resolve_role(&self, user_id: Uuid, project_id: Uuid) -> Result<ProjectRole, String> {
        let mut conn = self.conn()?;
        let direct: Option<String> = diesel::sql_query(
            "SELECT role FROM project_members \
             WHERE project_id = $1 AND user_id = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<String>(&mut conn)
        .optional()
        .map_err(|e| format!("resolve project role: {e}"))?;

        if let Some(role) = direct {
            return Ok(ProjectRole::parse(&role).unwrap_or(ProjectRole::Viewer));
        }

        let groups = self.user_group_names(&mut conn, user_id)?;
        if groups.is_empty() {
            return Ok(ProjectRole::Viewer);
        }
        let mut best = ProjectRole::Viewer;
        for name in &groups {
            let group_role: Option<String> = diesel::sql_query(
                "SELECT role FROM project_members \
                 WHERE project_id = $1 AND group_name = $2",
            )
            .bind::<diesel::sql_types::Uuid, _>(project_id)
            .bind::<diesel::sql_types::Text, _>(name)
            .get_result::<String>(&mut conn)
            .optional()
            .map_err(|e| format!("resolve group role: {e}"))?;
            if let Some(role) = group_role {
                if let Some(r) = ProjectRole::parse(&role) {
                    best = best.max(r);
                }
            }
        }
        Ok(best)
    }

    pub fn require_role(&self, user_id: Uuid, project_id: Uuid, min: ProjectRole) -> Result<ProjectRole, String> {
        let role = self.resolve_role(user_id, project_id)?;
        if role < min {
            return Err(format!(
                "forbidden: role '{role}' requires '{min}' on project {project_id}"
            ));
        }
        Ok(role)
    }

    pub fn set_user_role(&self, project_id: Uuid, user_id: Uuid, role: ProjectRole) -> Result<(), String> {
        if user_id == Uuid::nil() {
            return Err("cannot grant membership to the anonymous user".to_string());
        }
        let mut conn = self.conn()?;
        diesel::sql_query(
            "INSERT INTO project_members (project_id, user_id, role) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (project_id, user_id) WHERE user_id IS NOT NULL \
             DO UPDATE SET role = EXCLUDED.role",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(role.as_str())
        .execute(&mut conn)
        .map_err(|e| format!("set user role: {e}"))?;
        Ok(())
    }

    pub fn remove_user(&self, project_id: Uuid, user_id: Uuid) -> Result<bool, String> {
        let mut conn = self.conn()?;
        let deleted = diesel::sql_query(
            "DELETE FROM project_members \
             WHERE project_id = $1 AND user_id = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut conn)
        .map_err(|e| format!("remove user member: {e}"))?;
        Ok(deleted > 0)
    }

    pub fn set_group_role(&self, project_id: Uuid, group_name: &str, role: ProjectRole) -> Result<(), String> {
        let name = group_name.trim();
        if name.is_empty() {
            return Err("group name must not be empty".to_string());
        }
        let mut conn = self.conn()?;
        diesel::sql_query(
            "INSERT INTO project_members (project_id, group_name, role) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (project_id, group_name) WHERE group_name IS NOT NULL \
             DO UPDATE SET role = EXCLUDED.role",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(role.as_str())
        .execute(&mut conn)
        .map_err(|e| format!("set group role: {e}"))?;
        Ok(())
    }

    pub fn remove_group(&self, project_id: Uuid, group_name: &str) -> Result<bool, String> {
        let mut conn = self.conn()?;
        let deleted = diesel::sql_query(
            "DELETE FROM project_members \
             WHERE project_id = $1 AND group_name = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(group_name)
        .execute(&mut conn)
        .map_err(|e| format!("remove group member: {e}"))?;
        Ok(deleted > 0)
    }

    pub fn list_members(&self, project_id: Uuid) -> Result<Vec<ProjectMember>, String> {
        let mut conn = self.conn()?;
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            project_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            user_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            group_name: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            role: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            added_at: chrono::DateTime<chrono::Utc>,
        }
        let rows = diesel::sql_query(
            "SELECT project_id, user_id, group_name, role, added_at \
             FROM project_members WHERE project_id = $1 \
             ORDER BY added_at",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .load::<Row>(&mut conn)
        .map_err(|e| format!("list members: {e}"))?;
        Ok(rows
            .into_iter()
            .map(|r| ProjectMember {
                project_id: r.project_id,
                user_id: r.user_id,
                group_name: r.group_name,
                role: ProjectRole::parse(&r.role).unwrap_or(ProjectRole::Viewer),
                added_at: r.added_at,
            })
            .collect())
    }

    pub fn org_role(&self, user_id: Uuid) -> Result<String, String> {
        let mut conn = self.conn()?;
        let groups = self.user_group_names(&mut conn, user_id)?;
        for name in &groups {
            if name.to_lowercase().contains("admin") {
                return Ok("admin".to_string());
            }
        }
        Ok("user".to_string())
    }

    pub fn is_org_admin(&self, user_id: Uuid) -> Result<bool, String> {
        self.org_role(user_id).map(|r| r == "admin")
    }

    pub fn transfer_ownership(&self, project_id: Uuid, new_owner_id: Uuid) -> Result<(), String> {
        if new_owner_id == Uuid::nil() {
            return Err("cannot transfer ownership to the anonymous user".to_string());
        }
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE project_members SET role = 'admin' \
             WHERE project_id = $1 AND role = 'owner'",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .execute(&mut conn)
        .map_err(|e| format!("demote previous owners: {e}"))?;
        self.set_user_role(project_id, new_owner_id, ProjectRole::Owner)
    }
}
