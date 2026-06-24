use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
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
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => return Json(SetupStatusResponse { setup_complete: true }),
    };

    let result: Result<BotCountResult, _> = sql_query("SELECT COUNT(*) as count FROM bots")
        .get_result(&mut conn);

    let setup_complete = match result {
        Ok(res) => res.count > 0,
        Err(_) => true, // Fallback safe to prevent locking the screen on db errors
    };

    Json(SetupStatusResponse { setup_complete })
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

        let query = format!(
            "INSERT INTO bots (id, name, slug, org_id, is_active, created_at, updated_at, llm_provider, llm_config, context_provider, context_config, description, is_public) \
             VALUES ('{}', '{}', '{}', '{}', true, NOW(), NOW(), '{}', '{{}}', 'openai', '{{}}', '{}', true) ON CONFLICT DO NOTHING",
            bot_id, bot_name, bot_name, org_id, llm_provider, bot_purpose.replace("'", "''")
        );

        match sql_query(query).execute(&mut conn) {
            Ok(_) => {
                log::info!("Setup Wizard: Initial bot '{}' created successfully", bot_name);
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
