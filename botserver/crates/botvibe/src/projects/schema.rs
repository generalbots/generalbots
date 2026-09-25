//! `projects::schema` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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
    source_control VARCHAR(50) NOT NULL DEFAULT 'native',
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
ALTER TABLE vibe_projects ADD COLUMN IF NOT EXISTS source_control VARCHAR(50) NOT NULL DEFAULT 'native';
CREATE INDEX IF NOT EXISTS idx_vibe_projects_source_control ON vibe_projects(source_control);
";

/// Kinds of Vibe projects (REST API enum surface).
/// #1291 — the former `custom` kind is now `apps`; `custom` remains an
/// accepted input alias (and matches legacy DB rows on read).
/// #1372 — unknown kinds are REJECTED, never silently coerced: `web`/`site`
/// map to `website` (static HTMX pages served from the proxy container) and
/// `app`/`custom`/`node` map to `apps` (VM-backed custom projects; python
/// frameworks belong here too). Only VM-backed kinds (`bot`, `apps`) spawn
/// dev VMs — `website` always runs on the proxy container (#1371).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Bot,
    Website,
    Apps,
}

impl ProjectKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bot => "bot",
            Self::Website => "website",
            Self::Apps => "apps",
        }
    }

    /// #1372 — strict parse with explicit aliases; unknown kinds return an
    /// error naming the valid values instead of silently defaulting to
    /// `apps` (the VM-provisioning kind).
    pub fn parse_strict(s: &str) -> Result<Self, String> {
        match s {
            "bot" => Ok(Self::Bot),
            // Static-site aliases: HTMX/HTML pages, no VM.
            "website" | "web" | "site" | "static" | "html" | "htmx" => Ok(Self::Website),
            // VM-backed custom-app aliases: node (and python) run in a VM.
            "apps" | "app" | "custom" | "node" | "nodejs" => Ok(Self::Apps),
            _ => Err(format!(
                "unknown project_type '{s}': valid values are bot, website (aliases: web, site, static, html, htmx) and apps (aliases: app, custom, node, nodejs)"
            )),
        }
    }

    /// Legacy lenient parse — kept only for tests and non-user-facing reads.
    pub fn parse(s: &str) -> Self {
        Self::parse_strict(s).unwrap_or(Self::Apps)
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
    /// Source-control mode: `native` (workspace-only, VM syncs from the
    /// workspace), `git` (Forgejo-backed, VM syncs from the repo and
    /// Deploy creates per-deploy branches + dev→prod promotion) or
    /// `github` (clone of an external repository, see `payload.clone_url`).
    pub source_control: String,
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
    /// `native` (default), `git` (Forgejo-backed) or `github` (clone an
    /// external repository) — see `Project::source_control`.
    pub source_control: Option<String>,
    /// External repository URL for `source_control = "github"` (the
    /// repository is cloned into the workspace at creation).
    pub clone_url: Option<String>,
    /// Free-form "what do you want to build?" prompt from the New Project
    /// dialog. When present, the LLM scaffolds the starter files from it
    /// instead of a hardcoded template; the built-in template remains the
    /// offline fallback (#1312).
    pub description: Option<String>,
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
    pub source_control: Option<String>,
    pub status: Option<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ProjectRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) project_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) repository: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) framework: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) custom_domain: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) source_control: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) environment: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub(crate) payload: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) updated_at: DateTime<Utc>,
}

impl ProjectRow {
    pub(crate) fn into_project(self) -> Project {
        Project {
            id: self.id,
            org_id: self.org_id,
            branch_id: self.branch_id,
            name: self.name,
            project_type: self.project_type,
            repository: self.repository,
            framework: self.framework,
            custom_domain: self.custom_domain,
            source_control: self.source_control,
            status: self.status,
            environment: self.environment,
            payload: self.payload,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
