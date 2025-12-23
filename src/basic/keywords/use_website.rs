use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

/// Register USE_WEBSITE keyword in BASIC
/// Runtime mode: Associates a website collection with the current session (like USE KB)
/// Preprocessing mode: Registers website for crawling (handled in compiler/mod.rs)
pub fn use_website_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["USE", "WEBSITE", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?;
                let url_str = url.to_string().trim_matches('"').to_string();

                trace!(
                    "USE WEBSITE command executed: {} for session: {}",
                    url_str,
                    user_clone.id
                );

                // Validate URL
                let is_valid = url_str.starts_with("http://") || url_str.starts_with("https://");
                if !is_valid {
                    return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Invalid URL format. Must start with http:// or https://".into(),
                        rhai::Position::NONE,
                    )));
                }

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let url_for_task = url_str.clone();
                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            associate_website_with_session(
                                &state_for_task,
                                &user_for_task,
                                &url_for_task,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(message)) => Ok(Dynamic::from(message)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "USE WEBSITE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("USE WEBSITE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// Associate website with session (runtime behavior - like USE KB)
/// This only associates an already-crawled website with the session
async fn associate_website_with_session(
    state: &AppState,
    user: &UserSession,
    url: &str,
) -> Result<String, String> {
    info!("Associating website {} with session {}", url, user.id);

    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    // Create collection name for this website
    let collection_name = format!("website_{}", sanitize_url_for_collection(url));

    // Check if website has been crawled for this bot
    let website_status = check_website_crawl_status(&mut conn, &user.bot_id, url)?;

    match website_status {
        WebsiteCrawlStatus::NotRegistered => {
            return Err(format!(
                "Website {} has not been registered for crawling. It should be added to the script for preprocessing.",
                url
            ));
        }
        WebsiteCrawlStatus::Pending => {
            // Website is registered but not yet crawled - allow association but warn
            info!("Website {} is pending crawl, associating anyway", url);
        }
        WebsiteCrawlStatus::Crawled => {
            // Website is fully crawled and ready
            info!("Website {} is already crawled and ready", url);
        }
        WebsiteCrawlStatus::Failed => {
            return Err(format!(
                "Website {} crawling failed. Please check the logs.",
                url
            ));
        }
    }

    // Associate website collection with session (like session_kb_associations)
    add_website_to_session(&mut conn, &user.id, &user.bot_id, url, &collection_name)?;

    Ok(format!(
        "Website {} is now available in this conversation.",
        url
    ))
}

/// Website crawl status enum
enum WebsiteCrawlStatus {
    NotRegistered,
    Pending,
    Crawled,
    Failed,
}

/// Check website crawl status for this bot
fn check_website_crawl_status(
    conn: &mut PgConnection,
    bot_id: &Uuid,
    url: &str,
) -> Result<WebsiteCrawlStatus, String> {
    #[derive(QueryableByName)]
    struct CrawlStatus {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        crawl_status: Option<i32>,
    }

    let query =
        diesel::sql_query("SELECT crawl_status FROM website_crawls WHERE bot_id = $1 AND url = $2")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(url);

    let result: Result<CrawlStatus, _> = query.get_result(conn);

    match result {
        Ok(status) => match status.crawl_status {
            Some(0) => Ok(WebsiteCrawlStatus::Pending),
            Some(1) => Ok(WebsiteCrawlStatus::Crawled),
            Some(2) => Ok(WebsiteCrawlStatus::Failed),
            _ => Ok(WebsiteCrawlStatus::NotRegistered),
        },
        Err(_) => Ok(WebsiteCrawlStatus::NotRegistered),
    }
}

/// Register website for background crawling (called from preprocessing)
/// This is called during script compilation, not runtime
pub fn register_website_for_crawling(
    conn: &mut PgConnection,
    bot_id: &Uuid,
    url: &str,
) -> Result<(), String> {
    // Get website configuration with defaults
    let expires_policy = "1d"; // Default, would read from bot config

    let query = diesel::sql_query(
        "INSERT INTO website_crawls (id, bot_id, url, expires_policy, crawl_status, next_crawl)
         VALUES (gen_random_uuid(), $1, $2, $3, 0, NOW())
         ON CONFLICT (bot_id, url) DO UPDATE SET next_crawl =
         CASE
            WHEN website_crawls.crawl_status = 2 THEN NOW()  -- Failed, retry now
            ELSE website_crawls.next_crawl  -- Keep existing schedule
         END",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(url)
    .bind::<diesel::sql_types::Text, _>(expires_policy);

    query
        .execute(conn)
        .map_err(|e| format!("Failed to register website for crawling: {}", e))?;

    info!("Website {} registered for crawling for bot {}", url, bot_id);
    Ok(())
}

/// Execute USE_WEBSITE during preprocessing (called from compiler)
/// This registers the website for crawling but doesn't associate it with any session
pub fn execute_use_website_preprocessing(
    conn: &mut PgConnection,
    url: &str,
    bot_id: Uuid,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    trace!("Preprocessing USE_WEBSITE: {}, bot_id: {:?}", url, bot_id);

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!(
            "Invalid URL format: {}. Must start with http:// or https://",
            url
        )
        .into());
    }

    // Register for crawling
    register_website_for_crawling(conn, &bot_id, url)?;

    Ok(serde_json::json!({
        "command": "use_website",
        "url": url,
        "bot_id": bot_id.to_string(),
        "status": "registered_for_crawling"
    }))
}

/// Add website to session (like USE KB)
fn add_website_to_session(
    conn: &mut PgConnection,
    session_id: &Uuid,
    bot_id: &Uuid,
    url: &str,
    collection_name: &str,
) -> Result<(), String> {
    // Add to session_website_associations table (similar to session_kb_associations)
    let assoc_id = Uuid::new_v4();

    diesel::sql_query(
        "INSERT INTO session_website_associations
         (id, session_id, bot_id, website_url, collection_name, is_active, added_at)
         VALUES ($1, $2, $3, $4, $5, true, NOW())
         ON CONFLICT (session_id, website_url)
         DO UPDATE SET is_active = true, added_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(assoc_id)
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(url)
    .bind::<diesel::sql_types::Text, _>(collection_name)
    .execute(conn)
    .map_err(|e| format!("Failed to add website to session: {}", e))?;

    info!(
        " Added website '{}' to session {} (collection: {})",
        url, session_id, collection_name
    );

    Ok(())
}

/// Clear websites from session (companion to USE_WEBSITE)
pub fn clear_websites_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["CLEAR", "WEBSITES"], true, move |_context, _inputs| {
            info!(
                "CLEAR WEBSITES keyword executed for session: {}",
                user_clone.id
            );

            let session_id = user_clone.id;
            let conn = state_clone.conn.clone();

            let result = std::thread::spawn(move || clear_all_websites(conn, session_id)).join();

            match result {
                Ok(Ok(count)) => {
                    info!(
                        "Successfully cleared {} websites from session {}",
                        count, user_clone.id
                    );
                    Ok(Dynamic::from(format!(
                        "{} website(s) removed from conversation",
                        count
                    )))
                }
                Ok(Err(e)) => {
                    error!("Failed to clear websites: {}", e);
                    Err(format!("CLEAR_WEBSITES failed: {}", e).into())
                }
                Err(e) => {
                    error!("Thread panic in CLEAR_WEBSITES: {:?}", e);
                    Err("CLEAR_WEBSITES failed: thread panic".into())
                }
            }
        })
        .unwrap();
}

/// Clear all websites from session
fn clear_all_websites(
    conn_pool: crate::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<usize, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let rows_affected = diesel::sql_query(
        "UPDATE session_website_associations
         SET is_active = false
         WHERE session_id = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to clear websites: {}", e))?;

    Ok(rows_affected)
}

/// Get active websites for a session
pub fn get_active_websites_for_session(
    conn_pool: &crate::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<Vec<(String, String)>, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    #[derive(QueryableByName, Debug)]
    struct ActiveWebsiteResult {
        #[diesel(sql_type = diesel::sql_types::Text)]
        website_url: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        collection_name: String,
    }

    let results: Vec<ActiveWebsiteResult> = diesel::sql_query(
        "SELECT website_url, collection_name
         FROM session_website_associations
         WHERE session_id = $1 AND is_active = true
         ORDER BY added_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .load(&mut conn)
    .map_err(|e| format!("Failed to get active websites: {}", e))?;

    Ok(results
        .into_iter()
        .map(|r| (r.website_url, r.collection_name))
        .collect())
}

/// Sanitize URL for use as collection name
fn sanitize_url_for_collection(url: &str) -> String {
    url.replace("http://", "")
        .replace("https://", "")
        .replace('/', "_")
        .replace(':', "_")
        .replace('.', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}
