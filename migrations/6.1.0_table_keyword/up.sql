-- Migration for TABLE keyword support
-- Stores dynamic table definitions created via BASIC TABLE...END TABLE syntax

-- Table to store dynamic table definitions (metadata)
CREATE TABLE IF NOT EXISTS dynamic_table_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    table_name VARCHAR(255) NOT NULL,
    connection_name VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,

    -- Ensure unique table name per bot and connection
    CONSTRAINT unique_bot_table_connection UNIQUE (bot_id, table_name, connection_name),

    -- Foreign key to bots table
    CONSTRAINT fk_dynamic_table_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Table to store field definitions for dynamic tables
CREATE TABLE IF NOT EXISTS dynamic_table_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_definition_id UUID NOT NULL,
    field_name VARCHAR(255) NOT NULL,
    field_type VARCHAR(100) NOT NULL,
    field_length INTEGER,
    field_precision INTEGER,
    is_key BOOLEAN DEFAULT false,
    is_nullable BOOLEAN DEFAULT true,
    default_value TEXT,
    reference_table VARCHAR(255),
    field_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure unique field name per table definition
    CONSTRAINT unique_table_field UNIQUE (table_definition_id, field_name),

    -- Foreign key to table definitions
    CONSTRAINT fk_field_table_definition
        FOREIGN KEY (table_definition_id)
        REFERENCES dynamic_table_definitions(id)
        ON DELETE CASCADE
);

-- Table to store external database connections (from config.csv conn-* entries)
CREATE TABLE IF NOT EXISTS external_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    connection_name VARCHAR(255) NOT NULL,
    driver VARCHAR(100) NOT NULL,
    server VARCHAR(255) NOT NULL,
    port INTEGER,
    database_name VARCHAR(255),
    username VARCHAR(255),
    password_encrypted TEXT,
    additional_params JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_connected_at TIMESTAMPTZ,

    -- Ensure unique connection name per bot
    CONSTRAINT unique_bot_connection UNIQUE (bot_id, connection_name),

    -- Foreign key to bots table
    CONSTRAINT fk_external_connection_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_bot_id
    ON dynamic_table_definitions(bot_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_name
    ON dynamic_table_definitions(table_name);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_connection
    ON dynamic_table_definitions(connection_name);

CREATE INDEX IF NOT EXISTS idx_dynamic_table_fields_table_id
    ON dynamic_table_fields(table_definition_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_fields_name
    ON dynamic_table_fields(field_name);

CREATE INDEX IF NOT EXISTS idx_external_connections_bot_id
    ON external_connections(bot_id);
CREATE INDEX IF NOT EXISTS idx_external_connections_name
    ON external_connections(connection_name);

-- Create trigger to update updated_at timestamp for dynamic_table_definitions
CREATE OR REPLACE FUNCTION update_dynamic_table_definitions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER dynamic_table_definitions_updated_at_trigger
    BEFORE UPDATE ON dynamic_table_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_dynamic_table_definitions_updated_at();

-- Create trigger to update updated_at timestamp for external_connections
CREATE OR REPLACE FUNCTION update_external_connections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER external_connections_updated_at_trigger
    BEFORE UPDATE ON external_connections
    FOR EACH ROW
    EXECUTE FUNCTION update_external_connections_updated_at();
