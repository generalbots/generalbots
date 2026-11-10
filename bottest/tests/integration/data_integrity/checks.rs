//! (a) Migrations produce the schema `botschema` declares — and the seeded
//! dataset really is a nested org -> branch -> bot tree.

use uuid::Uuid;

use super::common::{connect, database_url, ensure_dataset, repo_root, strict};
use super::{dataset, migrations, schema};

/// Every migration on disk was applied, and nothing else was.
#[test]
fn migrations_apply_from_scratch() {
    let strict = strict();
    let Some(mut conn) = connect(strict) else {
        return;
    };
    let dir = repo_root().join("botserver").join("migrations");
    let on_disk = match migrations::migration_versions(&dir) {
        Ok(versions) => versions,
        Err(e) => skip_or_fail!(strict, (), "cannot read {}: {e}", dir.display()),
    };
    let applied = match migrations::applied_versions(&mut conn) {
        Ok(versions) => versions,
        Err(e) => skip_or_fail!(
            strict,
            (),
            "{e} — apply the migrations first: (cd botserver && diesel migration run)"
        ),
    };

    assert!(
        on_disk.len() > 100,
        "only {} migration directories found under {} — wrong working tree?",
        on_disk.len(),
        dir.display()
    );
    let missing: Vec<&String> = on_disk.iter().filter(|v| !applied.contains(v)).collect();
    assert!(
        missing.is_empty(),
        "{} migration(s) were never applied, e.g. {:?}",
        missing.len(),
        missing.iter().take(5).collect::<Vec<_>>()
    );
    let extra: Vec<&String> = applied.iter().filter(|v| !on_disk.contains(v)).collect();
    assert!(
        extra.is_empty(),
        "the database has {} migration(s) that are not on disk, e.g. {:?}",
        extra.len(),
        extra.iter().take(5).collect::<Vec<_>>()
    );
    assert_eq!(
        on_disk.len(),
        applied.len(),
        "migration count on disk ({}) differs from applied ({})",
        on_disk.len(),
        applied.len()
    );
    eprintln!(
        "ok: {} migrations applied, every one of them present on disk",
        applied.len()
    );
}

/// The migrated schema is the schema `botschema` declares.
#[test]
fn live_schema_matches_botschema() {
    let strict = strict();
    let Some(mut conn) = connect(strict) else {
        return;
    };
    let schema_dir = repo_root().join("botschema").join("src");
    let declared = match schema::read_botschema(&schema_dir) {
        Ok(tables) => tables,
        Err(e) => skip_or_fail!(strict, (), "cannot read botschema: {e}"),
    };
    assert!(
        declared.len() > 250,
        "the botschema parser found only {} tables — parser regression, not a schema fact",
        declared.len()
    );
    let live = match schema::read_live(&mut conn) {
        Ok(tables) => tables,
        Err(e) => skip_or_fail!(strict, (), "{e}"),
    };
    let drift = schema::compare(&declared, &live);
    for line in drift.additive.iter().take(20) {
        eprintln!("additive drift (allowed, Diesel never selects it): {line}");
    }
    if drift.additive.len() > 20 {
        eprintln!(
            "... {} more additive difference(s) suppressed",
            drift.additive.len() - 20
        );
    }
    assert!(
        drift.fatal.is_empty(),
        "{} fatal schema drift(s) between botschema and the migrated database:\n{}",
        drift.fatal.len(),
        drift.fatal.join("\n")
    );
    eprintln!(
        "ok: {} tables and {} declared columns match the migrated database",
        declared.len(),
        declared.values().map(|cols| cols.len()).sum::<usize>()
    );
}

/// The dataset is a real org -> branch -> bot tree, not flat UUIDs.
#[test]
fn manifest_scopes_cover_every_org_branch_and_bot() {
    let strict = strict();
    let Some(mut conn) = connect(strict) else {
        return;
    };
    let Some(url) = database_url(strict) else {
        return;
    };
    let Some(manifest) = ensure_dataset(strict, &mut conn, &url) else {
        return;
    };

    let mut failures = Vec::new();
    for org in &manifest.orgs {
        let org_id = match Uuid::parse_str(&org.org_id) {
            Ok(id) => id,
            Err(e) => {
                failures.push(format!("{}: bad org id: {e}", org.slug));
                continue;
            }
        };
        let org_filter = format!("org_id = '{org_id}'::uuid");
        let tenant_id = match Uuid::parse_str(&org.tenant_id) {
            Ok(id) => id,
            Err(e) => {
                failures.push(format!("{}: bad tenant id: {e}", org.slug));
                continue;
            }
        };
        let checks: [(&str, &str, Uuid); 2] = [
            (
                "organization",
                "SELECT count(*) AS row_count FROM organizations \
                 WHERE org_id = $1 AND slug LIKE 'integrity-%'",
                org_id,
            ),
            (
                "tenant",
                "SELECT count(*) AS row_count FROM tenants WHERE id = $1",
                tenant_id,
            ),
        ];
        for (what, sql, id) in checks {
            match dataset::exists_with(&mut conn, sql, id) {
                Ok(true) => {}
                Ok(false) => failures.push(format!("{}: {what} row is missing", org.slug)),
                Err(e) => failures.push(format!("{}: {what}: {e}", org.slug)),
            }
        }
        let branch_list = org
            .branch_ids
            .iter()
            .map(|id| format!("'{id}'::uuid"))
            .collect::<Vec<_>>()
            .join(", ");
        for raw in &org.branch_ids {
            let id = match Uuid::parse_str(raw) {
                Ok(id) => id,
                Err(e) => {
                    failures.push(format!("{}: bad branch id: {e}", org.slug));
                    continue;
                }
            };
            let sql = format!(
                "SELECT count(*) AS row_count FROM branches WHERE id = $1 AND {org_filter}"
            );
            match dataset::exists_with(&mut conn, &sql, id) {
                Ok(true) => {}
                Ok(false) => {
                    failures.push(format!("{}: branch {id} is not under the org", org.slug))
                }
                Err(e) => failures.push(format!("{}: branch {id}: {e}", org.slug)),
            }
        }
        for raw in &org.bot_ids {
            let id = match Uuid::parse_str(raw) {
                Ok(id) => id,
                Err(e) => {
                    failures.push(format!("{}: bad bot id: {e}", org.slug));
                    continue;
                }
            };
            let sql = format!(
                "SELECT count(*) AS row_count FROM bots WHERE id = $1 AND {org_filter} \
                 AND branch_id IN ({branch_list})"
            );
            match dataset::exists_with(&mut conn, &sql, id) {
                Ok(true) => {}
                Ok(false) => failures.push(format!(
                    "{}: bot {id} is not nested under one of the org's branches",
                    org.slug
                )),
                Err(e) => failures.push(format!("{}: bot {id}: {e}", org.slug)),
            }
        }
        for raw in &org.user_ids {
            let id = match Uuid::parse_str(raw) {
                Ok(id) => id,
                Err(e) => {
                    failures.push(format!("{}: bad user id: {e}", org.slug));
                    continue;
                }
            };
            match dataset::exists_with(
                &mut conn,
                "SELECT count(*) AS row_count FROM users WHERE id = $1",
                id,
            ) {
                Ok(true) => {}
                Ok(false) => failures.push(format!("{}: user {id} is missing", org.slug)),
                Err(e) => failures.push(format!("{}: user {id}: {e}", org.slug)),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} dataset tree failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    eprintln!(
        "ok: {} orgs, {} branches, {} bots, {} users form the seeded tree",
        manifest.orgs.len(),
        manifest
            .orgs
            .iter()
            .map(|o| o.branch_ids.len())
            .sum::<usize>(),
        manifest.orgs.iter().map(|o| o.bot_ids.len()).sum::<usize>(),
        manifest.orgs.iter().map(|o| o.user_ids.len()).sum::<usize>()
    );
}
