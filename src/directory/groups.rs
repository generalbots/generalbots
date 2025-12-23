
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



#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub members: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub members: Option<Vec<String>>,
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
    pub groups: Vec<GroupInfo>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct GroupInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
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




pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating group: {}", req.name);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


    let metadata_key = format!("group_{}", Uuid::new_v4());
    let metadata_value = serde_json::json!({
        "name": req.name,
        "description": req.description,
        "members": req.members.unwrap_or_default(),
        "created_at": chrono::Utc::now().to_rfc3339()
    })
    .to_string();


    match client
        .http_post(format!("{}/metadata/organization", client.api_url()))
        .await
        .json(&serde_json::json!({
            "key": metadata_key,
            "value": metadata_value
        }))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Group created successfully: {}", metadata_key);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("Group '{}' created successfully", req.name)),
                group_id: Some(metadata_key),
            }))
        }
        Ok(response) => {
            error!("Failed to create group: {}", response.status());
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to create group: {}", response.status()),
                    details: None,
                }),
            ))
        }
        Err(e) => {
            error!("Error creating group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e.to_string()),
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


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


    match client
        .http_put(format!(
            "{}/metadata/organization/{}",
            client.api_url(),
            group_id
        ))
        .await
        .json(&serde_json::json!({
            "value": serde_json::Value::Object(update_data).to_string()
        }))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Group updated successfully: {}", group_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("Group '{}' updated successfully", group_id)),
                group_id: Some(group_id),
            }))
        }
        Ok(response) => {
            error!("Failed to update group: {}", response.status());
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to update group: {}", response.status()),
                    details: None,
                }),
            ))
        }
        Err(e) => {
            error!("Error updating group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e.to_string()),
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


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


    match client
        .http_get(format!("{}/metadata/organization", client.api_url()))
        .await
        .query(&[
            ("limit", per_page.to_string()),
            ("offset", ((page - 1) * per_page).to_string()),
        ])
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let metadata: Vec<serde_json::Value> = response.json().await.unwrap_or_default();

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
        Ok(response) => {
            error!("Failed to list groups: {}", response.status());
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to list groups: {}", response.status()),
                    details: None,
                }),
            ))
        }
        Err(e) => {
            error!("Error listing groups: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e.to_string()),
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


    match client
        .http_get(format!(
            "{}/metadata/organization/{}",
            client.api_url(),
            group_id
        ))
        .await
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let metadata: serde_json::Value = response.json().await.unwrap_or_default();

            if let Some(value_str) = metadata.get("value").and_then(|v| v.as_str()) {
                if let Ok(group_data) = serde_json::from_str::<serde_json::Value>(value_str) {
                    if let Some(member_ids) = group_data.get("members").and_then(|m| m.as_array()) {

                        let mut members = Vec::new();

                        for member_id in member_ids {
                            if let Some(user_id) = member_id.as_str() {

                                if let Ok(user_response) = client
                                    .http_get(format!("{}/users/{}", client.api_url(), user_id))
                                    .await
                                    .send()
                                    .await
                                {
                                    if user_response.status().is_success() {
                                        if let Ok(user_data) =
                                            user_response.json::<serde_json::Value>().await
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
        Ok(response) => {
            error!("Failed to get group members: {}", response.status());
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Group not found".to_string(),
                    details: None,
                }),
            ))
        }
        Err(e) => {
            error!("Error getting group members: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Internal error: {}", e),
                    details: Some(e.to_string()),
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


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
