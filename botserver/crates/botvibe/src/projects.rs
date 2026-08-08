//! #743 — dynamic project registry for the Vibe agent.
//!
//! Replaces the hardcoded project/workspace modeling with a DB-backed
//! registry (`vibe_projects`): projects are created, listed, updated and
//! deleted through the REST API, so the underlying VM/tooling layers can
//! operate on whatever projects actually exist. Scoped by `branch_id`
//! (multi-tenant; nil UUID = global default-bot scope).

use crate::types::DbPool;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub const VIBE_PROJECTS_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS vibe_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    name VARCHAR(255) NOT NULL,
    project_type VARCHAR(50) NOT NULL DEFAULT 'bot',
    repository VARCHAR(255) NOT NULL DEFAULT 'generalbots',
    framework VARCHAR(255),
    custom_domain VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    environment VARCHAR(50) NOT NULL DEFAULT 'development',
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_vibe_projects_name_branch ON vibe_projects(branch_id, name);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_branch ON vibe_projects(branch_id);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_status ON vibe_projects(status);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_type ON vibe_projects(project_type);
";

/// Kinds of Vibe projects (REST API enum surface).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Bot,
    Website,
    Custom,
}

impl ProjectKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bot => "bot",
            Self::Website => "website",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "bot" => Self::Bot,
            "website" => Self::Website,
            _ => Self::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub project_type: String,
    pub repository: String,
    pub framework: Option<String>,
    pub custom_domain: Option<String>,
    pub status: String,
    pub environment: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub project_type: Option<String>,
    pub repository: Option<String>,
    pub framework: Option<String>,
    pub custom_domain: Option<String>,
    pub environment: Option<String>,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub project_type: Option<String>,
    pub repository: Option<String>,
    pub framework: Option<String>,
    pub custom_domain: Option<String>,
    pub environment: Option<String>,
    pub status: Option<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub branch_id: Option<Uuid>,
    pub project_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// DB-backed project registry.
#[derive(Clone)]
pub struct ProjectRegistry {
    pool: DbPool,
}

impl ProjectRegistry {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, String> {
        self.pool.get().map_err(|e| format!("db pool: {e}"))
    }

    /// Idempotent schema bootstrap (mirror of the 6.5.49 migration).
    pub fn ensure_schema(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(VIBE_PROJECTS_SCHEMA)
            .execute(&mut conn)
            .map_err(|e| format!("vibe_projects schema: {e}"))?;
        Ok(())
    }

    pub fn create(&self, req: &CreateProjectRequest) -> Result<Project, String> {
        let mut conn = self.conn()?;
        let org_id = req.org_id.unwrap_or_else(Uuid::nil);
        let branch_id = req.branch_id.unwrap_or_else(Uuid::nil);
        let project_type = ProjectKind::from_str(req.project_type.as_deref().unwrap_or("bot"));
        let repository = req.repository.clone().unwrap_or_else(|| req.name.clone());
        let framework = req.framework.clone().unwrap_or_default();
        let custom_domain = req.custom_domain.clone().unwrap_or_default();
        let environment = req.environment.clone().unwrap_or_else(|| "development".to_string());
        let payload = serde_json::json!({});

        let id = Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO vibe_projects \
             (id, org_id, branch_id, name, project_type, repository, framework, custom_domain, \
              status, environment, payload, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(&req.name)
        .bind::<diesel::sql_types::Text, _>(project_type.as_str())
        .bind::<diesel::sql_types::Text, _>(&repository)
        .bind::<diesel::sql_types::Text, _>(&framework)
        .bind::<diesel::sql_types::Text, _>(&custom_domain)
        .bind::<diesel::sql_types::Text, _>("pending")
        .bind::<diesel::sql_types::Text, _>(&environment)
        .bind::<diesel::sql_types::Jsonb, _>(&payload)
        .execute(&mut conn)
        .map_err(|e| format!("insert project: {e}"))?;

        Ok(Project {
            id,
            org_id,
            branch_id,
            name: req.name.clone(),
            project_type: project_type.as_str().to_string(),
            repository,
            framework: req.framework.clone(),
            custom_domain: req.custom_domain.clone(),
            status: "pending".to_string(),
            environment,
            payload,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn list(&self, query: &ListProjectsQuery) -> Result<Vec<Project>, String> {
        let mut conn = self.conn()?;
        let branch_id = query.branch_id.unwrap_or_else(Uuid::nil);
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut sql = String::from(
            "SELECT id, org_id, branch_id, name, project_type, repository, framework, \
             custom_domain, status, environment, payload, created_at, updated_at \
             FROM vibe_projects WHERE branch_id = $1",
        );
        let mut binds: Vec<String> = Vec::new();
        if let Some(ref pt) = query.project_type {
            sql.push_str(" AND project_type = $");
            sql.push_str(&(2 + binds.len()).to_string());
            binds.push(pt.clone());
        }
        if let Some(ref st) = query.status {
            sql.push_str(" AND status = $");
            sql.push_str(&(2 + binds.len()).to_string());
            binds.push(st.clone());
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT $");
        let limit_idx = 2 + binds.len();
        sql.push_str(&limit_idx.to_string());
        sql.push_str(" OFFSET $");
        sql.push_str(&(limit_idx + 1).to_string());

        let mut query_builder = diesel::sql_query(sql)
            .into_boxed::<diesel::pg::Pg>()
            .bind::<diesel::sql_types::Uuid, _>(branch_id);
        for b in &binds {
            query_builder = query_builder.bind::<diesel::sql_types::Text, _>(b.clone());
        }
        query_builder = query_builder
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset);

        query_builder
            .load::<ProjectRow>(&mut conn)
            .map(|rows| rows.into_iter().map(ProjectRow::into_project).collect())
            .map_err(|e| format!("list projects: {e}"))
    }

    pub fn get(&self, id: Uuid) -> Result<Option<Project>, String> {
        let mut conn = self.conn()?;
        let row = diesel::sql_query(
            "SELECT id, org_id, branch_id, name, project_type, repository, framework, \
             custom_domain, status, environment, payload, created_at, updated_at \
             FROM vibe_projects WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result::<ProjectRow>(&mut conn)
        .optional()
        .map_err(|e| format!("get project: {e}"))?;
        Ok(row.map(ProjectRow::into_project))
    }

    /// Append a deployment record to the project payload (deploy history;
    /// consumed by #772 rollback and the UI deployment list).
    pub fn append_deployment(&self, id: Uuid, record: &serde_json::Value) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE vibe_projects
             SET payload = jsonb_set(
                   payload,
                   '{deployments}',
                   COALESCE(payload->'deployments', '[]'::jsonb) || $2::jsonb
                 ),
                 updated_at = NOW()
             WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Jsonb, _>(record)
        .execute(&mut conn)
        .map_err(|e| format!("append deployment: {e}"))?;
        Ok(())
    }

    pub fn update(&self, id: Uuid, req: &UpdateProjectRequest) -> Result<bool, String> {
        let mut conn = self.conn()?;
        let mut assignments: Vec<String> = Vec::new();
        let mut binds: Vec<String> = Vec::new();
        let mut payload_bind: Option<serde_json::Value> = None;

        if let Some(ref name) = req.name {
            binds.push(name.clone());
            assignments.push(format!("name = ${}", binds.len()));
        }
        if let Some(ref pt) = req.project_type {
            binds.push(ProjectKind::from_str(pt).as_str().to_string());
            assignments.push(format!("project_type = ${}", binds.len()));
        }
        if let Some(ref repo) = req.repository {
            binds.push(repo.clone());
            assignments.push(format!("repository = ${}", binds.len()));
        }
        if let Some(ref fw) = req.framework {
            binds.push(fw.clone());
            assignments.push(format!("framework = ${}", binds.len()));
        }
        if let Some(ref cd) = req.custom_domain {
            binds.push(cd.clone());
            assignments.push(format!("custom_domain = ${}", binds.len()));
        }
        if let Some(ref env) = req.environment {
            binds.push(env.clone());
            assignments.push(format!("environment = ${}", binds.len()));
        }
        if let Some(ref st) = req.status {
            binds.push(st.clone());
            assignments.push(format!("status = ${}", binds.len()));
        }
        if let Some(ref p) = req.payload {
            payload_bind = Some(p.clone());
            assignments.push(format!("payload = ${}", binds.len() + 1));
        }

        if assignments.is_empty() {
            return Ok(false);
        }
        assignments.push("updated_at = NOW()".to_string());

        let mut sql = String::from("UPDATE vibe_projects SET ");
        sql.push_str(&assignments.join(", "));
        sql.push_str(&format!(" WHERE id = ${}", binds.len() + (payload_bind.is_some() as usize) + 1));

        let mut query_builder = diesel::sql_query(sql).into_boxed::<diesel::pg::Pg>();
        for b in &binds {
            query_builder = query_builder.bind::<diesel::sql_types::Text, _>(b.clone());
        }
        if let Some(p) = payload_bind {
            query_builder = query_builder.bind::<diesel::sql_types::Jsonb, _>(p);
        }
        query_builder = query_builder.bind::<diesel::sql_types::Uuid, _>(id);

        let affected = query_builder
            .execute(&mut conn)
            .map_err(|e| format!("update project: {e}"))?;
        Ok(affected > 0)
    }

    pub fn delete(&self, id: Uuid) -> Result<bool, String> {
        let mut conn = self.conn()?;
        let affected = diesel::sql_query("DELETE FROM vibe_projects WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| format!("delete project: {e}"))?;
        Ok(affected > 0)
    }
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ProjectRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    project_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    repository: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    framework: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    custom_domain: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    environment: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    payload: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: DateTime<Utc>,
}

impl ProjectRow {
    fn into_project(self) -> Project {
        Project {
            id: self.id,
            org_id: self.org_id,
            branch_id: self.branch_id,
            name: self.name,
            project_type: self.project_type,
            repository: self.repository,
            framework: self.framework,
            custom_domain: self.custom_domain,
            status: self.status,
            environment: self.environment,
            payload: self.payload,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub type ProjectRegistryRef = Arc<ProjectRegistry>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_kind_round_trip() {
        assert_eq!(ProjectKind::from_str("bot"), ProjectKind::Bot);
        assert_eq!(ProjectKind::from_str("website"), ProjectKind::Website);
        assert_eq!(ProjectKind::from_str("bogus"), ProjectKind::Custom);
        assert_eq!(ProjectKind::Bot.as_str(), "bot");
        assert_eq!(ProjectKind::Website.as_str(), "website");
        assert_eq!(ProjectKind::Custom.as_str(), "custom");
    }

    #[test]
    fn list_query_defaults_are_bounded() {
        let q = ListProjectsQuery {
            branch_id: None,
            project_type: None,
            status: None,
            limit: None,
            offset: None,
        };
        assert_eq!(q.branch_id, None);
        assert_eq!(q.limit.unwrap_or(100).min(500), 100);
        assert_eq!(q.offset.unwrap_or(0).max(0), 0);
        let big = ListProjectsQuery { limit: Some(9001), ..q };
        assert_eq!(big.limit.unwrap_or(100).min(500), 500);
        let neg = ListProjectsQuery { offset: Some(-3), ..q };
        assert_eq!(neg.offset.unwrap_or(0).max(0), 0);
    }
}
