-- Migration 6.0.10: Add unique constraint for system_automations upsert
-- Description: Creates a unique constraint matching the ON CONFLICT target in set_schedule.rs

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'system_automations_bot_kind_param_unique'
    ) THEN
        ALTER TABLE public.system_automations
        ADD CONSTRAINT system_automations_bot_kind_param_unique
        UNIQUE (bot_id, kind, param);
    END IF;
END
$$;
