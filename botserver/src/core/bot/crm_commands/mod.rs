//! #1441 P3 — CRM chat-command executors (`crm.pipeline.forecast`,
//! `crm.leads.create`, `crm.leads.report`, `crm.deal.create`,
//! `crm.deal.move-stage`, `crm.contact.create`). The dispatch arms live in
//! `api_catalog.rs`; every helper here is branch-scoped through `branch_scope`.
//!
//! #1730 — the executors outgrew the 450-line limit, so they are split by
//! responsibility into sibling modules. Every command is re-exported here, so
//! the `crm_commands::*` paths used by the dispatcher are unchanged.

mod contacts;
mod deals;
mod forecast;
mod leads;
mod report;
mod rows;

pub(crate) use contacts::crm_create_contact_command;
pub(crate) use deals::{crm_create_deal_command, crm_move_deal_stage_command};
pub(crate) use forecast::crm_forecast_command;
pub(crate) use leads::crm_create_lead_command;
pub(crate) use report::crm_pipeline_report_command;
