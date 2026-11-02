pub(crate) use botbasic_types::UserSession;
pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use log::{debug, trace};
pub(crate) use rhai::{Array, Dynamic, Engine, EvalAltResult, Map, Position};
pub(crate) use scraper::{Html, Selector};

use reqwest::Url;
use std::sync::Arc;
use std::time::Duration;

mod static_selector;
mod register_scrape_images_keyword;

pub use static_selector::*;
use register_scrape_images_keyword::*;
