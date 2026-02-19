use super::admin_types::*;
use crate::core::shared::models::core::OrganizationInvitation;
use crate::core::shared::state::AppState;
use crate::core::urls::ApiUrls;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use log::{error, info, warn};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list_invitations(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use crate::core::shared::models::schema::organization_invitations::dsl::*;

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let results = organization_invitations
        .filter(status.eq("pending"))
        .filter(expires_at.gt(Utc::now()))
        .order_by(created_at.desc())
        .load::<OrganizationInvitation>(&mut conn);

    match results {
        Ok(invites) => {
            let responses: Vec<InvitationResponse> = invites
                .into_iter()
                .map(|inv| InvitationResponse {
                    id: inv.id,
                    email: inv.email,
                    role: inv.role,
                    message: inv.message,
                    created_at: inv.created_at,
                    token: inv.token,
                })
                .collect();

            (StatusCode::OK, Json(BulkInvitationResponse { invitations: responses })).into_response()
        }
        Err(e) => {
            error!("Failed to list invitations: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to list invitations"})),
            )
                .into_response()
        }
    }
}

pub async fn create_invitation(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Json(request): Json<CreateInvitationRequest>,
) -> impl IntoResponse {
    use crate::core::shared::models::schema::organization_invitations::dsl::*;

    let _bot_id = bot_id;
    let invitation_id = Uuid::new_v4();
    let token = format!("{}{}", invitation_id, Uuid::new_v4());
    let expires_at = Utc::now() + Duration::days(7);
    let accept_url = format!("{}/accept-invitation?token={}", ApiUrls::get_app_url(), token);

    let body = format!(
        "You have been invited to join our organization as a {}.\n\nClick on link below to accept the invitation:\n{}\n\nThis invitation will expire in 7 days.",
        request.role, accept_url
    );

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let new_invitation = OrganizationInvitation {
        id: invitation_id,
        org_id: Uuid::new_v4(),
        email: request.email.clone(),
        role: request.role.clone(),
        status: "pending".to_string(),
        message: request.custom_message.clone(),
        invited_by: Uuid::new_v4(),
        token: Some(token.clone()),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        expires_at: Some(expires_at),
        accepted_at: None,
        accepted_by: None,
    };

    match diesel::insert_into(organization_invitations)
        .values(&new_invitation)
        .execute(&mut conn)
    {
        Ok(_) => {
            info!("Created invitation for {} with role {}", request.email, request.role);
            (
                StatusCode::OK,
                Json(InvitationResponse {
                    id: invitation_id,
                    email: request.email,
                    role: request.role,
                    message: request.custom_message,
                    created_at: Utc::now(),
                    token: Some(token),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to create invitation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create invitation"})),
            )
                .into_response()
        }
    }
}

pub async fn create_bulk_invitations(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BulkInvitationRequest>,
) -> impl IntoResponse {
    use crate::core::shared::models::schema::organization_invitations::dsl::*;

    info!("Creating {} bulk invitations", request.emails.len());

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let mut responses = Vec::new();

    for email in &request.emails {
        let invitation_id = Uuid::new_v4();
        let token = format!("{}{}", invitation_id, Uuid::new_v4());
        let expires_at = Utc::now() + Duration::days(7);

        let new_invitation = OrganizationInvitation {
            id: invitation_id,
            org_id: Uuid::new_v4(),
            email: email.clone(),
            role: request.role.clone(),
            status: "pending".to_string(),
            message: request.custom_message.clone(),
            invited_by: Uuid::new_v4(),
            token: Some(token.clone()),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            expires_at: Some(expires_at),
            accepted_at: None,
            accepted_by: None,
        };

        match diesel::insert_into(organization_invitations)
            .values(&new_invitation)
            .execute(&mut conn)
        {
            Ok(_) => {
                info!("Created invitation for {} with role {}", email, request.role);
                responses.push(InvitationResponse {
                    id: invitation_id,
                    email: email.clone(),
                    role: request.role.clone(),
                    message: request.custom_message.clone(),
                    created_at: Utc::now(),
                    token: Some(token),
                });
            }
            Err(e) => {
                error!("Failed to create invitation for {}: {}", email, e);
            }
        }
    }

    (StatusCode::OK, Json(BulkInvitationResponse { invitations: responses })).into_response()
}

pub async fn get_invitation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::core::shared::models::schema::organization_invitations::dsl::*;

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    match organization_invitations
        .filter(id.eq(id))
        .first::<OrganizationInvitation>(&mut conn)
    {
        Ok(invitation) => {
            let response = InvitationResponse {
                id: invitation.id,
                email: invitation.email,
                role: invitation.role,
                message: invitation.message,
                created_at: invitation.created_at,
                token: invitation.token,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(diesel::result::Error::NotFound) => {
            warn!("Invitation not found: {}", id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Invitation not found"})),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to get invitation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get invitation"})),
            )
                .into_response()
        }
    }
}

pub async fn cancel_invitation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::core::shared::models::schema::organization_invitations::dsl::*;

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    match diesel::update(organization_invitations.filter(id.eq(id)))
        .set((
            status.eq("cancelled"),
            updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)
    {
        Ok(0) => {
            warn!("Invitation not found for cancellation: {}", id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Invitation not found"})),
            )
                .into_response()
        }
        Ok(_) => {
            info!("Cancelled invitation: {}", id);
            (
                StatusCode::OK,
                Json(serde_json::json!({"success": true, "message": "Invitation cancelled"})),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to cancel invitation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to cancel invitation"})),
            )
                .into_response()
        }
    }
}

pub async fn resend_invitation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::core::shared::models::schema::organization_invitations::dsl::*;

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    match organization_invitations
        .filter(id.eq(id))
        .first::<OrganizationInvitation>(&mut conn)
    {
        Ok(invitation) => {
            if invitation.status != "pending" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Invitation is not pending"})),
                )
                    .into_response();
            }

            let new_expires_at = Utc::now() + Duration::days(7);

            match diesel::update(organization_invitations.filter(id.eq(id)))
                .set((
                    updated_at.eq(Utc::now()),
                    expires_at.eq(new_expires_at),
                ))
                .execute(&mut conn)
            {
                Ok(_) => {
                    info!("Resent invitation: {}", id);
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({"success": true, "message": "Invitation resent"})),
                    )
                        .into_response()
                }
                Err(e) => {
                    error!("Failed to resend invitation: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to resend invitation"})),
                    )
                        .into_response()
                }
            }
        }
        Err(diesel::result::Error::NotFound) => {
            warn!("Invitation not found for resending: {}", id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Invitation not found"})),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to get invitation for resending: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get invitation"})),
            )
                .into_response()
        }
    }
}
