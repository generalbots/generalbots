pub(crate) use botbasic_types::UserSession;
pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use log::trace;
pub(crate) use reqwest::{header::HeaderMap, header::HeaderName, header::HeaderValue, Client, Method};
pub(crate) use rhai::{Dynamic, Engine, Map};
pub(crate) use serde_json::{json, Value};
pub(crate) use std::sync::{Arc, Mutex};

thread_local! {
    static HTTP_HEADERS: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

mod register_http_operations;
mod register_set_header_keyword;
mod execute_graphql;

pub use register_http_operations::*;
pub use register_set_header_keyword::*;
use execute_graphql::*;
