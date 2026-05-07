use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;
use crate::core::urls::ApiUrls;

pub use botmonitoring::{
    DefaultMonitoringUrls, DistributedTracingService, MetricsCollector, MonitoringState,
    MonitoringUrls, TraceContext,
};

pub mod real_time {
    pub use botmonitoring::real_time::MetricsCollector;
}

pub mod tracing {
    pub use botmonitoring::tracing::{DistributedTracingService, TraceContext};
}

struct BotServerMonitoringUrls;

impl MonitoringUrls for BotServerMonitoringUrls {
    fn monitoring_dashboard() -> &'static str { ApiUrls::MONITORING_DASHBOARD }
    fn monitoring_services() -> &'static str { ApiUrls::MONITORING_SERVICES }
    fn monitoring_resources() -> &'static str { ApiUrls::MONITORING_RESOURCES }
    fn monitoring_logs() -> &'static str { ApiUrls::MONITORING_LOGS }
    fn monitoring_llm() -> &'static str { ApiUrls::MONITORING_LLM }
    fn monitoring_health() -> &'static str { ApiUrls::MONITORING_HEALTH }
    fn monitoring_timestamp() -> &'static str { ApiUrls::MONITORING_TIMESTAMP }
    fn monitoring_bots() -> &'static str { ApiUrls::MONITORING_BOTS }
    fn monitoring_services_status() -> &'static str { ApiUrls::MONITORING_SERVICES_STATUS }
    fn monitoring_resources_bars() -> &'static str { ApiUrls::MONITORING_RESOURCES_BARS }
    fn monitoring_activity_latest() -> &'static str { ApiUrls::MONITORING_ACTIVITY_LATEST }
    fn monitoring_metric_sessions() -> &'static str { ApiUrls::MONITORING_METRIC_SESSIONS }
    fn monitoring_metric_messages() -> &'static str { ApiUrls::MONITORING_METRIC_MESSAGES }
    fn monitoring_metric_response_time() -> &'static str { ApiUrls::MONITORING_METRIC_RESPONSE_TIME }
    fn monitoring_trend_sessions() -> &'static str { ApiUrls::MONITORING_TREND_SESSIONS }
    fn monitoring_rate_messages() -> &'static str { ApiUrls::MONITORING_RATE_MESSAGES }
    fn monitoring_sessions_panel() -> &'static str { ApiUrls::MONITORING_SESSIONS_PANEL }
    fn monitoring_messages_panel() -> &'static str { ApiUrls::MONITORING_MESSAGES_PANEL }
}

impl MonitoringState for AppState {
    fn active_session_count(&self) -> usize {
        self.session_manager
            .try_lock()
            .map(|sm| sm.active_count())
            .unwrap_or(0)
    }

    fn is_db_healthy(&self) -> bool {
        self.conn.get().is_ok()
    }
}

pub fn configure(state: &Arc<AppState>) -> Router {
    botmonitoring::configure::<AppState, BotServerMonitoringUrls>()
        .with_state(state.clone())
}
