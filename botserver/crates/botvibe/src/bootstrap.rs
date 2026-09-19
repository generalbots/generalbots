//! Reform #1500/#1504 — Vibe bootstrap for the branch's default bot.
//!
//! Every branch owns a default bot; the reform makes that bot a **Vibe
//! project** so people start vibing the base bot immediately:
//!
//! - `ensure_branch_default_project` creates the `bot`-kind, git-mode project
//!   row idempotently (safe to call from signup, Drive monitor scans and the
//!   boot backfill) and **adopts** the branch's existing `bots` row instead of
//!   duplicating it — the adopted row keeps its identity, LLM config and
//!   channels; only `payload.bot_id` links project ↔ bot.
//! - `ensure_bot_rows` guarantees the two-environment model: the PROD bot
//!   (adopted or created) and the `{bot}-test` TEST twin (always created,
//!   `origin='vibe'`), whose database is the project's `_dev` twin per #1386.
//!
//! The boot backfill lives in `bootstrap_backfill.rs`; both share the
//! `pub(crate)` SQL helpers in this module.

use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::OptionalExtension;
use uuid::Uuid;

use crate::projects::Project;

/// Fallback `vibe_projects` DDL for cold-start bootstraps (the registry's
/// schema ensure runs at boot; this keeps the bootstrap self-sufficient when
/// it fires before the first router mount, e.g. signup hook on a fresh DB).
pub(crate) const VIBE_PROJECTS_FALLBACK_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS vibe_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    name VARCHAR(255) NOT NULL,
    project_type VARCHAR(32) NOT NULL DEFAULT 'bot',
    repository VARCHAR(255) NOT NULL DEFAULT '',
    framework VARCHAR(64) NOT NULL DEFAULT '',
    custom_domain VARCHAR(255) NOT NULL DEFAULT '',
    source_control VARCHAR(16) NOT NULL DEFAULT 'git',
    status VARCHAR(32) NOT NULL DEFAULT 'ready',
    environment VARCHAR(16) NOT NULL DEFAULT 'development',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT vibe_projects_branch_name_unique UNIQUE (branch_id, name)
);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_branch ON vibe_projects(branch_id);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_type ON vibe_projects(project_type);
";

/// Project-name → bot slug (single source; the REST path reuses this).
pub fn bot_slug(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace('_', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

/// ALM (Forgejo) org for a branch **slug** — the readable "org-branch
/// organization" of the reform (#1503): every project of the branch lives in
/// this one org, repos named after the bot/web/app.
pub fn alm_org_from_slug(branch_slug: &str) -> String {
    let cleaned: String = branch_slug
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    let cleaned = cleaned.trim_matches('-');
    if cleaned.is_empty() {
        crate::vm_lifecycle::VmLifecycle::alm_org(Uuid::nil())
    } else {
        cleaned.to_string()
    }
}

/// Identity of the `bots` row(s) a bot-kind project owns/adopts.
#[derive(Debug)]
pub struct BotRowRef {
    pub id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
}

impl BotRowRef {
    pub fn from_project(project: &Project) -> Self {
        Self {
            id: project.id,
            org_id: project.org_id,
            branch_id: project.branch_id,
            name: project.name.clone(),
        }
    }
}

#[derive(diesel::QueryableByName)]
struct BotIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct ProjectIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

/// Insert a `bots` row. `ON CONFLICT (slug) DO NOTHING` keeps another
/// branch's bot untouchable.
fn insert_bot_row(
    conn: &mut PgConnection,
    id: Uuid,
    name: &str,
    slug: &str,
    description: &str,
    org_id: Uuid,
    branch_id: Uuid,
    database_name: &str,
) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO bots (id, name, slug, description, org_id, branch_id, \
             database_name, llm_provider, llm_config, context_provider, context_config, \
             is_active, is_public, origin, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'openai', '{}'::jsonb, 'openai', '{}'::jsonb, \
             true, true, 'vibe', NOW(), NOW()) \
         ON CONFLICT (slug) DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(name)
    .bind::<diesel::sql_types::Text, _>(slug)
    .bind::<diesel::sql_types::Text, _>(description)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(database_name)
    .execute(conn)
    .map_err(|e| format!("insert bots row {slug}: {e}"))?;
    Ok(())
}

/// Look up the branch's bot for `slug-or-name` (returns its id).
fn find_branch_bot(
    conn: &mut PgConnection,
    branch_id: Uuid,
    slug_or_name: &str,
) -> Option<BotIdRow> {
    diesel::sql_query(
        "SELECT id FROM bots WHERE branch_id = $1 AND (slug = $2 OR name = $2) \
         ORDER BY created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(slug_or_name)
    .get_result::<BotIdRow>(conn)
    .optional()
    .ok()
    .flatten()
}

/// Ensure the two-environment bot pair for a bot-kind project (#1504):
/// PROD `{bot}` (adopt the branch's existing bot when present, otherwise
/// create) and TEST `{bot}-test` (always vibe-owned, `_dev` database).
/// `payload.bot_id` records the adoption so downstream flows reuse it.
pub fn ensure_bot_rows(
    pool: &botcore::shared::utils::DbPool,
    bot: &BotRowRef,
    description: Option<&str>,
) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("vibe bootstrap: pool unavailable: {e}");
            return;
        }
    };
    if let Err(e) = ensure_bot_rows_conn(&mut conn, bot, description) {
        log::error!("vibe bootstrap: bot pair for project {}: {e}", bot.id);
    }
}

pub(crate) fn ensure_bot_rows_conn(
    conn: &mut PgConnection,
    bot: &BotRowRef,
    description: Option<&str>,
) -> Result<(), String> {
    let prod_slug = bot_slug(&bot.name);
    let test_slug = format!("{prod_slug}-test");
    let desc = description
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .unwrap_or("Vibe bot project");

    // PROD: adopt (preferred) or create.
    match find_branch_bot(conn, bot.branch_id, &prod_slug) {
        Some(row) => {
            let payload_patch = serde_json::json!({ "bot_id": row.id.to_string() });
            diesel::sql_query(
                "UPDATE vibe_projects SET payload = payload || $2::jsonb, updated_at = NOW() \
                 WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(bot.id)
            .bind::<diesel::sql_types::Jsonb, _>(&payload_patch)
            .execute(conn)
            .map_err(|e| format!("link project {} to bot: {e}", bot.id))?;
            log::info!(
                "vibe bootstrap: project {} adopted bot {} (branch {})",
                bot.id,
                row.id,
                bot.branch_id
            );
        }
        None => {
            let id = Uuid::new_v4();
            let database_name =
                crate::project_db::project_database_name(bot.branch_id, &bot.name, "production");
            insert_bot_row(
                conn,
                id,
                &bot.name,
                &prod_slug,
                desc,
                bot.org_id,
                bot.branch_id,
                &database_name,
            )?;
            log::info!(
                "vibe bootstrap: created PROD bot {prod_slug} (branch {})",
                bot.branch_id
            );
        }
    }

    // TEST twin: always fresh, never adopted, `_dev` database (#1386).
    if find_branch_bot(conn, bot.branch_id, &test_slug).is_none() {
        let id = Uuid::new_v4();
        let database_name =
            crate::project_db::project_database_name(bot.branch_id, &bot.name, "test");
        insert_bot_row(
            conn,
            id,
            &format!("{}-test", bot.name),
            &test_slug,
            &format!("Vibe TEST bot for {}", bot.name),
            bot.org_id,
            bot.branch_id,
            &database_name,
        )?;
        log::info!(
            "vibe bootstrap: created TEST bot {test_slug} (branch {})",
            bot.branch_id
        );
    }
    Ok(())
}

/// Create the branch's default bot project (#1500). Idempotent: an existing
/// `(branch_id, name)` project is returned untouched (bot pair re-checked).
pub(crate) fn ensure_branch_default_project_conn(
    conn: &mut PgConnection,
    org_id: Uuid,
    branch_id: Uuid,
    name: &str,
) -> Result<Uuid, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || branch_id.is_nil() {
        return Err("vibe bootstrap: branch name/id required".to_string());
    }
    if let Err(e) = conn.batch_execute(VIBE_PROJECTS_FALLBACK_SCHEMA) {
        log::warn!("vibe bootstrap: schema fallback failed (may already exist): {e}");
    }

    let existing = diesel::sql_query(
        "SELECT id FROM vibe_projects WHERE branch_id = $1 AND name = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(trimmed)
    .get_result::<ProjectIdRow>(conn)
    .optional()
    .map_err(|e| format!("vibe bootstrap lookup: {e}"))?;

    let project_id = match existing {
        Some(row) => row.id,
        None => {
            let id = Uuid::new_v4();
            let payload = serde_json::json!({ "bootstrap": true, "source": "workspace" });
            diesel::sql_query(
                "INSERT INTO vibe_projects \
                 (id, org_id, branch_id, name, project_type, repository, framework, custom_domain, \
                  source_control, status, environment, payload, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, 'bot', $4, '', '', 'git', 'ready', 'development', $5, NOW(), NOW()) \
                 ON CONFLICT (branch_id, name) DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(id)
            .bind::<diesel::sql_types::Uuid, _>(org_id)
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .bind::<diesel::sql_types::Text, _>(trimmed)
            .bind::<diesel::sql_types::Jsonb, _>(&payload)
            .execute(conn)
            .map_err(|e| format!("vibe bootstrap insert: {e}"))?;
            let row = diesel::sql_query(
                "SELECT id FROM vibe_projects WHERE branch_id = $1 AND name = $2",
            )
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .bind::<diesel::sql_types::Text, _>(trimmed)
            .get_result::<ProjectIdRow>(conn)
            .map_err(|e| format!("vibe bootstrap reselect: {e}"))?;
            log::info!(
                "vibe bootstrap: default bot project '{}' created for branch {}",
                trimmed,
                branch_id
            );
            row.id
        }
    };

    ensure_bot_rows_conn(
        conn,
        &BotRowRef {
            id: project_id,
            org_id,
            branch_id,
            name: trimmed.to_string(),
        },
        Some(&format!("Default bot of branch {trimmed}")),
    )?;
    Ok(project_id)
}

/// Pool-based wrapper (Drive monitors, signup hook, backfill).
pub fn ensure_branch_default_project(
    pool: &botcore::shared::utils::DbPool,
    org_id: Uuid,
    branch_id: Uuid,
    name: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
    ensure_branch_default_project_conn(&mut conn, org_id, branch_id, name)
}

#[cfg(test)]
mod tests {
    use super::bot_slug;

    #[test]
    fn bot_slug_normalizes() {
        assert_eq!(bot_slug("My Bot_Name"), "my-bot-name");
        assert_eq!(bot_slug("cristo"), "cristo");
        assert_eq!(bot_slug("weird!!name"), "weirdname");
    }

    #[test]
    fn alm_org_slug_is_url_safe() {
        assert_eq!(super::alm_org_from_slug("My Branch"), "mybranch");
        assert_eq!(super::alm_org_from_slug("cristo"), "cristo");
    }
}
