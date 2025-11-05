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
