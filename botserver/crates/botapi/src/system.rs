use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use botcore::shared::state::AppState;

pub fn configure_system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/system/versions", get(get_versions))
        .route("/api/system/check-updates", post(check_updates))
        .route("/api/setup/status", get(get_setup_status))
        .route("/api/setup/configure", post(configure_setup))
}

#[derive(Serialize)]
pub struct SystemVersionsResponse {
    pub botserver: String,
    pub botui: String,
    pub rust: String,
    pub postgresql: String,
    pub valkey: String,
    pub minio: String,
    pub qdrant: String,
    pub vault: String,
    pub zitadel: String,
}

pub async fn get_versions(
    State(_state): State<Arc<AppState>>,
) -> Json<SystemVersionsResponse> {
    Json(SystemVersionsResponse {
        botserver: env!("CARGO_PKG_VERSION").to_string(),
        botui: "6.3.1".to_string(),
        rust: "1.75.0".to_string(),
        postgresql: "16.1".to_string(),
        valkey: "8.0.2".to_string(),
        minio: "2026.03.15".to_string(),
        qdrant: "1.7.0".to_string(),
        vault: "1.15.0".to_string(),
        zitadel: "2.45.0".to_string(),
    })
}

#[derive(Deserialize)]
pub struct CheckUpdateRequest {
    pub component: String,
}

#[derive(Serialize)]
pub struct CheckUpdateResponse {
    pub component: String,
    pub current: String,
    pub latest: String,
    pub update_available: bool,
}

pub async fn check_updates(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<CheckUpdateRequest>,
) -> Json<CheckUpdateResponse> {
    let current_ver = if payload.component == "botserver" {
        env!("CARGO_PKG_VERSION").to_string()
    } else {
        "6.3.1".to_string()
    };
    Json(CheckUpdateResponse {
        component: payload.component,
        current: current_ver.clone(),
        latest: current_ver,
        update_available: false,
    })
}

use diesel::prelude::*;
use diesel::sql_query;

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub setup_complete: bool,
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

pub async fn get_setup_status(
    State(state): State<Arc<AppState>>,
) -> Json<SetupStatusResponse> {
    let wizard_enabled = std::env::var("GB_FEATURE_SETUP_WIZARD")
        .unwrap_or_else(|_| "false".to_string())
        == "true";

    if !wizard_enabled {
        return Json(SetupStatusResponse { setup_complete: true });
    }

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => return Json(SetupStatusResponse { setup_complete: true }),
    };

    let result: Result<BotCountResult, _> = sql_query("SELECT COUNT(*) as count FROM bots")
        .get_result(&mut conn);

    match result {
        Ok(res) => Json(SetupStatusResponse { setup_complete: res.count > 0 }),
        Err(_) => Json(SetupStatusResponse { setup_complete: true }),
    }
}

pub async fn configure_setup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfigureSetupRequest>,
) -> Json<ConfigureSetupResponse> {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => return Json(ConfigureSetupResponse {
            success: false,
            setup_complete: false,
            error: Some(format!("Database connection error: {}", e)),
        }),
    };

    if payload.step == 4 {
        let bot_name = payload.data.bot_name.unwrap_or_else(|| "My Assistant".to_string());
        let bot_purpose = payload.data.bot_purpose.unwrap_or_else(|| "".to_string());
        let llm_provider = payload.data.llm_provider.unwrap_or_else(|| "openai".to_string());

        let org_res: Result<OrgSetupResult, _> = sql_query("SELECT org_id FROM organizations WHERE slug = 'default' LIMIT 1")
            .get_result(&mut conn);

        let org_id = match org_res {
            Ok(res) => res.org_id,
            Err(_) => uuid::Uuid::nil(),
        };

        let bot_id = uuid::Uuid::new_v4();
        let slug = bot_name.to_lowercase().replace(' ', "-");

        let query = format!(
            "INSERT INTO bots (id, name, slug, org_id, is_active, created_at, updated_at, llm_provider, llm_config, context_provider, context_config, description, is_public) \
             VALUES ('{}', '{}', '{}', '{}', true, NOW(), NOW(), '{}', '{{}}', 'openai', '{{}}', '{}', true) \
             ON CONFLICT (slug, org_id) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description, llm_provider = EXCLUDED.llm_provider, updated_at = NOW()",
            bot_id, bot_name.replace("'", "''"), slug, org_id, llm_provider, bot_purpose.replace("'", "''")
        );

        match sql_query(query).execute(&mut conn) {
            Ok(_) => {
                log::info!("Setup Wizard: Bot '{}' created/updated successfully", bot_name);

                let template = payload.data.bot_template.filter(|t| !t.is_empty());
                if let Some(template_path) = &template {
                    if let Err(e) = apply_bot_template(&state, &slug, template_path).await {
                        log::warn!("Setup Wizard: Template '{}' applied with warnings: {}", template_path, e);
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
                error: Some(format!("Failed to create initial bot: {}", e)),
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
    let templates_dir = std::env::var("BOT_TEMPLATES_DIR")
        .unwrap_or_else(|_| "work/templates/bots".to_string());
    let src_dir = PathBuf::from(&templates_dir).join(template_path);

    if !src_dir.exists() {
        return Err(format!("Template directory not found: {}", src_dir.display()));
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
        .map_err(|e| format!("Failed to read template dir: {}", e))?
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
            format!("{bot_slug}.{}", dir_str.trim_start_matches(|c: char| !c.is_ascii_punctuation()))
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
                    log::warn!("Setup Wizard: Could not read {}: {}", file_path.display(), e);
                    continue;
                }
            };

            let ct = guess_content_type(file_path);
            let _ = drive.put_object(bucket, &key, data, Some(ct)).await;
            files_uploaded += 1;
        }
    }

    log::info!("Setup Wizard: Applied template '{}' to bot '{}' ({} files)", template_path, bot_slug, files_uploaded);
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

fn guess_content_type(path: &Path) -> &'static str {
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
