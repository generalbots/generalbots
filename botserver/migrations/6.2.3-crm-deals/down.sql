-- Rollback: 6.2.3-crm-deals
-- ============================================

-- 1. Drop indexes
DROP INDEX IF EXISTS idx_crm_deals_org_bot;
DROP INDEX IF EXISTS idx_crm_deals_contact;
DROP INDEX IF EXISTS idx_crm_deals_account;
DROP INDEX IF EXISTS idx_crm_deals_stage;
DROP INDEX IF EXISTS idx_crm_deals_owner;
DROP INDEX IF EXISTS idx_crm_deals_source;

-- 2. Remove deal_id from crm_activities
ALTER TABLE crm_activities DROP COLUMN IF EXISTS deal_id;

-- 3. Drop crm_deals table
DROP TABLE IF EXISTS crm_deals CASCADE;

-- 4. Drop crm_deal_segments table
DROP TABLE IF EXISTS crm_deal_segments;

-- 5. Recreate old tables (for rollback)
-- Note: These need to be recreated from backup or previous schema
-- This is a placeholder - in production, you'd restore from backup
