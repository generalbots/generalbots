-- Rollback Migration 6.1.1: Revert UUID columns back to TEXT
-- This reverts the id columns from UUID back to TEXT

-- For bot_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'bot_configuration'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE bot_configuration
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For server_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'server_configuration'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE server_configuration
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For tenant_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'tenant_configuration'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE tenant_configuration
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For model_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'model_configurations'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE model_configurations
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For connection_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'connection_configurations'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE connection_configurations
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For component_installations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_installations'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE component_installations
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For component_logs
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_logs'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE component_logs
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- For gbot_config_sync
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'gbot_config_sync'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE gbot_config_sync
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;
