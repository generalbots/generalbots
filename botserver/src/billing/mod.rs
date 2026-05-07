use axum::Router;
use std::sync::Arc;

use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;

pub use botbilling::{
    ProductConfig, BrandingConfig, PlanConfig, PlanPrice, BillingPeriod, PlanLimits,
    StripeConfig, LimitValue, BillingAlertNotification,
};

pub mod api {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::bot::get_default_bot;
    use crate::core::shared::state::AppState;

    pub fn configure_billing_api_routes(app_state: &Arc<AppState>) -> Router<()> {
        let state = Arc::new(botbilling::api::BillingApiState {
            pool: Arc::new(app_state.conn.clone()),
            get_default_bot: Some(get_default_bot as botbilling::GetDefaultBotFn),
        });
        botbilling::api::configure_billing_api_routes()
            .with_state(state)
    }
}

pub mod billing_ui {
    use axum::Router;
    use std::sync::Arc;

    use crate::core::bot::get_default_bot;
    use crate::core::shared::state::AppState;

    pub fn configure_billing_routes(app_state: &Arc<AppState>) -> Router<()> {
        let state = Arc::new(botbilling::api::BillingApiState {
            pool: Arc::new(app_state.conn.clone()),
            get_default_bot: Some(get_default_bot as botbilling::GetDefaultBotFn),
        });
        botbilling::billing_ui::configure_billing_routes()
            .with_state(state)
    }
}

pub mod alerts {
    pub use botbilling::alerts::*;
}

pub mod invoice {
    pub use botbilling::invoice::*;
}

pub mod lifecycle {
    pub use botbilling::lifecycle::*;
}

pub mod meters {
    pub use botbilling::meters::*;
}

pub mod middleware {
    pub use botbilling::middleware::*;
}

pub mod plans {
    pub use botbilling::plans::*;
}

pub mod quotas {
    pub use botbilling::quotas::*;
}

pub mod stripe_integration {
    pub use botbilling::stripe_integration::*;
}

pub mod testing {
    pub use botbilling::testing::*;
}
