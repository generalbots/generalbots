pub mod tax;
pub mod video;
pub mod vision;
pub mod fraud;
pub mod erp;
pub mod integrations;
pub mod itsm;
pub mod hr;
pub mod banking;
pub mod sales;
pub mod pos;
pub mod handoff;
pub mod kyc;
pub mod timeclock;
pub mod m365;
pub mod minutes;
pub mod templates_app;

use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .merge(tax::routes())
        .merge(video::routes())
        .merge(vision::routes())
        .merge(fraud::routes())
        .merge(erp::routes())
        .merge(integrations::routes())
        .merge(itsm::routes())
        .merge(hr::routes())
        .merge(banking::routes())
        .merge(sales::routes())
        .merge(pos::routes())
        .merge(handoff::routes())
        .merge(kyc::routes())
        .merge(timeclock::routes())
        .merge(m365::routes())
        .merge(minutes::routes())
        .merge(templates_app::routes())
}
