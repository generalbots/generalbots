use axum::Router;

pub fn configure_dashboards_routes(state: std::sync::Arc<botdashboards::DashboardsState>) -> Router {
    botdashboards::configure_dashboards_routes(state.clone()).with_state(state)
}

pub mod ui {
    use axum::Router;
    pub fn configure_dashboards_ui_routes() -> Router<std::sync::Arc<botdashboards::DashboardsState>> {
        botdashboards::ui::configure_dashboards_ui_routes()
    }
}
