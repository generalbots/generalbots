pub mod account_deletion;

use axum::{
    extract::{Path, Query, State},
    response::Html,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<diesel::PgConnection>>;

diesel::table! {
    organizations (org_id) {
        org_id -> Uuid,
        tenant_id -> Uuid,
        name -> Text,
        slug -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    bots (id) {
        id -> Uuid,
    }
}

diesel::table! {
    legal_documents (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        slug -> Varchar,
        title -> Varchar,
        content -> Text,
        document_type -> Varchar,
        version -> Varchar,
        effective_date -> Timestamptz,
        is_active -> Bool,
        requires_acceptance -> Bool,
        metadata -> Jsonb,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    legal_document_versions (id) {
        id -> Uuid,
        document_id -> Uuid,
        version -> Varchar,
        content -> Text,
        change_summary -> Nullable<Text>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    cookie_consents (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Nullable<Uuid>,
        session_id -> Nullable<Varchar>,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        country_code -> Nullable<Varchar>,
        consent_necessary -> Bool,
        consent_analytics -> Bool,
        consent_marketing -> Bool,
        consent_preferences -> Bool,
        consent_functional -> Bool,
        consent_version -> Varchar,
        consent_given_at -> Timestamptz,
        consent_updated_at -> Timestamptz,
        consent_withdrawn_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    consent_history (id) {
        id -> Uuid,
        consent_id -> Uuid,
        action -> Varchar,
        previous_consents -> Jsonb,
        new_consents -> Jsonb,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    legal_acceptances (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        document_id -> Uuid,
        document_version -> Varchar,
        accepted_at -> Timestamptz,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    data_deletion_requests (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        request_type -> Varchar,
        status -> Varchar,
        reason -> Nullable<Text>,
        requested_at -> Timestamptz,
        scheduled_for -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        confirmation_token -> Varchar,
        confirmed_at -> Nullable<Timestamptz>,
        processed_by -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    data_export_requests (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        user_id -> Uuid,
        status -> Varchar,
        format -> Varchar,
        include_sections -> Jsonb,
        requested_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        file_url -> Nullable<Text>,
        file_size -> Nullable<Int4>,
        expires_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(legal_documents -> organizations (org_id));
diesel::joinable!(legal_documents -> bots (bot_id));
diesel::joinable!(legal_document_versions -> legal_documents (document_id));
diesel::joinable!(cookie_consents -> organizations (org_id));
diesel::joinable!(cookie_consents -> bots (bot_id));
diesel::joinable!(consent_history -> cookie_consents (consent_id));
diesel::joinable!(legal_acceptances -> organizations (org_id));
diesel::joinable!(legal_acceptances -> bots (bot_id));
diesel::joinable!(legal_acceptances -> legal_documents (document_id));
diesel::joinable!(data_deletion_requests -> organizations (org_id));
diesel::joinable!(data_deletion_requests -> bots (bot_id));
diesel::joinable!(data_export_requests -> organizations (org_id));
diesel::joinable!(data_export_requests -> bots (bot_id));

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

pub type GetDefaultBotFn = fn(&mut diesel::PgConnection) -> (Uuid, String);

pub async fn handle_record_consent(
    State(pool): State<Arc<DbPool>>,
    Json(req): Json<CookieConsentRequest>,
) -> Result<Json<CookieConsentResponse>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot_stub(&mut conn);
        let org_id = Uuid::nil();
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
    State(pool): State<Arc<DbPool>>,
    Path(consent_id): Path<Uuid>,
) -> Result<Json<Option<CookieConsent>>, LegalError> {
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
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<ConsentBySessionQuery>,
) -> Result<Json<Option<CookieConsent>>, LegalError> {
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
    State(pool): State<Arc<DbPool>>,
    Path(consent_id): Path<Uuid>,
    Json(req): Json<CookieConsentRequest>,
) -> Result<Json<CookieConsent>, LegalError> {
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
    State(_pool): State<Arc<DbPool>>,
) -> Result<Json<CookiePolicy>, LegalError> {
    Ok(Json(get_default_cookie_policy()))
}

pub async fn handle_list_documents(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<LegalDocument>>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot_stub(&mut conn);

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
    State(pool): State<Arc<DbPool>>,
    Path(slug): Path<String>,
) -> Result<Json<Option<LegalDocument>>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot_stub(&mut conn);

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
    State(pool): State<Arc<DbPool>>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<LegalDocument>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot_stub(&mut conn);
        let org_id = Uuid::nil();
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
    State(pool): State<Arc<DbPool>>,
    Path(slug): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<LegalDocument>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot_stub(&mut conn);
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
    State(pool): State<Arc<DbPool>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<DataDeletionRequest>,
) -> Result<Json<DataDeletionResult>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot_stub(&mut conn);
        let org_id = Uuid::nil();
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
    State(pool): State<Arc<DbPool>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<DataExportRequest>,
) -> Result<Json<UserDataExport>, LegalError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| LegalError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot_stub(&mut conn);
        let org_id = Uuid::nil();
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

pub fn configure_legal_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/api/legal/consent", post(handle_record_consent))
        .route("/api/legal/consent/:consent_id", get(handle_get_consent).put(handle_update_consent))
        .route("/api/legal/consent/session", get(handle_get_consent_by_session))
        .route("/api/legal/cookies/policy", get(handle_get_cookie_policy))
        .route("/api/legal/documents", get(handle_list_documents).post(handle_create_document))
        .route("/api/legal/documents/:slug", get(handle_get_document).put(handle_update_document))
        .route("/api/legal/gdpr/delete/:user_id", post(handle_request_data_deletion))
        .route("/api/legal/gdpr/export/:user_id", post(handle_export_user_data))
}

fn get_default_bot_stub(conn: &mut diesel::PgConnection) -> (Uuid, String) {
    use diesel::dsl::exists;
    use diesel::select;
    let bot_exists: bool = select(exists(bots::table.find(Uuid::nil())))
        .get_result(conn)
        .unwrap_or(false);
    if bot_exists {
        (Uuid::nil(), "Default".to_string())
    } else {
        (Uuid::nil(), "Default".to_string())
    }
}

pub async fn handle_legal_list_page(State(_pool): State<Arc<DbPool>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Legal Documents</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 1400px; margin: 0 auto; padding: 24px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.header h1 { font-size: 28px; color: #1a1a1a; }
.btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-primary:hover { background: #0052a3; }
.tabs { display: flex; gap: 4px; margin-bottom: 24px; border-bottom: 1px solid #e0e0e0; }
.tab { padding: 12px 24px; background: none; border: none; cursor: pointer; font-size: 14px; color: #666; border-bottom: 2px solid transparent; }
.tab.active { color: #0066cc; border-bottom-color: #0066cc; }
.stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }
.stat-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.stat-value { font-size: 28px; font-weight: 600; color: #1a1a1a; }
.stat-label { font-size: 13px; color: #666; margin-top: 4px; }
.document-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 24px; }
.document-card { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; }
.document-card:hover { transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
.document-icon { width: 48px; height: 48px; border-radius: 8px; background: #e3f2fd; display: flex; align-items: center; justify-content: center; font-size: 24px; margin-bottom: 16px; }
.document-title { font-size: 16px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
.document-meta { font-size: 13px; color: #666; margin-bottom: 12px; }
.document-status { display: inline-block; padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 500; }
.status-active { background: #e8f5e9; color: #2e7d32; }
.status-draft { background: #f5f5f5; color: #666; }
.status-expired { background: #ffebee; color: #c62828; }
.status-review { background: #fff3e0; color: #ef6c00; }
.filters { display: flex; gap: 12px; margin-bottom: 24px; }
.search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; }
.filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
.empty-state { text-align: center; padding: 80px 24px; color: #666; }
.empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
</style>
</head>
<body>
<div class="container">
<div class="header">
<h1>Legal Documents</h1>
<button class="btn btn-primary" onclick="createDocument()">+ New Document</button>
</div>
<div class="stats-row">
<div class="stat-card">
<div class="stat-value" id="totalDocs">0</div>
<div class="stat-label">Total Documents</div>
</div>
<div class="stat-card">
<div class="stat-value" id="activeDocs">0</div>
<div class="stat-label">Active</div>
</div>
<div class="stat-card">
<div class="stat-value" id="pendingReview">0</div>
<div class="stat-label">Pending Review</div>
</div>
<div class="stat-card">
<div class="stat-value" id="expiringDocs">0</div>
<div class="stat-label">Expiring Soon</div>
</div>
</div>
<div class="tabs">
<button class="tab active" data-type="all">All Documents</button>
<button class="tab" data-type="policies">Policies</button>
<button class="tab" data-type="contracts">Contracts</button>
<button class="tab" data-type="agreements">Agreements</button>
<button class="tab" data-type="consents">Consent Forms</button>
</div>
<div class="filters">
<input type="text" class="search-box" placeholder="Search legal documents..." id="searchInput" oninput="filterDocuments()">
<select class="filter-select" id="statusFilter" onchange="filterDocuments()">
<option value="">All Status</option>
<option value="active">Active</option>
<option value="draft">Draft</option>
<option value="review">Under Review</option>
<option value="expired">Expired</option>
</select>
<select class="filter-select" id="sortBy" onchange="filterDocuments()">
<option value="updated">Recently Updated</option>
<option value="created">Recently Created</option>
<option value="name">Name A-Z</option>
<option value="expiry">Expiry Date</option>
</select>
</div>
<div class="document-grid" id="documentGrid">
<div class="empty-state">
<h3>No legal documents yet</h3>
<p>Create your first legal document to get started</p>
</div>
</div>
</div>
<script>
let documents = [];
let currentType = 'all';

document.querySelectorAll('.tab').forEach(tab => {
tab.addEventListener('click', () => {
document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
tab.classList.add('active');
currentType = tab.dataset.type;
filterDocuments();
});
});

async function loadDocuments() {
try {
const response = await fetch('/api/legal/documents');
documents = await response.json();
renderDocuments(documents);
updateStats();
} catch (e) {
console.error('Failed to load documents:', e);
}
}

function renderDocuments(docs) {
const grid = document.getElementById('documentGrid');
if (!docs || docs.length === 0) {
grid.innerHTML = '<div class="empty-state"><h3>No documents found</h3><p>Create a new document to get started</p></div>';
return;
}

grid.innerHTML = docs.map(d => `
<div class="document-card" onclick="openDocument('${d.id}')">
<div class="document-icon">${getDocIcon(d.document_type)}</div>
<div class="document-title">${d.title}</div>
<div class="document-meta">
${d.document_type || 'Document'} • Version ${d.version || '1.0'} • Updated ${formatDate(d.updated_at)}
</div>
<span class="document-status status-${d.status || 'draft'}">${(d.status || 'draft').charAt(0).toUpperCase() + (d.status || 'draft').slice(1)}</span>
</div>
`).join('');
}

function getDocIcon(type) {
const icons = {
policy: '📋',
contract: '📝',
agreement: '🤝',
consent: '✅',
terms: '📜',
privacy: '🔒'
};
return icons[type] || '📄';
}

function formatDate(dateStr) {
if (!dateStr) return 'Never';
const date = new Date(dateStr);
const now = new Date();
const diff = now - date;
if (diff < 86400000) return 'Today';
if (diff < 172800000) return 'Yesterday';
return date.toLocaleDateString();
}

function updateStats() {
document.getElementById('totalDocs').textContent = documents.length;
document.getElementById('activeDocs').textContent = documents.filter(d => d.status === 'active').length;
document.getElementById('pendingReview').textContent = documents.filter(d => d.status === 'review').length;
const now = new Date();
const thirtyDays = 30 * 24 * 60 * 60 * 1000;
document.getElementById('expiringDocs').textContent = documents.filter(d => {
if (!d.expiry_date) return false;
const expiry = new Date(d.expiry_date);
return expiry - now < thirtyDays && expiry > now;
}).length;
}

function filterDocuments() {
const query = document.getElementById('searchInput').value.toLowerCase();
const status = document.getElementById('statusFilter').value;

let filtered = documents;

if (currentType !== 'all') {
filtered = filtered.filter(d => d.document_type === currentType.slice(0, -1));
}

if (query) {
filtered = filtered.filter(d =>
d.title.toLowerCase().includes(query) ||
(d.description && d.description.toLowerCase().includes(query))
);
}

if (status) {
filtered = filtered.filter(d => d.status === status);
}

renderDocuments(filtered);
}

function createDocument() {
window.location = '/suite/legal/new';
}

function openDocument(id) {
window.location = `/suite/legal/${id}`;
}

loadDocuments();
</script>
</body>
</html>"#;
Html(html.to_string())
}

pub async fn handle_legal_detail_page(
    State(_pool): State<Arc<DbPool>>,
    Path(doc_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Legal Document</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
.container {{ max-width: 1000px; margin: 0 auto; padding: 24px; }}
.back-link {{ color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }}
.document-header {{ background: white; border-radius: 12px; padding: 32px; margin-bottom: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
.document-title {{ font-size: 28px; font-weight: 600; margin-bottom: 16px; }}
.document-meta {{ display: flex; gap: 24px; color: #666; margin-bottom: 16px; flex-wrap: wrap; }}
.document-status {{ display: inline-block; padding: 6px 16px; border-radius: 20px; font-size: 13px; font-weight: 500; }}
.status-active {{ background: #e8f5e9; color: #2e7d32; }}
.status-draft {{ background: #f5f5f5; color: #666; }}
.document-actions {{ display: flex; gap: 12px; margin-top: 20px; }}
.btn {{ padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }}
.btn-primary {{ background: #0066cc; color: white; }}
.btn-outline {{ background: white; border: 1px solid #ddd; color: #333; }}
.btn-danger {{ background: #ffebee; color: #c62828; }}
.document-content {{ background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
.content-body {{ line-height: 1.8; color: #333; }}
.content-body h2 {{ margin: 24px 0 12px; font-size: 20px; }}
.content-body h3 {{ margin: 20px 0 10px; font-size: 16px; }}
.content-body p {{ margin-bottom: 16px; }}
.content-body ul, .content-body ol {{ margin-bottom: 16px; padding-left: 24px; }}
.content-body li {{ margin-bottom: 8px; }}
.version-history {{ background: white; border-radius: 12px; padding: 24px; margin-top: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
.version-item {{ display: flex; justify-content: space-between; align-items: center; padding: 12px 0; border-bottom: 1px solid #f0f0f0; }}
.version-item:last-child {{ border-bottom: none; }}
.version-info {{ font-size: 14px; }}
.version-date {{ color: #666; font-size: 13px; }}
</style>
</head>
<body>
<div class="container">
<a href="/suite/legal" class="back-link">← Back to Legal Documents</a>
<div class="document-header">
<h1 class="document-title" id="docTitle">Loading...</h1>
<div class="document-meta">
<span id="docType">Document</span>
<span id="docVersion">Version 1.0</span>
<span id="docUpdated">Updated: -</span>
<span id="docExpiry"></span>
</div>
<span class="document-status status-draft" id="docStatus">Draft</span>
<div class="document-actions">
<button class="btn btn-primary" onclick="editDocument()">Edit Document</button>
<button class="btn btn-outline" onclick="downloadPdf()">Download PDF</button>
<button class="btn btn-outline" onclick="viewHistory()">Version History</button>
<button class="btn btn-danger" onclick="deleteDocument()">Delete</button>
</div>
</div>
<div class="document-content">
<div class="content-body" id="docContent">
<p>Loading document content...</p>
</div>
</div>
<div class="version-history" id="versionHistory" style="display: none;">
<h3 style="margin-bottom: 16px;">Version History</h3>
<div id="versionList"></div>
</div>
</div>
<script>
const docId = '{doc_id}';

async function loadDocument() {{
try {{
const response = await fetch(`/api/legal/documents/${{docId}}`);
const doc = await response.json();
if (doc) {{
document.getElementById('docTitle').textContent = doc.title;
document.getElementById('docType').textContent = doc.document_type || 'Document';
document.getElementById('docVersion').textContent = `Version ${{doc.version || '1.0'}}`;
document.getElementById('docUpdated').textContent = `Updated: ${{new Date(doc.updated_at).toLocaleDateString()}}`;

if (doc.expiry_date) {{
document.getElementById('docExpiry').textContent = `Expires: ${{new Date(doc.expiry_date).toLocaleDateString()}}`;
}}

const statusEl = document.getElementById('docStatus');
statusEl.textContent = (doc.status || 'draft').charAt(0).toUpperCase() + (doc.status || 'draft').slice(1);
statusEl.className = `document-status status-${{doc.status || 'draft'}}`;

document.getElementById('docContent').innerHTML = doc.content || '<p>No content available</p>';
}}
}} catch (e) {{
console.error('Failed to load document:', e);
}}
}}

function editDocument() {{
window.location = `/suite/legal/${{docId}}/edit`;
}}

async function downloadPdf() {{
window.open(`/api/legal/documents/${{docId}}/pdf`, '_blank');
}}

function viewHistory() {{
const history = document.getElementById('versionHistory');
history.style.display = history.style.display === 'none' ? 'block' : 'none';
}}

async function deleteDocument() {{
if (!confirm('Are you sure you want to delete this document? This action cannot be undone.')) return;
try {{
await fetch(`/api/legal/documents/${{docId}}`, {{ method: 'DELETE' }});
window.location = '/suite/legal';
}} catch (e) {{
alert('Failed to delete document');
}}
}}

loadDocument();
</script>
</body>
</html>"#);
Html(html)
}

pub async fn handle_legal_new_page(State(_pool): State<Arc<DbPool>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Create Legal Document</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 900px; margin: 0 auto; padding: 24px; }
.back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
.form-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
h1 { font-size: 24px; margin-bottom: 24px; }
.form-group { margin-bottom: 20px; }
.form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
.form-group textarea { min-height: 300px; resize: vertical; font-family: inherit; }
.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-primary:hover { background: #0052a3; }
.btn-secondary { background: #f5f5f5; color: #333; }
.form-actions { display: flex; gap: 12px; justify-content: flex-end; }
.template-buttons { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 16px; }
.template-btn { padding: 8px 16px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; font-size: 13px; }
.template-btn:hover { background: #f5f5f5; border-color: #0066cc; }
</style>
</head>
<body>
<div class="container">
<a href="/suite/legal" class="back-link">← Back to Legal Documents</a>
<div class="form-card">
<h1>Create Legal Document</h1>
<form id="documentForm">
<div class="form-row">
<div class="form-group">
<label>Document Type</label>
<select id="documentType" required>
<option value="">Select type...</option>
<option value="policy">Policy</option>
<option value="contract">Contract</option>
<option value="agreement">Agreement</option>
<option value="consent">Consent Form</option>
<option value="terms">Terms of Service</option>
<option value="privacy">Privacy Policy</option>
</select>
</div>
<div class="form-group">
<label>Status</label>
<select id="status">
<option value="draft">Draft</option>
<option value="review">Under Review</option>
<option value="active">Active</option>
</select>
</div>
</div>
<div class="form-group">
<label>Document Title</label>
<input type="text" id="title" required placeholder="Enter document title">
</div>
<div class="form-group">
<label>Description</label>
<input type="text" id="description" placeholder="Brief description of this document">
</div>
<div class="form-row">
<div class="form-group">
<label>Effective Date</label>
<input type="date" id="effectiveDate">
</div>
<div class="form-group">
<label>Expiry Date (optional)</label>
<input type="date" id="expiryDate">
</div>
</div>
<div class="form-group">
<label>Document Content</label>
<div class="template-buttons">
<button type="button" class="template-btn" onclick="insertTemplate('privacy')">Privacy Policy Template</button>
<button type="button" class="template-btn" onclick="insertTemplate('terms')">Terms of Service Template</button>
<button type="button" class="template-btn" onclick="insertTemplate('cookie')">Cookie Policy Template</button>
</div>
<textarea id="content" placeholder="Enter the document content here..."></textarea>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="saveDraft()">Save as Draft</button>
<button type="button" class="btn btn-secondary" onclick="window.location='/suite/legal'">Cancel</button>
<button type="submit" class="btn btn-primary">Create Document</button>
</div>
</form>
</div>
</div>
<script>
const templates = {
privacy: `<h2>Privacy Policy</h2>
<p>Last updated: [DATE]</p>

<h3>1. Information We Collect</h3>
<p>We collect information you provide directly to us, including...</p>

<h3>2. How We Use Your Information</h3>
<p>We use the information we collect to...</p>

<h3>3. Information Sharing</h3>
<p>We do not share your personal information except...</p>

<h3>4. Data Security</h3>
<p>We implement appropriate security measures to protect...</p>

<h3>5. Your Rights</h3>
<p>You have the right to access, correct, or delete your personal data...</p>

<h3>6. Contact Us</h3>
<p>If you have questions about this Privacy Policy, please contact us at...</p>`,
terms: `<h2>Terms of Service</h2>
<p>Last updated: [DATE]</p>

<h3>1. Acceptance of Terms</h3>
<p>By accessing or using our services, you agree to be bound by these Terms...</p>

<h3>2. Use of Services</h3>
<p>You may use our services only in compliance with these Terms...</p>

<h3>3. User Accounts</h3>
<p>You are responsible for maintaining the confidentiality of your account...</p>

<h3>4. Intellectual Property</h3>
<p>All content and materials available through our services are protected...</p>

<h3>5. Limitation of Liability</h3>
<p>To the fullest extent permitted by law, we shall not be liable...</p>

<h3>6. Governing Law</h3>
<p>These Terms shall be governed by and construed in accordance with...</p>`,
cookie: `<h2>Cookie Policy</h2>
<p>Last updated: [DATE]</p>

<h3>1. What Are Cookies</h3>
<p>Cookies are small text files stored on your device when you visit our website...</p>

<h3>2. Types of Cookies We Use</h3>
<ul>
<li><strong>Essential Cookies:</strong> Required for basic site functionality</li>
<li><strong>Analytics Cookies:</strong> Help us understand how visitors use our site</li>
<li><strong>Marketing Cookies:</strong> Used to deliver relevant advertisements</li>
</ul>

<h3>3. Managing Cookies</h3>
<p>You can control cookies through your browser settings...</p>

<h3>4. Contact Us</h3>
<p>For questions about our Cookie Policy, contact us at...</p>`
};

function insertTemplate(type) {
const content = document.getElementById('content');
const today = new Date().toLocaleDateString();
content.value = templates[type].replace('[DATE]', today);
}

function saveDraft() {
document.getElementById('status').value = 'draft';
document.getElementById('documentForm').dispatchEvent(new Event('submit'));
}

document.getElementById('documentForm').addEventListener('submit', async (e) => {
e.preventDefault();

const data = {
document_type: document.getElementById('documentType').value,
title: document.getElementById('title').value,
description: document.getElementById('description').value || null,
content: document.getElementById('content').value,
status: document.getElementById('status').value,
effective_date: document.getElementById('effectiveDate').value || null,
expiry_date: document.getElementById('expiryDate').value || null
};

try {
const response = await fetch('/api/legal/documents', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify(data)
});

if (response.ok) {
const doc = await response.json();
window.location = `/suite/legal/${doc.id}`;
} else {
alert('Failed to create document');
}
} catch (e) {
alert('Error: ' + e.message);
}
});
</script>
</body>
</html>"#;
Html(html.to_string())
}

pub fn configure_legal_ui_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/suite/legal", get(handle_legal_list_page))
        .route("/suite/legal/new", get(handle_legal_new_page))
        .route("/suite/legal/:id", get(handle_legal_detail_page))
}
