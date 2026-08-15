use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};


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
    pool: DbPool,
}

/// Row shape of the persisted `organization_invitations` table (raw SQL —
/// botcore does not own the diesel schema for this table).
#[derive(diesel::QueryableByName, Debug, Clone)]
struct InvitationRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    role: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    message: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    invited_by: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    invited_by_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    token: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    groups: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    updated_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    expires_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    accepted_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    accepted_by: Option<Uuid>,
}

impl InvitationRow {
    fn to_invitation(&self) -> OrganizationInvitation {
        OrganizationInvitation {
            id: self.id,
            organization_id: self.org_id,
            email: self.email.clone(),
            role: self.role.parse().unwrap_or(InvitationRole::Member),
            groups: self
                .groups
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            invited_by: self.invited_by,
            invited_by_name: self.invited_by_name.clone().unwrap_or_default(),
            status: match self.status.as_str() {
                "accepted" => InvitationStatus::Accepted,
                "declined" => InvitationStatus::Declined,
                "expired" => InvitationStatus::Expired,
                "revoked" => InvitationStatus::Revoked,
                _ => InvitationStatus::Pending,
            },
            token: self.token.clone().unwrap_or_default(),
            message: self.message.clone(),
            expires_at: self.expires_at.unwrap_or_else(Utc::now),
            created_at: self.created_at,
            updated_at: self.updated_at.unwrap_or(self.created_at),
            accepted_at: self.accepted_at,
            accepted_by: self.accepted_by,
        }
    }
}

#[derive(Debug, Clone)]
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
    /// Constructs a DB-backed service. All persistence goes through the
    /// `organization_invitations` table — invites survive restarts, and
    /// accepting binds the user to the org (`user_organizations`).
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> Result<diesel::PgConnection, String> {
        self.pool.get().map_err(|e| format!("DB pool error: {e}"))
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
        let groups_json =
            serde_json::to_string(&params.groups).unwrap_or_else(|_| "[]".to_string());

        let pool = self.pool.clone();
        let conn_result = tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
            sql_query(
                "INSERT INTO organization_invitations \
                 (id, org_id, email, role, status, message, invited_by, invited_by_name, \
                  token, groups, created_at, updated_at, expires_at) \
                 VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8, $9::jsonb, $10, $10, $11)",
            )
            .bind::<diesel::sql_types::Uuid, _>(invitation_id)
            .bind::<diesel::sql_types::Uuid, _>(params.organization_id)
            .bind::<diesel::sql_types::Text, _>(&email_lower)
            .bind::<diesel::sql_types::Text, _>(params.role.as_str())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(params.message.as_deref())
            .bind::<diesel::sql_types::Uuid, _>(params.invited_by)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(params.invited_by_name))
            .bind::<diesel::sql_types::Text, _>(&token)
            .bind::<diesel::sql_types::Text, _>(&groups_json)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now + Duration::days(params.expires_in_days))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to persist invitation: {e}"))?;
            Ok::<_, String>(())
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

        let _ = conn_result;

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
        let invitation = self.get_invitation_by_token(token).await.ok_or("Invalid invitation token")?;

        if invitation.status != InvitationStatus::Pending {
            return Err(format!(
                "Invitation is not pending (status: {:?})",
                invitation.status
            ));
        }

        if invitation.expires_at < Utc::now() {
            self.update_status(&invitation.id, InvitationStatus::Expired, None, None)
                .await?;
            return Err("Invitation has expired".to_string());
        }

        let now = Utc::now();
        let pool = self.pool.clone();
        let org_id = invitation.organization_id;
        let role = invitation.role.as_str().to_string();
        tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
            sql_query(
                "UPDATE organization_invitations \
                 SET status = 'accepted', accepted_at = $1, accepted_by = $2, updated_at = $1 \
                 WHERE id = $3",
            )
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(invitation.id)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update invitation: {e}"))?;

            // Bind the user to the org so the suite scopes them correctly
            // (this is the missing collaborative glue — before this, accept
            // only flipped an in-memory flag and the user never joined).
            let binding_id = Uuid::new_v4();
            sql_query(
                "INSERT INTO user_organizations (id, user_id, org_id, role, is_default, joined_at) \
                 VALUES ($1, $2, $3, $4, false, $5) \
                 ON CONFLICT (user_id, org_id) DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(binding_id)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(org_id)
            .bind::<diesel::sql_types::Text, _>(&role)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to bind user to organization: {e}"))?;
            Ok::<_, String>(())
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

        Ok(AcceptInvitationResponse {
            success: true,
            organization_id: org_id,
            organization_name: "Organization".to_string(),
            role,
            message: "Successfully joined the organization".to_string(),
        })
    }

    async fn update_status(
        &self,
        invitation_id: &Uuid,
        status: InvitationStatus,
        accepted_at: Option<DateTime<Utc>>,
        accepted_by: Option<Uuid>,
    ) -> Result<(), String> {
        let now = Utc::now();
        let pool = self.pool.clone();
        let id = *invitation_id;
        let status_str = match status {
            InvitationStatus::Pending => "pending",
            InvitationStatus::Accepted => "accepted",
            InvitationStatus::Declined => "declined",
            InvitationStatus::Expired => "expired",
            InvitationStatus::Revoked => "revoked",
        }
        .to_string();
        tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
            sql_query(
                "UPDATE organization_invitations \
                 SET status = $1, accepted_at = $2, accepted_by = $3, updated_at = $4 \
                 WHERE id = $5",
            )
            .bind::<diesel::sql_types::Text, _>(&status_str)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(accepted_at)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(accepted_by)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update invitation status: {e}"))?;
            Ok::<_, String>(())
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
    }

    pub async fn decline_invitation(&self, token: &str) -> Result<(), String> {
        let invitation = self.get_invitation_by_token(token).await.ok_or("Invalid invitation token")?;
        if invitation.status != InvitationStatus::Pending {
            return Err("Only pending invitations can be declined".to_string());
        }
        self.update_status(&invitation.id, InvitationStatus::Declined, None, None)
            .await
    }

    pub async fn revoke_invitation(&self, invitation_id: Uuid) -> Result<(), String> {
        let invitation = self.get_invitation(invitation_id).await.ok_or("Invitation not found")?;
        if invitation.status != InvitationStatus::Pending {
            return Err("Only pending invitations can be revoked".to_string());
        }
        self.update_status(&invitation_id, InvitationStatus::Revoked, None, None)
            .await
    }

    pub async fn resend_invitation(
        &self,
        invitation_id: Uuid,
        organization_name: &str,
        extend_expiry: bool,
    ) -> Result<OrganizationInvitation, String> {
        let mut invitation = self.get_invitation(invitation_id).await.ok_or("Invitation not found")?;

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

        // Persist the refreshed token/status/expiry.
        let pool = self.pool.clone();
        let id = invitation.id;
        let status_str = "pending";
        let expires = invitation.expires_at;
        tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
            sql_query(
                "UPDATE organization_invitations \
                 SET status = $1, expires_at = $2, updated_at = $3 WHERE id = $4",
            )
            .bind::<diesel::sql_types::Text, _>(status_str)
            .bind::<diesel::sql_types::Timestamptz, _>(expires)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update invitation: {e}"))?;
            Ok::<_, String>(())
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

        let invitation_clone = invitation.clone();
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
        let status_filter_str = status_filter.map(|s| match s {
            InvitationStatus::Pending => "pending",
            InvitationStatus::Accepted => "accepted",
            InvitationStatus::Declined => "declined",
            InvitationStatus::Expired => "expired",
            InvitationStatus::Revoked => "revoked",
        }.to_string());

        let pool = self.pool.clone();
        let result = tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
            let mut sql = String::from(
                "SELECT id, org_id, email, role, status, message, invited_by, invited_by_name, \
                        token, groups, created_at, updated_at, expires_at, accepted_at, accepted_by \
                 FROM organization_invitations WHERE org_id = $1",
            );
            if status_filter_str.is_some() {
                sql.push_str(" AND status = $2");
            }
            sql.push_str(" ORDER BY created_at DESC LIMIT 500");
            let mut q = sql_query(&sql)
                .bind::<diesel::sql_types::Uuid, _>(organization_id);
            if let Some(ref s) = status_filter_str {
                q = q.bind::<diesel::sql_types::Text, _>(s);
            }
            q.load::<InvitationRow>(&mut conn)
                .map_err(|e| format!("Failed to list invitations: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"));

        let rows = match result {
            Ok(Ok(rows)) => rows,
            _ => Vec::new(),
        };

        let all: Vec<InvitationResponse> = rows
            .iter()
            .map(|r| self.to_response(&r.to_invitation(), "Organization"))
            .collect();

        let total = all.len() as u32;
        let total_pages = total.div_ceil(per_page.max(1));
        let start = ((page.saturating_sub(1)) * per_page.max(1)) as usize;
        let end = (start + per_page as usize).min(all.len());

        InvitationListResponse {
            invitations: all[start..end.min(all.len())].to_vec(),
            total,
            page,
            per_page,
            total_pages,
        }
    }

    pub async fn get_invitation(&self, invitation_id: Uuid) -> Option<OrganizationInvitation> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().ok()?;
            sql_query(
                "SELECT id, org_id, email, role, status, message, invited_by, invited_by_name, \
                        token, groups, created_at, updated_at, expires_at, accepted_at, accepted_by \
                 FROM organization_invitations WHERE id = $1 LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(invitation_id)
            .get_result::<InvitationRow>(&mut conn)
            .ok()
            .map(|r| r.to_invitation())
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn get_invitation_by_token(&self, token: &str) -> Option<OrganizationInvitation> {
        let pool = self.pool.clone();
        let token_owned = token.to_string();
        tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().ok()?;
            sql_query(
                "SELECT id, org_id, email, role, status, message, invited_by, invited_by_name, \
                        token, groups, created_at, updated_at, expires_at, accepted_at, accepted_by \
                 FROM organization_invitations WHERE token = $1 LIMIT 1",
            )
            .bind::<diesel::sql_types::Text, _>(&token_owned)
            .get_result::<InvitationRow>(&mut conn)
            .ok()
            .map(|r| r.to_invitation())
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn cleanup_expired_invitations(&self) {
        let pool = self.pool.clone();
        let _ = tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().ok()?;
            sql_query(
                "UPDATE organization_invitations \
                 SET status = 'expired', updated_at = $1 \
                 WHERE status = 'pending' AND expires_at < $1",
            )
            .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
            .execute(&mut conn)
            .ok()
        })
        .await;
    }

    async fn find_pending_invitation(
        &self,
        organization_id: &Uuid,
        email: &str,
    ) -> Option<OrganizationInvitation> {
        let pool = self.pool.clone();
        let org = *organization_id;
        let email_owned = email.to_string();
        tokio::task::spawn_blocking(move || {
            use diesel::sql_query;
            use diesel::RunQueryDsl;
            let mut conn = pool.get().ok()?;
            sql_query(
                "SELECT id, org_id, email, role, status, message, invited_by, invited_by_name, \
                        token, groups, created_at, updated_at, expires_at, accepted_at, accepted_by \
                 FROM organization_invitations \
                 WHERE org_id = $1 AND email = $2 AND status = 'pending' LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(org)
            .bind::<diesel::sql_types::Text, _>(&email_owned)
            .get_result::<InvitationRow>(&mut conn)
            .ok()
            .map(|r| r.to_invitation())
        })
        .await
        .ok()
        .flatten()
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

        #[cfg(feature = "mail")]
        {
            // Real SMTP delivery via env-configured server (MAIL_HOST/PORT/
            // USER/PASS/FROM — secrets come from the environment/Vault, never
            // hardcoded). Best-effort: a failure logs and never fails the
            // invite creation.
            use lettre::transport::smtp::authentication::Credentials;
            use lettre::{Message, SmtpTransport, Transport};

            let host = std::env::var("MAIL_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port: u16 = std::env::var("MAIL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(587);
            let user = std::env::var("MAIL_USER").unwrap_or_default();
            let pass = std::env::var("MAIL_PASS").unwrap_or_default();
            let from = std::env::var("MAIL_FROM").unwrap_or_else(|_| "notifications@pragmatismo.com.br".to_string());
            if host == "localhost" && user.is_empty() {
                log::warn!("MAIL_HOST not configured — invitation email not delivered");
                return;
            }

            let invite_url = std::env::var("INVITE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:5000/invitations/accept".to_string());
            let body = format!(
                "You have been invited to join {org_name}.\n\n\
                 Accept your invitation: {invite_url}?token={token}\n\n\
                 This invitation expires at {expires}.",
                token = invitation.token,
                expires = invitation.expires_at.format("%Y-%m-%d %H:%M UTC"),
            );

            match Message::builder()
                .from(from.parse().map_err(|e| e.to_string()).ok())
                .to(invitation.email.parse().map_err(|e| e.to_string()).ok())
                .subject(format!("Invitation to join {org_name}"))
                .body(body)
            {
                Ok(email) => {
                    let mailer = SmtpTransport::relay(&host)
                        .ok()
                        .map(|m| m.port(port).credentials(Credentials::new(user, pass)).build());
                    if let Some(mailer) = mailer {
                        if let Err(e) = mailer.send(&email) {
                            log::warn!("Failed to deliver invitation email: {e}");
                        }
                    } else {
                        log::warn!("Failed to build SMTP transport for {host}:{port}");
                    }
                }
                Err(e) => log::warn!("Failed to build invitation email: {e}"),
            }
        }
    }
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/organizations/:org_id/invitations", get(list_invitations))
        .route(
            "/organizations/{org_id}/invitations",
            post(create_invitation),
        )
        .route("/organizations/:org_id/invitations/bulk", post(bulk_invite))
        .route(
            "/organizations/{org_id}/invitations/{invitation_id}",
            get(get_invitation),
        )
        .route(
            "/organizations/{org_id}/invitations/{invitation_id}",
            delete(revoke_invitation),
        )
        .route(
            "/organizations/{org_id}/invitations/{invitation_id}/resend",
            post(resend_invitation),
        )
        .route("/invitations/accept", post(accept_invitation))
        .route("/invitations/decline", post(decline_invitation))
        .route("/invitations/validate/:token", get(validate_invitation))
}

async fn list_invitations(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Query(params): Query<ListInvitationsQuery>,
) -> Result<Json<InvitationListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

    let role: InvitationRole = req.role.parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid role"})),
        )
    })?;

    let expires_in_days = req.expires_in_days.unwrap_or(7).clamp(1, 30);

    // The inviter must be a real user (FK users(id)); resolve from headers
    // like the accept path instead of a random UUID.
    let invited_by = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .or_else(|| {
            botsecurity_core::tenant::user_id_from_claims(&headers).and_then(|sub| {
                match Uuid::parse_str(&sub) {
                    Ok(u) => Some(u),
                    Err(_) => Some(Uuid::new_v5(
                        &Uuid::NAMESPACE_DNS,
                        format!("zitadel:{sub}").as_bytes(),
                    )),
                }
            })
        })
        .unwrap_or(Uuid::nil());

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
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(req): Json<BulkInviteRequest>,
) -> Result<Json<BulkInviteResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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

    let invited_by = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .or_else(|| {
            botsecurity_core::tenant::user_id_from_claims(&headers).and_then(|sub| {
                match Uuid::parse_str(&sub) {
                    Ok(u) => Some(u),
                    Err(_) => Some(Uuid::new_v5(
                        &Uuid::NAMESPACE_DNS,
                        format!("zitadel:{sub}").as_bytes(),
                    )),
                }
            })
        })
        .unwrap_or(Uuid::nil());

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
    State(state): State<Arc<AppState>>,
    Path((org_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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
    State(state): State<Arc<AppState>>,
    Path((_org_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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
    State(state): State<Arc<AppState>>,
    Path((_org_id, invitation_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ResendInvitationRequest>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<AcceptInvitationRequest>,
) -> Result<Json<AcceptInvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

    // Resolve the accepting user from the request: X-User-ID header first
    // (loopback/chat executor), then the JWT sub claim (derived to the stable
    // UUID the RBAC layer uses). Never a random UUID — the org binding must
    // point at the real user or the accept is meaningless.
    let user_id = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .or_else(|| {
            botsecurity_core::tenant::user_id_from_claims(&headers).and_then(|sub| {
                match Uuid::parse_str(&sub) {
                    Ok(u) => Some(u),
                    Err(_) => Some(Uuid::new_v5(
                        &Uuid::NAMESPACE_DNS,
                        format!("zitadel:{sub}").as_bytes(),
                    )),
                }
            })
        })
        .or(req.user_id)
        .unwrap_or(Uuid::nil());

    if user_id == Uuid::nil() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required to accept an invitation"})),
        ));
    }

    match service.accept_invitation(&req.token, user_id).await {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )),
    }
}

async fn decline_invitation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AcceptInvitationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let service = InvitationService::new(state.conn.clone());

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

    /// Builds a DB-backed service for tests. Requires `DATABASE_URL` (the
    /// `organization_invitations` table must exist); returns `None` when the
    /// env var is absent so unit runs without a database are skipped.
    fn test_service() -> Option<InvitationService> {
        use diesel::r2d2::ConnectionManager;
        let url = std::env::var("DATABASE_URL").ok()?;
        let manager = ConnectionManager::<diesel::PgConnection>::new(url);
        let pool = Pool::new(manager).ok()?;
        Some(InvitationService::new(pool))
    }

    #[tokio::test]
    async fn test_create_invitation() {
        let Some(service) = test_service() else { return; };
        let org_id = Uuid::new_v4();
 
        let params = crate::organization_invitations::CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Test Org",
            email: "test@example.com",
            role: InvitationRole::Member,
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
        let Some(service) = test_service() else { return; };
        let org_id = Uuid::new_v4();
 
        let params = crate::organization_invitations::CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Test Org",
            email: "test@example.com",
            role: InvitationRole::Member,
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
        let Some(service) = test_service() else { return; };
        let org_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();
        let user_id = Uuid::new_v4();
 
        let params = crate::organization_invitations::CreateInvitationParams {
            organization_id: org_id,
            organization_name: "Test Org",
            email: "test@example.com",
            role: InvitationRole::Member,
            groups: vec![],
            invited_by,
            invited_by_name: "Admin",
            message: None,
            expires_in_days: 7,
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

use std::sync::Arc;
use uuid::Uuid;
use crate::shared::state::AppState;
use crate::shared::utils::DbPool;