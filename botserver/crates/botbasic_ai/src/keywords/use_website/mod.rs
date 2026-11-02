pub(crate) use botbasic_types::UserSession;
pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use diesel::prelude::*;
pub(crate) use log::{debug, info, trace};
pub(crate) use rhai::{Dynamic, Engine};

enum WebsiteCrawlStatus {
    NotRegistered,
    Pending,
    Crawled,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_sanitization() {
        assert_eq!(
            sanitize_url_for_collection("https://docs.example.com/path"),
            "docs_example_com_path"
        );
        // The port is part of the sanitised name: it must survive the
        // replacement of `/`, `:` and `.` by `_`.
        assert_eq!(
            sanitize_url_for_collection("http://test.site:9000"),
            "test_site_9000"
        );
    }
}

use std::sync::Arc;
use uuid::Uuid;

mod parse_refresh_interval;
mod register_use_website_function;
mod check_website_crawl_status;
mod get_active_websites_for_session;

pub use parse_refresh_interval::*;
pub use register_use_website_function::*;
pub use check_website_crawl_status::*;
pub use get_active_websites_for_session::*;
