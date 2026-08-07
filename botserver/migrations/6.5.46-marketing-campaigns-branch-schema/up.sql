-- 6.5.46-marketing-campaigns-branch-schema
-- The campaign feature's Diesel model (crates/botmarketing/src/schema.rs)
-- reads branch_id, campaign_type, starts_at, ends_at and a numeric budget,
-- but migration 6.2.4-01 created marketing_campaigns with org_id/bot_id,
-- channel, scheduled_at, completed_at and a double-precision budget.
--
-- Migration 6.5.36-schema-reconcile performs the same renames but was never
-- applied to some existing databases (schema drift). This migration is a
-- self-contained, idempotent reconciliation: it converges both freshly
-- migrated and drifted databases to the compiled model without data loss.
--
-- Every step is guarded so re-running is a no-op (fresh DBs, where 6.5.36
-- already renamed the columns, skip straight through).

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'channel')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'campaign_type') THEN
        ALTER TABLE marketing_campaigns RENAME COLUMN channel TO campaign_type;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'scheduled_at')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'starts_at') THEN
        ALTER TABLE marketing_campaigns RENAME COLUMN scheduled_at TO starts_at;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'completed_at')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'ends_at') THEN
        ALTER TABLE marketing_campaigns RENAME COLUMN completed_at TO ends_at;
    END IF;
END $$;

-- Branch scoping is the sole tenant boundary for campaigns; org_id/bot_id are
-- legacy FK columns no longer referenced by the model (#731). Ensure the
-- columns exist and match the model.
ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS branch_id uuid;
ALTER TABLE marketing_campaigns ALTER COLUMN org_id DROP NOT NULL;
ALTER TABLE marketing_campaigns ALTER COLUMN bot_id DROP NOT NULL;

-- Backfill branch_id from the owning bot row where the column was added late.
UPDATE marketing_campaigns mc
SET branch_id = b.branch_id
FROM bots b
WHERE mc.branch_id IS NULL AND b.id = mc.bot_id;

-- Fallback for campaigns without a resolvable bot: keep them under a nil
-- branch so they remain visible to global scope.
ALTER TABLE marketing_campaigns ALTER COLUMN branch_id SET DEFAULT '00000000-0000-0000-0000-000000000000'::uuid;

ALTER TABLE marketing_campaigns ALTER COLUMN budget TYPE numeric USING budget::numeric;

-- The branch_id index is created by 6.5.23-branch-scope-cleanup; keep only
-- that one to avoid a redundant duplicate index.
