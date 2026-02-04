ALTER TABLE bots ADD COLUMN IF NOT EXISTS database_name VARCHAR(255) NULL;
ALTER TABLE bots ADD COLUMN IF NOT EXISTS tenant_id UUID NULL;

CREATE INDEX IF NOT EXISTS idx_bots_database_name ON bots(database_name);
CREATE INDEX IF NOT EXISTS idx_bots_tenant_id ON bots(tenant_id);

COMMENT ON COLUMN bots.database_name IS 'Name of the PostgreSQL database for this bot (bot_{name})';
COMMENT ON COLUMN bots.tenant_id IS 'Tenant/organization ID for multi-tenant isolation';
