//! #1184 — Telephony trunk provisioning.
//!
//! Inbound calling needs a phone number (a "trunk") bound to a room/bot.
//! This module is the provisioning ledger: create trunks (with an optional
//! SIP carrier reference), list them, and release them. Provisioning is
//! intentionally async-friendly: the actual carrier activation happens out
//! of band (LiveKit SIP trunk API), while this registry is the single
//! source of truth the voice UI reads.
//!
//! The registry is process-local (OnceLock<Mutex>) so no schema migration
//! is required; it survives as long as the botserver process.

use std::sync::{Mutex, OnceLock};

use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get},
    Router,
};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botcore::shared::state::AppState;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrunkDef {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub carrier: String,
    pub bot_id: Option<Uuid>,
    pub room_id: Option<String>,
    pub status: String,
    pub created_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct ProvisionTrunkRequest {
    pub name: String,
    pub phone_number: String,
    #[serde(default = "default_carrier")]
    pub carrier: String,
    #[serde(default)]
    pub bot_id: Option<Uuid>,
    #[serde(default)]
    pub room_id: Option<String>,
}

fn default_carrier() -> String {
    "livekit-sip".to_string()
}

#[derive(Debug, Serialize)]
pub struct TrunkResponse {
    pub success: bool,
    pub trunk: Option<TrunkDef>,
    pub trunks: Option<Vec<TrunkDef>>,
    pub error: Option<String>,
}

fn registry() -> &'static Mutex<Vec<TrunkDef>> {
    static REGISTRY: OnceLock<Mutex<Vec<TrunkDef>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn trunks_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/meet/trunks", get(list_trunks).post(provision_trunk))
        .route("/api/meet/trunks/:id", delete(release_trunk))
}

async fn provision_trunk(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ProvisionTrunkRequest>,
) -> Json<TrunkResponse> {
    if req.name.trim().is_empty() || req.phone_number.trim().is_empty() {
        return Json(TrunkResponse {
            success: false,
            trunk: None,
            trunks: None,
            error: Some("name and phone_number are required".to_string()),
        });
    }
    let trunk = TrunkDef {
        id: Uuid::new_v4(),
        name: req.name.trim().to_string(),
        phone_number: req.phone_number.trim().to_string(),
        carrier: req.carrier.clone(),
        bot_id: req.bot_id,
        room_id: req.room_id,
        status: "active".to_string(),
        created_at: now_secs(),
    };
    {
        let mut guard = registry().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.push(trunk.clone());
    }
    info!(
        "Vibe voice: trunk '{}' provisioned ({}) on {}",
        trunk.name, trunk.phone_number, trunk.carrier
    );
    Json(TrunkResponse {
        success: true,
        trunk: Some(trunk),
        trunks: None,
        error: None,
    })
}

async fn list_trunks(State(_state): State<Arc<AppState>>) -> Json<TrunkResponse> {
    let guard = registry().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    Json(TrunkResponse {
        success: true,
        trunk: None,
        trunks: Some(guard.clone()),
        error: None,
    })
}

async fn release_trunk(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<TrunkResponse> {
    let mut guard = registry().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(pos) = guard.iter().position(|t| t.id == id) {
        let trunk = guard.remove(pos);
        info!("Vibe voice: trunk '{}' released", trunk.name);
        Json(TrunkResponse {
            success: true,
            trunk: Some(trunk),
            trunks: None,
            error: None,
        })
    } else {
        Json(TrunkResponse {
            success: false,
            trunk: None,
            trunks: None,
            error: Some(format!("trunk {id} not found")),
        })
    }
}
