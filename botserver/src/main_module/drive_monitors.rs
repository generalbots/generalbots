//! drive_monitors - extracted from bootstrap.rs

use botcore::shared::state::AppState;
use diesel::RunQueryDsl;
use log::{error, info, trace, warn};
use std::sync::Arc;


pub(crate) async fn start_drive_monitors(
    app_state: Arc<AppState>,
    _pool: &botcore::shared::utils::DbPool,
  ) {
    use botcore::shared::memory_monitor::register_thread;
    use botcore::shared::models::schema::bots;
    use diesel::prelude::*;

    let drive_monitor_state = app_state.clone();
    let pool_clone = _pool.clone();
    let state_for_scan = app_state.clone();
    let scan_pool = _pool.clone();

    tokio::spawn(async move {
        register_thread("drive-monitor", "drive");

        // Get LOAD_ONLY from env to filter which bots to load
        let load_only: Vec<String> = std::env::var("LOAD_ONLY")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        if !load_only.is_empty() {
            info!("LOAD_ONLY filter active: {:?}", load_only);
        }

        // Step 1: Discover bots from S3 buckets (*.gbai) and auto-create missing
    log::info!("Drive client status: {:?}", state_for_scan.drive.is_some());
    if let Some(s3_client) = &state_for_scan.drive {
        match s3_client.list_all_buckets().await {
            Ok(buckets) => {
                for bucket in buckets {
                    let name = bucket;
                    if !name.ends_with(".gbai") {
                        continue;
                    }
                    let bot_name = name.strip_suffix(".gbai").unwrap_or(&name).to_string();

                        // Filter by LOAD_ONLY if specified
                        if !load_only.is_empty() && !load_only.contains(&bot_name) {
                            trace!("Skipping bot '{}' (not in LOAD_ONLY)", bot_name);
                            continue;
                        }

                        let exists = {
                            let pool_check = pool_clone.clone();
                            let bn = bot_name.to_string();
                            tokio::task::spawn_blocking(move || {
                                let mut conn = match pool_check.get() {
                                    Ok(c) => c,
                                    Err(_) => return false,
                                };
                                bots::dsl::bots
                                    .filter(bots::dsl::name.eq(&bn))
                                    .select(bots::dsl::id)
                                    .first::<uuid::Uuid>(&mut conn)
                                    .is_ok()
                            })
                            .await
                            .unwrap_or(false)
                        };

                        if !exists {
                            // If LOAD_ONLY is set, we must explicitly verify bot_name is included before auto-creation
                            if !load_only.is_empty() && !load_only.contains(&bot_name) {
                                trace!("Skipping auto-creation for bot '{}' (not in LOAD_ONLY)", bot_name);
                                continue;
                            }
                            
                            info!("Auto-creating bot '{}' from S3 bucket '{}'", bot_name, name);
                            let create_state = state_for_scan.clone();
                            let bn = bot_name.to_string();
                            let pool_create = pool_clone.clone();
                            match tokio::task::spawn_blocking(move || {
                                create_bot_from_drive(&create_state, &pool_create, &bn)
                            })
                            .await
                            {
                                Ok(Err(e)) => {
                                    error!("Failed to create bot '{}': {}", bot_name, e);
                                    continue;
                                }
                                Err(e) => {
                                    error!("Task failed to create bot '{}': {}", bot_name, e);
                                    continue;
                                }
                                Ok(Ok(())) => {
                                    info!("Bot '{}' created successfully", bot_name);
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to list S3 buckets for bot discovery: {}", e),
            }
        }

        // Step 2: Start DriveMonitor for each active bot
        let bots_to_monitor = tokio::task::spawn_blocking({
            let pool_clone = pool_clone.clone();
            move || {
                use uuid::Uuid;
                let mut conn = match pool_clone.get() {
                    Ok(conn) => conn,
                    Err(_) => return Vec::new(),
                };
                bots::dsl::bots.filter(bots::dsl::is_active.eq(true))
                    .select((bots::dsl::id, bots::dsl::name))
                    .load::<(Uuid, String)>(&mut conn)
                    .unwrap_or_default()
            }
        })
        .await
        .unwrap_or_default();

        info!("Found {} active bots to monitor", bots_to_monitor.len());

        // Apply LOAD_ONLY filter to monitoring as well
        let load_only_for_monitor: Vec<String> = std::env::var("LOAD_ONLY")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        // Track actively monitored bot names for periodic scan
        let mut monitored_bots: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (bot_id, bot_name) in bots_to_monitor {
            // Filter by LOAD_ONLY if specified
            if !load_only_for_monitor.is_empty() && !load_only_for_monitor.contains(&bot_name) {
                trace!("Skipping monitoring for bot '{}' (not in LOAD_ONLY)", bot_name);
                continue;
            }

            monitored_bots.insert(bot_name.clone());

            let bucket_name = format!("{}.gbai", bot_name);
            let monitor_state = drive_monitor_state.clone();
            let bot_id_clone = bot_id;
            let bucket_name_clone = bucket_name.clone();

            tokio::spawn(async move {
                use crate::drive::drive_monitor::DriveMonitor;
                register_thread(&format!("drive-monitor-{}", bot_name), "drive");
                trace!("DriveMonitor::new starting for bot: {}", bot_name);
                let monitor =
                    DriveMonitor::new(monitor_state, bucket_name_clone, bot_id_clone);
                trace!(
                    "DriveMonitor::new done for bot: {}, calling start_monitoring...",
                    bot_name
                );
                info!(
                    "Starting DriveMonitor for bot: {} (bucket: {})",
                    bot_name, bucket_name
                );
                if let Err(e) = monitor.start_monitoring().await {
                    error!("DriveMonitor failed for bot {}: {}", bot_name, e);
                }
                trace!(
                    "DriveMonitor start_monitoring returned for bot: {}",
                    bot_name
                );
            });
        }

        // Step 3: Periodic bucket re-scan to discover new bots
        let scan_state = app_state.clone();
        tokio::spawn(async move {
            register_thread("drive-scan", "drive");
            let scan_interval = std::env::var("DRIVE_SCAN_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5u64);

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(scan_interval)).await;

                if let Some(s3_client) = &scan_state.drive {
                    match s3_client.list_all_buckets().await {
                        Ok(buckets) => {
                            for bucket in buckets {
                                if !bucket.ends_with(".gbai") {
                                    continue;
                                }
                                let bot_name = bucket.strip_suffix(".gbai").unwrap_or(&bucket).to_string();

                                if monitored_bots.contains(&bot_name) {
                                    continue;
                                }

                                // Reload LOAD_ONLY periodically to prevent unwanted bots
                                let load_only_scan: Vec<String> = std::env::var("LOAD_ONLY")
                                    .ok()
                                    .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                                    .unwrap_or_default();
                                if !load_only_scan.is_empty() && !load_only_scan.contains(&bot_name) {
                                    trace!("Periodic scan: skipping bot '{}' (not in LOAD_ONLY)", bot_name);
                                    continue;
                                }

                                let exists = {
                                    let pool_check = scan_pool.clone();
                                    let bn = bot_name.clone();
                                    tokio::task::spawn_blocking(move || {
                                        use diesel::prelude::*;
                                        use botcore::shared::models::schema::bots;
                                        let mut conn = match pool_check.get() {
                                            Ok(c) => c,
                                            Err(_) => return false,
                                        };
                                        bots::dsl::bots
                                            .filter(bots::dsl::name.eq(&bn))
                                            .select(bots::dsl::id)
                                            .first::<uuid::Uuid>(&mut conn)
                                            .is_ok()
                                    })
                                    .await
                                    .unwrap_or(false)
                                };

                                if exists {
                                    monitored_bots.insert(bot_name);
                                    continue;
                                }

                                info!("Periodic scan: auto-creating bot '{}' from S3 bucket '{}'", bot_name, bucket);
                                let create_state = scan_state.clone();
                                let bn = bot_name.clone();
                                let pool_create = scan_pool.clone();
                                let created = match tokio::task::spawn_blocking(move || {
                                    create_bot_from_drive(&create_state, &pool_create, &bn)
                                }).await {
                                    Ok(Ok(())) => true,
                                    Ok(Err(e)) => {
                                        error!("Periodic scan: failed to create bot '{}': {}", bot_name, e);
                                        false
                                    }
                                    Err(e) => {
                                        error!("Periodic scan: task failed for bot '{}': {}", bot_name, e);
                                        false
                                    }
                                };

                                if created {
                                    info!("Bot '{}' auto-created via periodic scan", bot_name);
                                    monitored_bots.insert(bot_name.clone());

                                    // Start DriveMonitor for the new bot
                                    let mon_pool = scan_pool.clone();
                                    let mn = bot_name.clone();
                                    tokio::task::spawn_blocking(move || {
                                        use uuid::Uuid;
                                        use diesel::prelude::*;
                                        use botcore::shared::models::schema::bots;
                                        let mut conn = match mon_pool.get() {
                                            Ok(c) => c,
                                            Err(_) => return None,
                                        };
                                        bots::dsl::bots
                                            .filter(bots::dsl::name.eq(&mn))
                                            .select(bots::dsl::id)
                                            .first::<Uuid>(&mut conn)
                                            .ok()
                                    }).await.ok().flatten().map(|new_bot_id| {
                                        let bucket_name = format!("{}.gbai", bot_name);
                                        let mon_state = scan_state.clone();
                                        tokio::spawn(async move {
                                            use crate::drive::drive_monitor::DriveMonitor;
                                            register_thread(&format!("drive-monitor-{}", bot_name), "drive");
                                            let monitor = DriveMonitor::new(mon_state, bucket_name, new_bot_id);
                                            info!("Starting DriveMonitor for newly discovered bot: {}", bot_name);
                                            if let Err(e) = monitor.start_monitoring().await {
                                                error!("DriveMonitor failed for new bot {}: {}", bot_name, e);
                                            }
                                        });
                                    });
                                }
                            }
                        }
                        Err(e) => warn!("Periodic bucket re-scan failed: {}", e),
                    }
                }
            }
        });
    });
}

fn create_bot_from_drive(
    _state: &Arc<botcore::shared::state::AppState>,
    pool: &botcore::shared::utils::DbPool,
    bot_name: &str,
) -> Result<(), String> {
    use diesel::sql_query;
    use uuid::Uuid;

    // CRITICAL: Respect LOAD_ONLY - never create bots not in the safelist
    if !botcore::bot_database::is_bot_allowed_by_load_only(bot_name) {
        return Err(format!(
            "Bot '{}' not allowed by LOAD_ONLY filter - refusing to create in database",
            bot_name
        ));
    }

    let mut conn = pool.get().map_err(|e| e.to_string())?;

    sql_query(
        "INSERT INTO tenants (id, name, slug, created_at)          VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant', 'default', NOW())          ON CONFLICT (slug) DO NOTHING",
    )
    .execute(&mut conn)
    .ok();

    sql_query(
        "INSERT INTO organizations (org_id, tenant_id, name, slug, created_at)          VALUES ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Default Organization', 'default', NOW())          ON CONFLICT (slug) DO NOTHING",
    )
    .execute(&mut conn)
    .ok();

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgResult {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: uuid::Uuid,
    }

    let org_result: OrgResult = sql_query("SELECT org_id FROM organizations WHERE slug = 'default' LIMIT 1")
        .get_result(&mut conn)
        .map_err(|e| format!("Failed to get default org: {}", e))?;

    let bot_id = Uuid::new_v4();
    let bot_id_str = bot_id.to_string();
    let org_id_str = org_result.org_id.to_string();

    // Try to insert, if conflict on slug, update instead
    let result = sql_query(format!(
        "INSERT INTO bots (id, name, slug, org_id, is_active, created_at, llm_provider, llm_config, context_provider, context_config) \
         VALUES ('{}', '{}', '{}', '{}', true, NOW(), 'openai', '{}', 'website', '{}') \
         ON CONFLICT (slug) DO UPDATE SET is_active = true",
        bot_id_str, bot_name, bot_name, org_id_str, "{}", "{}"
    ))
    .execute(&mut conn);

    if result.is_err() {
        // Bot might already exist with different id, try to update by name
        sql_query(format!(
            "UPDATE bots SET is_active = true, slug = '{}', llm_provider = 'openai', llm_config = '{}', context_provider = 'openai', context_config = '{}' WHERE name = '{}'",
            bot_name, "{}", "{}", bot_name
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update bot '{}': {}", bot_name, e))?;
    }

    let db_name = format!("bot_{}", bot_name.replace('-', "_"));
    let _ = sql_query(format!(
        "SELECT 'CREATE DATABASE {}' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '{}')",
        db_name, db_name
    ))
    .execute(&mut conn);

    // Verify the bot was actually inserted
    let exists: bool = sql_query(format!(
        "SELECT 1 FROM bots WHERE name = '{}' LIMIT 1",
        bot_name
    ))
    .execute(&mut conn)
    .is_ok();

    if !exists {
        return Err(format!("Bot '{}' was not found in database after insert", bot_name));
    }

    info!("Bot '{}' created successfully with id {}", bot_name, bot_id);
    Ok(())
}

pub(crate) async fn start_drive_compiler(app_state: Arc<AppState>) {
use crate::drive::drive_compiler::DriveCompiler;

let compiler = DriveCompiler::new(app_state.clone());
if let Err(e) = compiler.start_compiling().await {
error!("Failed to start DriveCompiler: {}", e);
} else {
trace!("DriveCompiler started - compiling .bas files from drive_files");
}
}
