use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

/// Parse refresh interval string (e.g., "1d", "1w", "1m", "1y") into days
/// Returns the number of days for the refresh interval
fn parse_refresh_interval(interval: &str) -> Result<i32, String> {
    let interval_lower = interval.trim().to_lowercase();

    // Match patterns like "1d", "7d", "2w", "1m", "1y", etc.
    if interval_lower.ends_with('d') {
        let days: i32 = interval_lower[..interval_lower.len()-1]
            .parse()
            .map_err(|_| format!("Invalid days format: {}", interval))?;
        Ok(days)
    } else if interval_lower.ends_with('w') {
        let weeks: i32 = interval_lower[..interval_lower.len()-1]
            .parse()
            .map_err(|_| format!("Invalid weeks format: {}", interval))?;
        Ok(weeks * 7)
    } else if interval_lower.ends_with('m') {
        let months: i32 = interval_lower[..interval_lower.len()-1]
            .parse()
            .map_err(|_| format!("Invalid months format: {}", interval))?;
        Ok(months * 30) // Approximate month as 30 days
    } else if interval_lower.ends_with('y') {
        let years: i32 = interval_lower[..interval_lower.len()-1]
            .parse()
            .map_err(|_| format!("Invalid years format: {}", interval))?;
        Ok(years * 365) // Approximate year as 365 days
    } else {
        // Try to parse as plain number (assume days)
        interval.parse()
            .map_err(|_| format!("Invalid refresh interval format: {}. Use format like '1d', '1w', '1m', '1y'", interval))
    }
}

/// Convert days to expires_policy string format
fn days_to_expires_policy(days: i32) -> String {
    format!("{}d", days)
}

pub fn use_website_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    // Register syntax for USE WEBSITE "url" REFRESH "interval" (case insensitive)
    // Register both uppercase and lowercase variants
    engine
        .register_custom_syntax(
            ["USE", "WEBSITE", "$expr$", "REFRESH", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?;
                let url_str = url.to_string().trim_matches('"').to_string();

                let refresh = context.eval_expression_tree(&inputs[1])?;
                let refresh_str = refresh.to_string().trim_matches('"').to_string();

                trace!(
                    "USE WEBSITE command executed: {} REFRESH {} for session: {}",
                    url_str,
                    refresh_str,
                    user_clone.id
                );

                let is_valid = url_str.starts_with("http://") || url_str.starts_with("https://");
                if !is_valid {
                    return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Invalid URL format. Must start with http:// or https://".into(),
                        rhai::Position::NONE,
                    )));
                }

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let url_for_task = url_str;
                let refresh_for_task = refresh_str;
                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(_rt) = rt {
                        let result = associate_website_with_session_refresh(
                            &state_for_task,
                            &user_for_task,
                            &url_for_task,
                            &refresh_for_task,
                        );
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
        .expect("valid syntax registration");

    // Register syntax for USE WEBSITE "url" (without REFRESH)
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            ["USE", "WEBSITE", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?;
                let url_str = url.to_string().trim_matches('"').to_string();

                trace!(
                    "USE WEBSITE command executed: {} for session: {}",
                    url_str,
                    user_clone2.id
                );

                let is_valid = url_str.starts_with("http://") || url_str.starts_with("https://");
                if !is_valid {
                    return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Invalid URL format. Must start with http:// or https://".into(),
                        rhai::Position::NONE,
                    )));
                }

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();
                let url_for_task = url_str;
                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(_rt) = rt {
                        let result = associate_website_with_session(
                            &state_for_task,
                            &user_for_task,
                            &url_for_task,
                        );
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
        .expect("valid syntax registration");
}

/// Register USE_WEBSITE as a regular function instead of custom syntax
/// This avoids conflicts with other USE keywords (USE MODEL, USE KB, etc.)
pub fn register_use_website_function(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    // Register USE_WEBSITE(url, refresh) with both parameters (uppercase)
    engine.register_fn(
        "USE_WEBSITE",
        move |url: &str, refresh: &str| -> Dynamic {
            trace!(
                "USE_WEBSITE function called: {} REFRESH {} for session: {}",
                url,
                refresh,
                user_clone.id
            );

            let is_valid = url.starts_with("http://") || url.starts_with("https://");
            if !is_valid {
                return Dynamic::from(format!(
                    "ERROR: Invalid URL format: {}. Must start with http:// or https://",
                    url
                ));
            }

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();
            let url_for_task = url.to_string();
            let refresh_for_task = refresh.to_string();
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let _rt = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(_rt) => _rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to build tokio runtime: {}", e)));
                        return;
                    }
                };
                let result = associate_website_with_session_refresh(
                    &state_for_task,
                    &user_for_task,
                    &url_for_task,
                    &refresh_for_task,
                );
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(Ok(message)) => Dynamic::from(message),
                Ok(Err(e)) => Dynamic::from(format!("ERROR: {}", e)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Dynamic::from("ERROR: USE_WEBSITE timed out")
                }
                Err(e) => Dynamic::from(format!("ERROR: USE_WEBSITE failed: {}", e)),
            }
        },
    );

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    // Register use_website(url, refresh) with both parameters (lowercase for preprocessor)
    engine.register_fn(
        "use_website",
        move |url: &str, refresh: &str| -> Dynamic {
            trace!(
                "use_website function called: {} REFRESH {} for session: {}",
                url,
                refresh,
                user_clone2.id
            );

            let is_valid = url.starts_with("http://") || url.starts_with("https://");
            if !is_valid {
                return Dynamic::from(format!(
                    "ERROR: Invalid URL format: {}. Must start with http:// or https://",
                    url
                ));
            }

            let state_for_task = Arc::clone(&state_clone2);
            let user_for_task = user_clone2.clone();
            let url_for_task = url.to_string();
            let refresh_for_task = refresh.to_string();
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let _rt = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(_rt) => _rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to build tokio runtime: {}", e)));
                        return;
                    }
                };
                let result = associate_website_with_session_refresh(
                    &state_for_task,
                    &user_for_task,
                    &url_for_task,
                    &refresh_for_task,
                );
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(Ok(message)) => Dynamic::from(message),
                Ok(Err(e)) => Dynamic::from(format!("ERROR: {}", e)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Dynamic::from("ERROR: use_website timed out")
                }
                Err(e) => Dynamic::from(format!("ERROR: use_website failed: {}", e)),
            }
        },
    );

    let state_clone3 = Arc::clone(&state);
    let user_clone3 = user.clone();

    // Register USE_WEBSITE(url) with just URL (default refresh)
    engine.register_fn("USE_WEBSITE", move |url: &str| -> Dynamic {
        trace!(
            "USE_WEBSITE function called: {} for session: {}",
            url,
            user_clone3.id
        );

        let is_valid = url.starts_with("http://") || url.starts_with("https://");
        if !is_valid {
            return Dynamic::from(format!(
                "ERROR: Invalid URL format: {}. Must start with http:// or https://",
                url
            ));
        }

        let state_for_task = Arc::clone(&state_clone3);
        let user_for_task = user_clone3.clone();
        let url_for_task = url.to_string();
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let _rt = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(_rt) => _rt,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to build tokio runtime: {}", e)));
                    return;
                }
            };
            let result = associate_website_with_session(
                &state_for_task,
                &user_for_task,
                &url_for_task,
            );
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(message)) => Dynamic::from(message),
            Ok(Err(e)) => Dynamic::from(format!("ERROR: {}", e)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Dynamic::from("ERROR: USE_WEBSITE timed out")
            }
            Err(e) => Dynamic::from(format!("ERROR: USE_WEBSITE failed: {}", e)),
        }
    });

    let state_clone4 = Arc::clone(&state);
    let user_clone4 = user;

    // Register use_website(url) with just URL (default refresh, lowercase)
    engine.register_fn("use_website", move |url: &str| -> Dynamic {
        trace!(
            "use_website function called: {} for session: {}",
            url,
            user_clone4.id
        );

        let is_valid = url.starts_with("http://") || url.starts_with("https://");
        if !is_valid {
            return Dynamic::from(format!(
                "ERROR: Invalid URL format: {}. Must start with http:// or https://",
                url
            ));
        }

        let state_for_task = Arc::clone(&state_clone4);
        let user_for_task = user_clone4.clone();
        let url_for_task = url.to_string();
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let _rt = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(_rt) => _rt,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to build tokio runtime: {}", e)));
                    return;
                }
            };
            let result = associate_website_with_session(
                &state_for_task,
                &user_for_task,
                &url_for_task,
            );
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(message)) => Dynamic::from(message),
            Ok(Err(e)) => Dynamic::from(format!("ERROR: {}", e)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Dynamic::from("ERROR: use_website timed out")
            }
            Err(e) => Dynamic::from(format!("ERROR: use_website failed: {}", e)),
        }
    });

    info!("Registered USE_WEBSITE and use_website as function (preprocessed from USE WEBSITE)");
}

fn associate_website_with_session(
    state: &AppState,
    user: &UserSession,
    url: &str,
) -> Result<String, String> {
    associate_website_with_session_refresh(state, user, url, "1m") // Default: 1 month
}

fn associate_website_with_session_refresh(
    state: &AppState,
    user: &UserSession,
    url: &str,
    refresh_interval: &str,
) -> Result<String, String> {
    info!("Associating website {} with session {} (refresh: {})", url, user.id, refresh_interval);

    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    // Get bot name for collection naming
    #[derive(QueryableByName)]
    struct BotName {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    
    let bot_name_result: BotName = diesel::sql_query("SELECT name FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(&user.bot_id)
        .get_result(&mut conn)
        .map_err(|e| format!("Failed to get bot name: {}", e))?;

    let collection_name = format!("{}_website_{}", bot_name_result.name, sanitize_url_for_collection(url));

    let website_status = check_website_crawl_status(&mut conn, &user.bot_id, url)?;

    match website_status {
        WebsiteCrawlStatus::NotRegistered => {
            // Auto-register website for crawling instead of failing
            info!("Website {} not registered, auto-registering for crawling with refresh: {}", url, refresh_interval);
            register_website_for_crawling_with_refresh(&mut conn, &user.bot_id, url, refresh_interval)
                .map_err(|e| format!("Failed to register website: {}", e))?;

            return Ok(format!(
                "Website {} has been registered for crawling (refresh: {}). It will be available once crawling completes.",
                url, refresh_interval
            ));
        }
        WebsiteCrawlStatus::Pending => {
            info!("Website {} is pending crawl, associating anyway", url);
            // Update refresh policy if needed
            update_refresh_policy_if_shorter(&mut conn, &user.bot_id, url, refresh_interval)?;
        }
        WebsiteCrawlStatus::Crawled => {
            info!("Website {} is already crawled and ready", url);
            // Update refresh policy if needed
            update_refresh_policy_if_shorter(&mut conn, &user.bot_id, url, refresh_interval)?;
        }
        WebsiteCrawlStatus::Failed => {
            return Err(format!(
                "Website {} crawling failed. Please check the logs.",
                url
            ));
        }
    }

    add_website_to_session(&mut conn, &user.id, &user.bot_id, url, &collection_name)?;

    Ok(format!(
        "Website {} is now available in this conversation.",
        url
    ))
}

enum WebsiteCrawlStatus {
    NotRegistered,
    Pending,
    Crawled,
    Failed,
}

fn check_website_crawl_status(
    conn: &mut PgConnection,
    bot_id: &Uuid,
    url: &str,
) -> Result<WebsiteCrawlStatus, String> {
    #[derive(QueryableByName)]
    struct CrawlStatus {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::SmallInt>)]
        crawl_status: Option<i16>,
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

pub fn register_website_for_crawling(
    conn: &mut PgConnection,
    bot_id: &Uuid,
    url: &str,
) -> Result<(), String> {
    register_website_for_crawling_with_refresh(conn, bot_id, url, "1m") // Default: 1 month
}

pub fn register_website_for_crawling_with_refresh(
    conn: &mut PgConnection,
    bot_id: &Uuid,
    url: &str,
    refresh_interval: &str,
) -> Result<(), String> {
    let days = parse_refresh_interval(refresh_interval)
        .map_err(|e| format!("Invalid refresh interval: {}", e))?;

    let expires_policy = days_to_expires_policy(days);

    let query = diesel::sql_query(
        "INSERT INTO website_crawls (id, bot_id, url, expires_policy, crawl_status, next_crawl, refresh_policy)
         VALUES (gen_random_uuid(), $1, $2, $3, 0, NOW(), $4)
         ON CONFLICT (bot_id, url) DO UPDATE SET
            next_crawl = CASE
                WHEN website_crawls.crawl_status = 2 THEN NOW()  -- Failed, retry now
                ELSE website_crawls.next_crawl  -- Keep existing schedule
            END,
            refresh_policy = CASE
                WHEN website_crawls.refresh_policy IS NULL THEN $4
                ELSE LEAST(website_crawls.refresh_policy, $4)  -- Use shorter interval
            END",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(url)
    .bind::<diesel::sql_types::Text, _>(expires_policy)
    .bind::<diesel::sql_types::Text, _>(refresh_interval);

    query
        .execute(conn)
        .map_err(|e| format!("Failed to register website for crawling: {}", e))?;

    info!("Website {} registered for crawling for bot {} with refresh policy: {}", url, bot_id, refresh_interval);
    Ok(())
}

/// Update refresh policy if the new interval is shorter than the existing one
fn update_refresh_policy_if_shorter(
    conn: &mut PgConnection,
    bot_id: &Uuid,
    url: &str,
    refresh_interval: &str,
) -> Result<(), String> {
    // Get current record to compare in Rust (no SQL business logic!)
    #[derive(QueryableByName)]
    struct CurrentRefresh {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        refresh_policy: Option<String>,
    }

    let current = diesel::sql_query(
        "SELECT refresh_policy FROM website_crawls WHERE bot_id = $1 AND url = $2"
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(url)
    .get_result::<CurrentRefresh>(conn)
    .ok();

    let new_days = parse_refresh_interval(refresh_interval)
        .map_err(|e| format!("Invalid refresh interval: {}", e))?;

    // Check if we should update (no policy exists or new interval is shorter)
    let should_update = match &current {
        Some(c) if c.refresh_policy.is_some() => {
            let existing_days = if let Some(ref policy) = c.refresh_policy {
                parse_refresh_interval(policy).unwrap_or(i32::MAX)
            } else {
                i32::MAX
            };
            new_days < existing_days
        }
        _ => true, // No existing policy, so update
    };

    if should_update {
        let expires_policy = days_to_expires_policy(new_days);

        diesel::sql_query(
            "UPDATE website_crawls SET refresh_policy = $3, expires_policy = $4
             WHERE bot_id = $1 AND url = $2"
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(url)
        .bind::<diesel::sql_types::Text, _>(refresh_interval)
        .bind::<diesel::sql_types::Text, _>(expires_policy)
        .execute(conn)
        .map_err(|e| format!("Failed to update refresh policy: {}", e))?;
    }

    Ok(())
}

pub fn execute_use_website_preprocessing(
    conn: &mut PgConnection,
    url: &str,
    bot_id: Uuid,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    execute_use_website_preprocessing_with_refresh(conn, url, bot_id, "1m") // Default: 1 month
}

pub fn execute_use_website_preprocessing_with_refresh(
    conn: &mut PgConnection,
    url: &str,
    bot_id: Uuid,
    refresh_interval: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    trace!("Preprocessing USE_WEBSITE: {}, bot_id: {:?}, refresh: {}", url, bot_id, refresh_interval);

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!(
            "Invalid URL format: {}. Must start with http:// or https://",
            url
        )
        .into());
    }

    register_website_for_crawling_with_refresh(conn, &bot_id, url, refresh_interval)?;

    Ok(serde_json::json!({
        "command": "use_website",
        "url": url,
        "bot_id": bot_id.to_string(),
        "refresh_policy": refresh_interval,
        "status": "registered_for_crawling"
    }))
}

fn add_website_to_session(
    conn: &mut PgConnection,
    session_id: &Uuid,
    bot_id: &Uuid,
    url: &str,
    collection_name: &str,
) -> Result<(), String> {
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

pub fn clear_websites_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(["CLEAR", "WEBSITES"], true, move |_context, _inputs| {
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
        .expect("valid syntax registration");
}

fn clear_all_websites(
    conn_pool: crate::core::shared::utils::DbPool,
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

pub fn get_active_websites_for_session(
    conn_pool: &crate::core::shared::utils::DbPool,
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

fn sanitize_url_for_collection(url: &str) -> String {
    url.replace("http://", "")
        .replace("https://", "")
        .replace(['/', ':', '.'], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_sanitization() {
        assert_eq!(
            sanitize_url_for_collection("https://docs.example.com/path"),
            "docs_example_com_path"
        );
        assert_eq!(
            sanitize_url_for_collection("http://test.site:9000"),
            "test_site_8080"
        );
    }
}
