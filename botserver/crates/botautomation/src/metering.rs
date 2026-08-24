//! Hourly compute-usage accounting backed by the `compute_usage_hourly`
//! table. Quantities accumulate across upserts within the same clock hour.

use crate::schema::compute_usage_hourly;
use chrono::{Timelike, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// Accumulates `qty` of `resource` for `org_id` into the current hour bucket.
/// A `None` org is attributed to the nil UUID sentinel so the NOT NULL
/// primary-key column stays satisfied.
pub fn record_usage(
    conn: &mut PgConnection,
    org_id: Option<Uuid>,
    resource: &str,
    qty: f64,
) -> QueryResult<usize> {
    use compute_usage_hourly as t;

    let org = org_id.unwrap_or(Uuid::nil());
    let now = Utc::now();
    let hour = now
        .with_minute(0)
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .map(|d| d.naive_utc())
        .unwrap_or_else(|| now.naive_utc());
    let resource = resource.chars().take(32).collect::<String>();

    diesel::insert_into(t::table)
        .values((
            t::org_id.eq(org),
            t::hour.eq(hour),
            t::resource.eq(resource),
            t::quantity.eq(qty),
            t::updated_at.eq(now),
        ))
        .on_conflict((t::org_id, t::hour, t::resource))
        .do_update()
        .set((
            t::quantity.eq(t::quantity + diesel::dsl::excluded(t::quantity)),
            t::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
}
