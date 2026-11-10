//! (b) Tenant isolation on the seeded dataset: a scoped read returns its own
//! rows and only its own rows, and no scope can read another scope's ids.
//!
//! This extends `tenant_isolation.rs` (which drives the HTTP surface with two
//! orgs) instead of duplicating it: here the same tenancy model is proven at the
//! data layer, over every data-owning table in the dataset, with branches and
//! nested bots rather than nil UUIDs.

use uuid::Uuid;

use super::common::{connect, database_url, ensure_dataset, strict};
use super::dataset;

/// A scoped query returns its own rows — and only its own rows.
#[test]
fn tenant_scoped_reads_return_only_own_rows() {
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

    let failures = dataset::verify(&mut conn, &manifest);
    assert!(
        failures.is_empty(),
        "{} scoped-read failure(s) on {} rows (seed {}):\n{}",
        failures.len(),
        manifest.total_rows,
        manifest.seed,
        failures.join("\n")
    );
    eprintln!(
        "ok: {} tables / {} rows (seed {}, scale {}) partition exactly by org, branch, bot and NULL scope",
        manifest.tables.len(),
        manifest.total_rows,
        manifest.seed,
        manifest.scale
    );
}

/// Cross-tenant reads return zero rows, including a forged scope id.
#[test]
fn cross_tenant_reads_return_zero_rows() {
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
    let mut probes = 0_usize;
    for (table, expectation) in &manifest.tables {
        let ids = dataset::scope_ids(expectation);
        if ids.len() < 2 {
            failures.push(format!("{table}: only {} scope(s) seeded", ids.len()));
            continue;
        }
        for (own, _) in &ids {
            for (foreign, _) in &ids {
                if own == foreign {
                    continue;
                }
                probes += 1;
                match dataset::leaked_rows(&mut conn, table, &expectation.scope, *own, *foreign) {
                    Ok(0) => {}
                    Ok(leaked) => failures.push(format!(
                        "{table}: scope {own} read {leaked} row(s) owned by {foreign}"
                    )),
                    Err(e) => failures.push(format!("{table}: {e}")),
                }
            }
        }
        // A scope id nobody owns must never match a row: the forged-token case.
        let forged = Uuid::new_v4();
        probes += 1;
        match dataset::probe(&mut conn, table, &expectation.scope, forged) {
            Ok(found) if found.row_count == 0 => {}
            Ok(found) => failures.push(format!(
                "{table}: forged scope {forged} matched {} row(s)",
                found.row_count
            )),
            Err(e) => failures.push(format!("{table}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} cross-tenant read(s) escaped their scope over {probes} probes:\n{}",
        failures.len(),
        failures.join("\n")
    );
    eprintln!("ok: {probes} cross-scope probes returned zero foreign rows");
}
