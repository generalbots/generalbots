-- Add slug column to bots table for drive-based bot creation
ALTER TABLE public.bots ADD COLUMN IF NOT EXISTS slug VARCHAR(255);

-- Create unique constraint on slug (without WHERE clause for ON CONFLICT to work)
ALTER TABLE public.bots ADD CONSTRAINT bots_slug_key UNIQUE (slug);

-- Backfill slug from name for existing bots
UPDATE public.bots SET slug = name WHERE slug IS NULL;
