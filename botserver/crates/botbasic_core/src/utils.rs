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

/// Suffix `drive_monitors`' legacy naming appends to a tenant slug when it
/// creates an organization row (`cristo` -> `cristo-org`). It is a database
/// alias, never part of a bucket name.
pub const LEGACY_ORG_SUFFIX: &str = "-org";

/// Bucket that hosts an org's workspaces: `{base}.gborg`.
///
/// `organizations.slug` carries two conventions: signup stores the plain name
/// (`beiner`), while Drive discovery's legacy naming appends `-org`
/// (`cristo-org`) — `ensure_tenant_and_org` treats both as aliases of the same
/// org (#779). The physical bucket is always named after the plain tenant
/// (`cristo.gborg`), so the legacy suffix is stripped before building the name.
pub fn org_drive_bucket(org_slug: &str) -> String {
    let slug = org_slug.trim();
    let stripped = slug.strip_suffix(LEGACY_ORG_SUFFIX).unwrap_or(slug);
    let base = if stripped.is_empty() { slug } else { stripped };
    format!("{base}{GBORG_SUFFIX}")
}

/// Where a bot's Drive files physically live.
///
/// A bot that belongs to an org is materialised in the org workspace bucket
/// (`{slug}.gborg`), inside its branch workspace (`{branch}.gbai/`), with the
/// bot directory as an S3 key prefix (`{branch}.gbai/{bot}.gbdrive/…`). A
/// standalone bot owns its `{bot}.gbai` bucket outright. Both shapes are real
/// in prod, so the bucket cannot be derived from the bot name alone — the org
/// and the branch decide.
///
/// `org_slug: None` → standalone layout, `Some(slug)` → org layout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BotDriveLocation {
    pub bucket: String,
    /// Prefix every Drive key of this bot starts with, including the trailing
    /// separator (empty for the standalone layout, where `{bot}.gbdrive/` is
    /// already the whole key).
    pub key_prefix: String,
    /// Directory prefix of this bot's `{bot}.gb*` folders — not just Drive
    /// files, but also `.gbdialog`, `.gbkb` and `.gbot`. Empty for a
    /// standalone bot (its bucket holds only its own tree), the branch
    /// workspace (`{branch}.gbai/`) inside a shared org bucket. Callers that
    /// seed or delete a single bot must append `{bot}.gb` themselves
    /// (`bot_tree_prefix`) so sibling bots of the same branch survive.
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

    /// Exact prefix of this bot's whole tree (`{…}{bot}.gb`): every
    /// `{bot}.gb*` directory of the bot, and nothing of a sibling. Safe to
    /// scan or delete — `cristo.gb` does not match `cristo-test.gbdrive/…`.
    pub fn bot_tree_prefix(&self, bot_name: &str) -> String {
        format!("{}{bot_name}.gb", self.bot_prefix)
    }
}

/// Resolves the bucket and key prefixes for a bot's Drive tree.
///
/// `branch_slug` and `org_slug` must be the `branches.slug` and
/// `organizations.slug` columns, never the UUIDs. The branch decides the
/// workspace prefix inside the org bucket (`{branch}.gbai/`) — a bot whose
/// branch differs from its name (`oppbot` in branch `opportunity-oppbot`)
/// would otherwise write outside its own workspace. It falls back to the bot
/// name only when the caller has no branch information, which is correct for
/// every single-bot branch (`beiner`, `cristo`, …).
pub fn resolve_bot_drive_location(
    bot_name: &str,
    branch_slug: Option<&str>,
    org_slug: Option<&str>,
) -> BotDriveLocation {
    match org_slug.map(str::trim).filter(|s| !s.is_empty()) {
        Some(slug) => {
            let workspace = branch_slug
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(bot_name);
            BotDriveLocation {
                bucket: org_drive_bucket(slug),
                key_prefix: format!("{workspace}.gbai/{bot_name}.gbdrive/"),
                bot_prefix: format!("{workspace}.gbai/"),
            }
        }
        None => BotDriveLocation {
            bucket: format!("{bot_name}.gbai"),
            key_prefix: format!("{bot_name}.gbdrive/"),
            bot_prefix: String::new(),
        },
    }
}

/// Row shape shared by the bot lookups: the bot name plus the two slugs that
/// decide its Drive layout. Both joins are LEFT so a bot whose org or branch
/// row is missing still resolves (to the standalone layout) instead of failing
/// the whole lookup. Raw SQL because the diesel join DSL for `branches` does
/// not type-check against the partial schema in `botbasic_types`.
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BotDriveScopeRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    branch_slug: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    org_slug: Option<String>,
}

const BOT_DRIVE_SCOPE_SELECT: &str =
    "SELECT b.name, br.slug AS branch_slug, o.slug AS org_slug \
     FROM bots b \
     LEFT JOIN branches br ON br.id = b.branch_id \
     LEFT JOIN organizations o ON o.org_id = b.org_id";

fn resolve_from_scope_row(row: BotDriveScopeRow) -> BotDriveLocation {
    resolve_bot_drive_location(
        &row.name,
        row.branch_slug.as_deref(),
        row.org_slug.as_deref(),
    )
}

/// Looks up a bot's name, branch and org slug, then resolves its Drive location.
///
/// This is the single replacement for the `format!("{bot_name}.gbai")` pattern
/// that every file keyword used: neither the bucket nor the workspace prefix
/// can be derived from the bot name alone, because an org-hosted bot lives in
/// `{slug}.gborg` under its branch's `{branch}.gbai/` workspace. Falls back to
/// the standalone layout when the bot has no org or the lookup fails, so a
/// transient DB error degrades to the legacy path instead of writing nowhere.
pub fn bot_drive_location_for(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    bot_id: uuid::Uuid,
) -> BotDriveLocation {
    use diesel::prelude::*;

    let row = diesel::sql_query(format!("{BOT_DRIVE_SCOPE_SELECT} WHERE b.id = $1 LIMIT 1"))
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<BotDriveScopeRow>(conn);

    match row {
        Ok(row) => resolve_from_scope_row(row),
        Err(e) => {
            // Without the bot name no correct key can be built, so surface it:
            // silently writing to a wrong key is how files become unreachable.
            log::error!("bot_drive_location_for: failed to resolve bot {bot_id}: {e}");
            resolve_bot_drive_location("", None, None)
        }
    }
}

/// Same as [`bot_drive_location_for`], for callers that only know the bot
/// *name* (URL path segments such as the media player's `/{bot}/{path}`).
///
/// The name is resolved to the bot row first, so the org slug — and therefore
/// the `{slug}.gborg` bucket and `{branch}.gbai/` workspace — is still honoured.
pub fn bot_drive_location_for_name(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    bot_name: &str,
) -> BotDriveLocation {
    use diesel::prelude::*;

    let row = diesel::sql_query(format!("{BOT_DRIVE_SCOPE_SELECT} WHERE b.name = $1 LIMIT 1"))
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .get_result::<BotDriveScopeRow>(conn);

    match row {
        Ok(row) => resolve_from_scope_row(row),
        Err(e) => {
            log::error!("bot_drive_location_for_name: failed to resolve bot '{bot_name}': {e}");
            resolve_bot_drive_location(bot_name, None, None)
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
        let loc = resolve_bot_drive_location("beiner", None, None);
        assert_eq!(loc.bucket, "beiner.gbai");
        assert_eq!(loc.key_prefix, "beiner.gbdrive/");
        assert_eq!(loc.key_for("media/2026/10/a.mp4"), "beiner.gbdrive/media/2026/10/a.mp4");
    }

    #[test]
    fn org_bot_uses_gborg_bucket_with_branch_prefix() {
        // Prod shape: org slug `beiner`, branch `beiner`, bot `beiner` ->
        // bucket `beiner.gborg`, key prefix `beiner.gbai/beiner.gbdrive/`.
        let loc = resolve_bot_drive_location("beiner", Some("beiner"), Some("beiner"));
        assert_eq!(loc.bucket, "beiner.gborg");
        assert_eq!(loc.key_prefix, "beiner.gbai/beiner.gbdrive/");
        assert_eq!(
            loc.key_for("media/2026/10/a.mp4"),
            "beiner.gbai/beiner.gbdrive/media/2026/10/a.mp4"
        );
    }

    #[test]
    fn legacy_org_suffix_is_not_part_of_the_bucket_name() {
        // Drive discovery creates orgs as `{tenant}-org` (`ensure_tenant_and_org`),
        // but the physical bucket is `{tenant}.gborg`: cristo-org -> cristo.gborg.
        let loc = resolve_bot_drive_location("cristo", Some("cristo"), Some("cristo-org"));
        assert_eq!(loc.bucket, "cristo.gborg");
        assert_eq!(loc.key_prefix, "cristo.gbai/cristo.gbdrive/");
        assert_eq!(loc.bot_prefix, "cristo.gbai/");
    }

    #[test]
    fn bucket_base_strips_only_a_trailing_legacy_suffix() {
        assert_eq!(org_drive_bucket("cristo-org"), "cristo.gborg");
        assert_eq!(org_drive_bucket("beiner"), "beiner.gborg");
        assert_eq!(org_drive_bucket("default"), "default.gborg");
        assert_eq!(org_drive_bucket("my-organization"), "my-organization.gborg");
        assert_eq!(org_drive_bucket("org-org"), "org.gborg");
    }

    #[test]
    fn branch_decides_the_workspace_prefix_not_the_bot_name() {
        // Prod shape: bot `oppbot` lives in branch `opportunity-oppbot`, whose
        // workspace inside `opportunity-oppbot.gborg` is `opportunity-oppbot.gbai/`.
        let loc = resolve_bot_drive_location(
            "oppbot",
            Some("opportunity-oppbot"),
            Some("opportunity-oppbot-org"),
        );
        assert_eq!(loc.bucket, "opportunity-oppbot.gborg");
        assert_eq!(loc.key_prefix, "opportunity-oppbot.gbai/oppbot.gbdrive/");
        assert_eq!(loc.bot_prefix, "opportunity-oppbot.gbai/");
    }

    #[test]
    fn second_branch_of_an_org_shares_the_org_bucket() {
        // `pragmatismo.gborg` hosts both `pragmatismo.gbai/` and
        // `PragmatismoGB.gbai/` — the org owns the bucket, the branch the workspace.
        let loc = resolve_bot_drive_location(
            "PragmatismoGB",
            Some("PragmatismoGB"),
            Some("pragmatismo"),
        );
        assert_eq!(loc.bucket, "pragmatismo.gborg");
        assert_eq!(loc.key_prefix, "PragmatismoGB.gbai/PragmatismoGB.gbdrive/");
    }

    #[test]
    fn default_org_bot_resolves_to_the_org_workspace() {
        // The instance `default` bot carries org slug `default`; its canonical
        // tree is `default.gborg/default.gbai/…` — the location the archive
        // pass and the htmx seeding in drive_utils already write to.
        let loc = resolve_bot_drive_location("default", Some("default"), Some("default"));
        assert_eq!(loc.bucket, "default.gborg");
        assert_eq!(loc.key_prefix, "default.gbai/default.gbdrive/");
    }

    #[test]
    fn branch_falls_back_to_the_bot_name_when_unknown() {
        let loc = resolve_bot_drive_location("beiner", None, Some("beiner"));
        assert_eq!(loc.key_prefix, "beiner.gbai/beiner.gbdrive/");
        for blank in [Some(""), Some("   ")] {
            let loc = resolve_bot_drive_location("beiner", blank, Some("beiner"));
            assert_eq!(
                loc.key_prefix, "beiner.gbai/beiner.gbdrive/",
                "branch {blank:?} must fall back to the bot name"
            );
        }
    }

    #[test]
    fn org_bot_differs_from_standalone_bot() {
        let standalone = resolve_bot_drive_location("beiner", Some("beiner"), None);
        let org = resolve_bot_drive_location("beiner", Some("beiner"), Some("beiner"));
        assert_ne!(standalone.bucket, org.bucket);
    }

    #[test]
    fn blank_or_missing_slug_falls_back_to_standalone() {
        for slug in [None, Some(""), Some("   ")] {
            let loc = resolve_bot_drive_location("beiner", Some("beiner"), slug);
            assert_eq!(loc.bucket, "beiner.gbai", "slug {slug:?} must not produce .gborg");
        }
    }

    #[test]
    fn key_for_tolerates_leading_slash() {
        let org = resolve_bot_drive_location("beiner", Some("beiner"), Some("beiner"));
        assert_eq!(
            org.key_for("/media/a.mp4"),
            "beiner.gbai/beiner.gbdrive/media/a.mp4"
        );
    }

    #[test]
    fn standalone_bot_prefix_is_empty() {
        // A standalone bucket holds only this bot's tree, so seeding and
        // deleting operate on the bucket root.
        let loc = resolve_bot_drive_location("beiner", None, None);
        assert_eq!(loc.bot_prefix, "");
    }

    #[test]
    fn org_bot_prefix_is_the_branch_workspace() {
        // Sibling bots of one branch share the workspace prefix; the bot name
        // is appended by the callers that seed or delete a bot's tree.
        let loc = resolve_bot_drive_location("beiner", Some("beiner"), Some("beiner"));
        assert_eq!(loc.bot_prefix, "beiner.gbai/");
    }

    #[test]
    fn org_bot_prefix_and_drive_prefix_agree_on_the_bot_tree() {
        // The Drive prefix is the bot prefix plus the Drive directory, so a
        // Drive key is always inside the bot's own tree.
        let loc = resolve_bot_drive_location("beiner", Some("beiner"), Some("beiner"));
        let drive_key = loc.key_for("media/a.mp4");
        assert!(
            drive_key.starts_with(&loc.bot_prefix),
            "{drive_key} must live under {}",
            loc.bot_prefix
        );
    }

    #[test]
    fn sibling_bots_in_one_org_have_disjoint_trees() {
        // Siblings share bucket AND branch workspace, so disjointness is a
        // property of the per-bot tree prefix, not of the workspace prefix.
        let a = resolve_bot_drive_location("bot_a", Some("acme"), Some("acme"));
        let b = resolve_bot_drive_location("bot_b", Some("acme"), Some("acme"));
        assert_eq!(a.bucket, b.bucket, "siblings share the org bucket");
        assert_eq!(a.bot_prefix, b.bot_prefix, "and the branch workspace");
        assert!(
            !b.key_for("x.txt").starts_with(&a.bot_tree_prefix("bot_a")),
            "bot_b's key must not fall inside bot_a's tree"
        );
    }

    #[test]
    fn bot_tree_prefix_does_not_reach_a_sibling_bot() {
        // Deleting `cristo` must not touch `cristo-test`: the tree prefix ends
        // at `cristo.gb`, and `cristo-test.gbdrive/…` does not match it.
        let loc = resolve_bot_drive_location("cristo", Some("cristo"), Some("cristo-org"));
        let tree = loc.bot_tree_prefix("cristo");
        assert_eq!(tree, "cristo.gbai/cristo.gb");
        assert!(
            !format!("cristo.gbai/cristo-test.gbdrive/a.bas").starts_with(&tree),
            "sibling bot must stay outside the delete prefix"
        );
        assert!(
            format!("cristo.gbai/{}", "cristo.gbdialog/x.bas").starts_with(&tree),
            "the bot's own tree must be covered"
        );
    }

    #[test]
    fn standalone_tree_prefix_covers_the_bucket_root_tree() {
        let loc = resolve_bot_drive_location("beiner", None, None);
        assert_eq!(loc.bot_tree_prefix("beiner"), "beiner.gb");
        assert!(loc.key_for("media/a.mp4").starts_with(&loc.bot_tree_prefix("beiner")));
    }
}
