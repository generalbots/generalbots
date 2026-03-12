-- Fix tenant/org/bot relationship model
-- Run this SQL to update the database schema

-- 1. Create tenants table if not exists
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Add tenant_id to organizations (if column doesn't exist)
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- 3. Add org_id to bots (replaces tenant_id)
ALTER TABLE bots ADD COLUMN IF NOT EXISTS org_id UUID REFERENCES organizations(org_id);

-- 4. Create default tenant if not exists
INSERT INTO tenants (id, name, slug, created_at)
VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant', 'default', NOW())
ON CONFLICT (slug) DO NOTHING;

-- 5. Create default organization linked to tenant if not exists
INSERT INTO organizations (org_id, tenant_id, name, slug, created_at)
VALUES ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Default Organization', 'default', NOW())
ON CONFLICT (org_id) DO NOTHING;

-- 6. Update existing bots to use the default organization
UPDATE bots SET org_id = '00000000-0000-0000-0000-000000000001' WHERE org_id IS NULL;

-- 7. Make org_id NOT NULL after update
ALTER TABLE bots ALTER COLUMN org_id SET NOT NULL;

-- 8. Add foreign key for org_id in crm_leads (already should exist, but ensure)
-- This assumes crm_leads.org_id references organizations(org_id)

-- 9. Update crm_contacts to use org_id properly (should already link to organizations)
-- 10. Drop old tenant_id column from bots if exists
ALTER TABLE bots DROP COLUMN IF EXISTS tenant_id;

-- 11. Create indexes
CREATE INDEX IF NOT EXISTS idx_organizations_tenant_id ON organizations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_bots_org_id ON bots(org_id);
