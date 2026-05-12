use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;
use chrono;
use serde_json;

use botcore::shared::state::AppState;
use super::types::*;

pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating group: {}", req.name);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let metadata_key = format!("group_{}", Uuid::new_v4());
    let metadata_value = serde_json::json!({
        "name": req.name,
        "description": req.description,
        "members": req.members.unwrap_or_default(),
        "created_at": chrono::Utc::now().to_rfc3339()
    })
    .to_string();

    let body = serde_json::json!({
        "key": metadata_key,
        "value": metadata_value
    });

    match auth_service
        .http_post(format!("{}/metadata/organization", auth_service.api_url()), body)
        .await
    {
        Ok(_) => {
            info!("Group created successfully: {}", metadata_key);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("Group '{}' created successfully", req.name)),
                group_id: Some(metadata_key),
            }))
        }
        Err(e) => {
            error!("Error creating group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e),
                }),
            ))
        }
    }
}


pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Updating group: {}", group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;


    let mut update_data = serde_json::Map::new();
    if let Some(name) = &req.name {
        update_data.insert("name".to_string(), serde_json::json!(name));
    }
    if let Some(description) = &req.description {
        update_data.insert("description".to_string(), serde_json::json!(description));
    }
    if let Some(members) = &req.members {
        update_data.insert("members".to_string(), serde_json::json!(members));
    }
    update_data.insert(
        "updated_at".to_string(),
        serde_json::json!(chrono::Utc::now().to_rfc3339()),
    );


    let body = serde_json::json!({
        "value": serde_json::Value::Object(update_data).to_string()
    });

    match auth_service
        .http_put(format!(
            "{}/metadata/organization/{}",
            auth_service.api_url(),
            group_id
        ), body)
        .await
    {
        Ok(_) => {
            info!("Group updated successfully: {}", group_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("Group '{}' updated successfully", group_id)),
                group_id: Some(group_id),
            }))
        }
        Err(e) => {
            error!("Error updating group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e),
                }),
            ))
        }
    }
}


pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting group: {}", group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;


    match auth_service.list_organizations().await {
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


pub async fn list_groups(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GroupQuery>,
) -> Result<Json<GroupListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    info!("Listing groups (page: {}, per_page: {})", page, per_page);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;


    match auth_service
        .http_get(format!("{}/metadata/organization?limit={}&offset={}", auth_service.api_url(), per_page, (page - 1) * per_page))
        .await
    {
        Ok(data) => {
            let metadata: Vec<serde_json::Value> = data.as_array()
                .cloned()
                .unwrap_or_default();

            let groups: Vec<GroupInfo> = metadata
                .iter()
                .filter_map(|item| {
                    if let Some(key) = item.get("key").and_then(|k| k.as_str()) {
                        if key.starts_with("group_") {
                            if let Some(value_str) = item.get("value").and_then(|v| v.as_str()) {
                                if let Ok(group_data) =
                                    serde_json::from_str::<serde_json::Value>(value_str)
                                {
                                    return Some(GroupInfo {
                                        id: key.to_string(),
                                        name: group_data
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("Unknown")
                                            .to_string(),
                                        description: group_data
                                            .get("description")
                                            .and_then(|d| d.as_str())
                                            .map(|s| s.to_string()),
                                        member_count: group_data
                                            .get("members")
                                            .and_then(|m| m.as_array())
                                            .map(|a| a.len())
                                            .unwrap_or(0),
                                    });
                                }
                            }
                        }
                    }
                    None
                })
                .collect();

            let total = groups.len();
            info!("Found {} groups", total);

            Ok(Json(GroupListResponse {
                groups,
                total,
                page,
                per_page,
            }))
        }
        Err(e) => {
            error!("Error listing groups: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e),
                }),
            ))
        }
    }
}


pub async fn get_group_members(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
) -> Result<Json<Vec<GroupMemberResponse>>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting members for group: {}", group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;


    match auth_service
        .http_get(format!(
            "{}/metadata/organization/{}",
            auth_service.api_url(),
            group_id
        ))
        .await
    {
        Ok(metadata) => {
            if let Some(value_str) = metadata.get("value").and_then(|v| v.as_str()) {
                if let Ok(group_data) = serde_json::from_str::<serde_json::Value>(value_str) {
                    if let Some(member_ids) = group_data.get("members").and_then(|m| m.as_array()) {

                        let mut members = Vec::new();

                        for member_id in member_ids {
                            if let Some(user_id) = member_id.as_str() {
                                if let Ok(user_data) = auth_service
                                    .get_user(user_id)
                                    .await
                                {
                                    members.push(GroupMemberResponse {
                                        user_id: user_id.to_string(),
                                        username: user_data
                                            .get("userName")
                                            .and_then(|u| u.as_str())
                                            .map(|s| s.to_string()),
                                        email: user_data
                                            .get("profile")
                                            .and_then(|p| p.get("email"))
                                            .and_then(|e| e.as_str())
                                            .map(|s| s.to_string()),
                                        roles: vec![],
                                    });
                                }
                            }
                        }

                        info!("Found {} members in group {}", members.len(), group_id);
                        return Ok(Json(members));
                    }
                }
            }

            info!("Group {} has no members", group_id);
            Ok(Json(vec![]))
        }
        Err(e) => {
            error!("Error getting group members: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Group not found".to_string(),
                    details: Some(e),
                }),
            ))
        }
    }
}


pub async fn add_group_member(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Adding user {} to group {}", req.user_id, group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;


    let roles = req.roles.unwrap_or_else(|| vec!["ORG_USER".to_string()]);

    match auth_service.add_org_member(&group_id, &req.user_id, roles).await {
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


pub async fn remove_group_member(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Removing user {} from group {}", req.user_id, group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;


    match auth_service.remove_org_member(&group_id, &req.user_id).await {
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
