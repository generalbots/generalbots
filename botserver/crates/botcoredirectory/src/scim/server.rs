use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use log::info;
use std::collections::HashMap;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::types::*;

pub fn configure_scim_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/scim/v2/Users", get(list_users).post(create_user))
        .route("/scim/v2/Users/:user_id", get(get_user).put(update_user).delete(delete_user))
        .route("/scim/v2/Users/:user_id/replace", post(replace_user))
        .route("/scim/v2/Groups", get(list_groups).post(create_group))
        .route("/scim/v2/Groups/:group_id", get(get_group).put(update_group).delete(delete_group))
        .route("/scim/v2/Groups/:group_id/replace", post(replace_group))
        .route("/scim/v2/Me", get(get_me))
        .route("/scim/v2/ServiceProviderConfig", get(service_provider_config))
        .route("/scim/v2/ResourceTypes", get(resource_types))
        .route("/scim/v2/Schemas", get(schemas))
}

async fn service_provider_config() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"],
        "patch": { "supported": true },
        "bulk": { "supported": false, "maxOperations": 0, "maxPayloadSize": 0 },
        "filter": { "supported": true, "maxResults": 100 },
        "changePassword": { "supported": false },
        "sort": { "supported": false },
        "etag": { "supported": false },
        "authenticationSchemes": [{
            "type": "oauthbearertoken",
            "name": "OAuth Bearer Token",
            "description": "Authentication scheme using the OAuth Bearer Token Standard",
            "specUri": "https://www.rfc-editor.org/info/rfc6750",
            "primary": true
        }]
    }))
}

async fn resource_types() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
        "resources": [{
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
            "id": "User",
            "name": "User",
            "endpoint": "/Users",
            "description": "User account",
            "schema": "urn:ietf:params:scim:schemas:core:2.0:User"
        }, {
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
            "id": "Group",
            "name": "Group",
            "endpoint": "/Groups",
            "description": "Group of users",
            "schema": "urn:ietf:params:scim:schemas:core:2.0:Group"
        }]
    }))
}

async fn schemas() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Schema"],
        "resources": [{
            "id": "urn:ietf:params:scim:schemas:core:2.0:User",
            "name": "User",
            "description": "Core User Schema",
            "attributes": [{
                "name": "userName",
                "type": "string",
                "required": true,
                "multiValued": false,
                "description": "Unique identifier for the User"
            }]
        }, {
            "id": "urn:ietf:params:scim:schemas:core:2.0:Group",
            "name": "Group",
            "description": "Core Group Schema",
            "attributes": [{
                "name": "displayName",
                "type": "string",
                "required": true,
                "multiValued": false,
                "description": "A human-readable name for the Group"
            }]
        }]
    }))
}

async fn get_me(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ScimUser>, (StatusCode, Json<ScimError>)> {
    Err((StatusCode::NOT_IMPLEMENTED, Json(ScimError {
        schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
        scim_type: None,
        detail: "GET /Me not supported - use specific user ID".to_string(),
        status: 501,
    })))
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ScimListResponse>, (StatusCode, Json<ScimError>)> {
    let start_index = params.get("startIndex")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    let count = params.get("count")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(20);

    info!("SCIM GET /Users startIndex={} count={}", start_index, count);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let filter = params.get("filter").map(|s| s.as_str());
    let users = match filter {
        Some(f) if f.starts_with("userName eq ") => {
            let query = f.trim_start_matches("userName eq ").trim_matches('"');
            match auth_service.search_users(query).await {
                Ok(u) => u,
                Err(_) => vec![],
            }
        }
        _ => {
            match auth_service.list_users(count as i64, (start_index - 1) as i64).await {
                Ok(data) => data
                    .as_array()
                    .cloned()
                    .unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    };

    let total = users.len() as u32;
    let resources: Vec<serde_json::Value> = users.iter()
        .map(|u| serde_json::to_value(ScimUser::from_zitadel_user(u, vec![])).unwrap_or_default())
        .collect();

    Ok(Json(ScimListResponse {
        schemas: vec!["urn:ietf:params:scim:api:messages:2.0:ListResponse".to_string()],
        total_results: total,
        start_index,
        items_per_page: count,
        resources,
    }))
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<ScimUser>, (StatusCode, Json<ScimError>)> {
    info!("SCIM GET /Users/{}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let user_data = auth_service.get_user(&user_id).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: Some("invalidValue".to_string()),
            detail: format!("User not found: {}", e),
            status: 404,
        }))
    })?;

    let memberships = auth_service.get_user_memberships(&user_id, 0, 100).await
        .unwrap_or_default();
    let groups: Vec<ScimGroupRef> = memberships.get("result")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("organizationId").and_then(|v| v.as_str()).map(|org_id| {
                        ScimGroupRef {
                            value: org_id.to_string(),
                            reference: Some(format!("/Groups/{}", org_id)),
                            display: None,
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Json(ScimUser::from_zitadel_user(&user_data, groups)))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(scim_user): Json<ScimUser>,
) -> Result<(StatusCode, Json<ScimUser>), (StatusCode, Json<ScimError>)> {
    info!("SCIM POST /Users - creating: {}", scim_user.user_name);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let body = scim_user.to_zitadel_json();
    let result = auth_service.http_post(
        format!("{}/v2/users", auth_service.api_url()),
        body,
    ).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: format!("Failed to create user: {}", e),
            status: 400,
        }))
    })?;

    let new_id = result.get("userId")
        .or_else(|| result.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut created = scim_user.clone();
    created.id = Some(new_id.clone());
    created.meta = Some(ScimMeta {
        resource_type: "User".to_string(),
        created: Some(chrono::Utc::now().to_rfc3339()),
        last_modified: Some(chrono::Utc::now().to_rfc3339()),
        location: Some(format!("/Users/{}", new_id)),
        version: None,
    });

    Ok((StatusCode::CREATED, Json(created)))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(scim_user): Json<ScimUser>,
) -> Result<Json<ScimUser>, (StatusCode, Json<ScimError>)> {
    info!("SCIM PUT /Users/{}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let body = scim_user.to_zitadel_json();
    auth_service.http_patch(
        format!("{}/v2/users/{}", auth_service.api_url(), user_id),
        body,
    ).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: format!("Failed to update user: {}", e),
            status: 400,
        }))
    })?;

    let mut updated = scim_user;
    updated.id = Some(user_id.clone());
    Ok(Json(updated))
}

async fn replace_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(scim_user): Json<ScimUser>,
) -> Result<Json<ScimUser>, (StatusCode, Json<ScimError>)> {
    update_user(State(state), Path(user_id), Json(scim_user)).await
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ScimError>)> {
    info!("SCIM DELETE /Users/{}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    auth_service.http_delete(
        format!("{}/v2/users/{}", auth_service.api_url(), user_id),
    ).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: format!("Failed to delete user: {}", e),
            status: 400,
        }))
    })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_groups(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ScimListResponse>, (StatusCode, Json<ScimError>)> {
    let start_index = params.get("startIndex")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    let count = params.get("count")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(20);

    info!("SCIM GET /Groups startIndex={} count={}", start_index, count);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let data = auth_service.http_get(
        format!("{}/metadata/organization?limit={}&offset={}", auth_service.api_url(), count, start_index - 1),
    ).await.unwrap_or_default();

    let metadata = data.as_array().cloned().unwrap_or_default();
    let groups: Vec<serde_json::Value> = metadata.iter()
        .filter_map(|item| {
            let key = item.get("key")?.as_str()?;
            if !key.starts_with("group_") { return None; }
            let value_str = item.get("value")?.as_str()?;
            let value: serde_json::Value = serde_json::from_str(value_str).ok()?;
            let display_name = value.get("name")?.as_str()?.to_string();
            let member_ids: Vec<String> = value.get("members")
                .and_then(|m| m.as_array())
                .map(|arr| arr.iter().filter_map(|m| m.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let members: Vec<ScimMember> = member_ids.iter().map(|id| {
                ScimMember {
                    value: id.clone(),
                    reference: Some(format!("/Users/{}", id)),
                    display: None,
                    member_type: Some("User".to_string()),
                }
            }).collect();

            let scim_group = ScimGroup {
                schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:Group".to_string()],
                id: Some(key.to_string()),
                external_id: None,
                display_name,
                members,
                meta: Some(ScimMeta {
                    resource_type: "Group".to_string(),
                    created: value.get("created_at").and_then(|v| v.as_str()).map(String::from),
                    last_modified: None,
                    location: Some(format!("/Groups/{}", key)),
                    version: None,
                }),
            };
            serde_json::to_value(scim_group).ok()
        })
        .collect();

    let total = groups.len() as u32;

    Ok(Json(ScimListResponse {
        schemas: vec!["urn:ietf:params:scim:api:messages:2.0:ListResponse".to_string()],
        total_results: total,
        start_index,
        items_per_page: count,
        resources: groups,
    }))
}

async fn get_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
) -> Result<Json<ScimGroup>, (StatusCode, Json<ScimError>)> {
    info!("SCIM GET /Groups/{}", group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let metadata = auth_service.http_get(
        format!("{}/metadata/organization/{}", auth_service.api_url(), group_id),
    ).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: Some("invalidValue".to_string()),
            detail: format!("Group not found: {}", e),
            status: 404,
        }))
    })?;

    let value_str = metadata.get("value").and_then(|v| v.as_str()).unwrap_or("{}");
    let value: serde_json::Value = serde_json::from_str(value_str).unwrap_or_default();

    let member_ids: Vec<String> = value.get("members")
        .and_then(|m| m.as_array())
        .map(|arr| arr.iter().filter_map(|m| m.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let members: Vec<ScimMember> = member_ids.iter().map(|id| {
        ScimMember {
            value: id.clone(),
            reference: Some(format!("/Users/{}", id)),
            display: None,
            member_type: Some("User".to_string()),
        }
    }).collect();

    Ok(Json(ScimGroup::from_metadata(&group_id, &value, members)))
}

async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(scim_group): Json<ScimGroup>,
) -> Result<(StatusCode, Json<ScimGroup>), (StatusCode, Json<ScimError>)> {
    info!("SCIM POST /Groups - creating: {}", scim_group.display_name);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let member_ids: Vec<String> = scim_group.members.iter().map(|m| m.value.clone()).collect();
    let metadata_key = format!("group_{}", scim_group.id.as_deref().unwrap_or("new"));
    let metadata_value = serde_json::json!({
        "name": scim_group.display_name,
        "members": member_ids,
        "created_at": chrono::Utc::now().to_rfc3339()
    }).to_string();

    let body = serde_json::json!({
        "key": metadata_key,
        "value": metadata_value
    });

    auth_service.http_post(
        format!("{}/metadata/organization", auth_service.api_url()),
        body,
    ).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: format!("Failed to create group: {}", e),
            status: 400,
        }))
    })?;

    let mut created = scim_group;
    created.id = Some(metadata_key.clone());
    created.meta = Some(ScimMeta {
        resource_type: "Group".to_string(),
        created: Some(chrono::Utc::now().to_rfc3339()),
        last_modified: Some(chrono::Utc::now().to_rfc3339()),
        location: Some(format!("/Groups/{}", metadata_key)),
        version: None,
    });

    Ok((StatusCode::CREATED, Json(created)))
}

async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(scim_group): Json<ScimGroup>,
) -> Result<Json<ScimGroup>, (StatusCode, Json<ScimError>)> {
    info!("SCIM PUT /Groups/{}", group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    let member_ids: Vec<String> = scim_group.members.iter().map(|m| m.value.clone()).collect();
    let metadata_value = serde_json::json!({
        "name": scim_group.display_name,
        "members": member_ids,
        "updated_at": chrono::Utc::now().to_rfc3339()
    }).to_string();

    let body = serde_json::json!({
        "value": metadata_value
    });

    auth_service.http_put(
        format!("{}/metadata/organization/{}", auth_service.api_url(), group_id),
        body,
    ).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: format!("Failed to update group: {}", e),
            status: 400,
        }))
    })?;

    let mut updated = scim_group;
    updated.id = Some(group_id);
    Ok(Json(updated))
}

async fn replace_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
    Json(scim_group): Json<ScimGroup>,
) -> Result<Json<ScimGroup>, (StatusCode, Json<ScimError>)> {
    update_group(State(state), Path(group_id), Json(scim_group)).await
}

async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ScimError>)> {
    info!("SCIM DELETE /Groups/{}", group_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "No auth service".to_string(),
            status: 500,
        }))
    })?.lock().await;

    auth_service.http_delete(
        format!("{}/metadata/organization/{}", auth_service.api_url(), group_id),
    ).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: format!("Failed to delete group: {}", e),
            status: 400,
        }))
    })?;

    Ok(StatusCode::NO_CONTENT)
}
