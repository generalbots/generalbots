use crate::shared::state::AppState;
use crate::core::middleware::AuthenticatedUser;
use super::types::*;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use diesel::prelude::*;
use log::warn;
use std::sync::Arc;
use uuid::Uuid;

fn extract_user_from_session(_state: &Arc<AppState>) -> Result<Uuid, String> {
    Ok(Uuid::new_v4())
}

fn encrypt_password(password: &str) -> String {
    general_purpose::STANDARD.encode(password.as_bytes())
}

pub async fn add_email_account(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmailAccountRequest>,
) -> Result<Json<ApiResponse<EmailAccountResponse>>, EmailError> {
    let Ok(current_user_id) = extract_user_from_session(&state) else {
        return Err(EmailError("Authentication required".to_string()));
    };

    let account_id = Uuid::new_v4();
    let encrypted_password = encrypt_password(&request.password);

    let resp_email = request.email.clone();
    let resp_display_name = request.display_name.clone();
    let resp_imap_server = request.imap_server.clone();
    let resp_imap_port = request.imap_port;
    let resp_smtp_server = request.smtp_server.clone();
    let resp_smtp_port = request.smtp_port;
    let resp_is_primary = request.is_primary;

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        use crate::shared::models::schema::user_email_accounts::dsl::{is_primary, user_email_accounts, user_id};
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;

        if request.is_primary {
            diesel::update(user_email_accounts.filter(user_id.eq(&current_user_id)))
                .set(is_primary.eq(false))
                .execute(&mut db_conn)
                .ok();
        }

        diesel::sql_query(
            "INSERT INTO user_email_accounts
            (id, user_id, email, display_name, imap_server, imap_port, smtp_server, smtp_port, username, password_encrypted, is_primary, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind::<diesel::sql_types::Uuid, _>(account_id)
        .bind::<diesel::sql_types::Uuid, _>(current_user_id)
        .bind::<diesel::sql_types::Text, _>(&request.email)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(request.display_name.as_ref())
        .bind::<diesel::sql_types::Text, _>(&request.imap_server)
        .bind::<diesel::sql_types::Integer, _>(i32::from(request.imap_port))
        .bind::<diesel::sql_types::Text, _>(&request.smtp_server)
        .bind::<diesel::sql_types::Integer, _>(i32::from(request.smtp_port))
        .bind::<diesel::sql_types::Text, _>(&request.username)
        .bind::<diesel::sql_types::Text, _>(&encrypted_password)
        .bind::<diesel::sql_types::Bool, _>(request.is_primary)
        .bind::<diesel::sql_types::Bool, _>(true)
        .execute(&mut db_conn)
        .map_err(|e| format!("Failed to insert account: {e}"))?;

        Ok::<_, String>(account_id)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(EmailAccountResponse {
            id: account_id.to_string(),
            email: resp_email,
            display_name: resp_display_name,
            imap_server: resp_imap_server,
            imap_port: resp_imap_port,
            smtp_server: resp_smtp_server,
            smtp_port: resp_smtp_port,
            is_primary: resp_is_primary,
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
        message: Some("Email account added successfully".to_string()),
    }))
}

pub async fn list_email_accounts_htmx(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(user_id) = extract_user_from_session(&state) else {
        return axum::response::Html(r#"
            <div class="account-item" onclick="document.getElementById('add-account-modal').showModal()">
                <span>+ Add email account</span>
            </div>
        "#.to_string());
    };

    let conn = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;

        diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts WHERE user_id = $1 AND is_active = true ORDER BY is_primary DESC"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<EmailAccountBasicRow>(&mut db_conn)
        .map_err(|e| format!("Query failed: {e}"))
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or_default();

    if accounts.is_empty() {
        return axum::response::Html(r#"
            <div class="account-item" onclick="document.getElementById('add-account-modal').showModal()">
                <span>+ Add email account</span>
            </div>
        "#.to_string());
    }

    let mut html = String::new();
    for account in accounts {
        let name = account
            .display_name
            .clone()
            .unwrap_or_else(|| account.email.clone());
        let primary_badge = if account.is_primary {
            r#"<span class="badge">Primary</span>"#
        } else {
            ""
        };
        use std::fmt::Write;
        let _ = write!(
            html,
            r#"<div class="account-item" data-account-id="{}">
                <span>{}</span>
                {}
            </div>"#,
            account.id, name, primary_badge
        );
    }

    axum::response::Html(html)
}

pub async fn list_email_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<EmailAccountResponse>>>, EmailError> {
    let Ok(current_user_id) = extract_user_from_session(&state) else {
        return Err(EmailError("Authentication required".to_string()));
    };

    let conn = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        use crate::shared::models::schema::user_email_accounts::dsl::{
            created_at, display_name, email, id, imap_port, imap_server, is_active, is_primary,
            smtp_port, smtp_server, user_email_accounts, user_id,
        };
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {e}"))?;

        let results = user_email_accounts
            .filter(user_id.eq(current_user_id))
            .filter(is_active.eq(true))
            .order((is_primary.desc(), created_at.desc()))
            .select((
                id,
                email,
                display_name,
                imap_server,
                imap_port,
                smtp_server,
                smtp_port,
                is_primary,
                is_active,
                created_at,
            ))
            .load::<(
                Uuid,
                String,
                Option<String>,
                String,
                i32,
                String,
                i32,
                bool,
                bool,
                chrono::DateTime<chrono::Utc>,
            )>(&mut db_conn)
            .map_err(|e| format!("Query failed: {e}"))?;

        Ok::<_, String>(results)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    let account_list: Vec<EmailAccountResponse> = accounts
        .into_iter()
        .map(
            |(
                acc_id,
                acc_email,
                acc_display_name,
                acc_imap_server,
                acc_imap_port,
                acc_smtp_server,
                acc_smtp_port,
                acc_is_primary,
                acc_is_active,
                acc_created_at,
            )| {
                EmailAccountResponse {
                    id: acc_id.to_string(),
                    email: acc_email,
                    display_name: acc_display_name,
                    imap_server: acc_imap_server,
                    imap_port: acc_imap_port as u16,
                    smtp_server: acc_smtp_server,
                    smtp_port: acc_smtp_port as u16,
                    is_primary: acc_is_primary,
                    is_active: acc_is_active,
                    created_at: acc_created_at.to_rfc3339(),
                }
            },
        )
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(account_list),
        message: None,
    }))
}

pub async fn delete_email_account(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, EmailError> {
    let account_uuid =
        Uuid::parse_str(&account_id).map_err(|_| EmailError("Invalid account ID".to_string()))?;

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {e}"))?;

        diesel::sql_query("UPDATE user_email_accounts SET is_active = false WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(account_uuid)
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to delete account: {e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Email account deleted".to_string()),
    }))
}
