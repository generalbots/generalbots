DROP INDEX IF EXISTS idx_dashboard_filters_dashboard;
DROP INDEX IF EXISTS idx_dashboard_data_sources_dashboard;
DROP INDEX IF EXISTS idx_dashboard_data_sources_org_bot;
DROP INDEX IF EXISTS idx_dashboard_widgets_dashboard;
DROP INDEX IF EXISTS idx_dashboards_template;
DROP INDEX IF EXISTS idx_dashboards_public;
DROP INDEX IF EXISTS idx_dashboards_owner;
DROP INDEX IF EXISTS idx_dashboards_org_bot;

DROP TABLE IF EXISTS dashboard_filters;
DROP TABLE IF EXISTS dashboard_widgets;
DROP TABLE IF EXISTS dashboard_data_sources;
DROP TABLE IF EXISTS dashboards;
