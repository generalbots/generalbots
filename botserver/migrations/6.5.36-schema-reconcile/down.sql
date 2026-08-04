-- =============================================================================
-- 6.5.36 Schema reconcile (down)
-- =============================================================================

-- Restore compliance_checks old column layout (best-effort).
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'check_type')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'framework') THEN
        ALTER TABLE compliance_checks RENAME COLUMN check_type TO framework;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'target_type')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'control_id') THEN
        ALTER TABLE compliance_checks RENAME COLUMN target_type TO control_id;
    END IF;
END $$;

ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS control_name TEXT;
ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS score NUMERIC DEFAULT 0;
ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS evidence JSONB DEFAULT '[]'::jsonb;
ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS notes TEXT;
ALTER TABLE compliance_checks DROP COLUMN IF EXISTS result;
ALTER TABLE compliance_checks DROP COLUMN IF EXISTS target_id;

-- Restore marketing_campaigns old column layout.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'campaign_type')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'channel') THEN
        ALTER TABLE marketing_campaigns RENAME COLUMN campaign_type TO channel;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'starts_at')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'scheduled_at') THEN
        ALTER TABLE marketing_campaigns RENAME COLUMN starts_at TO scheduled_at;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'ends_at')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketing_campaigns' AND column_name = 'completed_at') THEN
        ALTER TABLE marketing_campaigns RENAME COLUMN ends_at TO completed_at;
    END IF;
END $$;

-- Restore marketing_lists (drop new columns).
ALTER TABLE marketing_lists DROP COLUMN IF EXISTS description;
ALTER TABLE marketing_lists DROP COLUMN IF EXISTS member_count;
ALTER TABLE marketing_lists DROP COLUMN IF EXISTS is_dynamic;
ALTER TABLE marketing_lists DROP COLUMN IF EXISTS criteria;
