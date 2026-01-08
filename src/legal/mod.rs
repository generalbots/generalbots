pub mod account_deletion;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieConsentRequest {
    pub session_id: Option<String>,
    pub consents: HashMap<CookieCategory, bool>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

pub struct LegalService {
    consents: Arc<RwLock<HashMap<Uuid, CookieConsent>>>,
    documents: Arc<RwLock<HashMap<String, LegalDocument>>>,
    current_consent_version: String,
}

impl LegalService {
    pub fn new() -> Self {
        let service = Self {
            consents: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            current_consent_version: "1.0.0".to_string(),
        };

        tokio::spawn({
            let documents = service.documents.clone();
            async move {
                let mut docs = documents.write().await;
                for doc in Self::get_default_documents() {
                    docs.insert(doc.slug.clone(), doc);
                }
            }
        });

        service
    }

    fn get_default_documents() -> Vec<LegalDocument> {
        let now = Utc::now();
        vec![
            LegalDocument {
                id: "privacy".to_string(),
                title: "Privacy Policy".to_string(),
                slug: "privacy".to_string(),
                content: include_str!("templates/privacy.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::PrivacyPolicy,
            },
            LegalDocument {
                id: "terms".to_string(),
                title: "Terms of Service".to_string(),
                slug: "terms".to_string(),
                content: include_str!("templates/terms.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::TermsOfService,
            },
            LegalDocument {
                id: "cookies".to_string(),
                title: "Cookie Policy".to_string(),
                slug: "cookies".to_string(),
                content: include_str!("templates/cookies.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::CookiePolicy,
            },
            LegalDocument {
                id: "aup".to_string(),
                title: "Acceptable Use Policy".to_string(),
                slug: "aup".to_string(),
                content: include_str!("templates/aup.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::AcceptableUsePolicy,
            },
            LegalDocument {
                id: "dpa".to_string(),
                title: "Data Processing Agreement".to_string(),
                slug: "dpa".to_string(),
                content: include_str!("templates/dpa.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::DataProcessingAgreement,
            },
            LegalDocument {
                id: "gdpr".to_string(),
                title: "GDPR Compliance".to_string(),
                slug: "gdpr".to_string(),
                content: include_str!("templates/gdpr.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::GdprCompliance,
            },
            LegalDocument {
                id: "ccpa".to_string(),
                title: "CCPA Compliance".to_string(),
                slug: "ccpa".to_string(),
                content: include_str!("templates/ccpa.md").to_string(),
                version: "1.0.0".to_string(),
                effective_date: now,
                last_updated: now,
                document_type: LegalDocumentType::CcpaCompliance,
            },
        ]
    }

    pub async fn record_consent(
        &self,
        user_id: Option<Uuid>,
        session_id: Option<String>,
        ip_address: Option<String>,
        consents: HashMap<CookieCategory, bool>,
        user_agent: Option<String>,
    ) -> CookieConsent {
        let mut final_consents = consents;
        final_consents.insert(CookieCategory::Necessary, true);

        let now = Utc::now();
        let consent = CookieConsent {
            id: Uuid::new_v4(),
            user_id,
            session_id,
            ip_address,
            consents: final_consents,
            consent_given_at: now,
            consent_updated_at: now,
            consent_version: self.current_consent_version.clone(),
            user_agent,
            country_code: None,
        };

        let mut consents_store = self.consents.write().await;
        consents_store.insert(consent.id, consent.clone());

        consent
    }

    pub async fn update_consent(
        &self,
        consent_id: Uuid,
        consents: HashMap<CookieCategory, bool>,
    ) -> Option<CookieConsent> {
        let mut consents_store = self.consents.write().await;
        if let Some(consent) = consents_store.get_mut(&consent_id) {
            let mut final_consents = consents;
            final_consents.insert(CookieCategory::Necessary, true);

            consent.consents = final_consents;
            consent.consent_updated_at = Utc::now();
            consent.consent_version = self.current_consent_version.clone();
            return Some(consent.clone());
        }
        None
    }

    pub async fn get_consent(&self, consent_id: Uuid) -> Option<CookieConsent> {
        let consents = self.consents.read().await;
        consents.get(&consent_id).cloned()
    }

    pub async fn get_consent_by_session(&self, session_id: &str) -> Option<CookieConsent> {
        let consents = self.consents.read().await;
        consents
            .values()
            .find(|c| c.session_id.as_deref() == Some(session_id))
            .cloned()
    }

    pub async fn get_consent_by_user(&self, user_id: Uuid) -> Option<CookieConsent> {
        let consents = self.consents.read().await;
        consents.values().find(|c| c.user_id == Some(user_id)).cloned()
    }

    pub async fn get_document(&self, slug: &str) -> Option<LegalDocument> {
        let documents = self.documents.read().await;
        documents.get(slug).cloned()
    }

    pub async fn get_all_documents(&self) -> Vec<LegalDocument> {
        let documents = self.documents.read().await;
        documents.values().cloned().collect()
    }

    pub async fn update_document(&self, document: LegalDocument) {
        let mut documents = self.documents.write().await;
        documents.insert(document.slug.clone(), document);
    }

    pub fn get_cookie_policy(&self) -> CookiePolicy {
        let now = Utc::now();
        CookiePolicy {
            version: self.current_consent_version.clone(),
            effective_date: now,
            categories: vec![
                CookieCategoryInfo {
                    category: CookieCategory::Necessary,
                    name: "Necessary Cookies".to_string(),
                    description: "These cookies are essential for the website to function properly."
                        .to_string(),
                    is_required: true,
                    cookies: vec![
                        CookieInfo {
                            name: "session_id".to_string(),
                            provider: "General Bots".to_string(),
                            purpose: "User session management".to_string(),
                            expiry: "Session".to_string(),
                            cookie_type: CookieType::Session,
                        },
                        CookieInfo {
                            name: "csrf_token".to_string(),
                            provider: "General Bots".to_string(),
                            purpose: "Security protection against CSRF attacks".to_string(),
                            expiry: "Session".to_string(),
                            cookie_type: CookieType::Session,
                        },
                    ],
                },
                CookieCategoryInfo {
                    category: CookieCategory::Analytics,
                    name: "Analytics Cookies".to_string(),
                    description:
                        "These cookies help us understand how visitors interact with our website."
                            .to_string(),
                    is_required: false,
                    cookies: vec![CookieInfo {
                        name: "_ga".to_string(),
                        provider: "Google Analytics".to_string(),
                        purpose: "Distinguishes unique users".to_string(),
                        expiry: "2 years".to_string(),
                        cookie_type: CookieType::ThirdParty,
                    }],
                },
                CookieCategoryInfo {
                    category: CookieCategory::Marketing,
                    name: "Marketing Cookies".to_string(),
                    description: "These cookies are used to deliver relevant advertisements."
                        .to_string(),
                    is_required: false,
                    cookies: vec![],
                },
                CookieCategoryInfo {
                    category: CookieCategory::Preferences,
                    name: "Preference Cookies".to_string(),
                    description: "These cookies remember your preferences and settings.".to_string(),
                    is_required: false,
                    cookies: vec![
                        CookieInfo {
                            name: "theme".to_string(),
                            provider: "General Bots".to_string(),
                            purpose: "Remembers user theme preference".to_string(),
                            expiry: "1 year".to_string(),
                            cookie_type: CookieType::Persistent,
                        },
                        CookieInfo {
                            name: "language".to_string(),
                            provider: "General Bots".to_string(),
                            purpose: "Remembers user language preference".to_string(),
                            expiry: "1 year".to_string(),
                            cookie_type: CookieType::Persistent,
                        },
                    ],
                },
                CookieCategoryInfo {
                    category: CookieCategory::Functional,
                    name: "Functional Cookies".to_string(),
                    description: "These cookies enable enhanced functionality and personalization."
                        .to_string(),
                    is_required: false,
                    cookies: vec![],
                },
            ],
            last_updated: now,
        }
    }

    pub async fn delete_user_data(&self, user_id: Uuid) -> DataDeletionResult {
        let mut consents = self.consents.write().await;
        let initial_count = consents.len();
        consents.retain(|_, c| c.user_id != Some(user_id));
        let deleted_count = initial_count - consents.len();

        DataDeletionResult {
            user_id,
            consents_deleted: deleted_count,
            deleted_at: Utc::now(),
            confirmation_token: Uuid::new_v4().to_string(),
        }
    }

    pub async fn export_user_data(&self, user_id: Uuid) -> UserDataExport {
        let consents = self.consents.read().await;
        let user_consents: Vec<CookieConsent> = consents
            .values()
            .filter(|c| c.user_id == Some(user_id))
            .cloned()
            .collect();

        UserDataExport {
            user_id,
            exported_at: Utc::now(),
            consents: user_consents,
            format: "json".to_string(),
        }
    }

    pub fn consent_version(&self) -> &str {
        &self.current_consent_version
    }
}

impl Default for LegalService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionResult {
    pub user_id: Uuid,
    pub consents_deleted: usize,
    pub deleted_at: DateTime<Utc>,
    pub confirmation_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub user_id: Uuid,
    pub exported_at: DateTime<Utc>,
    pub consents: Vec<CookieConsent>,
    pub format: String,
}

async fn record_consent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CookieConsentRequest>,
) -> Result<Json<CookieConsentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    let consent = service
        .record_consent(None, req.session_id, None, req.consents, req.user_agent)
        .await;

    Ok(Json(CookieConsentResponse {
        id: consent.id,
        consents: consent.consents,
        consent_given_at: consent.consent_given_at,
        consent_version: consent.consent_version,
    }))
}

async fn get_consent(
    State(state): State<Arc<AppState>>,
    Path(consent_id): Path<Uuid>,
) -> Result<Json<CookieConsentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    match service.get_consent(consent_id).await {
        Some(consent) => Ok(Json(CookieConsentResponse {
            id: consent.id,
            consents: consent.consents,
            consent_given_at: consent.consent_given_at,
            consent_version: consent.consent_version,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Consent not found"})),
        )),
    }
}

async fn get_cookie_policy(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CookiePolicy>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    Ok(Json(service.get_cookie_policy()))
}

async fn get_legal_document(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<LegalDocument>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    match service.get_document(&slug).await {
        Some(doc) => Ok(Json(doc)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Document not found"})),
        )),
    }
}

async fn list_legal_documents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LegalDocument>>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    let docs = service.get_all_documents().await;
    Ok(Json(docs))
}

async fn request_data_deletion(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<DataDeletionResult>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    let result = service.delete_user_data(user_id).await;
    Ok(Json(result))
}

async fn export_user_data(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDataExport>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.legal_service.read().await;
    let export = service.export_user_data(user_id).await;
    Ok(Json(export))
}

pub fn configure(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/legal/consent", post(record_consent))
        .route("/legal/consent/:consent_id", get(get_consent))
        .route("/legal/cookies/policy", get(get_cookie_policy))
        .route("/legal/documents", get(list_legal_documents))
        .route("/legal/documents/:slug", get(get_legal_document))
        .route("/legal/gdpr/delete/:user_id", post(request_data_deletion))
        .route("/legal/gdpr/export/:user_id", get(export_user_data))
        .with_state(state)
}
