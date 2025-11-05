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
ADD CONSTRAINT system_automations_bot_kind_param_unique
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

-- Migration 6.0.6: Add unique constraint for system_automations
-- Fixes error: "there is no unique or exclusion constraint matching the ON CONFLICT specification"

ALTER TABLE public.system_automations
ADD CONSTRAINT system_automations_bot_kind_param_unique
UNIQUE (bot_id, kind, param);

-- Add index for the new constraint
CREATE INDEX IF NOT EXISTS idx_system_automations_bot_kind_param 
ON public.system_automations (bot_id, kind, param);


-- Migration 6.0.7: Fix clicks table primary key
-- Required by Diesel before we can run other migrations

-- Create new table with proper structure
CREATE TABLE IF NOT EXISTS public.new_clicks (
    id SERIAL PRIMARY KEY,
    campaign_id text NOT NULL,
    email text NOT NULL,
    updated_at timestamptz DEFAULT now() NULL,
    CONSTRAINT new_clicks_campaign_id_email_key UNIQUE (campaign_id, email)
);

-- Copy data from old table
INSERT INTO public.new_clicks (campaign_id, email, updated_at)
SELECT campaign_id, email, updated_at FROM public.clicks;

-- Drop old table and rename new one
DROP TABLE public.clicks;
ALTER TABLE public.new_clicks RENAME TO clicks;
