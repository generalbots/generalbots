//! Endpoint catalogue chunk: directory, document and editor domains.
//!
//! #1730 — this chunk of the catalogue outgrew the 450-line limit, so it is
//! split by domain into sibling modules. `all_routes()` chains them back in
//! catalogue order so `endpoint_inventory::all_routes()` keeps returning one
//! ordered iterator.

use crate::core::bot::api_catalog::ApiEndpoint;

mod directory_and_dns;
mod docs;
mod editor;

/// Every endpoint in this chunk, in catalogue order.
pub(crate) fn all_routes() -> impl Iterator<Item = &'static ApiEndpoint> {
    directory_and_dns::ROUTES
        .iter()
        .chain(docs::ROUTES)
        .chain(editor::ROUTES)
}
