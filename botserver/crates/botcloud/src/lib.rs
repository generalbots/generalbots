pub mod api;
pub mod integration;
pub mod notifier;
pub mod cloud_ui;
pub mod vouchers;
pub mod schema_ext;
pub mod stripe;
pub mod webhook;

use botbilling::api::BillingApiState;
use botbilling::stripe_integration::StripeClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorPayload {
    pub plan: String,
    pub period: String,
    pub storage: f64,
    pub ai: Vec<String>,
    pub total: f64,
    pub currency: String,
}

pub struct SaasService {
    pub billing_state: Arc<BillingApiState>,
    pub stripe: StripeClient,
    pub config: SaasConfig,
}

#[derive(Debug, Clone)]
pub struct SaasConfig {
    pub base_url: String,
    pub jwt_secret: String,
    pub mc_path: String,
    pub mc_alias: String,
    pub directory_api_url: Option<String>,
    pub directory_service_token: Option<String>,
}

impl SaasService {
    pub fn new(
        billing_state: Arc<BillingApiState>,
        stripe: StripeClient,
        config: SaasConfig,
    ) -> Self {
        Self {
            billing_state,
            stripe,
            config,
        }
    }

    pub fn pool(&self) -> &DbPool {
        &self.billing_state.pool
    }
}
