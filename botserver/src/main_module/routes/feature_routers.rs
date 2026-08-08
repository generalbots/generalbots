use axum::Router;
use std::sync::Arc;
use botcore::shared::state::AppState;

/// Resolves the real workspace branch for suite-mode callers that carry no
/// JWT (issue #734). Returns the branch flagged `is_default_for_branch`, or
/// `Uuid::nil()` (the fallback global scope) when no bot is flagged yet.
/// This replaces the previous hardcoded `Uuid::nil()` stub so suite-mode
/// handlers stop defaulting to the global shared scope.
fn resolve_default_branch(conn: &mut diesel::PgConnection) -> uuid::Uuid {
    use diesel::prelude::*;
    if let Some(branch) = diesel::sql_query(
        "SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE LIMIT 1",
    )
    .get_result::<DefaultBranchRow>(conn)
    .optional()
    .ok()
    .flatten()
    {
        return branch.branch_id;
    }
    uuid::Uuid::nil()
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DefaultBranchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: uuid::Uuid,
}

fn default_branch_fn(conn: &mut diesel::PgConnection) -> uuid::Uuid {
    resolve_default_branch(conn)
}

/// Resolves a working Zitadel management token for the SaaS cloud API.
///
/// The configured `service_token` may be stale or invalid (it is a leftover
/// from an older setup and is never renewed). Fall back to the long-lived
/// bootstrap admin PAT (`conf/directory/admin-pat.txt`) and validate the
/// chosen token against the management API so signup user creation and login
/// password checks never run with a dead token.
fn resolve_directory_service_token(api_url: &str, configured: Option<String>) -> Option<String> {
    let mut candidates: Vec<String> = Vec::new();
    if let Some(token) = configured {
        if !token.is_empty() {
            candidates.push(token);
        }
    }
    let pat_path = format!(
        "{}/conf/directory/admin-pat.txt",
        botcore::shared::utils::get_stack_path()
    );
    if let Ok(pat) = std::fs::read_to_string(&pat_path) {
        let pat = pat.trim().to_string();
        if !pat.is_empty() {
            candidates.push(pat);
        }
    }
    if api_url.is_empty() || candidates.is_empty() {
        return configured;
    }
    for token in &candidates {
        let valid = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()
            .and_then(|client| {
                client
                    .post(format!("{api_url}/management/v1/users/_search"))
                    .header("Authorization", format!("Bearer {token}"))
                    .json(&serde_json::json!({}))
                    .send()
                    .ok()
            })
            .map(|response| response.status().is_success())
            .unwrap_or(false);
        if valid {
            if configured.as_deref() != Some(token.as_str()) {
                tracing::info!("Using admin PAT as directory service token (configured token was invalid)");
            }
            return Some(token.clone());
        }
        tracing::warn!("Directory service token rejected by Zitadel, trying next candidate");
    }
    tracing::error!("No valid directory service token found - SaaS signup/login will fail");
    configured
}

fn resolve_default_branch_pool(
    _pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> (uuid::Uuid, uuid::Uuid) {
    let mut conn = match _pool.get() {
        Ok(c) => c,
        Err(_) => return (uuid::Uuid::nil(), uuid::Uuid::nil()),
    };
    let branch = resolve_default_branch(&mut conn);
    (branch, branch)
}

#[cfg(feature = "paper")]
pub(super) fn make_paper_state(app_state: &Arc<AppState>) -> Arc<crate::paper::state::PaperState> {
    let drive = app_state.drive.clone();
    let llm_provider = app_state.llm_provider.clone();

    let s3_put = {
        let drive = drive.clone();
        Arc::new(move |bucket: &str, key: &str, data: Vec<u8>, content_type: Option<&str>| {
            let drive = drive.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            let ct = content_type.map(|s| s.to_string());
            Box::pin(async move {
                match drive.as_ref() {
                    Some(d) => d.put_object(&bucket, &key, data, ct.as_deref()).await,
                    None => Err("Drive service not available".to_string()),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        }) as crate::paper::state::S3PutFn
    };

    let s3_get = {
        let drive = drive.clone();
        Arc::new(move |bucket: &str, key: &str| {
            let drive = drive.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            Box::pin(async move {
                match drive.as_ref() {
                    Some(d) => d.get_object(&bucket, &key).await,
                    None => Err("Drive service not available".to_string()),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
        }) as crate::paper::state::S3GetFn
    };

    let s3_delete = {
        let drive = drive.clone();
        Arc::new(move |bucket: &str, key: &str| {
            let drive = drive.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            Box::pin(async move {
                match drive.as_ref() {
                    Some(d) => d.delete_object(&bucket, &key).await,
                    None => Err("Drive service not available".to_string()),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        }) as crate::paper::state::S3DeleteFn
    };

    let s3_list = {
        let drive = drive.clone();
        Arc::new(move |bucket: &str, prefix: &str| {
            let drive = drive.clone();
            let bucket = bucket.to_string();
            let prefix = prefix.to_string();
            Box::pin(async move {
                match drive.as_ref() {
                    Some(d) => d.list_objects(&bucket, Some(&prefix)).await,
                    None => Err("Drive service not available".to_string()),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>>
        }) as crate::paper::state::S3ListFn
    };

    let call_llm = {
        let llm = llm_provider.clone();
        Arc::new(move |prompt: &str, context: &str| {
            let llm = llm.clone();
            let prompt = prompt.to_string();
            let context = context.to_string();
            Box::pin(async move {
                match llm.as_ref() {
                    Some(l) => l.generate_with_context(&prompt, &context).await,
                    None => Err("LLM service not available".to_string()),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
        }) as crate::paper::state::CallLlmFn
    };

    Arc::new(crate::paper::state::PaperState {
        conn: app_state.conn.clone(),
        bucket_name: app_state.bucket_name.clone(),
        s3_put,
        s3_get,
        s3_delete,
        s3_list,
        call_llm,
    })
}

#[cfg(feature = "designer")]
pub(super) fn make_designer_router(app_state: &Arc<AppState>) -> Router<()> {
    let make_state = || Arc::new(botdesigner::DesignerState {
        conn: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn: &mut diesel::PgConnection| {
            let branch = resolve_default_branch(conn);
            (branch, "default".to_string())
        }),
        get_designer_error_context: Arc::new(|_err: &str| -> Option<String> { None }),
        get_content_type: Arc::new(|_p: &str| -> &'static str { "text/html" }),
        get_stack_path: Arc::new(|| "/opt/gbo/stack".to_string()),
        load_from_drive: Arc::new(|_: &str, _: &str| -> Result<String, String> { Err("not available".to_string()) }),
        write_to_drive: Arc::new(|_: &str, _: &str, _: &[u8], _: &str| -> Result<(), String> { Err("not available".to_string()) }),
        call_llm: Arc::new(|_: &str, _: &serde_json::Value| -> Result<String, String> { Ok(String::new()) }),
        get_config: Arc::new(|_: &str, _: &str, _: Option<&str>| -> Result<String, String> { Ok(String::new()) }),
        bucket_name: app_state.bucket_name.clone(),
        site_path: None,
    });
    let state = make_state();
    Router::new()
        .merge(crate::designer::designer_api::configure_designer_routes().with_state(state.clone()))
        .merge(crate::designer::ui::configure_designer_ui_routes().with_state(state.clone()))
        .merge(crate::designer::plugin_manifest::configure_plugin_routes().with_state(state))
}

#[cfg(feature = "dashboards")]
pub(super) fn make_dashboards_router(app_state: &Arc<AppState>) -> Router<()> {
    fn default_bot_fn(conn: &mut diesel::PgConnection) -> uuid::Uuid {
        resolve_default_branch(conn)
    }
    Router::new()
        .merge(crate::dashboards::configure_dashboards_routes(Arc::new(botdashboards::DashboardsState {
            pool: app_state.conn.clone(),
            get_default_bot: default_bot_fn,
        })))
        .merge(crate::dashboards::ui::configure_dashboards_ui_routes().with_state(Arc::new(botdashboards::DashboardsState {
            pool: app_state.conn.clone(),
            get_default_bot: default_bot_fn,
        })))
}

#[cfg(feature = "canvas")]
pub(super) fn make_canvas_router(app_state: &Arc<AppState>) -> Router<()> {
    let make_state = || Arc::new(botcanvas::CanvasState {
        pool: Arc::new(app_state.conn.clone()),
        get_bot_context: Arc::new(
            resolve_default_branch_pool as fn(&diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>) -> (uuid::Uuid, uuid::Uuid),
        ),
    });
    let state = make_state();
    Router::new()
        .merge(crate::canvas::configure_canvas_routes().with_state(state.clone()))
        .merge(crate::canvas::configure_canvas_ui_routes().with_state(state))
}

#[cfg(feature = "social")]
pub(super) fn make_social_router(app_state: &Arc<AppState>) -> Router<()> {
    let make_state = || Arc::new(botsocial::SocialState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn: &mut _| {
            let branch = resolve_default_branch(conn);
            (branch, "default".to_string())
        }),
    });
    let state = make_state();
    Router::new()
        .merge(crate::social::configure_social_routes().with_state(state.clone()))
        .merge(crate::social::configure_social_ui_routes().with_state(state))
}

pub(super) fn make_billing_router(app_state: &Arc<AppState>) -> Router<()> {
    let make_state = || Arc::new(botbilling::api::BillingApiState {
        pool: Arc::new(app_state.conn.clone()),
        get_default_bot: Some(
            (|conn: &mut diesel::PgConnection| {
                use diesel::prelude::*;
                #[derive(diesel::QueryableByName)]
                #[diesel(check_for_backend(diesel::pg::Pg))]
                struct BranchRow {
                    #[diesel(sql_type = diesel::sql_types::Uuid)]
                    branch_id: uuid::Uuid,
                }
                diesel::sql_query(
                    "SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE ORDER BY created_at ASC LIMIT 1",
                )
                .get_result::<BranchRow>(conn)
                .map(|r| r.branch_id)
                .unwrap_or_else(|_| uuid::Uuid::nil())
            }) as fn(&mut diesel::PgConnection) -> uuid::Uuid,
        ),
    });
    Router::new()
        .merge(crate::billing::billing_ui::configure_billing_routes().with_state(make_state()))
        .merge(crate::billing::billing_admin::configure_admin_billing_routes().with_state(make_state()))
        .merge(crate::billing::erp_ui::configure_erp_ui_routes().with_state(make_state()))
        .merge(crate::billing::api::configure_billing_api_routes().with_state(make_state()))
}

pub(super) fn make_saas_router(app_state: &Arc<AppState>) -> Router<()> {
    use botcloud::{SaasService, SaasConfig, stripe::StripeClient, cloud_ui, api};
    let stripe_secret = match std::env::var("STRIPE_SECRET_KEY") {
        Ok(key) => key,
        Err(_) => {
            tracing::info!("STRIPE_SECRET_KEY not set — Stripe operations will fail at runtime");
            String::new()
        }
    };
    let stripe = StripeClient::new(stripe_secret, None);

    // Configure mc alias from AppState drive config (loaded from Vault)
    let mc_path = std::env::var("MC_PATH").unwrap_or_else(|_| "/tmp/mc".to_string());
    let mc_alias = std::env::var("MC_ALIAS").unwrap_or_else(|_| "local".to_string());
    if let Some(ref cfg) = app_state.config {
        let endpoint = &cfg.drive.endpoint;
        let access_key = &cfg.drive.access_key;
        let secret_key = &cfg.drive.secret_key;
        if !access_key.is_empty() && !secret_key.is_empty() {
            std::process::Command::new(&mc_path)
                .args(["alias", "set", &mc_alias, endpoint, access_key, secret_key, "--api", "s3v4"])
                .output()
                .ok();
        } else {
            tracing::info!("Drive credentials from Vault are empty, mc alias not configured");
        }
    }

    // Load all SaaS config from directory_config.json (written during init from Vault)
    let (base_url, jwt_secret, templates_dir, mc_path, mc_alias,
         directory_api_url, directory_service_token, directory_external_domain) = {
        let config_path = format!("{}/conf/system/directory_config.json", botcore::shared::utils::get_stack_path());
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        let j = |key: &str| json.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
                        let configured_token = j("service_token");
                        let api_url = j("base_url");
                        let service_token = resolve_directory_service_token(
                            api_url.as_deref().unwrap_or_default(),
                            configured_token,
                        );
                        if service_token != configured_token {
                            // Persist the working token so future boots reuse it.
                            if let Some(new_token) = &service_token {
                                if let Ok(mut json) = content.clone().parse::<serde_json::Value>() {
                                    json["service_token"] = serde_json::Value::String(new_token.clone());
                                    let _ = std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap_or(content.clone()));
                                }
                            }
                        }
                        (
                            j("saas_base_url").unwrap_or_default(),
                            j("saas_jwt_secret").unwrap_or_else(crate::main_module::directory_setup::resolve_saas_jwt_secret),
                            j("bot_templates_dir").unwrap_or_else(|| "work/templates/bots".to_string()),
                            j("mc_path").unwrap_or_else(|| "/tmp/mc".to_string()),
                            j("mc_alias").unwrap_or_else(|| "local".to_string()),
                            api_url,
                            service_token,
                            j("external_domain"),
                        )
                    }
                    Err(_) => (String::new(), crate::main_module::directory_setup::resolve_saas_jwt_secret(), "work/templates/bots".to_string(),
                              "/tmp/mc".to_string(), "local".to_string(), None, None, None),
                }
            }
            Err(_) => (String::new(), crate::main_module::directory_setup::resolve_saas_jwt_secret(), "work/templates/bots".to_string(),
                      "/tmp/mc".to_string(), "local".to_string(), None, None, None),
        }
    };
    let saas_config = SaasConfig {
        base_url,
        jwt_secret,
        mc_path,
        mc_alias,
        templates_dir,
        directory_api_url,
        directory_service_token,
        directory_external_domain,
    };
    let saas_config_for_api = saas_config.clone();
    let saas_service = Arc::new(SaasService::new(
        Arc::new(botbilling::api::BillingApiState {
            pool: Arc::new(app_state.conn.clone()),
            get_default_bot: Some(botbilling::query_first_bot as fn(&mut diesel::PgConnection) -> uuid::Uuid),
        }),
        stripe,
        saas_config,
    ));
    Router::new()
        .merge(cloud_ui::configure_cloud_ui_routes().with_state(saas_service.clone()))
        .merge(api::configure_cloud_api_routes(saas_config_for_api).with_state(saas_service.clone()))
        .merge(botcloud::webhook::configure_webhook_routes().with_state(saas_service))
}

#[cfg(feature = "marketing")]
pub(super) fn make_marketing_router(app_state: &Arc<AppState>) -> Router<()> {
    let base = botmarketing::state::AppState {
        conn: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn: &mut diesel::PgConnection| {
            let branch = resolve_default_branch(conn);
            (branch, "default".to_string())
        }),
        send_email: Arc::new(|_: &str, _: &str, _: &str, _: uuid::Uuid, _: Option<&str>| -> Result<String, String> { Ok("stub".to_string()) }),
        send_whatsapp: Arc::new(|_: uuid::Uuid, _: &str, _: &str, _: Option<&str>, _: Option<&str>| -> Result<String, String> { Ok("stub".to_string()) }),
        get_config: Arc::new(|_: &uuid::Uuid, _: &str, _: Option<&str>| -> Result<String, String> { Ok("stub".to_string()) }),
        llm_generate: Arc::new(|_: &str, _: &serde_json::Value, _: &str, _: &str| -> Result<String, String> { Ok("stub".to_string()) }),
        worker: None,
    };
    let worker_state = Arc::new(base.clone());
    let marketing_state = Arc::new(
        base.with_worker(botmarketing::campaign::CampaignWorker::new(worker_state)),
    );
    crate::marketing::routes::configure_marketing_routes().with_state(marketing_state)
}

#[cfg(feature = "telegram")]
pub(super) fn make_telegram_router(app_state: &Arc<AppState>) -> Router<()> {
    crate::telegram::webhook::configure().with_state(Arc::new(bottelegram::ChannelState {
        conn: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn: &mut diesel::PgConnection| {
            let branch = resolve_default_branch(conn);
            (branch, "default".to_string())
        }),
        get_config: Arc::new(|_: &uuid::Uuid, _: &str, _: Option<&str>| -> Result<String, String> { Ok("stub".to_string()) }),
        stream_response: {
            let app_state = app_state.clone();
            Arc::new(move |msg: botlib::models::UserMessage, tx: tokio::sync::mpsc::Sender<botlib::models::BotResponse>| {
                let state = app_state.clone();
                tokio::spawn(async move {
                    let sink = crate::core::bot::pipeline::MpscChannelSink(tx);
                    let _ = crate::core::bot::pipeline::run_pipeline_for_channel(
                        &state, &msg, &sink,
                    ).await.map_err(|e| e.to_string())?;
                    Ok(())
                })
            })
        },
        attendant_broadcast: None,
    }))
}

#[cfg(feature = "instagram")]
pub(super) fn make_instagram_router(app_state: &Arc<AppState>) -> Router<()> {
    crate::instagram::webhook::configure().with_state(Arc::new(botinstagram::state::ChannelState {
        get_config: Arc::new(|_: &str, _: &str, _: Option<&str>| -> Result<String, String> { Ok("stub".to_string()) }),
        stream_response: {
            let app_state = app_state.clone();
            Arc::new(move |msg: botlib::models::UserMessage, tx: tokio::sync::mpsc::Sender<botlib::models::BotResponse>| {
                let state = app_state.clone();
                tokio::spawn(async move {
                    let sink = crate::core::bot::pipeline::MpscChannelSink(tx);
                    let _ = crate::core::bot::pipeline::run_pipeline_for_channel(
                        &state, &msg, &sink,
                    ).await.map_err(|e| e.to_string())?;
                    Ok(())
                })
            })
        },
        attendant_broadcast: None,
    }))
}

#[cfg(feature = "msteams")]
pub(super) fn make_msteams_router(app_state: &Arc<AppState>) -> Router<()> {
    crate::msteams::webhook::configure().with_state(Arc::new(botmsteams::state::ChannelState {
        conn: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn: &mut diesel::PgConnection| {
            let branch = resolve_default_branch(conn);
            (branch, "default".to_string())
        }),
        get_config: Arc::new(|_: &uuid::Uuid, _: &str, _: Option<&str>| -> Result<String, String> { Ok("stub".to_string()) }),
        stream_response: {
            let app_state = app_state.clone();
            Arc::new(move |msg: botlib::models::UserMessage, tx: tokio::sync::mpsc::Sender<botlib::models::BotResponse>| {
                let state = app_state.clone();
                tokio::spawn(async move {
                    let sink = crate::core::bot::pipeline::MpscChannelSink(tx);
                    let _ = crate::core::bot::pipeline::run_pipeline_for_channel(
                        &state, &msg, &sink,
                    ).await.map_err(|e| e.to_string())?;
                    Ok(())
                })
            })
        },
        attendant_broadcast: None,
    }))
}

#[cfg(feature = "attendant")]
pub(super) fn make_attendant_router(app_state: &Arc<AppState>) -> Router<()> {
    Router::new()
        .merge(crate::attendance::routes::configure_attendance_routes().with_state(Arc::new(botattendance::AttendanceConfig {
            pool: Arc::new(app_state.conn.clone()),
            master_key: botsecurity_crypto::encryption::load_master_encryption_key(),
            get_default_bot: Arc::new(|conn: &mut _| resolve_default_branch(conn)),
            llm_generate: Arc::new(|_: &str, _: &serde_json::Value, _: &str, _: &str| -> Result<String, Box<dyn std::error::Error + Send + Sync>> { Ok(String::new()) }),
            process_content: Arc::new(|_: &str, _: &str| -> String { String::new() }),
            config_get: Arc::new(|_: &uuid::Uuid, _: &str| -> String { String::new() }),
            send_bot_response: None,
            broadcast_notification: None,
            save_message: None,
        })))
}
