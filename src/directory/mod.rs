use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod client;
pub mod groups;
pub mod router;
pub mod users;

use self::client::{ZitadelClient, ZitadelConfig};

#[allow(dead_code)]
pub struct AuthService {
    client: Arc<ZitadelClient>,
}

impl std::fmt::Debug for AuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService")
            .field("client", &"Arc<ZitadelClient>")
            .finish()
    }
}

impl AuthService {
    pub async fn new(config: ZitadelConfig) -> anyhow::Result<Self> {
        let client = ZitadelClient::new(config).await?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub fn client(&self) -> &ZitadelClient {
        &self.client
    }
}

pub async fn auth_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_name = params.get("bot_name").cloned().unwrap_or_default();

    let user_id = {
        let mut sm = state.session_manager.lock().await;
        match sm.get_or_create_anonymous_user(None) {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to create anonymous user: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to create user" })),
                );
            }
        }
    };

    let (bot_id, bot_name) = match tokio::task::spawn_blocking({
        let bot_name = bot_name.clone();
        let conn = state.conn.clone();
        move || {
            let mut db_conn = conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;
            use crate::shared::models::schema::bots::dsl::*;
            use diesel::prelude::*;
            match bots
                .filter(name.eq(&bot_name))
                .filter(is_active.eq(true))
                .select((id, name))
                .first::<(Uuid, String)>(&mut db_conn)
                .optional()
            {
                Ok(Some((id_val, name_val))) => Ok((id_val, name_val)),
                Ok(None) => match bots
                    .filter(is_active.eq(true))
                    .select((id, name))
                    .first::<(Uuid, String)>(&mut db_conn)
                    .optional()
                {
                    Ok(Some((id_val, name_val))) => Ok((id_val, name_val)),
                    Ok(None) => Err("No active bots found".to_string()),
                    Err(e) => Err(format!("DB error: {}", e)),
                },
                Err(e) => Err(format!("DB error: {}", e)),
            }
        }
    })
    .await
    {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            error!("{}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            );
        }
        Err(e) => {
            error!("Spawn blocking failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "DB thread error" })),
            );
        }
    };

    let session = {
        let mut sm = state.session_manager.lock().await;
        match sm.get_or_create_user_session(user_id, bot_id, "Auth Session") {
            Ok(Some(sess)) => sess,
            Ok(None) => {
                error!("Failed to create session");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to create session" })),
                );
            }
            Err(e) => {
                error!("Failed to create session: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                );
            }
        }
    };

    let auth_script_path = format!("./work/{}.gbai/{}.gbdialog/auth.ast", bot_name, bot_name);
    if tokio::fs::metadata(&auth_script_path).await.is_ok() {
        let auth_script = match tokio::fs::read_to_string(&auth_script_path).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read auth script: {}", e);
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "user_id": session.user_id,
                        "session_id": session.id,
                        "status": "authenticated"
                    })),
                );
            }
        };

        let state_clone = Arc::clone(&state);
        let session_clone = session.clone();
        match tokio::task::spawn_blocking(move || {
            let script_service = crate::basic::ScriptService::new(state_clone, session_clone);
            match script_service.compile(&auth_script) {
                Ok(ast) => match script_service.run(&ast) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Script execution error: {}", e)),
                },
                Err(e) => Err(format!("Script compilation error: {}", e)),
            }
        })
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                error!("Auth script error: {}", e);
            }
            Err(e) => {
                error!("Auth script task error: {}", e);
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "user_id": session.user_id,
            "session_id": session.id,
            "status": "authenticated"
        })),
    )
}
