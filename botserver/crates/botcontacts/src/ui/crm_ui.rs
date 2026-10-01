//! #1441 — CRM HTMX fragment surface.
//!
//! The handlers live in `fragments/` (grids, search, stats, campaigns, record
//! detail); this module keeps the historical `ui::crm_ui::*` paths working so
//! the router registration and any other caller stay unchanged.

pub use super::fragments::campaigns::handle_crm_campaigns;
pub use super::fragments::lists::{handle_crm_accounts, handle_crm_contacts};
pub use super::fragments::pipeline_lists::{handle_crm_deals, handle_crm_opportunities};
pub use super::fragments::record_detail::handle_record_detail;
pub use super::fragments::search::{
    handle_crm_accounts_search, handle_crm_contacts_search, handle_crm_opportunities_search,
    handle_crm_search,
};
pub use super::fragments::stats::{
    handle_crm_count, handle_crm_pipeline, handle_crm_stats_avg_deal,
    handle_crm_stats_conversion_rate, handle_crm_stats_funnel, handle_crm_stats_forecast,
    handle_crm_stats_pipeline_value, handle_crm_stats_won_month,
};
