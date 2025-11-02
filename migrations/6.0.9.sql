-- Migration 6.0.9: Add bot_id column to system_automations
-- Description: Introduces a bot_id column to associate automations with a specific bot.
-- The column is added as UUID and indexed for efficient queries.

-- Add bot_id column if it does not exist
ALTER TABLE public.system_automations
ADD COLUMN IF NOT EXISTS bot_id UUID NOT NULL;

-- Create an index on bot_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_automations_bot_id
ON public.system_automations (bot_id);
