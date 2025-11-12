-- Drop indexes first
DROP INDEX IF EXISTS idx_gbot_sync_bot;
DROP INDEX IF EXISTS idx_component_logs_created;
DROP INDEX IF EXISTS idx_component_logs_level;
DROP INDEX IF EXISTS idx_component_logs_component;
DROP INDEX IF EXISTS idx_component_status;
DROP INDEX IF EXISTS idx_component_name;
DROP INDEX IF EXISTS idx_connection_config_active;
DROP INDEX IF EXISTS idx_connection_config_name;
DROP INDEX IF EXISTS idx_connection_config_bot;
DROP INDEX IF EXISTS idx_model_config_default;
DROP INDEX IF EXISTS idx_model_config_active;
DROP INDEX IF EXISTS idx_model_config_type;
DROP INDEX IF EXISTS idx_bot_config_key;
DROP INDEX IF EXISTS idx_bot_config_bot;
DROP INDEX IF EXISTS idx_tenant_config_key;
DROP INDEX IF EXISTS idx_tenant_config_tenant;
DROP INDEX IF EXISTS idx_server_config_type;
DROP INDEX IF EXISTS idx_server_config_key;

-- Drop tables
DROP TABLE IF EXISTS gbot_config_sync;
DROP TABLE IF EXISTS component_logs;
DROP TABLE IF EXISTS component_installations;
DROP TABLE IF EXISTS connection_configurations;
DROP TABLE IF EXISTS model_configurations;
DROP TABLE IF EXISTS bot_configuration;
DROP TABLE IF EXISTS tenant_configuration;
DROP TABLE IF EXISTS server_configuration;

-- Remove added columns if they exist
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'user_sessions' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE user_sessions DROP COLUMN tenant_id;
    END IF;
    
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'bots' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE bots DROP COLUMN tenant_id;
    END IF;
END $$;

-- Drop tenant indexes if they exist
DROP INDEX IF EXISTS idx_user_sessions_tenant;
DROP INDEX IF EXISTS idx_bots_tenant;

-- Remove default tenant
DELETE FROM tenants WHERE slug = 'default';
