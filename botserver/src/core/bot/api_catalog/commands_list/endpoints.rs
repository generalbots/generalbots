use super::ApiEndpoint;
use std::sync::OnceLock;

mod crm_sales_and_compliance;
mod files_user_and_commerce;
mod player_dashboards_and_vibe;

static ALL: OnceLock<Vec<ApiEndpoint>> = OnceLock::new();

pub fn all() -> &'static [ApiEndpoint] {
    ALL.get_or_init(|| {
        // Order follows the historical split order.
        let mut v = Vec::with_capacity(
            files_user_and_commerce::FILES_USER_AND_COMMERCE.len()
                + crm_sales_and_compliance::CRM_SALES_AND_COMPLIANCE.len()
                + player_dashboards_and_vibe::PLAYER_DASHBOARDS_AND_VIBE.len(),
        );
        v.extend_from_slice(files_user_and_commerce::FILES_USER_AND_COMMERCE);
        v.extend_from_slice(crm_sales_and_compliance::CRM_SALES_AND_COMPLIANCE);
        v.extend_from_slice(player_dashboards_and_vibe::PLAYER_DASHBOARDS_AND_VIBE);
        v
    })
    .as_slice()
}
