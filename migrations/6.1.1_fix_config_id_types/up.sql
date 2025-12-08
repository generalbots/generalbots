-- Migration 6.1.1: Fix bot_configuration id column type
-- The Diesel schema expects UUID but migration 6.0.4 created it as TEXT
-- This migration converts the id column from TEXT to UUID

-- For bot_configuration (main table that needs fixing)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'bot_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE bot_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Also fix server_configuration which has the same issue
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'server_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE server_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Also fix tenant_configuration which has the same issue
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'tenant_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE tenant_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Fix model_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'model_configurations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE model_configurations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Fix connection_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'connection_configurations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE connection_configurations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Fix component_installations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_installations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE component_installations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Fix component_logs
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_logs'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE component_logs
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- Fix gbot_config_sync
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'gbot_config_sync'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE gbot_config_sync
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;
