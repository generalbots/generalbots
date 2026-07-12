use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AddImapAccountForm {
    pub email: String,
    pub display_name: Option<String>,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
}

#[derive(diesel::QueryableByName, Debug)]
struct ConnectedAccountRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    metadata_json: String,
}

pub async fn handle_list_accounts_htmx(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let pool = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => return Err(format!("DB error: {e}")),
        };

        let rows: Vec<ConnectedAccountRow> = diesel::sql_query(
            "SELECT id, email, status, metadata_json
             FROM connected_accounts
             WHERE provider = 'imap' AND status = 'active'
             ORDER BY created_at DESC"
        )
            .load::<ConnectedAccountRow>(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?;

        Ok::<_, String>(rows)
    }).await;

    let accounts = match accounts {
        Ok(Ok(list)) => list,
        _ => Vec::new(),
    };

    if accounts.is_empty() {
        return Html(
            r#"<div class="empty-state">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="48" height="48">
                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                    <polyline points="22,6 12,13 2,6"></polyline>
                </svg>
                <h3>No email accounts connected</h3>
                <p>Add an IMAP email account to use it as a source for LLM context.</p>
            </div>"#.to_string(),
        );
    }

    let mut html = String::from(r#"<div class="accounts-grid">"#);
    for acct in &accounts {
        let acct_id = acct.id;
        let status = &acct.status;
        let status_class = if status == "active" { "active" } else { "error" };

        let meta: HashMap<String, String> = serde_json::from_str(&acct.metadata_json).unwrap_or_default();
        let imap_server = meta.get("imap_server").map(|s| s.as_str()).unwrap_or("");

        let _ = write!(
            html,
            r#"<div class="account-card">
                <div class="account-header">
                    <div class="account-icon imap">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                            <polyline points="22,6 12,13 2,6"></polyline>
                        </svg>
                    </div>
                    <div class="account-info">
                        <div class="account-email">{email}</div>
                        <div class="account-provider">IMAP — {imap_server}</div>
                    </div>
                    <span class="account-status {status_class}">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="6"/></svg>
                        {status}
                    </span>
                </div>
                <div class="account-actions">
                    <button class="danger" onclick="removeAccount('{acct_id}')" style="flex:1;padding:0.5rem;border:1px solid var(--border);border-radius:0.375rem;background:var(--bg-primary);color:var(--text-secondary);font-size:0.75rem;cursor:pointer;">
                        Remove
                    </button>
                </div>
            </div>"#,
            acct_id = acct_id,
            email = acct.email,
            imap_server = imap_server,
            status_class = status_class,
            status = status,
        );
    }
    html.push_str("</div>");
    Html(html)
}

pub async fn handle_add_imap_account(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AddImapAccountForm>,
) -> impl IntoResponse {
    let account_id = Uuid::new_v4();
    let vault_path = format!("gbo/users/imap/{}", account_id);
    let vault_path_for_vault = vault_path.clone();
    let vault_path_for_db = vault_path.clone();

    let mut vault_data = HashMap::new();
    vault_data.insert("imap_password".to_string(), form.password.clone());
    vault_data.insert("imap_server".to_string(), form.imap_server.clone());
    vault_data.insert("imap_port".to_string(), form.imap_port.to_string());
    vault_data.insert("smtp_server".to_string(), form.smtp_server.clone());
    vault_data.insert("smtp_port".to_string(), form.smtp_port.to_string());

    let vault_result = tokio::task::spawn_blocking(move || {
        let sm = match botcoresecrets::manager::SecretsManager::get() {
            Ok(s) => s,
            Err(e) => return Err(format!("Vault not available: {e}")),
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all().build()
            .map_err(|e| format!("Runtime build failed: {e}"))?;
        rt.block_on(sm.put_secret(&vault_path_for_vault, vault_data))
            .map_err(|e| format!("Vault write failed: {e}"))
    }).await;

    match vault_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            log::error!("Failed to save IMAP credentials to Vault: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Html(
                format!(r#"<div class="toast toast-error">Failed to save credentials: {e}</div>"#)
            ));
        }
        Err(e) => {
            log::error!("Vault task error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(
                r#"<div class="toast toast-error">Internal error saving credentials</div>"#.to_string()
            ));
        }
    }

    let metadata = serde_json::json!({
        "imap_server": form.imap_server,
        "imap_port": form.imap_port,
        "smtp_server": form.smtp_server,
        "smtp_port": form.smtp_port,
        "username": form.username,
        "vault_path": vault_path,
    });

    let pool = state.conn.clone();
    let email_clone = form.email.clone();
    let db_result = tokio::task::spawn_blocking(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => return Err(format!("DB connection: {e}")),
        };

        diesel::sql_query(
            "INSERT INTO connected_accounts
             (id, bot_id, email, provider, account_type, access_token, status, metadata_json, created_at, updated_at)
             VALUES ($1, $2, $3, 'imap', 'email', $4, 'active', $5, NOW(), NOW())"
        )
            .bind::<diesel::sql_types::Uuid, _>(account_id)
            .bind::<diesel::sql_types::Uuid, _>(Uuid::nil())
            .bind::<diesel::sql_types::Text, _>(&email_clone)
            .bind::<diesel::sql_types::Text, _>(&vault_path_for_db)
            .bind::<diesel::sql_types::Text, _>(&metadata.to_string())
            .execute(&mut conn)
            .map_err(|e| format!("DB insert failed: {e}"))?;

        Ok::<_, String>(())
    }).await;

    match db_result {
        Ok(Ok(_)) => {
            (StatusCode::OK, Html(
                r#"<div class="toast toast-success">IMAP account added as source!</div><script>htmx.trigger('#accounts-list', 'load');</script>"#.to_string()
            ))
        }
        Ok(Err(e)) => {
            log::error!("Failed to insert IMAP account: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Html(
                format!(r#"<div class="toast toast-error">Failed to save account: {e}</div>"#)
            ))
        }
        Err(e) => {
            log::error!("DB task error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Html(
                r#"<div class="toast toast-error">Internal error</div>"#.to_string()
            ))
        }
    }
}

pub async fn handle_delete_account(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let id = match Uuid::parse_str(&account_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Html(
            r#"<div class="toast toast-error">Invalid account ID</div>"#.to_string()
        )),
    };

    let pool = state.conn.clone();
    let db_result = tokio::task::spawn_blocking(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => return Err(format!("DB connection: {e}")),
        };

        diesel::sql_query("UPDATE connected_accounts SET status = 'removed' WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| format!("DB delete failed: {e}"))?;

        Ok::<_, String>(())
    }).await;

    match db_result {
        Ok(Ok(_)) => {
            (StatusCode::OK, Html(
                r#"<div class="toast toast-success">Account removed</div><script>htmx.trigger('#accounts-list', 'load');</script>"#.to_string()
            ))
        }
        _ => {
            (StatusCode::INTERNAL_SERVER_ERROR, Html(
                r#"<div class="toast toast-error">Failed to remove account</div>"#.to_string()
            ))
        }
    }
}
