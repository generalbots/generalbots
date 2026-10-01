//! #1441 — shared building blocks for the CRM HTMX fragment handlers.
//!
//! `crm_ui` (pipeline/search/stats) and the sibling modules under this tree
//! all render partial HTML for the suite CRM app, so the paging, sorting and
//! money helpers live here instead of being copy-pasted per handler.

pub mod campaigns;
pub mod lists;
pub mod pipeline_lists;
pub mod record_detail;
pub mod search;
pub mod stats;

use axum::http::HeaderMap;
use diesel::PgConnection;
use serde::Deserialize;
use std::cmp::Ordering;
use std::sync::Arc;
use uuid::Uuid;

use crate::CrateState;

/// `?q=` shared by the header search and the form `<option>` helpers.
#[derive(Debug, Default, Deserialize)]
pub struct TermQuery {
    pub q: Option<String>,
}

/// #1441 C2 — one grid query for every list fragment: paging plus the
/// whitelist sorting/filtering the suite tables expose as column headers.
#[derive(Debug, Default, Deserialize)]
pub struct GridQuery {
    pub stage: Option<String>,
    pub search: Option<String>,
    pub owner_id: Option<Uuid>,
    /// Rows are ranked in-process (see `lists::page`), so a page can never be
    /// half-applied; the candidate window is bounded by `SORT_WINDOW`.
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub dir: Option<String>,
}

pub const PAGE_SIZE: i64 = 50;

/// Upper bound of rows ranked in-process before a page is cut. Keeps sorting
/// deterministic and bounded for suite-sized branches.
pub const SORT_WINDOW: i64 = 5_000;

/// Branch scope for the caller, falling back to the crate default context.
pub fn branch_ctx(
    state: &Arc<CrateState>,
    headers: &HeaderMap,
    conn: &mut PgConnection,
) -> Uuid {
    crate::scope::branch_from_jwt(headers, conn).unwrap_or_else(|| state.get_bot_context())
}

/// `?dir=desc` sorts descending; anything else (including a missing value)
/// sorts ascending.
pub fn is_ascending(dir: Option<&str>) -> bool {
    !matches!(dir, Some(d) if d.eq_ignore_ascii_case("desc"))
}

/// Total ordering for optional sortable columns. `None` never panics on
/// non-comparable values (NaN) — unorderable values fall back to `Equal`.
pub fn order_by<T: PartialOrd + ?Sized>(left: Option<&T>, right: Option<&T>, asc: bool) -> Ordering {
    let ord = match (left, right) {
        (Some(l), Some(r)) => l.partial_cmp(r),
        (Some(_), None) => Some(Ordering::Greater),
        (None, Some(_)) => Some(Ordering::Less),
        (None, None) => Some(Ordering::Equal),
    };
    match ord {
        Some(Ordering::Equal) | None => Ordering::Equal,
        Some(other) if asc => other,
        Some(other) => other.reverse(),
    }
}

/// #1441 C4 — money is rendered with the record's own currency instead of a
/// hardcoded dollar sign.
pub fn format_money(value: f64, currency: Option<&str>) -> String {
    let code = currency.unwrap_or("USD").trim().to_uppercase();
    let symbol = match code.as_str() {
        "USD" => "$".to_string(),
        "EUR" => "€".to_string(),
        "BRL" => "R$".to_string(),
        "GBP" => "£".to_string(),
        "JPY" => "¥".to_string(),
        other => format!("{other} "),
    };
    let decimals = if code == "JPY" { 0 } else { 2 };
    format!("{symbol}{value:.decimals$}")
}

/// Renders the ‹ Prev / Page N of M / Next › row for a grid fragment. The
/// path is parameterized so each grid keeps its own filters on paging.
pub fn pager_bar(path: &str, extra: &str, total: i64, offset: i64) -> String {
    if total <= PAGE_SIZE {
        return String::new();
    }
    let page = offset / PAGE_SIZE + 1;
    let pages = (total + PAGE_SIZE - 1) / PAGE_SIZE;
    let href = |target: i64| format!("/api/ui/crm/{path}?offset={target}{extra}");
    let prev = if offset >= PAGE_SIZE {
        format!(
            r#"<button class="pager-btn" hx-get="{}" hx-target="closest tbody" hx-swap="innerHTML">&#8249; Prev</button>"#,
            href(offset - PAGE_SIZE)
        )
    } else {
        String::new()
    };
    let next = if offset + PAGE_SIZE < total {
        format!(
            r#"<button class="pager-btn" hx-get="{}" hx-target="closest tbody" hx-swap="innerHTML">Next &#8250;</button>"#,
            href(offset + PAGE_SIZE)
        )
    } else {
        String::new()
    };
    format!(
        r#"<tr class="pager-row"><td colspan="20">{prev}<span class="pager-label">Page {page} of {pages} · {total} records</span>{next}</td></tr>"#
    )
}

/// Shared row-action markup (#1441 B): every grid row exposes Edit/Delete so
/// the suite can drive `PUT`/`DELETE /api/crm/{entity}/:id` without bespoke
/// per-grid code.
pub fn row_actions(entity: &str, id: Uuid) -> String {
    format!(
        r#"<td class="row-actions"><button class="row-action" data-entity="{entity}" data-action="view" data-id="{id}">View</button><button class="row-action" data-entity="{entity}" data-action="edit" data-id="{id}">Edit</button><button class="row-action danger" data-entity="{entity}" data-action="delete" data-id="{id}">Delete</button></td>"#
    )
}
