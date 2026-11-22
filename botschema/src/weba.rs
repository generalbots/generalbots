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

// #2538 — read-only projections over the atomic snapshot. They carry the same
// branch_id boundary and let reporting code query sites/pages/submissions with
// Diesel without duplicating mutable state.
diesel::table! {
    weba_sites (id) {
        branch_id -> Uuid,
        id -> Uuid,
        name -> Text,
        slug -> Text,
        status -> Text,
        config -> Jsonb,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    weba_pages (id) {
        branch_id -> Uuid,
        id -> Uuid,
        site_id -> Uuid,
        path -> Text,
        title -> Text,
        content -> Text,
        is_index -> Bool,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    weba_form_submissions (id) {
        branch_id -> Uuid,
        id -> Uuid,
        site_id -> Uuid,
        slug -> Text,
        email -> Text,
        source -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(weba_pages -> weba_sites (site_id));
diesel::joinable!(weba_form_submissions -> weba_sites (site_id));
diesel::allow_tables_to_appear_in_same_query!(weba_snapshots, weba_sites, weba_pages, weba_form_submissions);
