-- 9.16-branch-id-isolation: Update FKs on org_id to reference branches(id) instead of organizations(org_id)
-- This implements the branch_id isolation mapping semantically onto the org_id column

-- 1. Drop existing foreign key constraints on the org_id column pointing to organizations
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN (
        SELECT tc.table_name, tc.constraint_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
          AND tc.table_schema = kcu.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
          AND tc.table_schema = 'public'
          AND kcu.column_name = 'org_id'
          AND tc.table_name IN ('billing_invoices', 'billing_recurring', 'billing_payments', 'billing_quotes', 'crm_contacts', 'crm_deals', 'crm_accounts')
    ) LOOP
        EXECUTE 'ALTER TABLE ' || quote_ident(r.table_name) || ' DROP CONSTRAINT ' || quote_ident(r.constraint_name);
    END LOOP;
END $$;

-- 2. Populate org_id for existing rows with their respective branch_id
-- If the record has a bot_id, we use that bot's branch_id.
-- Otherwise, we use the default branch of the organization.
-- Fallback to the default system branch if all else fails.
DO $$
DECLARE
    default_branch_id UUID := '00000000-0000-0000-0000-000000000001';
BEGIN

    -- Update billing_invoices
    UPDATE billing_invoices bi
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = bi.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = bi.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

    -- Update billing_recurring
    UPDATE billing_recurring br_table
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = br_table.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = br_table.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

    -- Update billing_payments
    UPDATE billing_payments bp
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = bp.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = bp.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

    -- Update billing_quotes
    UPDATE billing_quotes bq
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = bq.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = bq.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

    -- Update crm_contacts
    UPDATE crm_contacts cc
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = cc.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = cc.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

    -- Update crm_deals
    UPDATE crm_deals cd
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = cd.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = cd.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

    -- Update crm_accounts
    UPDATE crm_accounts ca
    SET org_id = COALESCE(
        (SELECT b.branch_id FROM bots b WHERE b.id = ca.bot_id),
        (SELECT br.id FROM branches br WHERE br.org_id = ca.org_id ORDER BY br.is_active DESC, br.created_at ASC LIMIT 1),
        default_branch_id
    );

END $$;

-- 3. Add the new foreign key constraints on the org_id column pointing to branches(id)
ALTER TABLE billing_invoices ADD CONSTRAINT billing_invoices_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE billing_recurring ADD CONSTRAINT billing_recurring_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE billing_payments ADD CONSTRAINT billing_payments_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE billing_quotes ADD CONSTRAINT billing_quotes_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE crm_contacts ADD CONSTRAINT crm_contacts_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE crm_deals ADD CONSTRAINT crm_deals_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE crm_accounts ADD CONSTRAINT crm_accounts_org_id_fkey FOREIGN KEY (org_id) REFERENCES branches(id) ON DELETE CASCADE;

-- 4. Create cloud_vouchers and cloud_voucher_redemptions tables
CREATE TABLE IF NOT EXISTS cloud_vouchers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code            VARCHAR(20) NOT NULL UNIQUE,
    plan            VARCHAR(50) NOT NULL DEFAULT 'shared',
    trial_days      INTEGER NOT NULL CHECK (trial_days >= 1 AND trial_days <= 180),
    max_uses        INTEGER NOT NULL DEFAULT 1,
    uses_count      INTEGER NOT NULL DEFAULT 0,
    created_by      UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vouchers_code ON cloud_vouchers(code);
CREATE INDEX IF NOT EXISTS idx_vouchers_expires ON cloud_vouchers(expires_at);

CREATE TABLE IF NOT EXISTS cloud_voucher_redemptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voucher_id      UUID NOT NULL REFERENCES cloud_vouchers(id) ON DELETE CASCADE,
    contact_id      UUID NOT NULL REFERENCES crm_contacts(id) ON DELETE CASCADE,
    org_id          UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    branch_id       UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    subscription_id UUID,
    trial_days      INTEGER NOT NULL,
    redeemed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_redemptions_voucher ON cloud_voucher_redemptions(voucher_id);
CREATE INDEX IF NOT EXISTS idx_redemptions_contact ON cloud_voucher_redemptions(contact_id);

