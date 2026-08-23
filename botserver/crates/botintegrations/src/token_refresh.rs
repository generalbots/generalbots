use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use diesel::prelude::*;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::oauth;
use crate::scope::ConnectionScope;
use crate::secrets::ConnectionVault;
use crate::state::IntegrationState;

#[derive(diesel::QueryableByName)]
struct ExpiringConnection {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    owner_user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    provider_slug: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    vault_path: String,
}

const TICK_SECONDS: u64 = 600;

fn refresh_url(slug: &str) -> Option<&'static str> {
    oauth::provider_config(slug).map(|config| config.token_url)
}

fn basic_client_auth(slug: &str) -> bool {
    oauth::provider_config(slug)
        .map(|config| config.basic_client_auth)
        .unwrap_or(false)
}

async fn refresh_one(
    http: &reqwest::Client,
    vault: &ConnectionVault,
    row: &ExpiringConnection,
) -> Result<Option<i64>, IntegrationError> {
    let Some(token_url) = refresh_url(&row.provider_slug) else {
        return Ok(None);
    };
    let scope = ConnectionScope {
        user_id: row.owner_user_id,
        org_id: row.org_id,
        branch_id: row.branch_id,
        bot_id: row.bot_id,
    };
    let envelope = vault.load_strict(&row.vault_path).await?;
    let refresh_token = envelope
        .get("refresh_token")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(IntegrationError::NotFound)?
        .to_string();

    let client_config_path = format!(
        "gbo/{}/{}/{}/integrations/oauth/{}",
        row.org_id, row.branch_id, row.bot_id, row.provider_slug
    );
    let client = vault.load_strict(&client_config_path).await?;
    let client_id = client
        .get("client_id")
        .and_then(Value::as_str)
        .ok_or(IntegrationError::NotFound)?
        .to_string();
    let client_secret = client
        .get("client_secret")
        .and_then(Value::as_str)
        .ok_or(IntegrationError::NotFound)?
        .to_string();

    let mut outgoing = http
        .post(token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
        ]);
    if basic_client_auth(&row.provider_slug) {
        let basic =
            base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));
        outgoing = outgoing.header("authorization", format!("Basic {basic}"));
    }
    let response = outgoing.send().await.map_err(|error| {
        log::warn!("token refresh request failed for {}: {error}", row.provider_slug);
        IntegrationError::VaultUnavailable
    })?;
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        log::warn!("token refresh returned {status} for {}", row.provider_slug);
        return Err(IntegrationError::VaultUnavailable);
    }
    let access_token = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or(IntegrationError::VaultUnavailable)?
        .to_string();
    let rotated = body
        .get("refresh_token")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or(refresh_token);
    let expires_in = body.get("expires_in").and_then(Value::as_i64);

    let mut next = json!({ "token": access_token, "refresh_token": rotated });
    if let Some(id_token) = body.get("id_token") {
        next["id_token"] = id_token.clone();
    }
    vault.store(&scope, row.id, &next).await?;
    Ok(expires_in)
}

async fn run_tick(state: &IntegrationState) {
    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::warn!("token refresh tick skipped, pool unavailable: {error}");
            return;
        }
    };
    let rows: Vec<ExpiringConnection> = diesel::sql_query(
        "SELECT id, org_id, branch_id, bot_id, owner_user_id, provider_slug, vault_path \
         FROM integration_connections \
         WHERE auth_kind = 'oauth2' AND status = 'active' \
           AND expires_at IS NOT NULL \
           AND expires_at < NOW() + INTERVAL '2 hours'",
    )
    .load(&mut conn)
    .unwrap_or_else(|error| {
        log::warn!("token refresh due query failed: {error:?}");
        Vec::new()
    });
    drop(conn);
    if rows.is_empty() {
        return;
    }
    let http = match reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
    {
        Ok(client) => client,
        Err(_) => return,
    };
    for row in rows {
        match refresh_one(&http, &state.vault, &row).await {
            Ok(Some(expires_in)) => {
                if let Ok(mut conn) = state.pool.get() {
                    diesel::sql_query(
                        "UPDATE integration_connections \
                         SET expires_at = NOW() + ($2 || ' seconds')::interval, \
                             last_refreshed_at = NOW(), credential_version = credential_version + 1 \
                         WHERE id = $1",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(row.id)
                    .bind::<diesel::sql_types::Text, _>(expires_in.to_string())
                    .execute(&mut conn)
                    .ok();
                }
                log::info!("token refreshed for {} connection {}", row.provider_slug, row.id);
            }
            Ok(None) => {}
            Err(error) => {
                log::warn!("token refresh failed for {} ({}): {:?}", row.provider_slug, row.id, error);
            }
        }
    }
}

/// Spawns the background token refresher keeping OAuth2 connections usable.
pub fn spawn(state: Arc<IntegrationState>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(TICK_SECONDS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            run_tick(&state).await;
        }
    });
    log::info!("integration token refresher started");
}
