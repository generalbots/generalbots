use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFe {
    pub id: Uuid,
    pub number: String,
    pub series: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub total: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub authorized_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFSe {
    pub id: Uuid,
    pub number: String,
    pub service_code: String,
    pub provider_cnpj: String,
    pub total: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTe {
    pub id: Uuid,
    pub number: String,
    pub sender_cnpj: String,
    pub recipient_cnpj: String,
    pub modality: String,
    pub total: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sped {
    pub id: Uuid,
    pub period: String,
    pub kind: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewNFe {
    pub number: String,
    pub series: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct NewNFSe {
    pub number: String,
    pub service_code: String,
    pub provider_cnpj: String,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCTe {
    pub number: String,
    pub sender_cnpj: String,
    pub recipient_cnpj: String,
    pub modality: String,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct NewSped {
    pub period: String,
    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub struct TaxCalculationRequest {
    pub service_value: String,
    pub branch_id: Option<String>,
    pub bot_id: Option<String>,
}

