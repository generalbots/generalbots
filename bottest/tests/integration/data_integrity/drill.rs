//! (c) Restore drill: `pg_dump` the seeded database, `pg_restore` it into a
//! scratch database, and prove every tenant kept its rows and its checksum.
//!
//! The scratch database is dropped afterwards, so the drill is repeatable on the
//! same instance. When the client tools are missing the test skips with the name
//! of the missing tool — or fails, under `INTEGRITY_STRICT=1`.

use std::collections::BTreeMap;

use diesel::prelude::*;

use super::common::{connect, database_url, ensure_dataset, strict};
use super::{dataset, restore};

#[test]
fn restore_round_trip_preserves_tenant_rows() {
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

    for tool in ["pg_dump", "pg_restore", "psql"] {
        if !restore::available(tool) {
            skip_or_fail!(
                strict,
                (),
                "{tool} is not on PATH; install postgresql-client to run the restore drill"
            );
        }
    }

    let pre_drill = dataset::verify(&mut conn, &manifest);
    assert!(
        pre_drill.is_empty(),
        "the source database is not clean before the drill ({} failure(s)):\n{}",
        pre_drill.len(),
        pre_drill.join("\n")
    );

    let mut before_counts = BTreeMap::new();
    for table in manifest.tables.keys() {
        match dataset::total_rows(&mut conn, table) {
            Ok(rows) => {
                before_counts.insert(table.clone(), rows);
            }
            Err(e) => panic!("cannot count {table} before the dump: {e}"),
        }
    }

    let dump_path = std::env::temp_dir().join(format!("gb-integrity-{}.dump", std::process::id()));
    let name = restore::scratch_name("gb_integrity_restore");
    let scratch = match restore::scratch_url(&url, &name) {
        Ok(scratch) => scratch,
        Err(e) => skip_or_fail!(strict, (), "{e}"),
    };

    let drill = (|| -> Result<(), String> {
        restore::dump(&url, &dump_path)?;
        restore::create_database(&url, &name)?;
        restore::restore(&scratch, &dump_path)
    })();
    if let Err(e) = drill {
        let _ = restore::drop_database(&url, &name);
        let _ = std::fs::remove_file(&dump_path);
        skip_or_fail!(strict, (), "dump/restore failed: {e}");
    }

    let mut restored = match PgConnection::establish(&scratch) {
        Ok(conn) => conn,
        Err(e) => {
            let _ = restore::drop_database(&url, &name);
            let _ = std::fs::remove_file(&dump_path);
            skip_or_fail!(strict, (), "cannot connect to the restored database: {e}");
        }
    };
    let mut failures = dataset::verify(&mut restored, &manifest);
    let mut after_counts = BTreeMap::new();
    for table in manifest.tables.keys() {
        match dataset::total_rows(&mut restored, table) {
            Ok(rows) => {
                after_counts.insert(table.clone(), rows);
            }
            Err(e) => failures.push(format!("cannot count {table} after the restore: {e}")),
        }
    }
    drop(restored);

    let cleanup = restore::drop_database(&url, &name);
    let _ = std::fs::remove_file(&dump_path);

    for (table, rows) in &before_counts {
        match after_counts.get(table) {
            Some(restored_rows) if restored_rows == rows => {}
            Some(restored_rows) => failures.push(format!(
                "{table}: {restored_rows} rows after the restore, {rows} before"
            )),
            None => failures.push(format!("{table}: missing after the restore")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} restore-drill failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    if let Err(e) = cleanup {
        eprintln!("warning: scratch database {name} was not dropped: {e}");
    }
    eprintln!(
        "ok: {} rows across {} tables survived pg_dump/pg_restore with per-tenant checksums intact",
        manifest.total_rows,
        manifest.tables.len()
    );
}
