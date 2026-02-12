use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{debug, trace};
use rhai::{Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostEngagement {
    pub likes: i64,
    pub comments: i64,
    pub shares: i64,
    pub views: i64,
    pub clicks: i64,
    pub reach: i64,
}

impl PostEngagement {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();
        map.insert("likes".into(), Dynamic::from(self.likes));
        map.insert("comments".into(), Dynamic::from(self.comments));
        map.insert("shares".into(), Dynamic::from(self.shares));
        map.insert("views".into(), Dynamic::from(self.views));
        map.insert("clicks".into(), Dynamic::from(self.clicks));
        map.insert("reach".into(), Dynamic::from(self.reach));
        Dynamic::from(map)
    }
}

pub fn get_instagram_metrics_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["GET", "INSTAGRAM", "METRICS", "$expr$"],
            false,
            move |context, inputs| {
                let post_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let post_id = post_id.trim_matches('"');

                trace!("GET INSTAGRAM METRICS: {}", post_id);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let post_id_owned = post_id.to_string();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            fetch_instagram_metrics(&state_for_task, &user_for_task, &post_id_owned)
                                .await
                        });
                        let _ = tx.send(result);
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(metrics)) => Ok(metrics.to_dynamic()),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GET INSTAGRAM METRICS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "GET INSTAGRAM METRICS timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    debug!("Registered GET INSTAGRAM METRICS keyword");
}

pub fn get_facebook_metrics_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["GET", "FACEBOOK", "METRICS", "$expr$"],
            false,
            move |context, inputs| {
                let post_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let post_id = post_id.trim_matches('"');

                trace!("GET FACEBOOK METRICS: {}", post_id);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let post_id_owned = post_id.to_string();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            fetch_facebook_metrics(&state_for_task, &user_for_task, &post_id_owned)
                                .await
                        });
                        let _ = tx.send(result);
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(metrics)) => Ok(metrics.to_dynamic()),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GET FACEBOOK METRICS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "GET FACEBOOK METRICS timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    debug!("Registered GET FACEBOOK METRICS keyword");
}

pub fn get_linkedin_metrics_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["GET", "LINKEDIN", "METRICS", "$expr$"],
            false,
            move |context, inputs| {
                let post_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let post_id = post_id.trim_matches('"');

                trace!("GET LINKEDIN METRICS: {}", post_id);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let post_id_owned = post_id.to_string();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            fetch_linkedin_metrics(&state_for_task, &user_for_task, &post_id_owned)
                                .await
                        });
                        let _ = tx.send(result);
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(metrics)) => Ok(metrics.to_dynamic()),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GET LINKEDIN METRICS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "GET LINKEDIN METRICS timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    debug!("Registered GET LINKEDIN METRICS keyword");
}

pub fn get_twitter_metrics_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["GET", "TWITTER", "METRICS", "$expr$"],
            false,
            move |context, inputs| {
                let post_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let post_id = post_id.trim_matches('"');

                trace!("GET TWITTER METRICS: {}", post_id);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let post_id_owned = post_id.to_string();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            fetch_twitter_metrics(&state_for_task, &user_for_task, &post_id_owned)
                                .await
                        });
                        let _ = tx.send(result);
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(metrics)) => Ok(metrics.to_dynamic()),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GET TWITTER METRICS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "GET TWITTER METRICS timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    debug!("Registered GET TWITTER METRICS keyword");
}

fn get_platform_credentials(
    state: &AppState,
    bot_id: Uuid,
    platform: &str,
) -> Result<Value, String> {
    use diesel::prelude::*;

    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
    let key = format!("{}_credentials", platform);

    #[derive(QueryableByName)]
    struct SettingRow {
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        value: Value,
    }

    let query = diesel::sql_query("SELECT value FROM bot_settings WHERE bot_id = $1 AND key = $2")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(&key);

    let result: Result<Vec<SettingRow>, _> = query.load(&mut *conn);

    match result {
        Ok(rows) if !rows.is_empty() => Ok(rows[0].value.clone()),
        _ => Err(format!("No {} credentials configured", platform)),
    }
}

async fn fetch_instagram_metrics(
    state: &AppState,
    user: &UserSession,
    post_id: &str,
) -> Result<PostEngagement, String> {
    let credentials = get_platform_credentials(state, user.bot_id, "instagram")?;

    let access_token = credentials
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Missing access_token")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("https://graph.facebook.com/v18.0/{}", post_id))
        .query(&[
            ("fields", "like_count,comments_count,impressions_count"),
            ("access_token", access_token),
        ])
        .send()
        .await
        .map_err(|e| format!("Instagram API error: {}", e))?;

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(PostEngagement {
        likes: data.get("like_count").and_then(|v| v.as_i64()).unwrap_or(0),
        comments: data
            .get("comments_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        views: data
            .get("impressions_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        ..Default::default()
    })
}

async fn fetch_facebook_metrics(
    state: &AppState,
    user: &UserSession,
    post_id: &str,
) -> Result<PostEngagement, String> {
    let credentials = get_platform_credentials(state, user.bot_id, "facebook")?;

    let access_token = credentials
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Missing access_token")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("https://graph.facebook.com/v18.0/{}", post_id))
        .query(&[
            (
                "fields",
                "likes.summary(true),comments.summary(true),shares",
            ),
            ("access_token", access_token),
        ])
        .send()
        .await
        .map_err(|e| format!("Facebook API error: {}", e))?;

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(PostEngagement {
        likes: data
            .get("likes")
            .and_then(|l| l.get("summary"))
            .and_then(|s| s.get("total_count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        comments: data
            .get("comments")
            .and_then(|c| c.get("summary"))
            .and_then(|s| s.get("total_count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        shares: data
            .get("shares")
            .and_then(|s| s.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        ..Default::default()
    })
}

async fn fetch_linkedin_metrics(
    state: &AppState,
    user: &UserSession,
    post_id: &str,
) -> Result<PostEngagement, String> {
    let credentials = get_platform_credentials(state, user.bot_id, "linkedin")?;

    let access_token = credentials
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Missing access_token")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://api.linkedin.com/v2/socialActions/{}/summary",
            post_id
        ))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("LinkedIn API error: {}", e))?;

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(PostEngagement {
        likes: data
            .get("likesSummary")
            .and_then(|l| l.get("totalLikes"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        comments: data
            .get("commentsSummary")
            .and_then(|c| c.get("totalFirstLevelComments"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        ..Default::default()
    })
}

async fn fetch_twitter_metrics(
    state: &AppState,
    user: &UserSession,
    post_id: &str,
) -> Result<PostEngagement, String> {
    let credentials = get_platform_credentials(state, user.bot_id, "twitter")?;

    let bearer_token = credentials
        .get("bearer_token")
        .and_then(|v| v.as_str())
        .ok_or("Missing bearer_token")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("https://api.twitter.com/2/tweets/{}", post_id))
        .query(&[("tweet.fields", "public_metrics")])
        .header("Authorization", format!("Bearer {}", bearer_token))
        .send()
        .await
        .map_err(|e| format!("Twitter API error: {}", e))?;

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let metrics = data
        .get("data")
        .and_then(|d| d.get("public_metrics"))
        .cloned()
        .unwrap_or_default();

    Ok(PostEngagement {
        likes: metrics
            .get("like_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        comments: metrics
            .get("reply_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        shares: metrics
            .get("retweet_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        views: metrics
            .get("impression_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_to_dynamic() {
        let engagement = PostEngagement {
            likes: 100,
            comments: 20,
            shares: 5,
            views: 1000,
            clicks: 50,
            reach: 500,
        };

        let dynamic = engagement.to_dynamic();
        assert!(dynamic.is_map());
    }
}
