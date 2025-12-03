-- Rollback Suite Applications Migration
-- Removes tables for: Paper (Documents), Designer (Dialogs), and analytics support

-- Drop indexes first
DROP INDEX IF EXISTS idx_research_history_created;
DROP INDEX IF EXISTS idx_research_history_user;
DROP INDEX IF EXISTS idx_analytics_daily_bot;
DROP INDEX IF EXISTS idx_analytics_daily_date;
DROP INDEX IF EXISTS idx_analytics_events_created;
DROP INDEX IF EXISTS idx_analytics_events_session;
DROP INDEX IF EXISTS idx_analytics_events_user;
DROP INDEX IF EXISTS idx_analytics_events_type;
DROP INDEX IF EXISTS idx_source_templates_category;
DROP INDEX IF EXISTS idx_designer_dialogs_updated;
DROP INDEX IF EXISTS idx_designer_dialogs_active;
DROP INDEX IF EXISTS idx_designer_dialogs_bot;
DROP INDEX IF EXISTS idx_paper_documents_updated;
DROP INDEX IF EXISTS idx_paper_documents_owner;

-- Drop tables
DROP TABLE IF EXISTS research_search_history;
DROP TABLE IF EXISTS analytics_daily_aggregates;
DROP TABLE IF EXISTS analytics_events;
DROP TABLE IF EXISTS source_templates;
DROP TABLE IF EXISTS designer_dialogs;
DROP TABLE IF EXISTS paper_documents;
