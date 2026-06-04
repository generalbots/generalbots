use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Produto {
    pub nome: String,
    pub quantidade: i64,
    pub preco_unitario: f64,
    pub ncm: String,
    pub cfop: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFe {
    pub id: Uuid,
    pub numero: i64,
    pub destinatario: String,
    pub produtos: Vec<Produto>,
    pub cfop: String,
    pub valor_total: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFSe {
    pub id: Uuid,
    pub numero: i64,
    pub prestador: String,
    pub tomador: String,
    pub valor: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTe {
    pub id: Uuid,
    pub numero: i64,
    pub remetente: String,
    pub destinatario: String,
    pub valor: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpedFile {
    pub id: Uuid,
    pub filename: String,
    pub tipo: String,
    pub periodo: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNFe {
    pub numero: i64,
    pub destinatario: String,
    pub produtos: Vec<Produto>,
    pub cfop: String,
    pub valor_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNFSe {
    pub numero: i64,
    pub prestador: String,
    pub tomador: String,
    pub valor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCTe {
    pub numero: i64,
    pub remetente: String,
    pub destinatario: String,
    pub valor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpedGenerateRequest {
    pub tipo: String,
    pub periodo: String,
}

#[derive(Debug, Default)]
pub struct TaxState {
    pub nfe: HashMap<Uuid, NFe>,
    pub nfse: HashMap<Uuid, NFSe>,
    pub cte: HashMap<Uuid, CTe>,
    pub sped: Vec<SpedFile>,
}



pub fn create_tax_state() -> SharedTaxState {
    Arc::new(RwLock::new(TaxState::default()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateNFe {
    pub destinatario: Option<String>,
    pub produtos: Option<Vec<Produto>>,
    pub cfop: Option<String>,
    pub valor_total: Option<f64>,
}

async fn list_nfe(
    State(state): State<SharedTaxState>,
) -> Result<Json<Vec<NFe>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.nfe.values().cloned().collect()))
}

async fn create_nfe(
    State(state): State<SharedTaxState>,
    Json(input): Json<CreateNFe>,
) -> Result<(StatusCode, Json<NFe>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nfe = NFe {
        id: Uuid::new_v4(),
        numero: input.numero,
        destinatario: input.destinatario,
        produtos: input.produtos,
        cfop: input.cfop,
        valor_total: input.valor_total,
        status: "pendente".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    data.nfe.insert(nfe.id, nfe.clone());
    Ok((StatusCode::CREATED, Json(nfe)))
}

async fn get_nfe(
    State(state): State<SharedTaxState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NFe>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    data.nfe.get(&id).cloned().ok_or(StatusCode::NOT_FOUND).map(Json)
}

async fn update_nfe(
    State(state): State<SharedTaxState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNFe>,
) -> Result<Json<NFe>, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nfe = data.nfe.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(d) = input.destinatario {
        nfe.destinatario = d;
    }
    if let Some(p) = input.produtos {
        nfe.produtos = p;
    }
    if let Some(c) = input.cfop {
        nfe.cfop = c;
    }
    if let Some(v) = input.valor_total {
        nfe.valor_total = v;
    }
    Ok(Json(nfe.clone()))
}

async fn authorize_nfe(
    State(state): State<SharedTaxState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NFe>, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nfe = data.nfe.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    nfe.status = "autorizada".to_string();
    Ok(Json(nfe.clone()))
}

async fn list_nfse(
    State(state): State<SharedTaxState>,
) -> Result<Json<Vec<NFSe>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.nfse.values().cloned().collect()))
}

async fn create_nfse(
    State(state): State<SharedTaxState>,
    Json(input): Json<CreateNFSe>,
) -> Result<(StatusCode, Json<NFSe>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nfse = NFSe {
        id: Uuid::new_v4(),
        numero: input.numero,
        prestador: input.prestador,
        tomador: input.tomador,
        valor: input.valor,
        status: "pendente".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    data.nfse.insert(nfse.id, nfse.clone());
    Ok((StatusCode::CREATED, Json(nfse)))
}

async fn list_cte(
    State(state): State<SharedTaxState>,
) -> Result<Json<Vec<CTe>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.cte.values().cloned().collect()))
}

async fn create_cte(
    State(state): State<SharedTaxState>,
    Json(input): Json<CreateCTe>,
) -> Result<(StatusCode, Json<CTe>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cte = CTe {
        id: Uuid::new_v4(),
        numero: input.numero,
        remetente: input.remetente,
        destinatario: input.destinatario,
        valor: input.valor,
        status: "pendente".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    data.cte.insert(cte.id, cte.clone());
    Ok((StatusCode::CREATED, Json(cte)))
}

async fn list_sped(
    State(state): State<SharedTaxState>,
) -> Result<Json<Vec<SpedFile>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.sped.clone()))
}

async fn generate_sped(
    State(state): State<SharedTaxState>,
    Json(input): Json<SpedGenerateRequest>,
) -> Result<(StatusCode, Json<SpedFile>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file = SpedFile {
        id: Uuid::new_v4(),
        filename: format!("SPED_{}_{}.txt", input.tipo, input.periodo),
        tipo: input.tipo,
        periodo: input.periodo,
        created_at: Utc::now().to_rfc3339(),
    };
    data.sped.push(file.clone());
    Ok((StatusCode::CREATED, Json(file)))
}

pub fn routes() -> Router {
    let state = std::sync::Arc::new(std::sync::RwLock::new(Default::default()));
    Router::new()
        .route("/api/tax/nfe", get(list_nfe).post(create_nfe))
        .route(
            "/api/tax/nfe/{id}",
            get(get_nfe).put(update_nfe),
        )
        .route("/api/tax/nfe/{id}/authorize", post(authorize_nfe))
        .route("/api/tax/nfse", get(list_nfse).post(create_nfse))
        .route("/api/tax/cte", get(list_cte).post(create_cte))
        .route("/api/tax/sped", get(list_sped))
        .route("/api/tax/sped/generate", post(generate_sped))
        .with_state(state)
}
