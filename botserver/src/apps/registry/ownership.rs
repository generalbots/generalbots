//! #2120 — owner and support status of every registry app.
//!
//! The supported matrix (`botserver/.product.matrix`, rendered in
//! `botbook/src/07-user-interface/app-support-matrix.md`) is the product
//! decision; this table is its Rust projection. Each registry entry carries
//! the owning team and the support status, so no app can ship without a name
//! behind it:
//!
//! | Status | Meaning |
//! |---|---|
//! | `Supported` | In `apps=` of at least one tier, covered by the matrix SLA. |
//! | `Preview` | Ships, but is excluded from support SLAs and hidden behind Preview mode. |
//! | `Hidden` | Compiled and routed, never advertised (`#1626`). |
//! | `Deprecated` | Announced for removal; may stay reachable, never in `apps=`. |
//!
//! The guard tests in `mod.rs` fail when an entry is missing here, when an
//! owner is empty, or when a deprecated app is advertised. The Python guard
//! (`scripts/product_profile_check.py`) re-checks the same table against the
//! matrix without compiling.

use serde::Serialize;

/// Owner recorded for an app that has no row in [`OWNERSHIP`]. It is never a
/// valid value: the registry guard test fails on it, and the runtime logs it.
pub const UNOWNED: &str = "unowned";

/// Support status of a registry app (#2120).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppStatus {
    /// Supported surface of at least one build tier.
    Supported,
    /// Ships, but outside the support SLA and behind Preview mode.
    Preview,
    /// Compiled and routed, never advertised.
    Hidden,
    /// Announced for removal; never advertised.
    Deprecated,
}

/// Owner and status of one registry id.
#[derive(Debug, Clone, Copy)]
pub struct AppOwnership {
    /// Owning team, e.g. `desktop` or `office`.
    pub owner: &'static str,
    /// Support status declared by the matrix.
    pub status: AppStatus,
}

/// Registry id → (owning team, support status). Keep sorted by id; the guard
/// test in `mod.rs` asserts that it covers the registry exactly once.
pub const OWNERSHIP: &[(&str, &str, AppStatus)] = &[
    ("about", "platform", AppStatus::Supported),
    ("analytics", "business", AppStatus::Preview),
    ("attendant", "business", AppStatus::Preview),
    ("automations", "chat-ai", AppStatus::Preview),
    ("banking", "business", AppStatus::Preview),
    ("billing", "business", AppStatus::Supported),
    ("browser", "platform", AppStatus::Supported),
    ("calculator", "desktop", AppStatus::Supported),
    ("calendar", "office", AppStatus::Supported),
    ("campaigns", "business", AppStatus::Preview),
    ("canvas", "office", AppStatus::Preview),
    ("chat", "chat-ai", AppStatus::Supported),
    // Superseded by `clock-widget`; the deprecation policy (#2120) keeps the
    // API for one release, so the entry stays registered but unadvertised.
    ("clock", "desktop", AppStatus::Deprecated),
    ("clock-widget", "desktop", AppStatus::Supported),
    ("compliance", "business", AppStatus::Preview),
    ("concierge", "chat-ai", AppStatus::Preview),
    ("crm", "business", AppStatus::Supported),
    ("database", "platform", AppStatus::Preview),
    ("designer", "office", AppStatus::Preview),
    ("desktop", "platform", AppStatus::Preview),
    ("docs", "office", AppStatus::Supported),
    ("drive", "desktop", AppStatus::Supported),
    ("editor", "platform", AppStatus::Preview),
    ("fraud", "business", AppStatus::Preview),
    ("goals", "office", AppStatus::Preview),
    ("handoff", "business", AppStatus::Preview),
    ("hr", "business", AppStatus::Preview),
    ("integrations", "platform", AppStatus::Supported),
    ("jukebox", "desktop", AppStatus::Supported),
    ("kyc", "business", AppStatus::Preview),
    ("learn", "chat-ai", AppStatus::Preview),
    ("lists", "business", AppStatus::Preview),
    ("mail", "office", AppStatus::Supported),
    ("meet", "office", AppStatus::Preview),
    ("memory", "chat-ai", AppStatus::Preview),
    ("minutes", "office", AppStatus::Preview),
    ("monitoring", "platform", AppStatus::Preview),
    ("notes", "desktop", AppStatus::Supported),
    ("o365", "office", AppStatus::Preview),
    ("paper", "office", AppStatus::Supported),
    ("people", "business", AppStatus::Preview),
    ("photos", "desktop", AppStatus::Supported),
    ("plan", "office", AppStatus::Preview),
    ("player", "desktop", AppStatus::Supported),
    ("plugins", "platform", AppStatus::Preview),
    ("pos", "business", AppStatus::Preview),
    ("products", "business", AppStatus::Supported),
    ("project", "office", AppStatus::Preview),
    ("recycle", "desktop", AppStatus::Preview),
    // #1626 — Research stays out of the default product.
    ("research", "chat-ai", AppStatus::Hidden),
    ("retail", "business", AppStatus::Preview),
    ("settings", "platform", AppStatus::Supported),
    ("sheet", "office", AppStatus::Supported),
    ("slides", "office", AppStatus::Supported),
    ("social", "business", AppStatus::Preview),
    ("store", "desktop", AppStatus::Preview),
    ("tasks", "office", AppStatus::Supported),
    ("tax", "business", AppStatus::Preview),
    ("templates", "office", AppStatus::Preview),
    ("terminal", "platform", AppStatus::Preview),
    ("tickets", "business", AppStatus::Preview),
    ("timeclock", "office", AppStatus::Preview),
    ("timer", "desktop", AppStatus::Supported),
    ("vibe", "chat-ai", AppStatus::Preview),
    ("video", "chat-ai", AppStatus::Preview),
    ("vision", "chat-ai", AppStatus::Preview),
    ("weather", "desktop", AppStatus::Supported),
    ("workspace", "office", AppStatus::Preview),
];

/// Ownership of `id`, or `None` when the registry entry has no matrix row.
pub fn ownership_of(id: &str) -> Option<AppOwnership> {
    OWNERSHIP
        .iter()
        .find(|(entry, _, _)| *entry == id)
        .map(|(_, owner, status)| AppOwnership {
            owner,
            status: *status,
        })
}
