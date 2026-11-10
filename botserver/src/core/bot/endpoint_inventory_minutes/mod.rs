//! Endpoint catalogue chunk: meetings, channels, workspace and UI domains.
//!
//! #1730 — this chunk of the catalogue outgrew the 450-line limit, so it is
//! split by domain into sibling modules. `all_routes()` chains them back in
//! catalogue order (paths stay sorted) so `endpoint_inventory::all_routes()`
//! keeps returning one ordered iterator.

use crate::core::bot::api_catalog::ApiEndpoint;

mod meetings_and_channels;
mod people_and_products;
mod access_and_reporting;
mod security_and_settings;
mod sheets;
mod slides_and_office;
mod tickets_and_time;
mod ui_endpoints;

/// Every endpoint in this chunk, in catalogue order.
pub(crate) fn all_routes() -> impl Iterator<Item = &'static ApiEndpoint> {
    meetings_and_channels::ROUTES
        .iter()
        .chain(people_and_products::ROUTES)
        .chain(access_and_reporting::ROUTES)
        .chain(security_and_settings::ROUTES)
        .chain(sheets::ROUTES)
        .chain(slides_and_office::ROUTES)
        .chain(tickets_and_time::ROUTES)
        .chain(ui_endpoints::ROUTES)
}
