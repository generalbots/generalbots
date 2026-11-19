use super::{bind, VerbEndpoint};
use std::sync::OnceLock;

mod browser_media_endpoints;
mod office_endpoints;

static DATA: OnceLock<Vec<VerbEndpoint>> = OnceLock::new();

pub fn all() -> &'static [VerbEndpoint] {
    DATA.get_or_init(|| {
        // Order follows the historical split order.
        let mut v = Vec::with_capacity(
            office_endpoints::OFFICE_ENDPOINTS.len()
                + browser_media_endpoints::BROWSER_MEDIA_ENDPOINTS.len(),
        );
        v.extend_from_slice(office_endpoints::OFFICE_ENDPOINTS);
        v.extend_from_slice(browser_media_endpoints::BROWSER_MEDIA_ENDPOINTS);
        v
    })
    .as_slice()
}
