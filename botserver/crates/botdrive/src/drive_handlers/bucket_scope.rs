//! Authoritative Drive bucket resolution for bot scope (#2928).
//!
//! The Drive UI used to guess `{bot}.gbai` when it could not discover an org
//! workspace. An org-owned bot has no such bucket: its files live in
//! `{org}.gborg/{branch}.gbai/`, so every listing against the guess was a 403.
//! The server already knows the real shape from the bot row (org + branch
//! slugs). This module exposes that knowledge as a small, testable resolver
//! and as `/api/files/bucket` for clients that could not discover the layout.
//!
//! Nothing here grants a new bucket. Every answer is checked against
//! `allowed_buckets_for`, which stays the single tenant-isolation gate.

use super::*;
use botbasic_core::utils::{resolve_bot_drive_location, BotDriveLocation};

/// Same projection as `botbasic_core`'s scope resolver. Local so this endpoint
/// can fail closed when the DB is unavailable: the shared resolver degrades to
/// the standalone layout on error, which is exactly the guess #2928 removes.
const BOT_SCOPE_SELECT: &str = "SELECT b.name, br.slug AS branch_slug, o.slug AS org_slug \
     FROM bots b \
     LEFT JOIN branches br ON br.id = b.branch_id \
     LEFT JOIN organizations o ON o.org_id = b.org_id";

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BotScopeRow {
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    branch_slug: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    org_slug: Option<String>,
}

fn load_bot_location(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    bot: &str,
) -> Option<BotDriveLocation> {
    let row = diesel::sql_query(format!("{BOT_SCOPE_SELECT} WHERE b.name = $1 LIMIT 1"))
        .bind::<Text, _>(bot)
        .get_result::<BotScopeRow>(conn);
    match row {
        Ok(row) => Some(resolve_bot_drive_location(
            &row.name,
            row.branch_slug.as_deref(),
            row.org_slug.as_deref(),
        )),
        Err(e) => {
            warn!("drive: bucket resolution failed for bot '{bot}': {e}");
            None
        }
    }
}

/// What a client needs to address a bot's Drive tree without guessing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DriveBucketContext {
    /// Bucket that actually holds the bot's files (`.gborg` for org bots).
    pub bucket: String,
    /// Branch workspace inside that bucket, i.e. the `{workspace}.gbai/` part
    /// of the key prefix. UI path builders must use this, not the bot name:
    /// `oppbot` lives in `opportunity-oppbot.gbai/`.
    pub workspace: String,
    pub bot: String,
    /// Full key prefix of the bot's Drive directory.
    pub key_prefix: String,
    pub is_org: bool,
}

/// Builds the response from a DB-resolved location. The bucket is never
/// derived from the bot name here.
pub(super) fn context_for(location: &BotDriveLocation, bot: &str) -> DriveBucketContext {
    let workspace = location
        .bot_prefix
        .trim_end_matches('/')
        .strip_suffix(".gbai")
        .unwrap_or_else(|| location.bot_prefix.trim_end_matches('/'));
    let workspace = if workspace.is_empty() {
        bot.to_string()
    } else {
        workspace.to_string()
    };
    DriveBucketContext {
        bucket: location.bucket.clone(),
        workspace,
        bot: bot.to_string(),
        key_prefix: location.key_prefix.clone(),
        is_org: location.bucket.ends_with(".gborg"),
    }
}

/// Reconciles a client-supplied bucket with the authoritative DB location.
///
/// - the caller must be entitled to the DB-resolved bucket first (platform
///   admins are the only exception, as in `resolve_bucket`)
/// - no override / correct override → the DB-resolved bucket
/// - the old `{bot}.gbai` guess for an org bot → corrected to that org bucket
/// - any other bucket — including another org's `.gborg` — is refused, so a
///   wrong tenant scope can never turn into a read or write of its files
pub(super) fn reconcile_bucket(
    location: &BotDriveLocation,
    bot: &str,
    requested: Option<&str>,
    allowed: &[String],
    platform_admin: bool,
) -> Result<DriveBucketContext, &'static str> {
    let context = context_for(location, bot);
    if !platform_admin && !allowed.iter().any(|a| a == &context.bucket) {
        return Err("Access denied for this bucket");
    }
    let guessed_standalone = format!("{bot}.gbai");
    if is_acceptable(
        requested,
        &context.bucket,
        &guessed_standalone,
        context.is_org,
    ) {
        Ok(context)
    } else {
        Err("Access denied for this bucket")
    }
}

fn is_acceptable(requested: Option<&str>, canonical: &str, guessed: &str, is_org: bool) -> bool {
    match requested {
        None => true,
        Some(bucket) => bucket == canonical || (is_org && bucket == guessed),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct DriveBucketParams {
    /// Bot whose Drive tree the caller wants to address. The suite sends
    /// `window.__INITIAL_BOT_NAME__` here.
    pub bot: Option<String>,
    /// Optional current client bucket, accepted only when it is already the
    /// authoritative bucket or the legacy standalone guess for this bot.
    pub bucket: Option<String>,
}

/// `GET /api/files/bucket?bot=<name>[&bucket=<current>]`
///
/// Returns the authoritative bucket, branch workspace and key prefix for the
/// bot, so a client that failed to discover the `.gborg` layout can ask the
/// server instead of fabricating `{bot}.gbai` (#2928).
pub async fn resolve_drive_bucket(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<DriveBucketParams>,
) -> Result<Json<DriveBucketContext>, (StatusCode, Json<serde_json::Value>)> {
    let bot = params
        .bot
        .filter(|name| !name.trim().is_empty())
        .or_else(|| user.metadata.get("bot_name").cloned())
        .unwrap_or_else(|| {
            state
                .bucket_name
                .trim_end_matches(".gborg")
                .trim_end_matches(".gbai")
                .to_string()
        });
    if bot.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "No bot specified"));
    }
    let mut conn = state.conn.get().map_err(|_| {
        err(
            StatusCode::SERVICE_UNAVAILABLE,
            "Drive storage not configured",
        )
    })?;
    // DB lookup, never name arithmetic: org and branch slugs decide the
    // bucket and the workspace prefix. No row (or no DB) is a failure, not
    // a standalone fallback.
    let location = load_bot_location(&mut conn, &bot)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Bot Drive location is not available"))?;
    let allowed = allowed_buckets_for(&state, &user);
    let context = reconcile_bucket(
        &location,
        &bot,
        params.bucket.as_deref(),
        &allowed,
        is_platform_admin(&state, &user),
    )
    .map_err(|message| err(StatusCode::FORBIDDEN, message))?;
    Ok(Json(context))
}
