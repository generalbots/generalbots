-- =============================================================================
-- 6.5.36 Schema reconcile
--
-- Reconciles database schema drift introduced by the branch-scope cleanup
-- (issue #707, commit 3a0a1f791). The Diesel schema.rs files for
-- botmarketing and botcompliance were regenerated to new column names, but no
-- migration updated the live tables. This migration aligns the database with
-- the compiled code without losing existing data.
--
-- Changes:
--   marketing_campaigns: channel->campaign_type, scheduled_at->starts_at,
--     completed_at->ends_at; budget double precision -> numeric;
--     org_id/bot_id relaxed (branch_id is the sole scope now).
--   marketing_lists: add description, member_count, is_dynamic, criteria;
--     org_id/bot_id relaxed.
--   compliance_checks: framework->check_type, control_id->target_type,
--     control_name+score folded into result jsonb, add target_id;
--     drop control_name/score/evidence/notes; org_id/bot_id relaxed.
-- =============================================================================

-- ---------------------------------------------------------------------------
-- 1. marketing_campaigns
-- ---------------------------------------------------------------------------
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

ALTER TABLE marketing_campaigns ALTER COLUMN budget TYPE numeric USING budget::numeric;
ALTER TABLE marketing_campaigns ALTER COLUMN org_id DROP NOT NULL;
ALTER TABLE marketing_campaigns ALTER COLUMN bot_id DROP NOT NULL;

-- ---------------------------------------------------------------------------
-- 2. marketing_lists
-- ---------------------------------------------------------------------------
ALTER TABLE marketing_lists ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE marketing_lists ADD COLUMN IF NOT EXISTS member_count INTEGER DEFAULT 0;
ALTER TABLE marketing_lists ADD COLUMN IF NOT EXISTS is_dynamic BOOLEAN DEFAULT FALSE;
ALTER TABLE marketing_lists ADD COLUMN IF NOT EXISTS criteria JSONB DEFAULT '{}'::jsonb;
ALTER TABLE marketing_lists ALTER COLUMN org_id DROP NOT NULL;
ALTER TABLE marketing_lists ALTER COLUMN bot_id DROP NOT NULL;

-- ---------------------------------------------------------------------------
-- 3. compliance_checks
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    has_old BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'framework'
    ) INTO has_old;

    IF has_old THEN
        ALTER TABLE compliance_checks RENAME COLUMN framework TO check_type;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'control_id') THEN
        ALTER TABLE compliance_checks RENAME COLUMN control_id TO target_type;
    END IF;
END $$;

ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS target_id UUID;
ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS result JSONB DEFAULT '{}'::jsonb;

DO $$
DECLARE
    has_result JSONB;
    has_control_name BOOLEAN;
    has_score BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'control_name'
    ) INTO has_control_name;
    SELECT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'score'
    ) INTO has_score;

    IF has_control_name OR has_score THEN
        UPDATE compliance_checks SET result = jsonb_build_object(
            'control_id', COALESCE(target_type::text, ''),
            'control_name', COALESCE(control_name::text, ''),
            'score', COALESCE(score::text, '0')
        );
    END IF;

    IF has_control_name THEN
        ALTER TABLE compliance_checks DROP COLUMN control_name;
    END IF;
    IF has_score THEN
        ALTER TABLE compliance_checks DROP COLUMN score;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'evidence') THEN
        ALTER TABLE compliance_checks DROP COLUMN evidence;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns
        WHERE table_name = 'compliance_checks' AND column_name = 'notes') THEN
        ALTER TABLE compliance_checks DROP COLUMN notes;
    END IF;
END $$;

ALTER TABLE compliance_checks ALTER COLUMN org_id DROP NOT NULL;
ALTER TABLE compliance_checks ALTER COLUMN bot_id DROP NOT NULL;
ALTER TABLE compliance_checks ALTER COLUMN status DROP NOT NULL;
ALTER TABLE compliance_checks ALTER COLUMN checked_at DROP NOT NULL;
ALTER TABLE compliance_checks ALTER COLUMN target_type DROP NOT NULL;
