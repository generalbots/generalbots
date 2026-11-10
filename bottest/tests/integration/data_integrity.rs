//! Data integrity: migrations, tenant isolation and restore on one dataset (#2117).
//!
//! Three properties that used to be tested separately — if at all — with
//! different fixtures: the migrations produce the schema `botschema` declares,
//! tenant-scoped queries never cross an org/branch/bot boundary, and a dump
//! restores with every row still in its own tenant.
//!
//! ## Local run
//!
//! ```bash
//! # 1. throwaway PostgreSQL (the CI job uses a postgres:16 service container)
//! export DATABASE_URL=postgres://gb:gb@127.0.0.1:5432/botserver
//!
//! # 2. apply every migration from scratch
//! (cd botserver && diesel migration run)
//!
//! # 3. generate the production-like dataset (deterministic, seed 2117)
//! python3 scripts/gen_integrity_dataset.py --out /tmp/gb-integrity --report
//!
//! # 4. run the suite — it loads the dataset, then dumps and restores it
//! cargo test -p bottest --features integration data_integrity \
//!   -- --nocapture --test-threads=1
//! ```
//!
//! `INTEGRITY_DATASET` overrides the dataset directory (default
//! `/tmp/gb-integrity`). `INTEGRITY_STRICT=1` turns every skip into a failure —
//! CI runs that way so a missing tool cannot masquerade as a pass.
//!
//! Skips are deliberate outside CI: without `DATABASE_URL`, without `psql`, or
//! without `pg_dump`/`pg_restore` the suite reports the exact reason and moves on.
//!
//! Layout: `checks` proves (a) migrations + schema, `isolation` proves (b)
//! tenant scoping on the seeded dataset, `drill` proves (c) the dump/restore
//! round-trip. `common` holds the environment/skip helpers, `dataset` the
//! manifest probes, `schema` the `botschema` comparison and `restore` the
//! external-tool wrappers.

#[macro_use]
mod common;

mod checks;
mod dataset;
mod drill;
mod isolation;
mod restore;
mod schema;
