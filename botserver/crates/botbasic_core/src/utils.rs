use rhai::Dynamic;
use serde_json::Value;
use std::path::PathBuf;

pub fn to_array(dynamic: &Dynamic) -> Vec<Dynamic> {
    if let Some(array) = dynamic.clone().try_cast::<Vec<Dynamic>>() {
        array
    } else if let Some(map) = dynamic.clone().try_cast::<rhai::Map>() {
        map.into_values().collect()
    } else {
        vec![dynamic.clone()]
    }
}

pub fn json_value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::from(n.to_string())
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => {
            let vec: Vec<Dynamic> = arr.iter().map(json_value_to_dynamic).collect();
            Dynamic::from(vec)
        }
        Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.clone().into(), json_value_to_dynamic(v));
            }
            Dynamic::from(rhai_map)
        }
    }
}

pub fn dynamic_to_json(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if let Ok(b) = value.as_bool() {
        Value::Bool(b)
    } else if let Ok(i) = value.as_int() {
        Value::from(i)
    } else if let Ok(f) = value.as_float() {
        Value::from(f)
    } else if let Some(s) = value.clone().try_cast::<String>() {
        Value::String(s)
    } else if let Some(arr) = value.clone().try_cast::<Vec<Dynamic>>() {
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if let Some(map) = value.clone().try_cast::<rhai::Map>() {
        let mut json_map = serde_json::Map::new();
        for (k, v) in map {
            json_map.insert(k.to_string(), dynamic_to_json(&v));
        }
        Value::Object(json_map)
    } else {
        Value::String(value.to_string())
    }
}

pub fn convert_date_to_iso_format(date_str: &str) -> String {
    let trimmed = date_str.trim();
    // Already ISO format (yyyy-MM-dd or yyyy-MM-dd HH:mm:ss)
    if trimmed.len() >= 10 && trimmed.as_bytes()[4] == b'-' && trimmed.as_bytes()[7] == b'-' {
        return trimmed.to_string();
    }
    // Brazilian format: dd/mm/aaaa or dd/mm/aaaa HH:mm:ss
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.splitn(3, '/').collect();
        if parts.len() == 3 {
            let day = parts[0];
            let month = parts[1];
            let rest = parts[2];
            let year = if rest.contains(' ') {
                rest.split(' ').next().unwrap_or(rest)
            } else {
                rest
            };
            // Validate numbers
            if day.chars().all(|c| c.is_ascii_digit()) && month.chars().all(|c| c.is_ascii_digit()) {
                let d: i32 = day.parse().unwrap_or(0);
                let m: i32 = month.parse().unwrap_or(0);
                if (1..=31).contains(&d) && (1..=12).contains(&m) {
                    let time_part = if rest.contains(' ') {
                        " ".to_string() + rest.split(' ').nth(1).unwrap_or("")
                    } else {
                        String::new()
                    };
                    return format!("{:04}-{:02}-{:02}{}", year, m, d, time_part);
                }
            }
        }
    }
    date_str.to_string()
}

pub fn get_work_path() -> String {
    std::env::var("GBO_WORK_PATH").unwrap_or_else(|_| "/opt/gbo/work".to_string())
}

/// Returns the current org ID for bot file path isolation.
/// Always returns Uuid::nil() until real multi-tenant auth is implemented.
pub fn current_org_id() -> uuid::Uuid {
    uuid::Uuid::nil()
}

/// Build a relative bot path with org isolation (.gborg wrapping).
/// Returns: "{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_bot_path(org_id: impl std::fmt::Display, bot_bucket: &str, sub_path: &str) -> String {
    format!("{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

/// Bucket suffix for an org tenant workspace.
pub const GBORG_SUFFIX: &str = ".gborg";

/// Where a bot's Drive files physically live.
///
/// A bot that belongs to an org is materialised in the org workspace bucket
/// (`{slug}.gborg`), with the bot directory as an S3 key prefix
/// (`{bot}.gbai/{bot}.gbdrive/…`). A standalone bot owns its `{bot}.gbai`
/// bucket outright. Both shapes are real in prod, so the bucket cannot be
/// derived from the bot name alone — the org slug decides.
///
/// `org_slug: None` → standalone layout, `Some(slug)` → org layout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BotDriveLocation {
    pub bucket: String,
    /// Prefix every Drive key of this bot starts with, including the trailing
    /// separator (empty for the standalone layout, where `{bot}.gbdrive/` is
    /// already the whole key).
    pub key_prefix: String,
    /// Prefix every key of this bot starts with — not just Drive files, but
    /// also `.gbdialog`, `.gbkb` and `.gbot`. Empty for a standalone bot
    /// (its bucket holds only its own tree), `{bot}.gbai/` inside a shared
    /// org bucket. Needed to seed or delete a bot without touching its
    /// sibling bots in the same org workspace.
    pub bot_prefix: String,
}

impl BotDriveLocation {
    /// Full S3 key for a Drive-relative path such as `media/2026/10/a.mp4`.
    pub fn key_for(&self, path: &str) -> String {
        format!("{}{}", self.key_prefix, path.trim_start_matches('/'))
    }

    /// Bucket plus the bot's Drive directory, i.e. what a LIST must target to
    /// enumerate this bot's files.
    pub fn drive_prefix(&self) -> String {
        self.key_prefix.clone()
    }
}

/// Resolves the bucket and key prefix for a bot's Drive files.
///
/// `org_slug` must be the organization slug (`organizations.slug`), not the
/// `org_id` UUID — prod buckets are named after the slug (`beiner.gborg`).
pub fn resolve_bot_drive_location(bot_name: &str, org_slug: Option<&str>) -> BotDriveLocation {
    match org_slug.map(str::trim).filter(|s| !s.is_empty()) {
        Some(slug) => BotDriveLocation {
            bucket: format!("{slug}{GBORG_SUFFIX}"),
            key_prefix: format!("{bot_name}.gbai/{bot_name}.gbdrive/"),
            bot_prefix: format!("{bot_name}.gbai/"),
        },
        None => BotDriveLocation {
            bucket: format!("{bot_name}.gbai"),
            key_prefix: format!("{bot_name}.gbdrive/"),
            bot_prefix: String::new(),
        },
    }
}

/// Looks up a bot's name and org slug, then resolves its Drive location.
///
/// This is the single replacement for the `format!("{bot_name}.gbai")` pattern
/// that every file keyword used: the bucket cannot be derived from the bot
/// name alone, because an org-hosted bot lives under `{slug}.gborg`. Falls back
/// to the standalone layout when the bot has no org or the lookup fails, so a
/// transient DB error degrades to the legacy path instead of writing nowhere.
pub fn bot_drive_location_for(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    bot_id: uuid::Uuid,
) -> BotDriveLocation {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use botbasic_types::schema::organizations::dsl::{
        organizations, org_id as org_id_col, slug as org_slug_col,
    };
    // Aliased: the bare names `name`/`org_id` are diesel unit structs, which a
    // local binding is not allowed to shadow.
    use botbasic_types::schema::bots::dsl::{
        bots, id as bot_id_col, name as bot_name_col, org_id as bot_org_id_col,
    };

    // Field order matches the `.select((bot_name_col, bot_org_id_col))` tuple.
    #[derive(diesel::Queryable)]
    struct BotOrgRow {
        name: String,
        org_id: uuid::Uuid,
    }

    let fetched = (|| -> Result<(String, Option<String>), diesel::result::Error> {
        let bot = bots
            .filter(bot_id_col.eq(bot_id))
            .select((bot_name_col, bot_org_id_col))
            .first::<BotOrgRow>(conn)?;

        // A bot without an org is a standalone bot: it owns its `.gbai` bucket.
        if bot.org_id == uuid::Uuid::nil() {
            return Ok((bot.name, None));
        }

        let org_slug: Option<String> = organizations
            .filter(org_id_col.eq(bot.org_id))
            .select(org_slug_col)
            .first::<String>(conn)
            .ok();

        Ok((bot.name, org_slug))
    })();

    match fetched {
        Ok((bot_name, org_slug)) => resolve_bot_drive_location(&bot_name, org_slug.as_deref()),
        Err(e) => {
            // Without the bot name no correct key can be built, so surface it:
            // silently writing to a wrong key is how files become unreachable.
            log::error!("bot_drive_location_for: failed to resolve bot {bot_id}: {e}");
            resolve_bot_drive_location("", None)
        }
    }
}

/// Same as [`bot_drive_location_for`], for callers that only know the bot
/// *name* (URL path segments such as the media player's `/{bot}/{path}`).
///
/// The name is resolved to the bot row first, so the org slug — and therefore
/// the `{slug}.gborg` bucket — is still honoured.
pub fn bot_drive_location_for_name(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    bot_name: &str,
) -> BotDriveLocation {
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
    }

    let bot_id = diesel::sql_query("SELECT id FROM bots WHERE name = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .get_result::<BotIdRow>(conn)
        .map(|row| row.id);

    match bot_id {
        Ok(bot_id) => bot_drive_location_for(conn, bot_id),
        Err(e) => {
            log::error!("bot_drive_location_for_name: failed to resolve bot '{bot_name}': {e}");
            resolve_bot_drive_location(bot_name, None)
        }
    }
}

/// Build an absolute bot path with org isolation.
/// Returns: "{work_root}/{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_absolute_bot_path(
    work_root: &str,
    org_id: impl std::fmt::Display,
    bot_bucket: &str,
    sub_path: &str,
) -> String {
    format!("{work_root}/{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

/// Get work path with org isolation suffix.
/// Returns: "{work_path}/{org_id}.gborg/"
pub fn get_org_work_path(org_id: impl std::fmt::Display) -> String {
    format!("{}/{org_id}.gborg/", get_work_path())
}

pub fn get_content_type(path: &str) -> String {
    let p = PathBuf::from(path);
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }.to_string()
}

pub fn get_default_bot(state: &std::sync::Arc<dyn botbasic_types::BasicRuntime>) -> Result<serde_json::Value, String> {
    let mut conn = state.db_pool().get().map_err(|e| e.to_string())?;
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Text;
    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = Text)]
        bot_id: String,
    }
    let rows = sql_query("SELECT bot_id FROM bots LIMIT 1")
        .load::<BotRow>(&mut conn)
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .next()
        .map(|r| serde_json::json!({"bot_id": r.bot_id}))
        .ok_or_else(|| "No bots found".to_string())
}

pub fn parse_filter(filter_str: &str) -> Result<(String, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    let trimmed = filter_str.trim();
    if trimmed == "1=1" {
        return Ok(("TRUE".to_string(), vec![]));
    }
    let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Invalid filter format. Expected 'KEY=VALUE'".into());
    }
    let column = parts[0].trim();
    let value = parts[1].trim();
    if !column.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Invalid column name in filter".into());
    }
    Ok((format!("{} = $1", column), vec![value.to_string()]))
}

#[cfg(test)]
mod bot_drive_location_tests {
    use super::*;

    #[test]
    fn standalone_bot_owns_its_gbai_bucket() {
        let loc = resolve_bot_drive_location("beiner", None);
        assert_eq!(loc.bucket, "beiner.gbai");
        assert_eq!(loc.key_prefix, "beiner.gbdrive/");
        assert_eq!(loc.key_for("media/2026/10/a.mp4"), "beiner.gbdrive/media/2026/10/a.mp4");
    }

    #[test]
    fn org_bot_uses_gborg_bucket_with_gbai_prefix() {
        // Prod shape: org slug `beiner`, bot `beiner` -> bucket `beiner.gborg`,
        // key prefix `beiner.gbai/beiner.gbdrive/`.
        let loc = resolve_bot_drive_location("beiner", Some("beiner"));
        assert_eq!(loc.bucket, "beiner.gborg");
        assert_eq!(loc.key_prefix, "beiner.gbai/beiner.gbdrive/");
        assert_eq!(
            loc.key_for("media/2026/10/a.mp4"),
            "beiner.gbai/beiner.gbdrive/media/2026/10/a.mp4"
        );
    }

    #[test]
    fn org_bot_differs_from_standalone_bot() {
        let standalone = resolve_bot_drive_location("beiner", None);
        let org = resolve_bot_drive_location("beiner", Some("beiner"));
        assert_ne!(standalone.bucket, org.bucket);
    }

    #[test]
    fn blank_or_missing_slug_falls_back_to_standalone() {
        for slug in [Some(""), Some("   ")] {
            let loc = resolve_bot_drive_location("beiner", slug);
            assert_eq!(loc.bucket, "beiner.gbai", "slug {slug:?} must not produce .gborg");
        }
    }

    #[test]
    fn key_for_tolerates_leading_slash() {
        let org = resolve_bot_drive_location("beiner", Some("beiner"));
        assert_eq!(
            org.key_for("/media/a.mp4"),
            "beiner.gbai/beiner.gbdrive/media/a.mp4"
        );
    }

    #[test]
    fn standalone_bot_prefix_is_empty() {
        // A standalone bucket holds only this bot's tree, so seeding and
        // deleting operate on the bucket root.
        let loc = resolve_bot_drive_location("beiner", None);
        assert_eq!(loc.bot_prefix, "");
    }

    #[test]
    fn org_bot_prefix_scopes_to_the_bot_inside_the_shared_bucket() {
        // Deleting an org bot must remove only `{beiner}.gbai/…`, never a
        // sibling bot's tree in the same `.gborg` bucket.
        let loc = resolve_bot_drive_location("beiner", Some("beiner"));
        assert_eq!(loc.bot_prefix, "beiner.gbai/");
    }

    #[test]
    fn org_bot_prefix_and_drive_prefix_agree_on_the_bot_tree() {
        // The Drive prefix is the bot prefix plus the Drive directory, so a
        // Drive key is always inside the bot's own tree.
        let loc = resolve_bot_drive_location("beiner", Some("beiner"));
        let drive_key = loc.key_for("media/a.mp4");
        assert!(
            drive_key.starts_with(&loc.bot_prefix),
            "{drive_key} must live under {}",
            loc.bot_prefix
        );
    }

    #[test]
    fn sibling_bots_in_one_org_have_disjoint_trees() {
        let a = resolve_bot_drive_location("bot_a", Some("acme"));
        let b = resolve_bot_drive_location("bot_b", Some("acme"));
        assert_eq!(a.bucket, b.bucket, "siblings share the org bucket");
        assert!(
            !b.key_for("x.txt").starts_with(&a.bot_prefix),
            "bot_b's key must not fall inside bot_a's tree"
        );
    }
}
