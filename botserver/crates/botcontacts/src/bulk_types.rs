//! #1441 P2 — bulk-action and CSV-import request/response types, split out of
//! `requests.rs` to respect the 450-line budget (same convention as
//! `calendar_types.rs` / `tasks_types.rs`).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// #1441 P2 — bulk action over a set of lead/opportunity or contact ids.
/// Every executed change writes one audit row per touched record. Named
/// `LeadBulkActionRequest`/`LeadBulkActionResult` to avoid clashing with the
/// contact-groups bulk types already defined further down this file.
#[derive(Debug, Deserialize)]
pub struct LeadBulkActionRequest {
    pub ids: Vec<Uuid>,
    /// Supported: `stage` (leads — requires `stage`), `owner` (requires
    /// `owner_id`), `delete` (leads or contacts).
    pub action: String,
    pub stage: Option<String>,
    pub owner_id: Option<Uuid>,
}

/// #1441 P2 — result of one bulk operation; `failed` carries the ids that
/// matched no row inside the caller's branch (never a foreign-branch id).
#[derive(Debug, Serialize)]
pub struct LeadBulkActionResult {
    pub updated: u64,
    pub deleted: u64,
    pub failed: Vec<Uuid>,
}

/// #1441 P2 — one CSV import row. All fields optional so partial spreadsheets
/// import cleanly; rows without any recognizable identity are reported in
/// `invalid` instead of silently dropped.
#[derive(Debug, Deserialize)]
pub struct CsvLeadRow {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
}

/// #1441 P2 — CSV import report; `errors` pairs a 1-based row number with the
/// failure reason so the operator can fix the spreadsheet precisely.
#[derive(Debug, Serialize)]
pub struct CsvImportReport {
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub errors: Vec<(usize, String)>,
}
