//! drive_monitors - extracted from bootstrap.rs
//! Processes .gborg buckets containing .gbai bot sub-directories

use botcore::shared::state::AppState;
use diesel::RunQueryDsl;
use log::{error, info, trace, warn};
use std::sync::Arc;
use uuid::Uuid;

use crate::basic::AppStateBasicRuntime;
use crate::basic::keywords::table_definition::process_table_definitions;

const DEFAULT_TENANT_ID: Uuid = Uuid::from_u128(1);
const DEFAULT_ORG_ID: Uuid = Uuid::from_u128(1);
const DEFAULT_BRANCH_ID: Uuid = Uuid::from_u128(1);
const DEFAULT_BOT_ID: Uuid = Uuid::from_u128(1);

pub(crate) async fn start_drive_monitors(
    app_state: Arc<AppState>,
    _pool: &botcore::shared::utils::DbPool,
) {
    use botcore::shared::memory_monitor::register_thread;

    let pool_clone = _pool.clone();
    let state_for_scan = app_state.clone();
    let scan_pool = _pool.clone();

    // Bootstrap: ensure branches table schema (migration 9.15) if diesel pipeline was blocked
    ensure_org_branches_schema(_pool);

    // Bootstrap: ensure default tenant/org/branch/bot exist
    if let Err(e) = ensure_bootstrap_defaults(_pool) {
        error!("Bootstrap defaults failed: {}", e);
    } else {
        info!("Bootstrap defaults ensured (tenant/org/branch/bot)");
    }

    tokio::spawn(async move {
        register_thread("drive-monitor", "drive");

        let load_only: Vec<String> = std::env::var("LOAD_ONLY")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        if !load_only.is_empty() {
            info!("LOAD_ONLY filter active: {:?}", load_only);
        }

        // Step 1: Discover bots from S3 .gborg buckets
        log::info!("Drive client status: {:?}", state_for_scan.drive.is_some());
        if let Some(s3_client) = &state_for_scan.drive {
            match s3_client.list_all_buckets().await {
                Ok(buckets) => {
                    for bucket in buckets {
                        if bucket.ends_with(".gborg") {
                            process_org_bucket(
                                &state_for_scan, &pool_clone, &bucket, &load_only,
                            ).await;
                        }
                    }
                }
                Err(e) => warn!("Failed to list S3 buckets for bot discovery: {}", e),
            }
        }

        // Step 2: Periodic bucket re-scan (bots are discovered via process_org_bucket)
        let load_only_for_monitor: Vec<String> = std::env::var("LOAD_ONLY")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let mut monitored_bots: std::collections::HashSet<String> = std::collections::HashSet::new();

        let mut last_list_error: Option<std::time::Instant> = None;
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
                            if last_list_error.is_some() {
                                info!("Drive scan recovered — S3 buckets now accessible");
                                last_list_error = None;
                            }
                            for bucket in buckets {
                                if bucket.ends_with(".gborg") {
                                    if let Err(e) = scan_org_bucket(
                                        &scan_state, &scan_pool, &bucket,
                                        &load_only_for_monitor, &mut monitored_bots,
                                    ).await {
                                        warn!("Periodic .gborg scan failed for {}: {}", bucket, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let now = std::time::Instant::now();
                            let should_log = last_list_error.map_or(true, |t| now.duration_since(t).as_secs() > 120);
                            if should_log {
                                warn!("Periodic bucket re-scan failed: {}", e);
                                last_list_error = Some(now);
                            }
                        }
                    }
                }
            }
        });
    });
}

/// Bootstrap: ensure default tenant/org/branch/bot exist for fresh installs
fn ensure_bootstrap_defaults(pool: &botcore::shared::utils::DbPool) -> Result<(), String> {
    use diesel::sql_query;

    let mut conn = pool.get().map_err(|e| e.to_string())?;

    if let Err(e) = sql_query(format!(
        "INSERT INTO tenants (id, name, slug, created_at) \
         VALUES ('{tid}', 'Default Tenant', 'default', NOW()) \
         ON CONFLICT (slug) DO NOTHING",
        tid = DEFAULT_TENANT_ID
    ))
    .execute(&mut conn) {
        warn!("Failed to insert default tenant: {}", e);
    }

    if let Err(e) = sql_query(format!(
        "INSERT INTO organizations (org_id, tenant_id, name, slug, created_at) \
         VALUES ('{oid}', '{tid}', 'Default Organization', 'default', NOW()) \
         ON CONFLICT (slug) DO NOTHING",
        oid = DEFAULT_ORG_ID,
        tid = DEFAULT_TENANT_ID
    ))
    .execute(&mut conn) {
        warn!("Failed to insert default organization: {}", e);
    }

    // Look up the actual org_id and tenant_id from the database
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgId { #[diesel(sql_type = diesel::sql_types::Uuid)] org_id: Uuid }

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct TenantId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid }

    let actual_org_id: Uuid = sql_query(
        "SELECT org_id FROM organizations WHERE slug = 'default' LIMIT 1"
    ).get_result::<OrgId>(&mut conn)
     .map(|r| r.org_id)
     .unwrap_or(DEFAULT_ORG_ID);

    let actual_tenant_id: Uuid = sql_query(
        "SELECT id FROM tenants WHERE slug = 'default' LIMIT 1"
    ).get_result::<TenantId>(&mut conn)
     .map(|r| r.id)
     .unwrap_or(DEFAULT_TENANT_ID);

    if let Err(e) = sql_query(format!(
        "INSERT INTO branches (id, org_id, tenant_id, slug, name, created_at) \
         VALUES ('{bid}', '{oid}', '{tid}', 'default', 'Default Branch', NOW()) \
         ON CONFLICT (org_id, slug) DO NOTHING",
        bid = DEFAULT_BRANCH_ID,
        oid = actual_org_id,
        tid = actual_tenant_id
    ))
    .execute(&mut conn) {
        warn!("Failed to insert default branch: {}", e);
    }

    // Look up the actual branch_id
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BranchId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid }

    let actual_branch_id: Uuid = sql_query(
        "SELECT id FROM branches WHERE org_id = $1 AND slug = 'default' LIMIT 1"
    ).bind::<diesel::sql_types::Uuid, _>(actual_org_id)
     .get_result::<BranchId>(&mut conn)
     .map(|r| r.id)
     .unwrap_or(DEFAULT_BRANCH_ID);

    if let Err(e) = sql_query(format!(
        "INSERT INTO bots (id, name, slug, org_id, branch_id, is_default_for_branch, \
                           is_active, created_at, updated_at, llm_provider, llm_config, \
                           context_provider, context_config, is_public) \
         VALUES ('{botid}', 'default', 'default', '{oid}', '{bid}', true, \
                 true, NOW(), NOW(), 'openai', '{{}}', 'openai', '{{}}', false) \
         ON CONFLICT (slug) DO NOTHING",
        botid = DEFAULT_BOT_ID,
        oid = actual_org_id,
        bid = actual_branch_id
    ))
    .execute(&mut conn) {
        warn!("Failed to insert default bot: {}", e);
    }

    info!("Default tenant, organization, branch, and bot ensured");
    Ok(())
}

/// Ensure the 9.15-org-branches migration schema exists (branches table + bots columns).
/// Runs independently of the diesel migration pipeline to handle environments where
/// pre-existing migrations block the pipeline from reaching 9.15.
fn ensure_org_branches_schema(pool: &botcore::shared::utils::DbPool) {
    use diesel::sql_query;
    if let Ok(mut conn) = pool.get() {
        // Ensure tenants table exists (pre-requisite for branches)
        if let Err(e) = sql_query(
            "CREATE TABLE IF NOT EXISTS tenants (
                id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name        VARCHAR(255) NOT NULL,
                slug        VARCHAR(255) NOT NULL UNIQUE,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to create tenants table: {}", e);
        }

        // Ensure tenant_id column exists on organizations (pre-requisite for branches)
        if let Err(e) = sql_query(
            "ALTER TABLE organizations ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id)"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to add tenant_id to organizations: {}", e);
        }

        if let Err(e) = sql_query(
            "CREATE TABLE IF NOT EXISTS branches (
                id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                org_id      UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
                tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                slug        VARCHAR(255) NOT NULL,
                name        VARCHAR(255) NOT NULL,
                description TEXT,
                is_active   BOOLEAN DEFAULT true,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(org_id, slug)
            )"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to create branches table: {}", e);
        }

        if let Err(e) = sql_query(
            "ALTER TABLE bots ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id)"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to add branch_id to bots: {}", e);
        }

        if let Err(e) = sql_query(
            "ALTER TABLE bots ADD COLUMN IF NOT EXISTS is_default_for_branch BOOLEAN DEFAULT false"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to add is_default_for_branch to bots: {}", e);
        }

        if let Err(e) = sql_query(
            "CREATE INDEX IF NOT EXISTS idx_branches_org ON branches(org_id)"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to create idx_branches_org: {}", e);
        }

        if let Err(e) = sql_query(
            "CREATE INDEX IF NOT EXISTS idx_branches_tenant ON branches(tenant_id)"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to create idx_branches_tenant: {}", e);
        }

        if let Err(e) = sql_query(
            "CREATE INDEX IF NOT EXISTS idx_branches_slug ON branches(slug)"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to create idx_branches_slug: {}", e);
        }

        if let Err(e) = sql_query(
            "CREATE INDEX IF NOT EXISTS idx_bots_branch ON bots(branch_id)"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to create idx_bots_branch: {}", e);
        }

        if let Err(e) = sql_query(
            "UPDATE bots SET org_id = '00000000-0000-0000-0000-000000000001' WHERE org_id IS NULL"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to update null org_ids: {}", e);
        }

        if let Err(e) = sql_query(
            "ALTER TABLE bots ALTER COLUMN org_id SET NOT NULL"
        ).execute(&mut conn) {
            warn!("ensure_org_branches_schema: failed to set org_id NOT NULL: {}", e);
        }

        trace!("ensure_org_branches_schema: branches table and bot columns ensured");
    }
}

/// Sync tables.bas for a bot within a .gborg bucket structure.
async fn sync_tables_for_org_bot(
    state: &Arc<AppState>,
    pool: &botcore::shared::utils::DbPool,
    bot_name: &str,
    bucket_name: &str,
    s3_prefix: &str,
) -> Result<(), String> {
    let bot_id = get_bot_id(pool, bot_name).await?;

    let content = match &state.drive {
        Some(s3) => {
            match s3.get_object_direct(bucket_name, &format!("{}{}.gbdialog/tables.bas", s3_prefix, bot_name)).await {
                Ok(data) => String::from_utf8(data).map_err(|e| format!("UTF-8 error: {e}"))?,
                Err(_) => {
                    trace!("tables.bas not found in S3 for org bot {}, skipping", bot_name);
                    return Ok(());
                }
            }
        }
        None => {
            trace!("S3 client not available, skipping table sync for org bot {}", bot_name);
            return Ok(());
        }
    };

    let runtime: Arc<dyn botbasic_types::BasicRuntime> = Arc::new(AppStateBasicRuntime(state.clone()));
    match process_table_definitions(runtime, bot_id, &content) {
        Ok(tables) => info!("Synced {} table definitions for org bot '{}'", tables.len(), bot_name),
        Err(e) => warn!("Failed to sync table definitions for org bot '{}': {}", bot_name, e),
    }
    Ok(())
}

async fn get_bot_id(
    pool: &botcore::shared::utils::DbPool,
    bot_name: &str,
) -> Result<Uuid, String> {
    let bn = bot_name.to_string();
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;
        use botcore::shared::models::schema::bots;
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(_) => return None,
        };
        bots::dsl::bots
            .filter(bots::dsl::name.eq(&bn))
            .select(bots::dsl::id)
            .first::<Uuid>(&mut conn)
            .ok()
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
    .ok_or_else(|| format!("Bot '{}' not found in database", bot_name))
}

/// Process a .gborg bucket (tenant) during initial discovery
async fn process_org_bucket(
    state: &Arc<AppState>,
    pool: &botcore::shared::utils::DbPool,
    bucket_name: &str,
    load_only: &[String],
) {
    let tenant_slug = bucket_name.strip_suffix(".gborg").unwrap_or(bucket_name).to_string();

    // For .gborg buckets, LOAD_ONLY filtering happens at the bot level
    // inside discover_and_create_bots, not at the tenant level.
    // This allows LOAD_ONLY=cristo to work even if the bucket is named differently.

    // Get S3 client to list prefixes
    let (tenant_id, org_id) = if let Ok(result) = ensure_tenant_and_org(pool, &tenant_slug) {
        result
    } else {
        return;
    };

    // Discover .gbai/ prefixes within the .gborg bucket
    if let Some(s3) = &state.drive {
        match s3.list_common_prefixes(bucket_name, "/").await {
            Ok(prefixes) => {
                for prefix in prefixes {
                    if !prefix.ends_with(".gbai/") {
                        continue;
                    }
                    let branch_slug = prefix.strip_suffix(".gbai/").unwrap_or(&prefix).to_string();

                    // Discover bot names within this branch
                    discover_and_create_bots(
                        state, pool, bucket_name, &prefix,
                        &branch_slug, tenant_id, org_id, load_only,
                    ).await;

                    // Start monitors for bots in this branch immediately
                    let bot_names = get_bot_names_in_branch(pool, &branch_slug, org_id).await;
                    for bot_name in bot_names {
                        start_org_bot_monitor(
                            state, pool, bucket_name, &bot_name,
                            &branch_slug, &tenant_slug, &prefix,
                            tenant_id, org_id,
                        ).await;
                    }
                }
            }
            Err(e) => error!("Failed to list prefixes in .gborg bucket '{}': {}", bucket_name, e),
        }
    }
}

/// Discover bot names within a branch prefix and create them
async fn discover_and_create_bots(
    state: &Arc<AppState>,
    pool: &botcore::shared::utils::DbPool,
    bucket_name: &str,
    branch_prefix: &str,
    branch_slug: &str,
    tenant_id: Uuid,
    org_id: Uuid,
    load_only: &[String],
) {
    if let Some(s3) = &state.drive {
        match s3.list_objects(bucket_name, Some(branch_prefix)).await {
            Ok(objects) => {
                // Extract unique bot names from object keys
                // Key format: {branch}.gbai/{bot}.gbdialog/file.bas
                let mut bot_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
                for key in &objects {
                    let relative = key.strip_prefix(branch_prefix).unwrap_or(key);
                    if let Some(dot_idx) = relative.find(".gbdialog") {
                        let bot_name = &relative[..dot_idx];
                        if !bot_name.is_empty() {
                            bot_names.insert(bot_name.to_string());
                        }
                    }
                }

                if bot_names.is_empty() && !load_only.iter().any(|s| s == branch_slug) {
                    // Create default bot (branch name == bot name)
                    bot_names.insert(branch_slug.to_string());
                }

                for bot_name in &bot_names {
                    if !load_only.is_empty() && !load_only.contains(bot_name) {
                        trace!("Skipping bot '{}' (not in LOAD_ONLY)", bot_name);
                        continue;
                    }

                    info!("Ensuring bot '{}' in branch '{}' (org: {}, tenant: {})",
                          bot_name, branch_slug, org_id, tenant_id);

                    let create_state = state.clone();
                    let bn = bot_name.clone();
                    let bs = branch_slug.to_string();
                    let bs_is_default = bn == bs;
                    let pool_create = pool.clone();
                    let ld = load_only.to_vec();
                    let created = match tokio::task::spawn_blocking(move || {
                        create_bot_from_drive(
                            &create_state, &pool_create, &bn,
                            Some((tenant_id, org_id, bs)),
                            Some(bs_is_default), &ld,
                        )
                    }).await {
                        Ok(Ok(_)) => {
                            info!("Bot '{}' created in org", bot_name);
                            true
                        }
                        Ok(Err(e)) => {
                            error!("Failed to create bot '{}' in org: {}", bot_name, e);
                            false
                        }
                        Err(e) => {
                            error!("Task failed for bot '{}' in org: {}", bot_name, e);
                            false
                        }
                    };

                    if created {
                        if let Err(e) = sync_tables_for_org_bot(
                            state, pool, bot_name, bucket_name, branch_prefix,
                        ).await {
                            warn!("Failed to sync tables for new org bot '{}': {}", bot_name, e);
                        }
                    }
                }
            }
            Err(e) => error!("Failed to list objects in '{}/{}': {}", bucket_name, branch_prefix, e),
        }
    }
}

/// Periodic re-scan for a .gborg bucket
async fn scan_org_bucket(
    state: &Arc<AppState>,
    pool: &botcore::shared::utils::DbPool,
    bucket_name: &str,
    load_only: &[String],
    monitored_bots: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    let tenant_slug = bucket_name.strip_suffix(".gborg").unwrap_or(bucket_name).to_string();

    let (tenant_id, org_id) = ensure_tenant_and_org(pool, &tenant_slug)?;

    if let Some(s3) = &state.drive {
        let prefixes = s3.list_common_prefixes(bucket_name, "/")
            .await
            .map_err(|e| format!("Failed to list prefixes: {}", e))?;

        for prefix in prefixes {
            if !prefix.ends_with(".gbai/") {
                continue;
            }
            let branch_slug = prefix.strip_suffix(".gbai/").unwrap_or(&prefix).to_string();
            let branch_key = format!("{}/", branch_slug);

            if monitored_bots.contains(&branch_key) {
                continue;
            }

            ensure_branch_exists(pool, &branch_slug, org_id, tenant_id)?;

            // Discover bots within branch
            discover_and_create_bots(
                state, pool, bucket_name, &prefix,
                &branch_slug, tenant_id, org_id, load_only,
            ).await;

            // Mark branch as monitored (track via prefix)
            monitored_bots.insert(branch_key);

            // Start monitors for newly created bots in this branch
            let bot_names = get_bot_names_in_branch(pool, &branch_slug, org_id).await;
            for bot_name in bot_names {
                let mon_key = format!("{}/{}", branch_slug, bot_name);
                if !monitored_bots.contains(&mon_key) {
                    monitored_bots.insert(mon_key);
                    start_org_bot_monitor(state, pool, bucket_name, &bot_name, &branch_slug,
                                          &tenant_slug, &prefix, tenant_id, org_id).await;
                }
            }
        }
    }

    Ok(())
}

/// Start a DriveMonitor for a bot within a .gborg bucket
async fn start_org_bot_monitor(
    state: &Arc<AppState>,
    pool: &botcore::shared::utils::DbPool,
    bucket_name: &str,
    bot_name: &str,
    branch_slug: &str,
    tenant_slug: &str,
    s3_prefix: &str,
    _tenant_id: Uuid,
    _org_id: Uuid,
) {
    let mon_pool = pool.clone();
    let mn = bot_name.to_string();
    let bot_id = tokio::task::spawn_blocking(move || {
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
    }).await.ok().flatten();

    if let Some(bot_id) = bot_id {
        let mon_state = state.clone();
        let bucket = bucket_name.to_string();
        let prefix = s3_prefix.to_string();
        let bn = bot_name.to_string();
        let bs = branch_slug.to_string();
        let ts = tenant_slug.to_string();
        let org_slug_for_spawn = format!("{}-org", ts);
        let bn_for_log = bn.clone();
        let bucket_for_log = bucket.clone();
        tokio::spawn(async move {
            use crate::drive::drive_monitor::DriveMonitor;
            use botcore::shared::memory_monitor::register_thread;
            register_thread(&format!("drive-monitor-{}", bn_for_log), "drive");
            let monitor = DriveMonitor::new_with_params(
                mon_state, bucket, bot_id, bn, bs,
                Some(prefix), ts, org_slug_for_spawn,
            );
            info!("Starting DriveMonitor for org bot: {} (bucket: {})", bn_for_log, bucket_for_log);
            if let Err(e) = monitor.start_monitoring().await {
                error!("DriveMonitor failed for org bot {}: {}", bn_for_log, e);
            }
        });
    }
}

/// Get bot names in a branch from the database
async fn get_bot_names_in_branch(
    pool: &botcore::shared::utils::DbPool,
    branch_slug: &str,
    org_id: Uuid,
) -> Vec<String> {
    let pool_clone = pool.clone();
    let bs = branch_slug.to_string();
    tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;
        use botcore::shared::models::schema::{branches, bots};
        let mut conn = match pool_clone.get() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let branch_id: Uuid = match branches::dsl::branches
            .filter(branches::dsl::slug.eq(&bs))
            .filter(branches::dsl::org_id.eq(org_id))
            .select(branches::dsl::id)
            .first::<Uuid>(&mut conn)
        {
            Ok(id) => id,
            Err(_) => return Vec::new(),
        };
        bots::dsl::bots
            .filter(bots::dsl::branch_id.eq(branch_id))
            .filter(bots::dsl::is_active.eq(true))
            .select(bots::dsl::name)
            .load::<String>(&mut conn)
            .unwrap_or_default()
    }).await.unwrap_or_default()
}

fn ensure_tenant_and_org(pool: &botcore::shared::utils::DbPool, tenant_slug: &str) -> Result<(Uuid, Uuid), String> {
    use diesel::sql_query;
    let mut conn = pool.get().map_err(|e| e.to_string())?;

    // Upsert tenant
    let tenant_id = Uuid::new_v4();
    sql_query(format!(
        "INSERT INTO tenants (id, name, slug, created_at) \
         VALUES ('{tid}', '{slug}', '{slug}', NOW()) \
         ON CONFLICT (slug) DO UPDATE SET name = '{slug}'",
        tid = tenant_id,
        slug = tenant_slug
    ))
    .execute(&mut conn)
    .map_err(|e| format!("Failed to upsert tenant '{}': {}", tenant_slug, e))?;

    // Get tenant_id
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct TenantId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid }
    let tid: Uuid = diesel::sql_query(
        "SELECT id FROM tenants WHERE slug = $1 LIMIT 1"
    )
    .bind::<diesel::sql_types::Text, _>(tenant_slug)
    .get_result::<TenantId>(&mut conn)
    .map_err(|e| format!("Failed to get tenant_id: {}", e))?
    .id;

    // Upsert default org for this tenant
    let org_id = Uuid::new_v4();
    let org_slug = format!("{}-org", tenant_slug);
    sql_query(format!(
        "INSERT INTO organizations (org_id, tenant_id, name, slug, created_at) \
         VALUES ('{oid}', '{tid}', '{slug}', '{orgslug}', NOW()) \
         ON CONFLICT (slug) DO NOTHING",
        oid = org_id,
        tid = tid,
        slug = tenant_slug,
        orgslug = org_slug
    ))
    .execute(&mut conn)
    .ok();

    // Get org_id
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgId { #[diesel(sql_type = diesel::sql_types::Uuid)] org_id: Uuid }
    let resolved_org_id: Uuid = diesel::sql_query(
        "SELECT org_id FROM organizations WHERE tenant_id = $1 AND slug = $2 LIMIT 1"
    )
    .bind::<diesel::sql_types::Uuid, _>(tid)
    .bind::<diesel::sql_types::Text, _>(&org_slug)
    .get_result::<OrgId>(&mut conn)
    .map_err(|e| format!("Failed to get org_id: {}", e))?
    .org_id;

    Ok((tid, resolved_org_id))
}

fn ensure_branch_exists(pool: &botcore::shared::utils::DbPool, branch_slug: &str, org_id: Uuid, tenant_id: Uuid) -> Result<Uuid, String> {
    use diesel::sql_query;
    let mut conn = pool.get().map_err(|e| e.to_string())?;

    let branch_id = Uuid::new_v4();
    sql_query(format!(
        "INSERT INTO branches (id, org_id, tenant_id, slug, name, created_at) \
         VALUES ('{bid}', '{oid}', '{tid}', '{slug}', '{slug}', NOW()) \
         ON CONFLICT (org_id, slug) DO NOTHING",
        bid = branch_id,
        oid = org_id,
        tid = tenant_id,
        slug = branch_slug
    ))
    .execute(&mut conn)
    .map_err(|e| format!("Failed to upsert branch '{}': {}", branch_slug, e))?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BranchId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid }
    let resolved_bid: Uuid = diesel::sql_query(
        "SELECT id FROM branches WHERE org_id = $1 AND slug = $2 LIMIT 1"
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Text, _>(branch_slug)
    .get_result::<BranchId>(&mut conn)
    .map_err(|e| format!("Failed to get branch_id: {}", e))?
    .id;

    Ok(resolved_bid)
}

fn create_bot_from_drive(
    _state: &Arc<botcore::shared::state::AppState>,
    pool: &botcore::shared::utils::DbPool,
    bot_name: &str,
    org_info: Option<(Uuid, Uuid, String)>, // (tenant_id, org_id, branch_slug)
    is_default: Option<bool>,
    load_only: &[String],
) -> Result<(), String> {
    use diesel::sql_query;

    if !load_only.is_empty() && !load_only.contains(&bot_name.to_string()) {
        return Err(format!(
            "Bot '{}' not allowed by LOAD_ONLY filter - refusing to create",
            bot_name
        ));
    }

    let mut conn = pool.get().map_err(|e| e.to_string())?;

    let (tenant_id, org_id, branch_slug) = match org_info {
        Some((tid, oid, bs)) => (tid, oid, bs),
        None => {
            // Legacy mode: resolve actual default tenant/org from database
            #[derive(diesel::QueryableByName)]
            #[diesel(check_for_backend(diesel::pg::Pg))]
            struct OrgId { #[diesel(sql_type = diesel::sql_types::Uuid)] org_id: Uuid }

            #[derive(diesel::QueryableByName)]
            #[diesel(check_for_backend(diesel::pg::Pg))]
            struct TenantId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid }

            let actual_tenant_id: Uuid = diesel::sql_query(
                "SELECT id FROM tenants WHERE slug = 'default' LIMIT 1"
            ).get_result::<TenantId>(&mut conn)
             .map(|r| r.id)
             .unwrap_or(DEFAULT_TENANT_ID);

            let actual_org_id: Uuid = diesel::sql_query(
                "SELECT org_id FROM organizations WHERE slug = 'default' LIMIT 1"
            ).get_result::<OrgId>(&mut conn)
             .map(|r| r.org_id)
             .unwrap_or(DEFAULT_ORG_ID);

            let bs = bot_name.to_string();
            (actual_tenant_id, actual_org_id, bs)
        }
    };

    let branch_id = ensure_branch_exists(pool, &branch_slug, org_id, tenant_id)?;
    let bot_id = Uuid::new_v4();
    let is_default = is_default.unwrap_or(true);

    let result = sql_query(format!(
        "INSERT INTO bots (id, name, slug, org_id, branch_id, is_default_for_branch, \
                          is_active, created_at, updated_at, llm_provider, llm_config, \
                          context_provider, context_config, is_public) \
         VALUES ('{bid}', '{name}', '{name}', '{oid}', '{branchid}', {isdef}, \
                 true, NOW(), NOW(), 'openai', '{{}}', 'openai', '{{}}', false) \
         ON CONFLICT (slug) DO UPDATE SET is_active = true, org_id = '{oid}', \
         branch_id = '{branchid}', updated_at = NOW()",
        bid = bot_id,
        name = bot_name,
        oid = org_id,
        branchid = branch_id,
        isdef = if is_default { "true" } else { "false" },
    ))
    .execute(&mut conn);

    if result.is_err() {
        sql_query(format!(
            "UPDATE bots SET is_active = true, org_id = '{oid}', branch_id = '{branchid}', \
             is_default_for_branch = {isdef}, updated_at = NOW(), \
             llm_provider = 'openai', llm_config = '{{}}', \
             context_provider = 'openai', context_config = '{{}}' \
             WHERE name = '{name}'",
            oid = org_id,
            branchid = branch_id,
            isdef = if is_default { "true" } else { "false" },
            name = bot_name
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update bot '{}': {}", bot_name, e))?;
    }

    let db_name = format!("bot_{}", bot_name.replace('-', "_"));
    let _ = sql_query(format!(
        "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_database WHERE datname = '{db}') THEN EXECUTE 'CREATE DATABASE {db}'; END IF; END $$;",
        db = db_name
    ))
    .execute(&mut conn);

    info!("Bot '{}' created (org: {}, branch: {})", bot_name, org_id, branch_id);
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
