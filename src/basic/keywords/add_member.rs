use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub fn add_member_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["ADD_MEMBER", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let group_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let user_email = context.eval_expression_tree(&inputs[1])?.to_string();
                let role = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!(
                    "ADD_MEMBER: group={}, user_email={}, role={} for user={}",
                    group_id,
                    user_email,
                    role,
                    user_clone.user_id
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_add_member(
                                &state_for_task,
                                &user_for_task,
                                &group_id,
                                &user_email,
                                &role,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send ADD_MEMBER result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(member_id)) => Ok(Dynamic::from(member_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("ADD_MEMBER failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "ADD_MEMBER timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("ADD_MEMBER thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // Register CREATE_TEAM for creating teams with workspace
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            &["CREATE_TEAM", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let name = context.eval_expression_tree(&inputs[0])?.to_string();
                let members_input = context.eval_expression_tree(&inputs[1])?;
                let workspace_template = context.eval_expression_tree(&inputs[2])?.to_string();

                let mut members = Vec::new();
                if members_input.is_array() {
                    let arr = members_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        members.push(item.to_string());
                    }
                } else {
                    members.push(members_input.to_string());
                }

                trace!(
                    "CREATE_TEAM: name={}, members={:?}, template={} for user={}",
                    name,
                    members,
                    workspace_template,
                    user_clone2.user_id
                );

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_create_team(
                                &state_for_task,
                                &user_for_task,
                                &name,
                                members,
                                &workspace_template,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send CREATE_TEAM result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(15)) {
                    Ok(Ok(team_id)) => Ok(Dynamic::from(team_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CREATE_TEAM failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "CREATE_TEAM timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

async fn execute_add_member(
    state: &AppState,
    user: &UserSession,
    group_id: &str,
    user_email: &str,
    role: &str,
) -> Result<String, String> {
    let member_id = Uuid::new_v4().to_string();

    // Validate role
    let valid_role = validate_role(role);

    // Get default permissions for role
    let permissions = get_permissions_for_role(&valid_role);

    // Save member to database
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let query = diesel::sql_query(
        "INSERT INTO group_members (id, group_id, user_email, role, permissions, added_by, added_at, is_active)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true)
         ON CONFLICT (group_id, user_email)
         DO UPDATE SET role = $4, permissions = $5, updated_at = $7"
    )
    .bind::<diesel::sql_types::Text, _>(&member_id)
    .bind::<diesel::sql_types::Text, _>(group_id)
    .bind::<diesel::sql_types::Text, _>(user_email)
    .bind::<diesel::sql_types::Text, _>(&valid_role)
    .bind::<diesel::sql_types::Jsonb, _>(&permissions);

    let user_id_str = user.user_id.to_string();
    let now = Utc::now();
    let query = query
        .bind::<diesel::sql_types::Text, _>(&user_id_str)
        .bind::<diesel::sql_types::Timestamptz, _>(&now);

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to add member: {}", e);
        format!("Failed to add member: {}", e)
    })?;

    // Send invitation email if new member
    send_member_invitation(state, group_id, user_email, &valid_role).await?;

    // Update group activity log
    log_group_activity(state, group_id, "member_added", user_email).await?;

    trace!(
        "Added {} to group {} as {} with permissions {:?}",
        user_email,
        group_id,
        valid_role,
        permissions
    );

    Ok(member_id)
}

async fn execute_create_team(
    state: &AppState,
    user: &UserSession,
    name: &str,
    members: Vec<String>,
    workspace_template: &str,
) -> Result<String, String> {
    let team_id = Uuid::new_v4().to_string();

    // Create the team/group
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let query = diesel::sql_query(
        "INSERT INTO groups (id, name, type, template, created_by, created_at, settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Text, _>(&team_id)
    .bind::<diesel::sql_types::Text, _>(name)
    .bind::<diesel::sql_types::Text, _>("team")
    .bind::<diesel::sql_types::Text, _>(workspace_template);

    let user_id_str = user.user_id.to_string();
    let now = Utc::now();
    let query = query
        .bind::<diesel::sql_types::Text, _>(&user_id_str)
        .bind::<diesel::sql_types::Timestamptz, _>(&now)
        .bind::<diesel::sql_types::Jsonb, _>(
            &serde_json::to_value(json!({
                "workspace_enabled": true,
                "chat_enabled": true,
                "file_sharing": true
            }))
            .unwrap(),
        );

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to create team: {}", e);
        format!("Failed to create team: {}", e)
    })?;

    // Add creator as admin
    execute_add_member(state, user, &team_id, &user.user_id.to_string(), "admin").await?;

    // Add all members
    for member_email in &members {
        let role = if member_email == &user.user_id.to_string() {
            "admin"
        } else {
            "member"
        };
        execute_add_member(state, user, &team_id, member_email, role).await?;
    }

    // Create workspace structure
    create_workspace_structure(state, &team_id, name, workspace_template).await?;

    // Create team communication channel
    create_team_channel(state, &team_id, name).await?;

    trace!(
        "Created team '{}' with {} members (ID: {})",
        name,
        members.len(),
        team_id
    );

    Ok(team_id)
}

fn validate_role(role: &str) -> String {
    match role.to_lowercase().as_str() {
        "admin" | "administrator" => "admin".to_string(),
        "contributor" | "editor" => "contributor".to_string(),
        "member" | "user" => "member".to_string(),
        "viewer" | "read" | "readonly" => "viewer".to_string(),
        "owner" => "owner".to_string(),
        _ => "member".to_string(), // Default role
    }
}

fn get_permissions_for_role(role: &str) -> serde_json::Value {
    match role {
        "owner" => json!({
            "read": true,
            "write": true,
            "delete": true,
            "manage_members": true,
            "manage_settings": true,
            "export_data": true
        }),
        "admin" => json!({
            "read": true,
            "write": true,
            "delete": true,
            "manage_members": true,
            "manage_settings": true,
            "export_data": true
        }),
        "contributor" => json!({
            "read": true,
            "write": true,
            "delete": false,
            "manage_members": false,
            "manage_settings": false,
            "export_data": true
        }),
        "member" => json!({
            "read": true,
            "write": true,
            "delete": false,
            "manage_members": false,
            "manage_settings": false,
            "export_data": false
        }),
        "viewer" => json!({
            "read": true,
            "write": false,
            "delete": false,
            "manage_members": false,
            "manage_settings": false,
            "export_data": false
        }),
        _ => json!({
            "read": true,
            "write": false,
            "delete": false,
            "manage_members": false,
            "manage_settings": false,
            "export_data": false
        }),
    }
}

async fn send_member_invitation(
    _state: &AppState,
    group_id: &str,
    user_email: &str,
    role: &str,
) -> Result<(), String> {
    // In a real implementation, send an actual email invitation
    trace!(
        "Invitation sent to {} for group {} with role {}",
        user_email,
        group_id,
        role
    );
    Ok(())
}

async fn log_group_activity(
    state: &AppState,
    group_id: &str,
    action: &str,
    details: &str,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let activity_id = Uuid::new_v4().to_string();

    let query = diesel::sql_query(
        "INSERT INTO group_activity_log (id, group_id, action, details, timestamp)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind::<diesel::sql_types::Text, _>(&activity_id)
    .bind::<diesel::sql_types::Text, _>(group_id)
    .bind::<diesel::sql_types::Text, _>(action)
    .bind::<diesel::sql_types::Text, _>(details);

    let now = Utc::now();
    let query = query.bind::<diesel::sql_types::Timestamptz, _>(&now);

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to log activity: {}", e);
        format!("Failed to log activity: {}", e)
    })?;

    Ok(())
}

async fn create_workspace_structure(
    state: &AppState,
    team_id: &str,
    team_name: &str,
    template: &str,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    // Define workspace structure based on template
    let folders = match template {
        "project" => vec![
            "Documents",
            "Meetings",
            "Resources",
            "Deliverables",
            "Archive",
        ],
        "sales" => vec!["Proposals", "Contracts", "Presentations", "CRM", "Reports"],
        "support" => vec![
            "Tickets",
            "Knowledge Base",
            "FAQs",
            "Training",
            "Escalations",
        ],
        _ => vec!["Documents", "Shared", "Archive"],
    };

    let workspace_base = format!(".gbdrive/workspaces/{}", team_name);

    for folder in folders {
        let folder_path = format!("{}/{}", workspace_base, folder);

        let folder_id = Uuid::new_v4().to_string();
        let query = diesel::sql_query(
            "INSERT INTO workspace_folders (id, team_id, path, name, created_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind::<diesel::sql_types::Text, _>(&folder_id)
        .bind::<diesel::sql_types::Text, _>(team_id)
        .bind::<diesel::sql_types::Text, _>(&folder_path)
        .bind::<diesel::sql_types::Text, _>(folder)
        .bind::<diesel::sql_types::Timestamptz, _>(&chrono::Utc::now());

        query.execute(&mut *conn).map_err(|e| {
            error!("Failed to create workspace folder: {}", e);
            format!("Failed to create workspace folder: {}", e)
        })?;
    }

    trace!("Created workspace structure for team {}", team_name);
    Ok(())
}

async fn create_team_channel(
    state: &AppState,
    team_id: &str,
    team_name: &str,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let channel_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let query = diesel::sql_query(
        "INSERT INTO communication_channels (id, team_id, name, type, created_at)
         VALUES ($1, $2, $3, 'team_chat', $4)",
    )
    .bind::<diesel::sql_types::Text, _>(&channel_id)
    .bind::<diesel::sql_types::Text, _>(team_id)
    .bind::<diesel::sql_types::Text, _>(team_name)
    .bind::<diesel::sql_types::Timestamptz, _>(&now);

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to create team channel: {}", e);
        format!("Failed to create team channel: {}", e)
    })?;

    trace!("Created communication channel for team {}", team_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_role() {
        assert_eq!(validate_role("admin"), "admin");
        assert_eq!(validate_role("ADMIN"), "admin");
        assert_eq!(validate_role("contributor"), "contributor");
        assert_eq!(validate_role("viewer"), "viewer");
        assert_eq!(validate_role("unknown"), "member");
    }

    #[test]
    fn test_get_permissions_for_role() {
        let admin_perms = get_permissions_for_role("admin");
        assert!(admin_perms.get("read").unwrap().as_bool().unwrap());
        assert!(admin_perms.get("write").unwrap().as_bool().unwrap());
        assert!(admin_perms
            .get("manage_members")
            .unwrap()
            .as_bool()
            .unwrap());

        let viewer_perms = get_permissions_for_role("viewer");
        assert!(viewer_perms.get("read").unwrap().as_bool().unwrap());
        assert!(!viewer_perms.get("write").unwrap().as_bool().unwrap());
        assert!(!viewer_perms.get("delete").unwrap().as_bool().unwrap());
    }
}
