DROP TRIGGER IF EXISTS trigger_db_saved_queries_updated_at ON database_saved_queries;
DROP FUNCTION IF EXISTS update_db_saved_queries_updated_at();
DROP TABLE IF EXISTS database_saved_queries;
DROP TABLE IF EXISTS database_query_history;
