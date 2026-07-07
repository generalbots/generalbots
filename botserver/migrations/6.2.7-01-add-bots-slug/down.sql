-- Remove slug column from bots table
ALTER TABLE public.bots DROP COLUMN IF EXISTS slug;
