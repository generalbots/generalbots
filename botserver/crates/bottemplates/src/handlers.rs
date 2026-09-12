use axum::body::Bytes;
use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use botcore::config::ConfigManager;
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

/// Resolves the caller's tenant branch from the server-minted JWT claims
/// (issue #734). Falls back to the global nil branch so anonymous/system
/// callers keep working, but every query is still constrained by the resolved
/// branch — a tenant can never see another tenant's rows.
fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

pub async fn list_templates(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] description: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] version: String,
        #[diesel(sql_type = diesel::sql_types::Text)] author: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, description, kind, version, author, created_at FROM app_templates WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "description": r.description, "kind": r.kind,
        "version": r.version, "author": r.author, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn preview_template(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
    }
    let row: Option<Row> = diesel::sql_query("SELECT name FROM app_templates WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .get_result(&mut conn).optional().map_err(db::map_diesel_err)?;
    let name = row.ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?.name;
    Ok(Json(serde_json::json!({"preview": {"id": id, "name": name, "files": [], "config": {}}})))
}

/// Optional body of `POST /api/templates/deploy/:id`. The Templates app sends
/// the user-chosen bot name; a BotFather token may be supplied so the Telegram
/// channel is provisioned in the same request (#1356). A caller that has nothing
/// to configure sends no body at all and the template name is used.
#[derive(Debug, Default, serde::Deserialize)]
pub struct DeployRequest {
    #[serde(default)]
    pub bot_name: Option<String>,
    #[serde(default)]
    pub telegram_token: Option<String>,
}

/// `POST /api/templates/deploy/:id` — stage the template's `.gbai` directory
/// into the caller org's Drive work tree (`{work}/{org}.gborg/{bot}.gbai/…`).
///
/// The drive monitor uploads that tree to MinIO and registers the bot — the same
/// mechanism bootstrap uses for the shipped catalog — and the org layout is what
/// every runtime resolver keys on (#1355). Before this the endpoint only inserted
/// a row and deployed nothing (#1354).
///
/// The deployed bot is named by the caller and the template's inner
/// `{template}.gbdialog` / `.gbot` directories are renamed to `{bot}.gbdialog` /
/// `.gbot`: the runtime resolves those directories by the bot name, so a bot whose
/// dialog directory kept the template name would load no scripts and no prompt.
pub async fn deploy_template(
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    // The body is optional: `Bytes` accepts an empty payload, unlike a strict JSON
    // extractor, which would reject a body-less deploy.
    let request: DeployRequest = if body.is_empty() {
        DeployRequest::default()
    } else {
        serde_json::from_slice(&body)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid deploy body: {e}")))?
    };

    #[derive(diesel::QueryableByName)]
    struct TemplateRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let template: Option<TemplateRow> = diesel::sql_query(
        "SELECT name FROM app_templates WHERE id = $1 AND branch_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    let template_name = template
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?
        .name;

    // Templates ship nested as `bots/<name>/<name>.gbai/`, so a single-level join
    // never matched; the directory is located by walking the tree (#1354).
    let source = crate::storage::find_template_dir(&template_name).ok_or((
        StatusCode::NOT_FOUND,
        format!("Template source directory not found for '{template_name}'"),
    ))?;

    // The deployed bot is named by the caller and defaults to the template name.
    let requested = request.bot_name.as_deref().unwrap_or("").trim().to_string();
    let bot_name = if requested.is_empty() {
        template_name.clone()
    } else {
        requested
    };
    validate_bot_name(&bot_name)?;

    let org_id = resolve_org_for_branch(&mut conn, branch);
    let target = std::path::PathBuf::from(botcore::shared::utils::get_org_work_path(org_id))
        .join(format!("{bot_name}.gbai"));
    let deploy_id = Uuid::new_v4();
    let now = Utc::now();
    if let Err(e) = copy_tree(&source, &target, &template_name, &bot_name) {
        log::error!("Template deploy staging failed for {template_name} as {bot_name}: {e}");
        insert_deploy(&mut conn, deploy_id, parsed, &branch, now, "failed")?;
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template staging failed: {e}"),
        ));
    }

    insert_deploy(&mut conn, deploy_id, parsed, &branch, now, "deployed")?;

    // Telegram provisioning is best-effort and always reported, never fatal: the
    // bot row is created by the drive monitor once the staged tree is uploaded, so
    // the token may legitimately have no bot to attach to yet (#1356).
    let telegram = match request.telegram_token.as_deref().map(str::trim) {
        Some(token) if !token.is_empty() => {
            provision_telegram_token(pool, &mut conn, &bot_name, branch, token)
        }
        _ => serde_json::json!({"status": "not_requested"}),
    };

    Ok(Json(serde_json::json!({"result": {
        "id": deploy_id,
        "template_id": id,
        "status": "deployed",
        "deployed_at": now,
        "target": "production",
        "bot_name": bot_name,
        "path": target.to_string_lossy(),
        "telegram": telegram,
    }})))
}

/// Bot names become Drive directory names and the bot `slug`, so they are kept to
/// a strict slug alphabet: no separators, no `..`, no spaces.
fn validate_bot_name(bot_name: &str) -> Result<(), (StatusCode, String)> {
    let length_ok = (1..=64).contains(&bot_name.len());
    let alphabet_ok = bot_name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if length_ok && alphabet_ok {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Bot name must be 1-64 characters of lowercase letters, digits, '-' or '_'".to_string(),
        ))
    }
}

/// The `bots` row for `bot_name`, created by the drive monitor after the staged
/// tree reaches MinIO.
fn resolve_bot_id(conn: &mut diesel::PgConnection, bot_name: &str) -> Option<Uuid> {
    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    match diesel::sql_query("SELECT id FROM bots WHERE slug = $1 OR name = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(bot_name.to_string())
        .get_result::<BotRow>(conn)
        .optional()
    {
        Ok(row) => row.map(|r| r.id),
        Err(e) => {
            log::warn!("Telegram provisioning: bot lookup for {bot_name} failed: {e}");
            None
        }
    }
}

/// Store the BotFather token for the deployed bot. Sensitive keys are written to
/// Vault at `secret/gbo/{org_id}/{branch_id}/{bot_id}` by the config manager, so
/// the token never lands in the database or a log line (#1356).
fn provision_telegram_token(
    pool: &botcore::shared::utils::DbPool,
    conn: &mut diesel::PgConnection,
    bot_name: &str,
    branch: Uuid,
    token: &str,
) -> serde_json::Value {
    let Some(bot_id) = resolve_bot_id(conn, bot_name) else {
        return serde_json::json!({
            "status": "pending",
            "detail": "The bot row is created by the drive monitor after the staged tree is uploaded; set the token again once the bot exists."
        });
    };
    let manager = ConfigManager::new(botcore::shared::utils::DbPool::clone(pool));
    match manager.set_config(&bot_id, "telegram-bot-token", token) {
        Ok(()) => serde_json::json!({
            "status": "provisioned",
            "bot_id": bot_id,
            "branch_id": branch,
        }),
        Err(e) => {
            log::error!("Telegram token provisioning failed for bot {bot_name}: {e}");
            serde_json::json!({
                "status": "failed",
                "detail": "The token could not be stored; see the server log."
            })
        }
    }
}

/// The org that owns `branch`; the nil org (global/admin scope) when the branch
/// is unknown.
fn resolve_org_for_branch(conn: &mut diesel::PgConnection, branch: Uuid) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    match diesel::sql_query("SELECT org_id FROM branches WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .get_result::<OrgRow>(conn)
        .optional()
    {
        Ok(Some(row)) => row.org_id,
        Ok(None) => {
            log::warn!("Template deploy: branch {branch} is not in the branches table; using the global org");
            Uuid::nil()
        }
        Err(e) => {
            log::error!("Template deploy: could not resolve the org for branch {branch}: {e}");
            Uuid::nil()
        }
    }
}

fn insert_deploy(
    conn: &mut diesel::PgConnection,
    deploy_id: Uuid,
    template_id: Uuid,
    branch: &Uuid,
    now: chrono::DateTime<Utc>,
    status: &str,
) -> Result<(), (StatusCode, String)> {
    diesel::sql_query(
        "INSERT INTO app_template_deploys (id, template_id, status, target, deployed_at, branch_id)
         VALUES ($1, $2, $3, 'production', $4, $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(deploy_id)
    .bind::<diesel::sql_types::Uuid, _>(template_id)
    .bind::<diesel::sql_types::Text, _>(status.to_string())
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(*branch)
    .execute(conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}

/// Recursively copy a directory tree, creating the destination and renaming the
/// template's `{template}.gb*` directories to `{bot}.gb*`. Fails with a message
/// naming the offending path.
fn copy_tree(
    src: &std::path::Path,
    dst: &std::path::Path,
    template_name: &str,
    bot_name: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("create {}: {e}", dst.display()))?;
    let entries = std::fs::read_dir(src).map_err(|e| format!("read {}: {e}", src.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("entry in {}: {e}", src.display()))?;
        let from = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let to = dst.join(rename_gb_dir(&file_name, template_name, bot_name));
        if from.is_dir() {
            copy_tree(&from, &to, template_name, bot_name)?;
        } else {
            std::fs::copy(&from, &to)
                .map_err(|e| format!("copy {} -> {}: {e}", from.display(), to.display()))?;
        }
    }
    Ok(())
}

/// `media-filing.gbdialog` becomes `{bot}.gbdialog` when a template is deployed
/// under a different bot name; every other entry is left untouched.
fn rename_gb_dir(file_name: &str, template_name: &str, bot_name: &str) -> String {
    if template_name == bot_name {
        return file_name.to_string();
    }
    match file_name.strip_prefix(&format!("{template_name}.gb")) {
        Some(kind) => format!("{bot_name}.gb{kind}"),
        None => file_name.to_string(),
    }
}