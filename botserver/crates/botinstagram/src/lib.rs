use std::sync::Arc;

pub mod adapter;
pub mod auth;
pub mod campaign;
pub mod channel;
pub mod handlers;
pub mod state;
pub mod types;
pub mod webhook;

pub use adapter::InstagramAdapter;
pub use campaign::{configure_campaign_routes, INSTAGRAM_CAMPAIGNS_PATH};
pub use channel::ChannelAdapter;
pub use state::ChannelState;

/// Assembles the whole `/api/instagram/*` surface: webhook/send plus the
/// campaign builder. Mounting this one router (instead of only the webhook
/// half) is what keeps `/api/instagram/campaigns` from 404ing on app open
/// (#2949). New Instagram route groups belong here.
pub fn configure_instagram_router() -> axum::Router<Arc<ChannelState>> {
    webhook::configure().merge(campaign::configure_campaign_routes())
}
