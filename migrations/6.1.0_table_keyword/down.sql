-- Drop triggers and functions
DROP TRIGGER IF EXISTS external_connections_updated_at_trigger ON external_connections;
DROP FUNCTION IF EXISTS update_external_connections_updated_at();

DROP TRIGGER IF EXISTS dynamic_table_definitions_updated_at_trigger ON dynamic_table_definitions;
DROP FUNCTION IF EXISTS update_dynamic_table_definitions_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_external_connections_name;
DROP INDEX IF EXISTS idx_external_connections_bot_id;

DROP INDEX IF EXISTS idx_dynamic_table_fields_name;
DROP INDEX IF EXISTS idx_dynamic_table_fields_table_id;

DROP INDEX IF EXISTS idx_dynamic_table_definitions_connection;
DROP INDEX IF EXISTS idx_dynamic_table_definitions_name;
DROP INDEX IF EXISTS idx_dynamic_table_definitions_bot_id;

-- Drop tables (order matters due to foreign keys)
DROP TABLE IF EXISTS external_connections;
DROP TABLE IF EXISTS dynamic_table_fields;
DROP TABLE IF EXISTS dynamic_table_definitions;
