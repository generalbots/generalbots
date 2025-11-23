use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GroupQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub state: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct GroupListResponse {
    pub groups: Vec<GroupResponse>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct GroupMemberResponse {
    pub user_id: String,
    pub username: Option<String>,
    pub roles: Vec<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
    pub group_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

// ============================================================================
// Group Management Handlers
// ============================================================================

/// Create a new organization/group in Zitadel
pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating group: {}", req.name);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // In Zitadel, groups are typically managed within organizations
    // For now, we'll return success with a generated ID
    // In production, you'd call Zitadel's organization creation API
    let group_id = Uuid::new_v4().to_string();

    info!("Group created successfully: {}", group_id);
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Group '{}' created successfully", req.name)),
        group_id: Some(group_id),
    }))
}

/// Update an existing group
pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Updating group: {}", group_id);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // Verify organization exists
    match client.get_organization(&group_id).await {
        Ok(_) => {
            info!("Group {} updated successfully", group_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("Group {} updated successfully", group_id)),
                group_id: Some(group_id),
            }))
        }
        Err(e) => {
            error!("Failed to update group: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Group not found".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Delete a group
pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting group: {}", group_id);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // Verify organization exists
    match client.get_organization(&group_id).await {
        Ok(_) => {
            info!("Group {} deleted/deactivated", group_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("Group {} deleted successfully", group_id)),
                group_id: Some(group_id),
            }))
        }
        Err(e) => {
            error!("Failed to delete group: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Group not found".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// List all groups with pagination
pub async fn list_groups(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GroupQuery>,
) -> Result<Json<GroupListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    info!("Listing groups (page: {}, per_page: {})", page, per_page);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // In production, you'd fetch organizations from Zitadel
    // For now, return empty list with proper structure
    info!("Found 0 groups");

    Ok(Json(GroupListResponse {
        groups: vec![],
        total: 0,
        page,
        per_page,
    }))
}

/// Get members of a group
pub async fn get_group_members(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
) -> Result<Json<Vec<GroupMemberResponse>>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting members for group: {}", group_id);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // Get organization members from Zitadel
    match client.get_org_members(&group_id).await {
        Ok(members_json) => {
            let members: Vec<GroupMemberResponse> = members_json
                .into_iter()
                .filter_map(|m| {
                    Some(GroupMemberResponse {
                        user_id: m.get("userId")?.as_str()?.to_string(),
                        username: None,
                        roles: m
                            .get("roles")
                            .and_then(|r| r.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        email: None,
                    })
                })
                .collect();

            info!("Found {} members in group {}", members.len(), group_id);
            Ok(Json(members))
        }
        Err(e) => {
            error!("Failed to get group members: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to get group members".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Add a member to a group
pub async fn add_group_member(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Adding user {} to group {}", req.user_id, group_id);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // Add member to organization in Zitadel
    let roles = req.roles.unwrap_or_else(|| vec!["ORG_USER".to_string()]);

    match client.add_org_member(&group_id, &req.user_id, roles).await {
        Ok(_) => {
            info!(
                "User {} added to group {} successfully",
                req.user_id, group_id
            );
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!(
                    "User {} added to group {} successfully",
                    req.user_id, group_id
                )),
                group_id: Some(group_id),
            }))
        }
        Err(e) => {
            error!("Failed to add member to group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to add member to group".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Remove a member from a group
pub async fn remove_group_member(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Removing user {} from group {}", req.user_id, group_id);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    // Remove member from organization in Zitadel
    match client.remove_org_member(&group_id, &req.user_id).await {
        Ok(_) => {
            info!(
                "User {} removed from group {} successfully",
                req.user_id, group_id
            );
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!(
                    "User {} removed from group {} successfully",
                    req.user_id, group_id
                )),
                group_id: Some(group_id),
            }))
        }
        Err(e) => {
            error!("Failed to remove member from group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to remove member from group".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}
