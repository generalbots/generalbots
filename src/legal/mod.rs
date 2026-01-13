pub mod account_deletion;
pub mod ui;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{
    consent_history, cookie_consents, data_deletion_requests, data_export_requests,
    legal_acceptances, legal_document_versions, legal_documents,
};
use crate::shared::state::AppState;

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = legal_documents)]
pub struct DbLegalDocument {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub document_type: String,
    pub version: String,
    pub effective_date: DateTime<Utc>,
    pub is_active: bool,
    pub requires_acceptance: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = legal_document_versions)]
pub struct DbDocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: String,
    pub content: String,
    pub change_summary: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = cookie_consents)]
pub struct DbCookieConsent {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub country_code: Option<String>,
    pub consent_necessary: bool,
    pub consent_analytics: bool,
    pub consent_marketing: bool,
    pub consent_preferences: bool,
    pub consent_functional: bool,
    pub consent_version: String,
    pub consent_given_at: DateTime<Utc>,
    pub consent_updated_at: DateTime<Utc>,
    pub consent_withdrawn_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = consent_history)]
pub struct DbConsentHistory {
    pub id: Uuid,
    pub consent_id: Uuid,
    pub action: String,
    pub previous_consents: serde_json::Value,
    pub new_consents: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = legal_acceptances)]
pub struct DbLegalAcceptance {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub document_id: Uuid,
    pub document_version: String,
    pub accepted_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = data_deletion_requests)]
pub struct DbDeletionRequest {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub request_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub confirmation_token: String,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = data_export_requests)]
pub struct DbExportRequest {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub format: String,
    pub include_sections: serde_json::Value,
    pub requested_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub file_url: Option<String>,
    pub file_size: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CookieCategory {
    Necessary,
    Analytics,
    Marketing,
    Preferences,
    Functional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieConsent {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub consents: HashMap<CookieCategory, bool>,
    pub consent_given_at: DateTime<Utc>,
    pub consent_updated_at: DateTime<Utc>,
    pub consent_version: String,
    pub user_agent: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CookieConsentRequest {
    pub session_id: Option<String>,
    pub consents: HashMap<CookieCategory, bool>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CookieConsentResponse {
    pub id: Uuid,
    pub consents: HashMap<CookieCategory, bool>,
    pub consent_given_at: DateTime<Utc>,
    pub consent_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookiePolicy {
    pub version: String,
    pub effective_date: DateTime<Utc>,
    pub categories: Vec<CookieCategoryInfo>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieCategoryInfo {
    pub category: CookieCategory,
    pub name: String,
    pub description: String,
    pub is_required: bool,
    pub cookies: Vec<CookieInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInfo {
    pub name: String,
    pub provider: String,
    pub purpose: String,
    pub expiry: String,
    pub cookie_type: CookieType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CookieType {
    Session,
    Persistent,
    FirstParty,
    ThirdParty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalDocument {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub version: String,
    pub effective_date: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub document_type: LegalDocumentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LegalDocumentType {
    PrivacyPolicy,
    TermsOfService,
    CookiePolicy,
    AcceptableUsePolicy,
    DataProcessingAgreement,
    GdprCompliance,
    CcpaCompliance,
}

impl std::fmt::Display for LegalDocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PrivacyPolicy => "privacy_policy",
            Self::TermsOfService => "terms_of_service",
            Self::CookiePolicy => "cookie_policy",
            Self::AcceptableUsePolicy => "acceptable_use_policy",
            Self::DataProcessingAgreement => "data_processing_agreement",
            Self::GdprCompliance => "gdpr_compliance",
            Self::CcpaCompliance => "ccpa_compliance",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for LegalDocumentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "privacy_policy" => Ok(Self::PrivacyPolicy),
            "terms_of_service" => Ok(Self::TermsOfService),
            "cookie_policy" => Ok(Self::CookiePolicy),
            "acceptable_use_policy" => Ok(Self::AcceptableUsePolicy),
            "data_processing_agreement" => Ok(Self::DataProcessingAgreement),
            "gdpr_compliance" => Ok(Self::GdprCompliance),
            "ccpa_compliance" => Ok(Self::CcpaCompliance),
            _ => Err(format!("Unknown document type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DataDeletionResult {
    pub user_id: Uuid,
    pub consents_deleted: i32,
    pub deleted_at: DateTime<Utc>,
    pub confirmation_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserDataExport {
    pub user_id: Uuid,
    pub exported_at: DateTime<Utc>,
    pub consents: Vec<CookieConsent>,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub document_type: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub document_type: LegalDocumentType,
    pub version: Option<String>,
    pub requires_acceptance: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_active: Option<bool>,
    pub requires_acceptance: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DataDeletionRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DataExportRequest {
    pub format: Option<String>,
    pub sections: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum LegalError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for LegalError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) | Self::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

fn db_consent_to_consent(db: DbCookieConsent) -> CookieConsent {
    let mut consents = HashMap::new();
    consents.insert(CookieCategory::Necessary, db.consent_necessary);
    consents.insert(CookieCategory::Analytics, db.consent_analytics);
    consents.insert(CookieCategory::Marketing, db.consent_marketing);
    consents.insert(CookieCategory::Preferences, db.consent_preferences);
    consents.insert(CookieCategory::Functional, db.consent_functional);

    CookieConsent {
        id: db.id,
        user_id: db.user_id,
        session_id: db.session_id,
        ip_address: db.ip_address,
        consents,
        consent_given_at: db.consent_given_at,
        consent_updated_at: db.consent_updated_at,
        consent_version: db.consent_version,
        user_agent: db.user_agent,
        country_code: db.country_code,
    }
}

fn db_document_to_document(db: DbLegalDocument) -> LegalDocument {
    let document_type: LegalDocumentType = db.document_type.parse().unwrap_or(LegalDocumentType::PrivacyPolicy);

    LegalDocument {
        id: db.id.to_string(),
        title: db.title,
        slug: db.slug,
        content: db.content,
        version: db.version,
        effective_date: db.effective_date,
        last_updated: db.updated_at,
        document_type,
    }
}

fn get_default_cookie_policy() -> CookiePolicy {
    let now = Utc::now();
    CookiePolicy {
        version: "1.0.0".to_string(),
        effective_date: now,
        last_updated: now,
        categories: vec![
            CookieCategoryInfo {
                category: CookieCategory::Necessary,
                name: "Necessary".to_string(),
                description: "Essential cookies required for the website to function properly.".to_string(),
                is_required: true,
                cookies: vec![
                    CookieInfo {
                        name: "session_id".to_string(),
                        provider: "General Bots".to_string(),
                        purpose: "Session management".to_string(),
                        expiry: "Session".to_string(),
                        cookie_type: CookieType::Session,
                    },
                ],
            },
            CookieCategoryInfo {
                category: CookieCategory::Analytics,
                name: "Analytics".to_string(),
                description: "Cookies that help us understand how visitors interact with our website.".to_string(),
                is_required: false,
                cookies: vec![],
            },
            CookieCategoryInfo {
                category: CookieCategory::Marketing,
                name: "Marketing".to_string(),
                description: "Cookies used to deliver relevant advertisements.".to_string(),
                is_required: false,
                cookies: vec![],
            },
            CookieCategoryInfo {
                category: CookieCategory::Preferences,
                name: "Preferences".to_string(),
                description: "Cookies that remember your preferences and settings.".to_string(),
                is_required: false,
                cookies: vec![],
            },
            CookieCategoryInfo {
                category: CookieCategory::Functional,
                name: "Functional".to_string(),
                description: "Cookies that enable enhanced functionality.".to_string(),
                is_required: false,
                cookies: vec![],
            },
        ],
    }
}

pub async fn handle_record_consent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CookieConsentRequest>,
) -> Result<Json<CookieConsentResponse>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let mut consents = req.consents;
        consents.insert(CookieCategory::Necessary, true);

        let db_consent = DbCookieConsent {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            user_id: None,
            session_id: req.session_id,
            ip_address: None,
            user_agent: req.user_agent,
            country_code: None,
            consent_necessary: true,
            consent_analytics: *consents.get(&CookieCategory::Analytics).unwrap_or(&false),
            consent_marketing: *consents.get(&CookieCategory::Marketing).unwrap_or(&false),
            consent_preferences: *consents.get(&CookieCategory::Preferences).unwrap_or(&false),
            consent_functional: *consents.get(&CookieCategory::Functional).unwrap_or(&false),
            consent_version: "1.0.0".to_string(),
            consent_given_at: now,
            consent_updated_at: now,
            consent_withdrawn_at: None,
            created_at: now,
        };

        diesel::insert_into(cookie_consents::table)
            .values(&db_consent)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(CookieConsentResponse {
            id: db_consent.id,
            consents,
            consent_given_at: db_consent.consent_given_at,
            consent_version: db_consent.consent_version,
        })
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_consent(
    State(state): State<Arc<AppState>>,
    Path(consent_id): Path<Uuid>,
) -> Result<Json<Option<CookieConsent>>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;

        let db_consent: Option<DbCookieConsent> = cookie_consents::table
            .find(consent_id)
            .first(&mut conn)
            .optional()
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(db_consent.map(db_consent_to_consent))
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ConsentBySessionQuery {
    pub session_id: String,
}

pub async fn handle_get_consent_by_session(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ConsentBySessionQuery>,
) -> Result<Json<Option<CookieConsent>>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;

        let db_consent: Option<DbCookieConsent> = cookie_consents::table
            .filter(cookie_consents::session_id.eq(&query.session_id))
            .order(cookie_consents::consent_given_at.desc())
            .first(&mut conn)
            .optional()
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(db_consent.map(db_consent_to_consent))
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_update_consent(
    State(state): State<Arc<AppState>>,
    Path(consent_id): Path<Uuid>,
    Json(req): Json<CookieConsentRequest>,
) -> Result<Json<CookieConsent>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut db_consent: DbCookieConsent = cookie_consents::table
            .find(consent_id)
            .first(&mut conn)
            .map_err(|_| LegalError::NotFound("Consent not found".to_string()))?;

        let previous_consents = serde_json::json!({
            "necessary": db_consent.consent_necessary,
            "analytics": db_consent.consent_analytics,
            "marketing": db_consent.consent_marketing,
            "preferences": db_consent.consent_preferences,
            "functional": db_consent.consent_functional,
        });

        let mut consents = req.consents;
        consents.insert(CookieCategory::Necessary, true);

        db_consent.consent_necessary = true;
        db_consent.consent_analytics = *consents.get(&CookieCategory::Analytics).unwrap_or(&false);
        db_consent.consent_marketing = *consents.get(&CookieCategory::Marketing).unwrap_or(&false);
        db_consent.consent_preferences = *consents.get(&CookieCategory::Preferences).unwrap_or(&false);
        db_consent.consent_functional = *consents.get(&CookieCategory::Functional).unwrap_or(&false);
        db_consent.consent_updated_at = now;

        let new_consents = serde_json::json!({
            "necessary": db_consent.consent_necessary,
            "analytics": db_consent.consent_analytics,
            "marketing": db_consent.consent_marketing,
            "preferences": db_consent.consent_preferences,
            "functional": db_consent.consent_functional,
        });

        diesel::update(cookie_consents::table.find(consent_id))
            .set(&db_consent)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        let history = DbConsentHistory {
            id: Uuid::new_v4(),
            consent_id,
            action: "update".to_string(),
            previous_consents,
            new_consents,
            ip_address: None,
            user_agent: req.user_agent,
            created_at: now,
        };

        diesel::insert_into(consent_history::table)
            .values(&history)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(db_consent_to_consent(db_consent))
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_cookie_policy(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<CookiePolicy>, LegalError> {
    Ok(Json(get_default_cookie_policy()))
}

pub async fn handle_list_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<LegalDocument>>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = legal_documents::table
            .filter(legal_documents::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(doc_type) = query.document_type {
            db_query = db_query.filter(legal_documents::document_type.eq(doc_type));
        }

        if query.active_only.unwrap_or(true) {
            db_query = db_query.filter(legal_documents::is_active.eq(true));
        }

        let db_docs: Vec<DbLegalDocument> = db_query
            .order(legal_documents::created_at.desc())
            .load(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        let docs: Vec<LegalDocument> = db_docs.into_iter().map(db_document_to_document).collect();
        Ok::<_, LegalError>(docs)
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_document(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Option<LegalDocument>>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let db_doc: Option<DbLegalDocument> = legal_documents::table
            .filter(legal_documents::bot_id.eq(bot_id))
            .filter(legal_documents::slug.eq(&slug))
            .filter(legal_documents::is_active.eq(true))
            .first(&mut conn)
            .optional()
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(db_doc.map(db_document_to_document))
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<LegalDocument>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let db_doc = DbLegalDocument {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            slug: req.slug,
            title: req.title,
            content: req.content,
            document_type: req.document_type.to_string(),
            version: req.version.unwrap_or_else(|| "1.0.0".to_string()),
            effective_date: now,
            is_active: true,
            requires_acceptance: req.requires_acceptance.unwrap_or(false),
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(legal_documents::table)
            .values(&db_doc)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(db_document_to_document(db_doc))
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_update_document(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<LegalDocument>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);
        let now = Utc::now();

        let mut db_doc: DbLegalDocument = legal_documents::table
            .filter(legal_documents::bot_id.eq(bot_id))
            .filter(legal_documents::slug.eq(&slug))
            .first(&mut conn)
            .map_err(|_| LegalError::NotFound("Document not found".to_string()))?;

        if let Some(title) = req.title {
            db_doc.title = title;
        }
        if let Some(content) = req.content {
            let version_record = DbDocumentVersion {
                id: Uuid::new_v4(),
                document_id: db_doc.id,
                version: db_doc.version.clone(),
                content: db_doc.content.clone(),
                change_summary: None,
                created_by: None,
                created_at: now,
            };

            diesel::insert_into(legal_document_versions::table)
                .values(&version_record)
                .execute(&mut conn)
                .map_err(|e| LegalError::Database(e.to_string()))?;

            db_doc.content = content;
        }
        if let Some(is_active) = req.is_active {
            db_doc.is_active = is_active;
        }
        if let Some(requires_acceptance) = req.requires_acceptance {
            db_doc.requires_acceptance = requires_acceptance;
        }
        db_doc.updated_at = now;

        diesel::update(legal_documents::table.find(db_doc.id))
            .set(&db_doc)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        Ok::<_, LegalError>(db_document_to_document(db_doc))
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_request_data_deletion(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<DataDeletionRequest>,
) -> Result<Json<DataDeletionResult>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();
        let token = Uuid::new_v4().to_string();

        let db_request = DbDeletionRequest {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            user_id,
            request_type: "full".to_string(),
            status: "pending".to_string(),
            reason: req.reason,
            requested_at: now,
            scheduled_for: Some(now + chrono::Duration::days(30)),
            completed_at: None,
            confirmation_token: token.clone(),
            confirmed_at: None,
            processed_by: None,
            notes: None,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(data_deletion_requests::table)
            .values(&db_request)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        let deleted_consents = diesel::delete(
            cookie_consents::table
                .filter(cookie_consents::bot_id.eq(bot_id))
                .filter(cookie_consents::user_id.eq(user_id)),
        )
        .execute(&mut conn)
        .unwrap_or(0);

        Ok::<_, LegalError>(DataDeletionResult {
            user_id,
            consents_deleted: deleted_consents as i32,
            deleted_at: now,
            confirmation_token: token,
        })
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_export_user_data(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<DataExportRequest>,
) -> Result<Json<UserDataExport>, LegalError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let format = req.format.unwrap_or_else(|| "json".to_string());
        let sections = req.sections.unwrap_or_else(|| vec!["all".to_string()]);

        let db_export = DbExportRequest {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            user_id,
            status: "completed".to_string(),
            format: format.clone(),
            include_sections: serde_json::to_value(&sections).unwrap_or_default(),
            requested_at: now,
            started_at: Some(now),
            completed_at: Some(now),
            file_url: None,
            file_size: None,
            expires_at: Some(now + chrono::Duration::days(7)),
            error_message: None,
            created_at: now,
        };

        diesel::insert_into(data_export_requests::table)
            .values(&db_export)
            .execute(&mut conn)
            .map_err(|e| LegalError::Database(e.to_string()))?;

        let db_consents: Vec<DbCookieConsent> = cookie_consents::table
            .filter(cookie_consents::bot_id.eq(bot_id))
            .filter(cookie_consents::user_id.eq(user_id))
            .load(&mut conn)
            .unwrap_or_default();

        let consents: Vec<CookieConsent> = db_consents.into_iter().map(db_consent_to_consent).collect();

        Ok::<_, LegalError>(UserDataExport {
            user_id,
            exported_at: now,
            consents,
            format,
        })
    })
    .await
    .map_err(|e| LegalError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub struct LegalService {}

impl LegalService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LegalService {
    fn default() -> Self {
        Self::new()
    }
}

pub fn configure_legal_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/legal/consent", post(handle_record_consent))
        .route("/api/legal/consent/:consent_id", get(handle_get_consent))
        .route("/api/legal/consent/:consent_id", put(handle_update_consent))
        .route("/api/legal/consent/session", get(handle_get_consent_by_session))
        .route("/api/legal/cookies/policy", get(handle_get_cookie_policy))
        .route("/api/legal/documents", get(handle_list_documents))
        .route("/api/legal/documents", post(handle_create_document))
        .route("/api/legal/documents/:slug", get(handle_get_document))
        .route("/api/legal/documents/:slug", put(handle_update_document))
        .route("/api/legal/gdpr/delete/:user_id", post(handle_request_data_deletion))
        .route("/api/legal/gdpr/export/:user_id", post(handle_export_user_data))
}
