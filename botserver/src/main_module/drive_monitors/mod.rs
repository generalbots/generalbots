//! drive_monitors - extracted from bootstrap.rs
//! Processes .gborg buckets containing .gbai bot sub-directories

pub(crate) use botcore::shared::state::AppState;
pub(crate) use diesel::RunQueryDsl;
pub(crate) use log::{error, info, trace, warn};
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;
pub(crate) use crate::basic::AppStateBasicRuntime;
pub(crate) use crate::basic::keywords::table_definition::process_table_definitions;

const DEFAULT_TENANT_ID: Uuid = Uuid::from_u128(1);
const DEFAULT_ORG_ID: Uuid = Uuid::from_u128(1);
const DEFAULT_BRANCH_ID: Uuid = Uuid::from_u128(1);
const DEFAULT_BOT_ID: Uuid = Uuid::from_u128(1);

mod start_drive_monitors;
mod ensure_org_branches_schema;
mod process_org_bucket;
mod ensure_tenant_and_org;

pub(crate) use start_drive_monitors::*;
use ensure_org_branches_schema::*;
use process_org_bucket::*;
pub(crate) use ensure_tenant_and_org::*;
