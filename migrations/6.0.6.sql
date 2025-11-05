-- Migration 6.0.6: Add unique constraint for system_automations
-- Fixes error: "there is no unique or exclusion constraint matching the ON CONFLICT specification"

ALTER TABLE public.system_automations
ADD CONSTRAINT system_automations_bot_kind_param_unique
UNIQUE (bot_id, kind, param);

-- Add index for the new constraint
CREATE INDEX IF NOT EXISTS idx_system_automations_bot_kind_param 
ON public.system_automations (bot_id, kind, param);
