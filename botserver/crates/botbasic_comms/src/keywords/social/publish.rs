//! #2056 — publish a BASIC `POST TO` to the external platform.
//!
//! The keyword used to insert a `pending` row (into a table that does not even
//! have those columns) and report success. Nothing drained the queue, so the
//! bot claimed a post that was never published.
//!
//! This module does the real work: it validates the platform, performs the
//! provider call, and records `published` or `failed` with the remote id and
//! permalink. A platform without a transport fails loudly and is never queued.
//! Scheduled posts are stored and published by [`spawn_drainer`].

use botbasic_types::{BasicRuntime, UserSession};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// Platforms with a working publish transport today.
const SUPPORTED_PLATFORMS: &[&str] = &["instagram", "facebook"];

/// A published post as the provider reported it.
#[derive(Debug, Clone)]
pub struct PublishedPost {
    pub remote_id: String,
    pub permalink: Option<String>,
}

/// Fails with a clear message when the platform has no transport. Callers use
/// this before inserting anything so an unsupported platform never leaves a
/// pending row behind.
pub fn ensure_supported(platform: &str) -> Result<(), String> {
    if SUPPORTED_PLATFORMS.contains(&platform.to_ascii_lowercase().as_str()) {
        return Ok(());
    }
    Err(format!(
        "POST TO: '{platform}' is not implemented. Supported platforms: {}",
        SUPPORTED_PLATFORMS.join(", ")
    ))
}

/// Credentials read from the bot config (`config_value`). Instagram needs the
/// business account id, Facebook the page id; both need the Graph token.
struct Credentials {
    token: String,
    account_id: String,
}

fn credentials(state: &dyn BasicRuntime, platform: &str) -> Result<Credentials, String> {
    let (token_key, account_key) = match platform {
        "instagram" => ("instagram-access-token", "instagram-account-id"),
        "facebook" => ("facebook-page-access-token", "facebook-page-id"),
        other => return Err(format!("POST TO: no credentials for '{other}'")),
    };
    let token = state
        .config_value(token_key)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!("POST TO {platform}: {token_key} is not configured for this bot")
        })?;
    let account_id = state
        .config_value(account_key)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!("POST TO {platform}: {account_key} is not configured for this bot")
        })?;
    Ok(Credentials { token, account_id })
}

async fn post_instagram(
    http: &reqwest::Client,
    creds: &Credentials,
    media: &str,
    caption: &str,
) -> Result<PublishedPost, String> {
    let base = format!("https://graph.facebook.com/v17.0/{}", creds.account_id);
    let container: serde_json::Value = http
        .post(format!("{base}/media"))
        .query(&[
            ("access_token", creds.token.as_str()),
            ("image_url", media),
            ("caption", caption),
        ])
        .send()
        .await
        .map_err(|e| format!("Instagram request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Instagram rejected the media container: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Instagram response unreadable: {e}"))?;
    let creation_id = container
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Instagram returned no creation id".to_string())?;
    let published: serde_json::Value = http
        .post(format!("{base}/media_publish"))
        .query(&[
            ("access_token", creds.token.as_str()),
            ("creation_id", creation_id),
        ])
        .send()
        .await
        .map_err(|e| format!("Instagram publish failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Instagram rejected the publish: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Instagram publish response unreadable: {e}"))?;
    let remote_id = published
        .get("id")
        .and_then(|value| value.as_str())
        .unwrap_or(creation_id)
        .to_string();
    Ok(PublishedPost {
        remote_id,
        permalink: None,
    })
}

async fn post_facebook(
    http: &reqwest::Client,
    creds: &Credentials,
    media: &str,
    caption: &str,
) -> Result<PublishedPost, String> {
    let url = format!("https://graph.facebook.com/v17.0/{}/feed", creds.account_id);
    let mut form: Vec<(&str, String)> = vec![
        ("message", caption.to_string()),
        ("access_token", creds.token.clone()),
    ];
    if !media.trim().is_empty() {
        form.push(("link", media.to_string()));
    }
    let body: serde_json::Value = http
        .post(url)
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("Facebook request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Facebook rejected the post: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Facebook response unreadable: {e}"))?;
    let remote_id = body
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Facebook returned no post id".to_string())?
        .to_string();
    Ok(PublishedPost {
        permalink: Some(format!("https://www.facebook.com/{remote_id}")),
        remote_id,
    })
}

/// Performs the provider call. Never called before the platform passed
/// [`ensure_supported`].
async fn publish_with_provider(
    state: &dyn BasicRuntime,
    platform: &str,
    media: &str,
    caption: &str,
) -> Result<PublishedPost, String> {
    ensure_supported(platform)?;
    let creds = credentials(state, platform)?;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| format!("HTTP client unavailable: {e}"))?;
    match platform {
        "instagram" => post_instagram(&http, &creds, media, caption).await,
        "facebook" => post_facebook(&http, &creds, media, caption).await,
        other => Err(format!("POST TO: no transport for '{other}'")),
    }
}

/// Records the attempt before the provider call, so a crash cannot hide it.
fn insert_pending(
    state: &dyn BasicRuntime,
    user: &UserSession,
    platform: &str,
    media: &str,
    caption: &str,
    scheduled_at: Option<DateTime<Utc>>,
) -> Result<Uuid, String> {
    let mut conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&user.user_id).ok();
    diesel::sql_query(
        "INSERT INTO social_external_posts
            (id, bot_id, user_id, platform, content, media_url, status, scheduled_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(user.bot_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(platform)
    .bind::<diesel::sql_types::Text, _>(caption)
    .bind::<diesel::sql_types::Text, _>(media)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(scheduled_at)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to record the post: {e}"))?;
    Ok(id)
}

fn mark_published(
    state: &dyn BasicRuntime,
    id: Uuid,
    published: &PublishedPost,
) -> Result<(), String> {
    let mut conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
    diesel::sql_query(
        "UPDATE social_external_posts
            SET status = 'published', remote_post_id = $2, permalink = $3,
                error = NULL, published_at = NOW(), updated_at = NOW(),
                attempts = attempts + 1
          WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&published.remote_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(published.permalink.as_deref())
    .execute(&mut conn)
    .map_err(|e| format!("Failed to record the published post: {e}"))?;
    Ok(())
}

fn mark_failed(state: &dyn BasicRuntime, id: Uuid, error: &str) -> Result<(), String> {
    let mut conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
    diesel::sql_query(
        "UPDATE social_external_posts
            SET status = 'failed', error = $2, updated_at = NOW(), attempts = attempts + 1
          WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(error)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to record the failed post: {e}"))?;
    Ok(())
}

/// Publishes now and returns the row id only after the provider answered with
/// success. A failure is recorded and returned to the caller.
pub async fn publish_now(
    state: &dyn BasicRuntime,
    user: &UserSession,
    platform: &str,
    media: &str,
    caption: &str,
) -> Result<Uuid, String> {
    ensure_supported(platform)?;
    let id = insert_pending(state, user, platform, media, caption, None)?;
    match publish_with_provider(state, platform, media, caption).await {
        Ok(published) => {
            mark_published(state, id, &published)?;
            Ok(id)
        }
        Err(error) => {
            let _ = mark_failed(state, id, &error);
            Err(error)
        }
    }
}

/// Stores a future post for [`spawn_drainer`]. Validation happens first so an
/// unsupported platform is refused instead of sleeping in the queue.
pub fn schedule(
    state: &dyn BasicRuntime,
    user: &UserSession,
    platform: &str,
    media: &str,
    caption: &str,
    scheduled_at: DateTime<Utc>,
) -> Result<Uuid, String> {
    ensure_supported(platform)?;
    insert_pending(state, user, platform, media, caption, Some(scheduled_at))
}

/// Publishes every due `pending` row. The state is needed for the config and
/// the pool; rows are claimed one at a time so two workers cannot publish the
/// same post twice.
pub async fn publish_due(state: &dyn BasicRuntime) -> usize {
    let Ok(mut conn) = state.db_pool().get() else {
        return 0;
    };
    #[derive(diesel::QueryableByName)]
    struct DueRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        platform: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        content: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        media_url: String,
    }
    let due: Vec<DueRow> = diesel::sql_query(
        "UPDATE social_external_posts
            SET status = 'publishing', updated_at = NOW()
          WHERE id IN (
              SELECT id FROM social_external_posts
               WHERE status = 'pending' AND scheduled_at IS NOT NULL AND scheduled_at <= NOW()
               ORDER BY scheduled_at
               LIMIT 10
               FOR UPDATE SKIP LOCKED
          )
          RETURNING id, bot_id, platform, content, media_url",
    )
    .load(&mut conn)
    .unwrap_or_default();
    drop(conn);

    let mut published = 0;
    for row in due {
        // A scheduled row carries its own payload; publish it without
        // re-inserting (the row already exists and is claimed). `bot_id` is
        // kept in the row for the audit trail.
        let _bot = row.bot_id;
        match publish_with_provider(state, &row.platform, &row.media_url, &row.content).await {
            Ok(result) => {
                if mark_published(state, row.id, &result).is_ok() {
                    published += 1;
                }
            }
            Err(error) => {
                let _ = mark_failed(state, row.id, &error);
            }
        }
    }
    published
}

/// Starts the scheduled-post worker. Safe to call once at boot; it ticks every
/// 60 s and only picks rows whose time has come.
pub fn spawn_drainer(state: Arc<dyn BasicRuntime>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            ticker.tick().await;
            let published = publish_due(&*state).await;
            if published > 0 {
                log::info!("POST TO: published {published} scheduled post(s)");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_platforms_with_a_transport_are_accepted() {
        assert!(ensure_supported("instagram").is_ok());
        assert!(ensure_supported("FACEBOOK").is_ok());
        let error = ensure_supported("myspace").expect_err("must fail");
        assert!(error.contains("not implemented"), "{error}");
        assert!(error.contains("instagram"), "{error}");
    }
}
