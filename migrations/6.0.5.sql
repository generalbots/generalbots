-- Migration 6.0.5: Add update-summary.bas scheduled automation
-- Description: Creates a scheduled automation that runs every minute to update summaries
-- This replaces the announcements system in legacy mode
-- Note: Bots are now created dynamically during bootstrap based on template folders

-- Add name column to system_automations if it doesn't exist
ALTER TABLE public.system_automations ADD COLUMN IF NOT EXISTS name VARCHAR(255);

-- Insert update-summary automation (runs every minute)
-- kind = 3 (Scheduled trigger)
-- schedule format: minute hour day month weekday
-- "* * * * *" = every minute
INSERT INTO public.system_automations (name, kind, target, param, schedule, is_active)
VALUES (
    'Update Summary',
    0,
    NULL,
    'update-summary.bas',
    '* * * * *',
    true
)
ON CONFLICT DO NOTHING;

-- Create index on name column for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_automations_name ON public.system_automations(name);
