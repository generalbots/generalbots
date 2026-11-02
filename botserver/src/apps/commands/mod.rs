//! Per-application command model — single source of truth for every action a
//! suite application exposes to the LLM (`__api_call__`), the command palette
//! and the unified Start Menu.
//!
//! Commands defined here are the ONLY declarative per-app action list. They are
//! consumed by:
//!   1. `GET /api/apps/catalog` (`apps/mod.rs`) — palette + frontend.
//!   2. `api_command_instructions()` (`core/bot/api_catalog.rs`) — chat/WhatsApp
//!      prompt (only `name: summary` is injected to keep the prompt compact).
//!   3. `api.find` / `apps.find` discovery — full params on demand.
//!   4. The frontend Start Menu / command palette.
//!
//! Deep-link parameters declare the canonical record key each app accepts via
//! `app://<appId>?<key>=<value>` so the frontend can open an app window already
//! contextualized to a record.

pub(crate) use serde::Serialize;

mod app_command;
mod app_deep_links;
mod deep_link_params_for_app;

pub use app_command::*;
pub use app_deep_links::*;
pub use deep_link_params_for_app::*;
