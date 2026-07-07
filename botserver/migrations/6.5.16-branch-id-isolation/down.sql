-- Revert 9.16-branch-id-isolation: Restore FKs on org_id to reference organizations(org_id)

-- 1. Drop the constraints pointing to branches(id)
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

-- 2. Restore org_id values to point back to the actual organization
-- Map from branch.org_id using the current value (which is branch_id)
DO $$
BEGIN
    UPDATE billing_invoices bi SET org_id = (SELECT org_id FROM branches WHERE id = bi.org_id);
    UPDATE billing_recurring br_table SET org_id = (SELECT org_id FROM branches WHERE id = br_table.org_id);
    UPDATE billing_payments bp SET org_id = (SELECT org_id FROM branches WHERE id = bp.org_id);
    UPDATE billing_quotes bq SET org_id = (SELECT org_id FROM branches WHERE id = bq.org_id);
    UPDATE crm_contacts cc SET org_id = (SELECT org_id FROM branches WHERE id = cc.org_id);
    UPDATE crm_deals cd SET org_id = (SELECT org_id FROM branches WHERE id = cd.org_id);
    UPDATE crm_accounts ca SET org_id = (SELECT org_id FROM branches WHERE id = ca.org_id);
END $$;

-- 3. Add original foreign keys pointing to organizations(org_id)
DROP TABLE IF EXISTS cloud_voucher_redemptions;
DROP TABLE IF EXISTS cloud_vouchers;

ALTER TABLE billing_invoices ADD CONSTRAINT billing_invoices_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE billing_recurring ADD CONSTRAINT billing_recurring_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE billing_payments ADD CONSTRAINT billing_payments_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE billing_quotes ADD CONSTRAINT billing_quotes_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE crm_contacts ADD CONSTRAINT crm_contacts_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE crm_deals ADD CONSTRAINT crm_deals_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE crm_accounts ADD CONSTRAINT crm_accounts_org_id_fkey FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
