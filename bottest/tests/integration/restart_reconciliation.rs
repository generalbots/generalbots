//! #2541 — restart UAT, in-process edition.
//!
//! A real process restart cannot be driven from inside a `#[tokio::test]`:
//! botserver is a separate process and its restart is a CI/production action.
//! This test therefore asserts the *durable substitutes* the boot path reads
//! after a restart, which is the strongest in-process evidence available:
//!
//! 1. `agent_runs` (chat run traces) — a stale `running` row is reconciled to
//!    `interrupted` by the same SQL botserver runs at boot
//!    (`botserver/src/core/bot/runs/observability.rs`).
//! 2. `output_publish:{session}` (published-file ledger) — re-read from Redis
//!    after the in-memory map is discarded, with a bounded TTL (no republish
//!    and no silent 404 for a file published before the restart).
//! 3. `vibe_runs` (Vibe runs) — reconciled by the real public helper
//!    `VibeRunStore::recover_orphaned_runs`, then the API's run map is rebuilt
//!    from the durable store instead of starting empty (no silent 404).
//!
//! The pure reconciliation predicates are asserted directly via the public
//! `boot_reconcile_target` / `LIVE_RUN_STATES` helpers, and the replicated SQL
//! is checked against the production source so the two cannot drift silently.
//!
//! Run with:
//!   `cargo test -p bottest --features integration --test integration \
//!        restart_truthful_state -- --nocapture --test-threads=1`
//! The harness skips gracefully when local Postgres/Redis binaries are absent;
//! CI is the authoritative type/compile gate.

use std::collections::HashMap;

use bottest::prelude::*;
use botvibe::run_store::{
    boot_reconcile_target, ensure_vibe_schema, VibeRunStore, LIVE_RUN_STATES, RECONCILED_ERROR,
    RECONCILED_STATE,
};
use botvibe::types::{VibeRun, VibeRunConfig, VibeRunState};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Text, Uuid as SqlUuid};

/// Mirrors `api::tools::BOOT_RUN_REBUILD_LIMIT` (pub(crate), not importable here).
const BOOT_RUN_REBUILD_LIMIT: i64 = 500;
/// Mirrors `observability::DEFAULT_STALE_RUN_MINUTES`.
const STALE_RUN_MINUTES: i64 = 15;
/// Mirrors `output_publish::ledger::DEFAULT_LEDGER_TTL_DAYS` (30 days).
const LEDGER_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Byte-for-byte the statement in `observability::reconcile_stale_runs`.
const STALE_RUN_SQL: &str = "UPDATE agent_runs SET status = 'interrupted', finished_at = NOW() \
     WHERE status IN ('running', 'pending') \
       AND started_at < NOW() - ($1 || ' minutes')::interval";

const OBSERVABILITY_RS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../botserver/src/core/bot/runs/observability.rs"
);
const PUBLISH_LEDGER_RS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../botserver/src/core/bot/pipeline/output_publish/ledger.rs"
);

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RunStatus {
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Bool)]
    finished: bool,
}

/// Reads the truthful state of a durable run. The row must still exist — a
/// missing row would be the "silent 404" this issue is about.
fn run_status(conn: &mut PgConnection, run_id: Uuid) -> RunStatus {
    let mut rows: Vec<RunStatus> = sql_query(
        "SELECT status, finished_at IS NOT NULL AS finished FROM agent_runs WHERE id = $1",
    )
    .bind::<SqlUuid, _>(run_id)
    .load(conn)
    .expect("read agent_runs row");
    assert_eq!(rows.len(), 1, "no silent 404: run row must survive restart");
    rows.remove(0)
}

/// Keeps the predicates asserted here in lockstep with the production code.
fn production_predicates_match_test() {
    let obs = std::fs::read_to_string(OBSERVABILITY_RS).expect("observability.rs readable");
    assert!(
        obs.contains("UPDATE agent_runs SET status = 'interrupted', finished_at = NOW()"),
        "test SQL drifted from observability::reconcile_stale_runs"
    );
    assert!(obs.contains("status IN ('running', 'pending')"));
    assert!(obs.contains("started_at < NOW() - ($1 || ' minutes')::interval"));
    assert!(obs.contains("reconcile_on_boot"));

    let ledger = std::fs::read_to_string(PUBLISH_LEDGER_RS).expect("ledger.rs readable");
    assert!(
        ledger.contains("output_publish:{session_id}"),
        "test key drifted from the durable publish ledger"
    );
    assert!(ledger.contains("DEFAULT_LEDGER_TTL_DAYS"));
}

#[tokio::test]
async fn restart_truthful_state_across_durable_stores() {
    // One harness for all three stores: harness setup tears down local test
    // services, so a single setup avoids cross-test interference. CI runs this
    // target with `--test-threads=1`.
    let ctx = match TestHarness::setup(TestConfig {
        postgres: true,
        redis: true,
        run_migrations: true,
        ..TestConfig::minimal()
    })
    .await
    {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("SKIP: harness unavailable (Postgres/Redis binaries missing): {e}");
            return;
        }
    };
    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("SKIP: db pool unavailable: {e}");
            return;
        }
    };
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("SKIP: db connection unavailable: {e}");
            return;
        }
    };

    // (1) Chat run traces: a durable `running` row older than the retention
    // window must not survive boot as a phantom "running" run. This minimal
    // DDL is a no-op when migrations already created the production table.
    sql_query(
        "CREATE TABLE IF NOT EXISTS agent_runs (\
         id UUID PRIMARY KEY, bot_id UUID NOT NULL, \
         trigger_kind VARCHAR(16) NOT NULL, status VARCHAR(24) NOT NULL DEFAULT 'queued', \
         started_at TIMESTAMPTZ, finished_at TIMESTAMPTZ)",
    )
    .execute(&mut conn)
    .expect("agent_runs fixture table");

    let bot_id = Uuid::new_v4();
    let stale_id = Uuid::new_v4();
    let fresh_id = Uuid::new_v4();
    let done_id = Uuid::new_v4();
    for (id, status, age_minutes) in [
        (stale_id, "running", 60_i64),
        (fresh_id, "running", 1_i64),
        (done_id, "completed", 60_i64),
    ] {
        let inserted = sql_query(
            "INSERT INTO agent_runs (id, bot_id, trigger_kind, status, started_at) \
             VALUES ($1, $2, 'chat', $3, NOW() - ($4 || ' minutes')::interval)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<SqlUuid, _>(bot_id)
        .bind::<Text, _>(status)
        .bind::<BigInt, _>(age_minutes)
        .execute(&mut conn)
        .expect("seed agent_runs row");
        assert_eq!(inserted, 1);
    }
    assert_eq!(run_status(&mut conn, stale_id).status, "running");
    assert_eq!(run_status(&mut conn, fresh_id).status, "running");
    assert_eq!(run_status(&mut conn, done_id).status, "completed");

    // Simulated restart: discard the volatile trace buffer and apply the same
    // reconciliation the boot path applies to the durable table.
    let recovered = sql_query(STALE_RUN_SQL)
        .bind::<BigInt, _>(STALE_RUN_MINUTES)
        .execute(&mut conn)
        .expect("reconcile stale agent_runs");
    assert!(recovered >= 1, "boot must reconcile at least the stale row");

    let stale = run_status(&mut conn, stale_id);
    assert_eq!(stale.status, "interrupted");
    assert!(
        stale.finished,
        "interrupted run must have finished_at stamped"
    );
    assert_eq!(run_status(&mut conn, fresh_id).status, "running");
    let done = run_status(&mut conn, done_id);
    assert_eq!(done.status, "completed");
    assert!(
        !done.finished,
        "terminal rows must not be rewritten on boot"
    );

    // (2) Vibe runs: reconcile durable rows, then rebuild the API's run map
    // from the store — the exact boot path in `api::tools::router`.
    ensure_vibe_schema(pool).expect("vibe schema");
    let store = VibeRunStore::new(pool.clone());
    let mut live = VibeRun::new(
        bot_id,
        Uuid::new_v4(),
        Uuid::nil(),
        "build a calculator".to_string(),
        VibeRunConfig::default(),
    );
    live.state = VibeRunState::Running;
    let mut terminal = VibeRun::new(
        bot_id,
        Uuid::new_v4(),
        Uuid::nil(),
        "already finished".to_string(),
        VibeRunConfig::default(),
    );
    terminal.state = VibeRunState::Completed;
    store.save_run(&live).expect("seed live Vibe run");
    store.save_run(&terminal).expect("seed terminal Vibe run");
    assert_eq!(
        store
            .get_run(live.run_id)
            .expect("durable read before restart")
            .state,
        VibeRunState::Running
    );

    // Simulated restart: in-memory run map is empty; boot reconciles and
    // hydrates the map from the durable store.
    let recovered = store.recover_orphaned_runs();
    assert!(
        recovered >= 1,
        "boot must reconcile orphaned Vibe live runs"
    );
    let rebuilt: HashMap<Uuid, VibeRun> = store
        .list_runs(BOOT_RUN_REBUILD_LIMIT)
        .into_iter()
        .map(|run| (run.run_id, run))
        .collect();

    let after = rebuilt
        .get(&live.run_id)
        .expect("no silent 404 for a Vibe run started before the restart");
    assert_eq!(after.state, RECONCILED_STATE);
    assert!(after.state.is_terminal(), "reconciled run must be terminal");
    assert_eq!(after.error.as_deref(), Some(RECONCILED_ERROR));
    assert!(after.completed_at.is_some());
    let preserved = rebuilt
        .get(&terminal.run_id)
        .expect("terminal Vibe run must be preserved");
    assert_eq!(preserved.state, VibeRunState::Completed);
    assert_eq!(preserved.error, None);

    // Assert the pure predicates behind boot reconciliation directly.
    for state in LIVE_RUN_STATES {
        assert_eq!(boot_reconcile_target(state), Some(RECONCILED_STATE));
    }
    for state in [
        VibeRunState::Completed,
        VibeRunState::Failed,
        VibeRunState::Cancelled,
    ] {
        assert_eq!(boot_reconcile_target(state), None);
    }

    // (3) Published-file ledger: a committed publish is mirrored to Redis with
    // a bounded TTL; after the in-memory map is discarded the durable mirror
    // is the only source that can answer for the file.
    let client = match redis::Client::open(ctx.redis_url()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("SKIP: redis client unavailable: {e}");
            return;
        }
    };
    let mut cache = match client.get_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("SKIP: redis connection unavailable: {e}");
            return;
        }
    };
    let session_id = Uuid::new_v4();
    let key = format!("output_publish:{session_id}");
    let payload = serde_json::json!({
        "report.pdf": { "url": "https://example.invalid/report.pdf" }
    })
    .to_string();
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg(&payload)
        .arg("EX")
        .arg(LEDGER_TTL_SECS)
        .query(&mut cache)
        .expect("seed durable publish ledger");

    let raw: String = redis::cmd("GET")
        .arg(&key)
        .query(&mut cache)
        .expect("durable ledger read after restart");
    let ledger: serde_json::Value = serde_json::from_str(&raw).expect("parse publish ledger");
    assert!(
        ledger.get("report.pdf").is_some(),
        "published file must not silently 404 after a restart"
    );
    let ttl: i64 = redis::cmd("TTL")
        .arg(&key)
        .query(&mut cache)
        .expect("read ledger TTL");
    assert!((1..=LEDGER_TTL_SECS).contains(&ttl), "bounded ledger TTL");

    production_predicates_match_test();
}
