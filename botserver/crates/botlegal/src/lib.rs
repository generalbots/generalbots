

pub mod account_deletion;

use axum::{
    extract::{Path, Query, State},
    response::Html,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

mod db_legal_document;
mod cookie_info;
mod handle_get_consent;
mod handle_request_data_deletion;
mod handle_legal_list_page;
mod handle_legal_detail_page;
mod handle_legal_new_page;

pub use db_legal_document::*;
pub use cookie_info::*;
pub use handle_get_consent::*;
pub use handle_request_data_deletion::*;
pub use handle_legal_list_page::*;
pub use handle_legal_detail_page::*;
pub use handle_legal_new_page::*;
