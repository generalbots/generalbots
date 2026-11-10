//! Fraud, inventory, GL, retail, banking, M365, weba, directory, social,
//! learn, meet, billing and SaaS router merges.

use axum::Router;
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn merge_commerce(
    app_state: &Arc<AppState>,
    api_router: &mut Router<Arc<AppState>>,
    sub_router: &mut Router<()>,
) {
    let _ = &api_router;
    #[cfg(feature = "fraud")]
    {
        let fraud_state = Arc::new(crate::fraud::FraudState::new(app_state.conn.clone()));
        *sub_router = sub_router
            .clone()
            .merge(crate::fraud::configure_fraud_routes().with_state(fraud_state));
    }

    #[cfg(feature = "inventory")]
    {
        let inventory_state = Arc::new(crate::inventory::InventoryState {
            pool: app_state.conn.clone(),
        });
        *sub_router = sub_router
            .clone()
            .merge(crate::inventory::configure_inventory_routes().with_state(inventory_state));
    }

    #[cfg(feature = "gl")]
    {
        let gl_state = Arc::new(crate::gl::GlState {
            pool: app_state.conn.clone(),
        });
        *sub_router = sub_router
            .clone()
            .merge(crate::gl::configure_gl_routes().with_state(gl_state));
    }

    #[cfg(feature = "retail")]
    {
        *sub_router = sub_router.clone().merge(botretail::configure());
    }

    #[cfg(feature = "banking")]
    {
        *api_router = api_router.clone().merge(botbanking::configure());
    }

    #[cfg(feature = "m365")]
    {
        *api_router = api_router.clone().merge(botm365::configure());
    }

    #[cfg(feature = "weba")]
    {
        let weba_state = Arc::new(crate::weba::WebaState::new());
        *sub_router = sub_router
            .clone()
            .merge(crate::weba::configure_routes(weba_state));
    }

    #[cfg(feature = "directory")]
    {
        *sub_router = sub_router.clone().merge(
            crate::directory::scim::server::configure_scim_routes().with_state(app_state.clone()),
        );
    }

    #[cfg(feature = "social")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_social_router(app_state));
    }

    #[cfg(feature = "learn")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::learn::ui::configure_learn_ui_routes());
        *sub_router = sub_router.clone().merge(
            crate::learn::creator::configure_learn_api_routes()
                .with_state(Arc::new(botlearn::GamificationService::new())),
        );
    }

    #[cfg(feature = "meet")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::meet::ui::configure_meet_ui_routes().with_state(app_state.clone()));
    }

    #[cfg(feature = "billing")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_billing_router(app_state));
    }

    #[cfg(feature = "saas")]
    {
        *sub_router = sub_router
            .clone()
            .merge(crate::main_module::routes::feature_routers::make_saas_router(app_state));
    }
}
