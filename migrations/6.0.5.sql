-- Migration 6.0.5: Add update-summary.bas scheduled automation
-- Description: Creates a scheduled automation that runs every minute to update summaries
-- This replaces the announcements system in legacy mode
-- Note: Bots are now created dynamically during bootstrap based on template folders

-- Add name column to system_automations if it doesn't exist
ALTER TABLE public.system_automations ADD COLUMN IF NOT EXISTS name VARCHAR(255);

-- Create index on name column for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_automations_name ON public.system_automations(name);

ALTER TABLE bot_configuration
ADD CONSTRAINT bot_configuration_config_key_unique UNIQUE (config_key);

-- Migration 6.0.9: Add bot_id column to system_automations
-- Description: Introduces a bot_id column to associate automations with a specific bot.
-- The column is added as UUID and indexed for efficient queries.

-- Add bot_id column if it does not exist
ALTER TABLE public.system_automations
ADD COLUMN IF NOT EXISTS bot_id UUID NOT NULL;

-- Create an index on bot_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_automations_bot_id
ON public.system_automations (bot_id);


ALTER TABLE public.system_automations
ADD CONSTRAINT IF NOT EXISTS system_automations_bot_kind_param_unique
UNIQUE (bot_id, kind, param);

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
