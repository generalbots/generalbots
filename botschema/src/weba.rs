//! WebA site-builder snapshot table (#2538, #1497).
//!
//! `table!` macros live only in this crate; feature crates re-export them.
//! Migration: `botserver/migrations/6.8.20-weba-sites`.

diesel::table! {
    weba_snapshots (branch_id) {
        branch_id -> Uuid,
        payload -> Jsonb,
        updated_at -> Timestamptz,
    }
}
