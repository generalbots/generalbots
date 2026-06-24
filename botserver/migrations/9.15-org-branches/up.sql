DO $$
BEGIN

-- 9.15-org-branches: Add branches table for Tenant > Branch > Bot hierarchy
-- Aligns with ISO 27001 tenant isolation (Forest/Domain model)

-- Ensure tenants table and organizations.tenant_id exist (pre-requisites)
CREATE TABLE IF NOT EXISTS tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    slug        VARCHAR(255) NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE organizations ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

CREATE TABLE IF NOT EXISTS branches (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    slug        VARCHAR(255) NOT NULL,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    is_active   BOOLEAN DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, slug)
);

ALTER TABLE bots ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id);
ALTER TABLE bots ADD COLUMN IF NOT EXISTS is_default_for_branch BOOLEAN DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_branches_org ON branches(org_id);
CREATE INDEX IF NOT EXISTS idx_branches_tenant ON branches(tenant_id);
CREATE INDEX IF NOT EXISTS idx_branches_slug ON branches(slug);
CREATE INDEX IF NOT EXISTS idx_bots_branch ON bots(branch_id);

END $$;

-- Bootstrap defaults: ensure tenant/org/branch/bot exist for fresh installs
DO $$
DECLARE
    v_org_id UUID;
    v_tenant_id UUID;
BEGIN
    INSERT INTO tenants (id, name, slug, created_at)
    VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant', 'default', NOW())
    ON CONFLICT (slug) DO NOTHING;

    INSERT INTO organizations (org_id, tenant_id, name, slug, created_at)
    VALUES ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Default Organization', 'default', NOW())
    ON CONFLICT (slug) DO NOTHING;

    -- Use the actual org_id from the database (not hardcoded)
    SELECT org_id INTO v_org_id FROM organizations WHERE slug = 'default';
    SELECT id INTO v_tenant_id FROM tenants WHERE slug = 'default';

    INSERT INTO branches (id, org_id, tenant_id, slug, name, created_at)
    VALUES ('00000000-0000-0000-0000-000000000001', v_org_id, v_tenant_id, 'default', 'Default Branch', NOW())
    ON CONFLICT (org_id, slug) DO NOTHING;
END $$;

-- Migrate existing bots: create a branch for each bot (1:1 mapping)
-- For bots without org_id, assign to default org
DO $$
DECLARE
    default_org_id UUID := '00000000-0000-0000-0000-000000000001';
    default_tenant_id UUID;
    rec RECORD;
BEGIN
    SELECT id INTO default_tenant_id FROM tenants WHERE slug = 'default';

    FOR rec IN SELECT id, name, org_id, COALESCE(org_id, default_org_id) AS resolved_org_id FROM bots LOOP
        INSERT INTO branches (id, org_id, tenant_id, slug, name, created_at)
        VALUES (gen_random_uuid(), rec.resolved_org_id, default_tenant_id, rec.name, rec.name, NOW())
        ON CONFLICT (org_id, slug) DO NOTHING;

        UPDATE bots b
        SET branch_id = br.id, is_default_for_branch = true
        FROM branches br
        WHERE b.id = rec.id
          AND br.org_id = rec.resolved_org_id
          AND br.slug = rec.name;
    END LOOP;
END $$;

-- Make branch_id and org_id NOT NULL after migration (safely)
DO $$
BEGIN
    -- Fill null org_ids with default org before making NOT NULL
    UPDATE bots SET org_id = '00000000-0000-0000-0000-000000000001' WHERE org_id IS NULL;
    IF NOT EXISTS (SELECT 1 FROM bots WHERE branch_id IS NULL) THEN
        ALTER TABLE bots ALTER COLUMN branch_id SET NOT NULL;
    END IF;
    ALTER TABLE bots ALTER COLUMN org_id SET NOT NULL;
END $$;
