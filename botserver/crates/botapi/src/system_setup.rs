//! Setup-wizard endpoints, split out of `system.rs` to keep both modules
//! comfortably under the repository's 450-line cap.
//!
//! #2747: the wizard's final step writes to `bots`. Every untrusted value is
//! passed as a Diesel bind parameter — no request data is ever interpolated
//! into SQL text. The endpoint is admin-gated by the RBAC route table
//! (`/api/setup/**` POST in `botsecurity-auth/rbac_middleware/routes_admin.rs`).

use axum::{extract::State, response::Json};
use botcore::shared::state::AppState;
use diesel::prelude::*;
use diesel::sql_query;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Parameterized upsert for the wizard's final step (#2747).
///
/// `$1`..`$6` are Diesel bind placeholders: `bot_id`, `name`, `slug`,
/// `org_id`, `llm_provider`, `description`. The only single-quoted tokens are
/// the two static, empty JSON literals for the config columns. Keeping the
/// statement in a constant makes the "no interpolation" property testable.
pub(crate) const BOT_SETUP_INSERT_SQL: &str =
    "INSERT INTO bots (id, name, slug, org_id, is_active, created_at, updated_at, \
     llm_provider, llm_config, context_provider, context_config, description, is_public) \
     VALUES ($1, $2, $3, $4, true, NOW(), NOW(), $5, '{}', 'openai', '{}', $6, true) \
     ON CONFLICT (slug, org_id) DO UPDATE SET name = EXCLUDED.name, \
     description = EXCLUDED.description, llm_provider = EXCLUDED.llm_provider, \
     updated_at = NOW()";

/// Derives a Drive-safe slug from a bot display name.
///
/// Bind parameters already make the SQL inert (#2747); normalization is
/// defense-in-depth because the same slug becomes an object-key prefix in
/// `apply_bot_template`. It keeps only lowercase ASCII letters, digits and
/// single hyphens, so quotes, semicolons and path separators cannot survive.
pub(crate) fn safe_bot_slug(bot_name: &str) -> String {
    let mut slug = String::with_capacity(bot_name.len());
    let mut last_was_separator = false;
    for ch in bot_name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('-');
            last_was_separator = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "assistant".to_string()
    } else {
        slug.to_string()
    }
}

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub setup_complete: bool,
}

/// #2767 — Shapes the only response this endpoint is allowed to return: a
/// single boolean the setup wizard needs, and nothing about *what* is
/// configured (no component versions, hosts, readiness details or counts).
///
/// `authenticated` is false for a caller whose identity cannot be verified;
/// such a caller always receives the same constant (`true`) regardless of the
/// real database state, so probing cannot fingerprint a fresh install. The
/// same constant is used for every unknown state (wizard disabled, DB
/// unavailable). Only an authenticated wizard can ever observe `false`.
pub(crate) fn shape_setup_status(
    authenticated: bool,
    wizard_enabled: bool,
    bot_count: Option<i64>,
) -> SetupStatusResponse {
    let setup_complete = match (authenticated && wizard_enabled, bot_count) {
        (true, Some(count)) => count > 0,
        _ => true,
    };
    SetupStatusResponse { setup_complete }
}

#[derive(Deserialize)]
pub struct ConfigureSetupRequest {
    pub step: u8,
    pub data: ConfigureSetupData,
}

#[derive(Deserialize)]
pub struct ConfigureSetupData {
    pub llm_provider: Option<String>,
    pub user_profile: Option<String>,
    pub bot_name: Option<String>,
    pub bot_purpose: Option<String>,
    pub bot_template: Option<String>,
    pub training_files: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct ConfigureSetupResponse {
    pub success: bool,
    pub setup_complete: bool,
    pub error: Option<String>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BotCountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct OrgSetupResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: uuid::Uuid,
}

pub async fn get_setup_status(State(state): State<Arc<AppState>>) -> Json<SetupStatusResponse> {
    let wizard_enabled = std::env::var("GB_FEATURE_SETUP_WIZARD")
        .map(|value| value == "true")
        .unwrap_or(false);

    let bot_count = if wizard_enabled {
        match state.conn.get() {
            Ok(mut conn) => sql_query("SELECT COUNT(*) as count FROM bots")
                .get_result::<BotCountResult>(&mut conn)
                .ok()
                .map(|res: BotCountResult| res.count),
            Err(_) => None,
        }
    } else {
        None
    };

    // This route is no longer anonymous (#2767): the auth middleware has
    // already authenticated the caller before the handler runs, so the wizard
    // bit is only ever computed for a trusted caller.
    Json(shape_setup_status(true, wizard_enabled, bot_count))
}

/// Creates the initial bot on wizard step 4.
///
/// Authorization is enforced before this handler runs by the RBAC middleware:
/// `POST /api/setup/configure` is registered with `Admin`/`SuperAdmin` roles.
pub async fn configure_setup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfigureSetupRequest>,
) -> Json<ConfigureSetupResponse> {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return Json(ConfigureSetupResponse {
                success: false,
                setup_complete: false,
                error: Some(format!("Database connection error: {e}")),
            })
        }
    };

    if payload.step == 4 {
        let bot_name = payload
            .data
            .bot_name
            .unwrap_or_else(|| "My Assistant".to_string());
        let bot_purpose = payload.data.bot_purpose.unwrap_or_else(|| "".to_string());
        let llm_provider = payload
            .data
            .llm_provider
            .unwrap_or_else(|| "openai".to_string());

        let org_res: Result<OrgSetupResult, _> =
            sql_query("SELECT org_id FROM organizations WHERE slug = 'default' LIMIT 1")
                .get_result(&mut conn);

        let org_id = match org_res {
            Ok(res) => res.org_id,
            Err(_) => uuid::Uuid::nil(),
        };

        let bot_id = uuid::Uuid::new_v4();
        let slug = safe_bot_slug(&bot_name);

        let insert = sql_query(BOT_SETUP_INSERT_SQL)
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(&bot_name)
            .bind::<diesel::sql_types::Text, _>(&slug)
            .bind::<diesel::sql_types::Uuid, _>(org_id)
            .bind::<diesel::sql_types::Text, _>(&llm_provider)
            .bind::<diesel::sql_types::Text, _>(&bot_purpose);

        match insert.execute(&mut conn) {
            Ok(_) => {
                log::info!(
                    "Setup Wizard: Bot '{}' created/updated successfully",
                    bot_name
                );

                let template = payload.data.bot_template.filter(|t| !t.is_empty());
                if let Some(template_path) = &template {
                    if let Err(e) = apply_bot_template(&state, &slug, template_path).await {
                        log::warn!(
                            "Setup Wizard: Template '{}' applied with warnings: {}",
                            template_path,
                            e
                        );
                    }
                }

                Json(ConfigureSetupResponse {
                    success: true,
                    setup_complete: true,
                    error: None,
                })
            }
            Err(e) => Json(ConfigureSetupResponse {
                success: false,
                setup_complete: false,
                error: Some(format!("Failed to create initial bot: {e}")),
            }),
        }
    } else {
        Json(ConfigureSetupResponse {
            success: true,
            setup_complete: false,
            error: None,
        })
    }
}

async fn apply_bot_template(
    state: &AppState,
    bot_slug: &str,
    template_path: &str,
) -> Result<(), String> {
    let templates_dir =
        std::env::var("BOT_TEMPLATES_DIR").unwrap_or_else(|_| "work/templates/bots".to_string());
    let src_dir = PathBuf::from(&templates_dir).join(template_path);

    if !src_dir.exists() {
        return Err(format!(
            "Template directory not found: {}",
            src_dir.display()
        ));
    }

    let drive = match &state.drive {
        Some(d) => d,
        None => return Err("Drive not available".to_string()),
    };

    let bucket = &state.bucket_name;
    if bucket.is_empty() {
        return Err("Bucket name is empty".to_string());
    }

    let mut files_uploaded = 0u64;
    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .map_err(|e| format!("Failed to read template dir: {e}"))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = entry.file_name();
        let dir_str = dir_name.to_string_lossy();
        let target_subdir = if dir_str.ends_with(".gbdialog")
            || dir_str.ends_with(".gbot")
            || dir_str.ends_with(".gbkb")
            || dir_str.ends_with(".gbdrive")
        {
            format!(
                "{bot_slug}.{}",
                dir_str.trim_start_matches(|c: char| !c.is_ascii_punctuation())
            )
        } else {
            dir_str.to_string()
        };

        let mut files: Vec<_> = match walk_dir(&path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("Setup Wizard: Could not walk dir {}: {}", path.display(), e);
                continue;
            }
        };
        files.sort();

        for file_path in &files {
            let relative = file_path
                .strip_prefix(&src_dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();
            let key = format!("{bot_slug}.gbai/{target_subdir}/{relative}");

            let data = match tokio::fs::read(file_path).await {
                Ok(d) => d,
                Err(e) => {
                    log::warn!(
                        "Setup Wizard: Could not read {}: {}",
                        file_path.display(),
                        e
                    );
                    continue;
                }
            };

            let ct = guess_content_type(file_path);
            let _ = drive.put_object(bucket, &key, data, Some(ct)).await;
            files_uploaded += 1;
        }
    }

    log::info!(
        "Setup Wizard: Applied template '{}' to bot '{}' ({} files)",
        template_path,
        bot_slug,
        files_uploaded
    );
    Ok(())
}

fn walk_dir(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                let sub = walk_dir(&path)?;
                files.extend(sub);
            } else {
                files.push(entry.path());
            }
        }
    }
    Ok(files)
}

pub(crate) fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("pdf") => "application/pdf",
        Some("bas") | Some("basic") => "text/plain",
        Some("gbkb") => "application/octet-stream",
        Some("ast") => "application/octet-stream",
        Some("csv") => "text/csv",
        Some("md") => "text/markdown",
        Some("yaml") | Some("yml") => "application/yaml",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}
