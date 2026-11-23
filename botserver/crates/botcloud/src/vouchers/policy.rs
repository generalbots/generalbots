//! Pure voucher policy helpers (#2069 #2868), split from the HTTP handlers
//! to keep both files focused and under the size budget.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Alphabet of a voucher code: uppercase ASCII and digits, nothing that needs
/// escaping in a URL or a chat message.
const VOUCHER_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Voucher codes are `GB<trial_days>-<8 random chars>`; the day count is part
/// of the code so support can read the trial length without a database lookup.
/// #2069 CLD-7 — the RNG is injected so the shape is testable.
pub(crate) fn voucher_code_with<R: rand::Rng>(trial_days: i32, rng: &mut R) -> String {
    let pick = |rng: &mut R| VOUCHER_ALPHABET[rng.random_range(0..VOUCHER_ALPHABET.len())] as char;
    let mut suffix = String::with_capacity(8);
    for _ in 0..8 {
        suffix.push(pick(rng));
    }
    format!("GB{trial_days}-{suffix}")
}

pub(crate) fn generate_voucher_code(trial_days: i32) -> String {
    voucher_code_with(trial_days, &mut rand::rng())
}

/// Trial length a voucher may grant (inclusive bounds).
pub(crate) const TRIAL_DAYS_RANGE: std::ops::RangeInclusive<i32> = 1..=180;

/// #2069 CLD-8 — redemption policy, without a database handle.
///
/// Returns the rejection reason the endpoint reports, or `None` when the
/// voucher may be redeemed. Precedence is exhausted → expired → duplicate.
pub(crate) fn redemption_rejection(
    uses_count: i32,
    max_uses: i32,
    expires_at: Option<DateTime<Utc>>,
    already_redeemed: bool,
    now: DateTime<Utc>,
) -> Option<&'static str> {
    if uses_count >= max_uses {
        return Some("Voucher has reached its maximum uses");
    }
    if let Some(expiry) = expires_at {
        if now > expiry {
            return Some("Voucher has expired");
        }
    }
    if already_redeemed {
        return Some("Voucher already redeemed by this user");
    }
    None
}

// #2868 — pure helpers so the NOT NULL / cross-tenant guards are testable.
/// #2868 — the (org_id, branch_id) pair shared by every row a redemption
/// writes. A nil org or branch is never a valid tenant scope.
pub(crate) fn redemption_scope(org_id: Uuid, branch_id: Uuid) -> Option<(Uuid, Uuid)> {
    (!org_id.is_nil() && !branch_id.is_nil()).then_some((org_id, branch_id))
}

/// #2868 — a prior redemption is in scope only when it carries the caller's
/// verified branch. Pure; refuses a cross-tenant replay of the same voucher.
pub(crate) fn redemption_matches_branch(row_branch: Option<Uuid>, caller_branch: Uuid) -> bool {
    !caller_branch.is_nil() && matches!(row_branch, Some(b) if !b.is_nil() && b == caller_branch)
}

/// #2868 — ordered columns of the trial subscription INSERT. `branch_id` is
/// NOT NULL in `billing_recurring` and branch-scoped subscription handlers
/// locate the trial by it, so it must never be omitted from the statement.
pub(crate) const TRIAL_SUBSCRIPTION_COLUMNS: &[&str] = &[
    "id", "org_id", "branch_id", "bot_id", "customer_name", "customer_email", "status",
    "frequency", "interval_count", "amount", "currency", "description", "next_invoice_date",
    "start_date", "last_invoice_id", "invoices_generated", "created_at", "updated_at",
];

/// #2868 — true only when `columns` binds `branch_id` and `org_id` exactly once.
pub(crate) fn insert_fields_are_branch_scoped(columns: &[&str]) -> bool {
    columns.iter().filter(|c| **c == "branch_id").count() == 1
        && columns.iter().filter(|c| **c == "org_id").count() == 1
}

