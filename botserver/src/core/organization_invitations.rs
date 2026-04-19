use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvitationRole {
    Owner,
    Admin,
    Manager,
    Member,
    Viewer,
    Guest,
}

impl std::str::FromStr for InvitationRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "manager" => Ok(Self::Manager),
            "member" => Ok(Self::Member),
            "viewer" => Ok(Self::Viewer),
            "guest" => Ok(Self::Guest),
            _ => Err(()),
        }
    }
}

impl InvitationRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Manager => "manager",
            Self::Member => "member",
            Self::Viewer => "viewer",
            Self::Guest => "guest",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInvitation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub role: InvitationRole,
    pub groups: Vec<String>,
    pub invited_by: Uuid,
    pub invited_by_name: String,
    pub status: InvitationStatus,
    pub token: String,
    pub message: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub accepted_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    pub email: String,
    pub role: String,
    #[serde(default)]
    pub groups: Vec<String>,
    pub message: Option<String>,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BulkInviteRequest {
    pub emails: Vec<String>,
    pub role: String,
    #[serde(default)]
    pub groups: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInvitationRequest {
    pub token: String,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ResendInvitationRequest {
    pub extend_expiry: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListInvitationsQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub email: String,
    pub role: String,
    pub groups: Vec<String>,
    pub invited_by_name: String,
    pub status: String,
    pub message: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub is_expired: bool,
}

#[derive(Debug, Serialize)]
pub struct BulkInviteResponse {
    pub successful: Vec<InvitationResponse>,
    pub failed: Vec<BulkInviteError>,
}

#[derive(Debug, Serialize)]
pub struct BulkInviteError {
    pub email: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct InvitationListResponse {
    pub invitations: Vec<InvitationResponse>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct AcceptInvitationResponse {
    pub success: bool,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub role: String,
    pub message: String,
}

pub struct InvitationService {
    invitations: Arc<RwLock<HashMap<Uuid, OrganizationInvitation>>>,
    invitations_by_token: Arc<RwLock<HashMap<String, Uuid>>>,
    invitations_by_org: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl Default for InvitationService {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CreateInvitationParams<'a> {
    pub organization_id: Uuid,
    pub organization_name: &'a str,
    pub email: &'a str,
    pub role: InvitationRole,
    pub groups: Vec<String>,
    pub invited_by: Uuid,
    pub invited_by_name: &'a str,
    pub message: Option<String>,
    pub expires_in_days: i64,
}

impl<'a> Default for CreateInvitationParams<'a> {
    fn default() -> Self {
        Self {
            organization_id: Uuid::default(),
            organization_name: "",
            email: "",
            role: InvitationRole::Member,
            groups: Vec::new(),
            invited_by: Uuid::default(),
            invited_by_name: "",
            message: None,
            expires_in_days: 7,
        }
    }
}

pub struct BulkInviteParams<'a> {
    pub organization_id: Uuid,
    pub organization_name: &'a str,
    pub emails: Vec<String>,
    pub role: InvitationRole,
    pub groups: Vec<String>,
    pub invited_by: Uuid,
    pub invited_by_name: &'a str,
    pub message: Option<String>,
}

impl InvitationService {
    pub fn new() -> Self {
        Self {
            invitations: Arc::new(RwLock::new(HashMap::new())),
            invitations_by_token: Arc::new(RwLock::new(HashMap::new())),
            invitations_by_org: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_invitation(
        &self,
        params: CreateInvitationParams<'_>,
    ) -> Result<OrganizationInvitation, String> {
        let email_lower = params.email.to_lowercase().trim().to_string();

        if !self.is_valid_email(&email_lower) {
            return Err("Invalid email address".to_string());
        }

        let existing = self
            .find_pending_invitation(&params.organization_id, &email_lower)
            .await;
        if existing.is_some() {
            return Err("An invitation already exists for this email".to_string());
        }

        let now = Utc::now();
        let invitation_id = Uuid::new_v4();
        let token = self.generate_secure_token();

        let invitation = OrganizationInvitation {
            id: invitation_id,
            organization_id: params.organization_id,
            email: email_lower,
            role: params.role,
            groups: params.groups,
            invited_by: params.invited_by,
            invited_by_name: params.invited_by_name.to_string(),
            status: InvitationStatus::Pending,
            token: token.clone(),
            message: params.message,
            expires_at: now + Duration::days(params.expires_in_days),
            created_at: now,
            updated_at: now,
            accepted_at: None,
            accepted_by: None,
        };

        {
            let mut invitations = self.invitations.write().await;
            invitations.insert(invitation_id, invitation.clone());
        }

        {
            let mut by_token = self.invitations_by_token.write().await;
            by_token.insert(token, invitation_id);
        }

        {
            let mut by_org = self.invitations_by_org.write().await;
            by_org
                .entry(params.organization_id)
                .or_default()
                .push(invitation_id);
        }

        self.send_invitation_email(&invitation, params.organization_name)
            .await;

        Ok(invitation)
    }

    pub async fn bulk_invite(&self, params: BulkInviteParams<'_>) -> BulkInviteResponse {
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for email in params.emails {
            match self
                .create_invitation(CreateInvitationParams {
                    organization_id: params.organization_id,
                    organization_name: params.organization_name,
                    email: &email,
                    role: params.role.clone(),
                    groups: params.groups.clone(),
                    invited_by: params.invited_by,
                    invited_by_name: params.invited_by_name,
                    message: params.message.clone(),
                    expires_in_days: 7,
                })
                .await
            {
                Ok(invitation) => {
                    successful.push(self.to_response(&invitation, params.organization_name));
                }
                Err(error) => {
                    failed.push(BulkInviteError { email, error });
                }
            }
        }

        BulkInviteResponse { successful, failed }
    }

    pub async fn accept_invitation(
        &self,
        token: &str,
        user_id: Uuid,
    ) -> Result<AcceptInvitationResponse, String> {
        let invitation_id = {
            let by_token = self.invitations_by_token.read().await;
            by_token.get(token).copied()
        };

        let invitation_id = invitation_id.ok_or("Invalid invitation token")?;

        let mut invitations = self.invitations.write().await;
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or("Invitation not found")?;

        if invitation.status != InvitationStatus::Pending {
            return Err(format!(
                "Invitation is not pending (status: {:?})",
                invitation.status
            ));
        }

        if invitation.expires_at < Utc::now() {
            invitation.status = InvitationStatus::Expired;
            invitation.updated_at = Utc::now();
            return Err("Invitation has expired".to_string());
        }

        let now = Utc::now();
        invitation.status = InvitationStatus::Accepted;
        invitation.accepted_at = Some(now);
        invitation.accepted_by = Some(user_id);
        invitation.updated_at = now;

        Ok(AcceptInvitationResponse {
            success: true,
            organization_id: invitation.organization_id,
            organization_name: "Organization".to_string(),
            role: invitation.role.as_str().to_string(),
            message: "Successfully joined the organization".to_string(),
        })
    }

    pub async fn decline_invitation(&self, token: &str) -> Result<(), String> {
        let invitation_id = {
            let by_token = self.invitations_by_token.read().await;
            by_token.get(token).copied()
        };

        let invitation_id = invitation_id.ok_or("Invalid invitation token")?;

        let mut invitations = self.invitations.write().await;
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or("Invitation not found")?;

        if invitation.status != InvitationStatus::Pending {
            return Err("Invitation is not pending".to_string());
        }

        invitation.status = InvitationStatus::Declined;
        invitation.updated_at = Utc::now();

        Ok(())
    }

    pub async fn revoke_invitation(&self, invitation_id: Uuid) -> Result<(), String> {
        let mut invitations = self.invitations.write().await;
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or("Invitation not found")?;

        if invitation.status != InvitationStatus::Pending {
            return Err("Only pending invitations can be revoked".to_string());
        }

        invitation.status = InvitationStatus::Revoked;
        invitation.updated_at = Utc::now();

        Ok(())
    }

    pub async fn resend_invitation(
        &self,
        invitation_id: Uuid,
        organization_name: &str,
        extend_expiry: bool,
    ) -> Result<OrganizationInvitation, String> {
        let mut invitations = self.invitations.write().await;
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or("Invitation not found")?;

        if invitation.status != InvitationStatus::Pending
            && invitation.status != InvitationStatus::Expired
        {
            return Err("Only pending or expired invitations can be resent".to_string());
        }

        let now = Utc::now();

        if extend_expiry || invitation.expires_at < now {
            invitation.expires_at = now + Duration::days(7);
        }

        invitation.status = InvitationStatus::Pending;
        invitation.updated_at = now;

        let invitation_clone = invitation.clone();
        drop(invitations);

        self.send_invitation_email(&invitation_clone, organization_name)
            .await;

        Ok(invitation_clone)
    }

    pub async fn list_invitations(
        &self,
        organization_id: Uuid,
        status_filter: Option<InvitationStatus>,
        page: u32,
        per_page: u32,
    ) -> InvitationListResponse {
        let org_invitation_ids = {
            let by_org = self.invitations_by_org.read().await;
            by_org.get(&organization_id).cloned().unwrap_or_default()
        };

        let invitations = self.invitations.read().await;

        let mut filtered: Vec<_> = org_invitation_ids
            .iter()
            .filter_map(|id| invitations.get(id))
            .filter(|inv| {
                if let Some(ref status) = status_filter {
                    &inv.status == status
                } else {
                    true
                }
            })
            .collect();

        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = filtered.len() as u32;
        let total_pages = total.div_ceil(per_page);
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(filtered.len());

        let page_items: Vec<InvitationResponse> = filtered[start..end]
            .iter()
            .map(|inv| self.to_response(inv, "Organization"))
            .collect();

        InvitationListResponse {
            invitations: page_items,
            total,
            page,
            per_page,
            total_pages,
        }
    }

    pub async fn get_invitation(&self, invitation_id: Uuid) -> Option<OrganizationInvitation> {
        let invitations = self.invitations.read().await;
        invitations.get(&invitation_id).cloned()
    }

    pub async fn get_invitation_by_token(&self, token: &str) -> Option<OrganizationInvitation> {
        let invitation_id = {
            let by_token = self.invitations_by_token.read().await;
            by_token.get(token).copied()
        };

        if let Some(id) = invitation_id {
            let invitations = self.invitations.read().await;
            invitations.get(&id).cloned()
        } else {
            None
        }
    }

    pub async fn cleanup_expired_invitations(&self) {
        let now = Utc::now();
        let mut invitations = self.invitations.write().await;

        for invitation in invitations.values_mut() {
            if invitation.status == InvitationStatus::Pending && invitation.expires_at < now {
                invitation.status = InvitationStatus::Expired;
                invitation.updated_at = now;
            }
        }
    }

    async fn find_pending_invitation(
        &self,
        organization_id: &Uuid,
        email: &str,
    ) -> Option<OrganizationInvitation> {
        let org_invitation_ids = {
            let by_org = self.invitations_by_org.read().await;
            by_org.get(organization_id).cloned().unwrap_or_default()
        };

        let invitations = self.invitations.read().await;

        for id in org_invitation_ids {
            if let Some(inv) = invitations.get(&id) {
                if inv.email == email && inv.status == InvitationStatus::Pending {
                    return Some(inv.clone());
                }
            }
        }

        None
    }

    fn to_response(
        &self,
        invitation: &OrganizationInvitation,
        org_name: &str,
    ) -> InvitationResponse {
        let now = Utc::now();
        InvitationResponse {
            id: invitation.id,
            organization_id: invitation.organization_id,
            organization_name: org_name.to_string(),
            email: invitation.email.clone(),
            role: invitation.role.as_str().to_string(),
            groups: invitation.groups.clone(),
            invited_by_name: invitation.invited_by_name.clone(),
            status: format!("{:?}", invitation.status).to_lowercase(),
            message: invitation.message.clone(),
            expires_at: invitation.expires_at,
            created_at: invitation.created_at,
            is_expired: invitation.expires_at < now,
        }
    }

    fn generate_secure_token(&self) -> String {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        let mut token = String::with_capacity(64);
        let hasher_builder = RandomState::new();

        for _ in 0..4 {
            let mut hasher = hasher_builder.build_hasher();
            hasher.write_u128(Uuid::new_v4().as_u128());
            hasher.write_i64(Utc::now().timestamp_nanos_opt().unwrap_or(0));
            token.push_str(&format!("{:016x}", hasher.finish()));
        }

        token
    }

    fn is_valid_email(&self, email: &str) -> bool {
        let email = email.trim();

        if email.is_empty() || email.len() > 254 {
            return false;
        }

        let at_pos = match email.find('@') {
            Some(pos) => pos,
            None => return false,
        };

        let local = &email[..at_pos];
        let domain = &email[at_pos + 1..];

        if local.is_empty() || local.len() > 64 {
            return false;
        }

        if domain.is_empty() || !domain.contains('.') {
            return false;
        }

        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.iter().any(|p| p.is_empty()) {
            return false;
        }

        true
    }

    async fn send_invitation_email(&self, invitation: &OrganizationInvitation, org_name: &str) {
        log::info!(
            "Sending invitation email to {} for organization {} (token: {}...)",
            invitation.email,
            org_name,
            &invitation.token[..16]
        );
    }
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/organizations/:org_id/invitations", get(list_invitations))
        .route(
            "/organizations/:org_id/invitations",
            post(create_invitation),
        )
        .route("/organizations/:org_id/invitations/bulk", post(bulk_invite))
        .route(
            "/organizations/:org_id/invitations/:invitation_id",
            get(get_invitation),
        )
        .route(
            "/organizations/:org_id/invitations/:invitation_id",
            delete(revoke_invitation),
        )
        .route(
            "/organizations/:org_id/invitations/:invitation_id/resend",
            post(resend_invitation),
        )
        .route("/invitations/accept", post(accept_invitation))
        .route("/invitations/decline", post(decline_invitation))
        .route("/invitations/validate/:token", get(validate_invitation))
}

async fn list_invitations(
    State(_state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Query(params): Query<ListInvitationsQuery>,
) -> Result<Json<InvitationListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    let status_filter = params.status.and_then(|s| match s.to_lowercase().as_str() {
        "pending" => Some(InvitationStatus::Pending),
        "accepted" => Some(InvitationStatus::Accepted),
        "declined" => Some(InvitationStatus::Declined),
        "expired" => Some(InvitationStatus::Expired),
        "revoked" => Some(InvitationStatus::Revoked),
        _ => None,
    });

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let response = service
        .list_invitations(org_id, status_filter, page, per_page)
        .await;

    Ok(Json(response))
}

async fn create_invitation(
    State(_state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    let role: InvitationRole = req.role.parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid role"})),
        )
    })?;

    let expires_in_days = req.expires_in_days.unwrap_or(7).clamp(1, 30);

    let invited_by = Uuid::new_v4();

    match service
        .create_invitation(CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Organization",
            email: &req.email,
            role,
            groups: req.groups,
            invited_by,
            invited_by_name: "Admin User",
            message: req.message,
            expires_in_days,
        })
        .await
    {
        Ok(invitation) => Ok(Json(service.to_response(&invitation, "Organization"))),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )),
    }
}

async fn bulk_invite(
    State(_state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<BulkInviteRequest>,
) -> Result<Json<BulkInviteResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    let role = req.role.parse::<InvitationRole>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid role"})),
        )
    })?;

    if req.emails.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No emails provided"})),
        ));
    }

    if req.emails.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Maximum 100 invitations per request"})),
        ));
    }

    let invited_by = Uuid::new_v4();

    let response = service
        .bulk_invite(BulkInviteParams {
            organization_id: org_id,
            organization_name: "Organization",
            emails: req.emails,
            role,
            groups: req.groups,
            invited_by,
            invited_by_name: "Admin User",
            message: req.message,
        })
        .await;

    Ok(Json(response))
}

async fn get_invitation(
    State(_state): State<Arc<AppState>>,
    Path((org_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    match service.get_invitation(invitation_id).await {
        Some(invitation) if invitation.organization_id == org_id => {
            Ok(Json(service.to_response(&invitation, "Organization")))
        }
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Invitation belongs to different organization"})),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Invitation not found"})),
        )),
    }
}

async fn revoke_invitation(
    State(_state): State<Arc<AppState>>,
    Path((_org_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    match service.revoke_invitation(invitation_id).await {
        Ok(()) => Ok(Json(
            serde_json::json!({"success": true, "message": "Invitation revoked"}),
        )),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )),
    }
}

async fn resend_invitation(
    State(_state): State<Arc<AppState>>,
    Path((_org_id, invitation_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ResendInvitationRequest>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    let extend_expiry = req.extend_expiry.unwrap_or(true);

    match service
        .resend_invitation(invitation_id, "Organization", extend_expiry)
        .await
    {
        Ok(invitation) => Ok(Json(service.to_response(&invitation, "Organization"))),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )),
    }
}

async fn accept_invitation(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AcceptInvitationRequest>,
) -> Result<Json<AcceptInvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    let user_id = req.user_id.unwrap_or_else(Uuid::new_v4);

    match service.accept_invitation(&req.token, user_id).await {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )),
    }
}

async fn decline_invitation(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AcceptInvitationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    match service.decline_invitation(&req.token).await {
        Ok(()) => Ok(Json(
            serde_json::json!({"success": true, "message": "Invitation declined"}),
        )),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )),
    }
}

async fn validate_invitation(
    State(_state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new();

    match service.get_invitation_by_token(&token).await {
        Some(invitation) => {
            if invitation.status != InvitationStatus::Pending {
                return Err((
                    StatusCode::GONE,
                    Json(serde_json::json!({
                        "error": "Invitation is no longer valid",
                        "status": format!("{:?}", invitation.status).to_lowercase()
                    })),
                ));
            }

            if invitation.expires_at < Utc::now() {
                return Err((
                    StatusCode::GONE,
                    Json(serde_json::json!({
                        "error": "Invitation has expired",
                        "expired_at": invitation.expires_at
                    })),
                ));
            }

            Ok(Json(service.to_response(&invitation, "Organization")))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Invalid invitation token"})),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_invitation() {
        let service = InvitationService::new();
        let org_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();

        let params = crate::core::organization_invitations::CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Test Org",
            email: "test@example.com".to_string(),
            role: "Member".to_string(),
            groups: vec![],
            ..Default::default()
        };

        let result = service.create_invitation(params).await;

        assert!(result.is_ok());
        let invitation = result.unwrap();
        assert_eq!(invitation.email, "test@example.com");
        assert_eq!(invitation.status, InvitationStatus::Pending);
    }

    #[tokio::test]
    async fn test_duplicate_invitation() {
        let service = InvitationService::new();
        let org_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();

        let params = crate::core::organization_invitations::CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Test Org",
            email: "test@example.com".to_string(),
            role: "Member".to_string(),
            groups: vec![],
            ..Default::default()
        };

        let first_result = service.create_invitation(params.clone()).await;

        assert!(first_result.is_ok());

        let second_result = service.create_invitation(params).await;

        assert!(second_result.is_err());
        assert_eq!(
            second_result.unwrap_err(),
            "An invitation already exists for this email"
        );
    }

    #[tokio::test]
    async fn test_accept_invitation() {
        let service = InvitationService::new();
        let org_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let params = crate::core::organization_invitations::CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Test Org",
            email: "test@example.com".to_string(),
            role: "Member".to_string(),
            groups: vec![],
            invited_by,
            invited_by_name: Some("Admin".to_string()),
            message: None,
            expires_in_days: Some(7),
        };
        let invitation = service.create_invitation(params).await.unwrap();

        let result = service.accept_invitation(&invitation.token, user_id).await;
        assert!(result.is_ok());

        result.unwrap();
        let updated = service.get_invitation(invitation.id).await.unwrap();
        assert_eq!(updated.status, InvitationStatus::Accepted);
        assert!(updated.accepted_at.is_some());
    }
}
