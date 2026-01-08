use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info};
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;
use uuid::Uuid;

#[derive(QueryableByName)]
struct AccountResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    provider: String,
}

#[derive(QueryableByName, Debug, Clone)]
pub struct ActiveAccountResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub account_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub provider: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub qdrant_collection: String,
}

pub fn register_use_account_keyword(
    engine: &mut Engine,
    state: Arc<AppState>,
    session: Arc<UserSession>,
) -> Result<(), Box<EvalAltResult>> {
    let state_clone = Arc::clone(&state);
    let session_clone = Arc::clone(&session);

    engine.register_custom_syntax(
        ["USE", "ACCOUNT", "$expr$"],
        true,
        move |context, inputs| {
            let email = context.eval_expression_tree(&inputs[0])?.to_string();

            info!(
                "USE ACCOUNT keyword executed - Email: {}, Session: {}",
                email, session_clone.id
            );

            let session_id = session_clone.id;
            let bot_id = session_clone.bot_id;
            let user_id = session_clone.user_id;
            let conn = state_clone.conn.clone();
            let email_clone = email.clone();

            let result = std::thread::spawn(move || {
                add_account_to_session(conn, session_id, bot_id, user_id, &email_clone)
            })
            .join();

            match result {
                Ok(Ok(_)) => {
                    info!("Account '{}' added to session {}", email, session_clone.id);
                    Ok(Dynamic::UNIT)
                }
                Ok(Err(e)) => {
                    error!("Failed to add account '{}': {}", email, e);
                    Err(format!("USE_ACCOUNT failed: {}", e).into())
                }
                Err(e) => {
                    error!("Thread panic in USE_ACCOUNT: {:?}", e);
                    Err("USE_ACCOUNT failed: thread panic".into())
                }
            }
        },
    )?;

    Ok(())
}

fn add_account_to_session(
    conn_pool: crate::shared::utils::DbPool,
    session_id: Uuid,
    bot_id: Uuid,
    user_id: Uuid,
    email: &str,
) -> Result<(), String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let account: Option<AccountResult> = diesel::sql_query(
        "SELECT id, email, provider FROM connected_accounts
         WHERE email = $1 AND (bot_id = $2 OR user_id = $3) AND status = 'active'",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result(&mut conn)
    .optional()
    .map_err(|e| format!("Failed to query account: {}", e))?;

    let Some(account) = account else {
        return Err(format!(
            "Account '{}' not found or not configured. Add it in Sources app.",
            email
        ));
    };

    let qdrant_collection = format!("account_{}_{}", account.provider, account.id);

    let assoc_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO session_account_associations
         (id, session_id, bot_id, account_id, email, provider, qdrant_collection, is_active)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true)
         ON CONFLICT (session_id, account_id)
         DO UPDATE SET is_active = true, added_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(assoc_id)
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Uuid, _>(account.id)
    .bind::<diesel::sql_types::Text, _>(&account.email)
    .bind::<diesel::sql_types::Text, _>(&account.provider)
    .bind::<diesel::sql_types::Text, _>(&qdrant_collection)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to add account association: {}", e))?;

    info!(
        "Added account '{}' ({}) to session {} (collection: {})",
        email, account.provider, session_id, qdrant_collection
    );

    Ok(())
}

pub fn get_active_accounts_for_session(
    conn_pool: &crate::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<Vec<ActiveAccountResult>, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let results: Vec<ActiveAccountResult> = diesel::sql_query(
        "SELECT account_id, email, provider, qdrant_collection
         FROM session_account_associations
         WHERE session_id = $1 AND is_active = true
         ORDER BY added_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .load(&mut conn)
    .map_err(|e| format!("Failed to get active accounts: {}", e))?;

    Ok(results)
}

pub fn parse_account_path(path: &str) -> Option<(String, String)> {
    if let Some(rest) = path.strip_prefix("account://") {
        if let Some(slash_pos) = rest.find('/') {
            let email = &rest[..slash_pos];
            let file_path = &rest[slash_pos + 1..];
            return Some((email.to_string(), file_path.to_string()));
        }
    }
    None
}

pub fn is_account_path(path: &str) -> bool {
    path.starts_with("account://")
}

pub async fn get_account_credentials(
    conn_pool: &crate::shared::utils::DbPool,
    email: &str,
    bot_id: Uuid,
) -> Result<AccountCredentials, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    #[derive(QueryableByName)]
    struct CredResult {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        provider: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        access_token: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        refresh_token: Option<String>,
    }

    let creds: CredResult = diesel::sql_query(
        "SELECT id, provider, access_token, refresh_token
         FROM connected_accounts
         WHERE email = $1 AND bot_id = $2 AND status = 'active'",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result(&mut conn)
    .map_err(|e| format!("Account not found: {}", e))?;

    Ok(AccountCredentials {
        account_id: creds.id,
        provider: creds.provider,
        access_token: creds.access_token,
        refresh_token: creds.refresh_token,
    })
}

#[derive(Debug, Clone)]
pub struct AccountCredentials {
    pub account_id: Uuid,
    pub provider: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_account_path() {
        let result = parse_account_path("account://user@gmail.com/Documents/file.pdf");
        assert!(result.is_some());
        let (email, path) = result.unwrap();
        assert_eq!(email, "user@gmail.com");
        assert_eq!(path, "Documents/file.pdf");
    }

    #[test]
    fn test_parse_account_path_invalid() {
        assert!(parse_account_path("local/file.pdf").is_none());
        assert!(parse_account_path("/absolute/path").is_none());
    }

    #[test]
    fn test_is_account_path() {
        assert!(is_account_path("account://user@gmail.com/file.pdf"));
        assert!(!is_account_path("local/file.pdf"));
        assert!(!is_account_path("file.pdf"));
    }
}
