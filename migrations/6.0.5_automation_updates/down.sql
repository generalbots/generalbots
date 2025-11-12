-- Revert clicks table changes
CREATE TABLE IF NOT EXISTS public.old_clicks (
    campaign_id text NOT NULL,
    email text NOT NULL,
    updated_at timestamptz DEFAULT now() NULL,
    CONSTRAINT clicks_campaign_id_email_key UNIQUE (campaign_id, email)
);

INSERT INTO public.old_clicks (campaign_id, email, updated_at)
SELECT campaign_id, email, updated_at FROM public.clicks;

DROP TABLE public.clicks;
ALTER TABLE public.old_clicks RENAME TO clicks;

-- Remove system_automations constraints and indexes
DROP INDEX IF EXISTS idx_system_automations_bot_kind_param;
ALTER TABLE public.system_automations DROP CONSTRAINT IF EXISTS system_automations_bot_kind_param_unique;

DROP INDEX IF EXISTS idx_system_automations_bot_id;
ALTER TABLE public.system_automations DROP COLUMN IF EXISTS bot_id;

DROP INDEX IF EXISTS idx_system_automations_name;
ALTER TABLE public.system_automations DROP COLUMN IF EXISTS name;

-- Remove bot_configuration constraint
ALTER TABLE bot_configuration DROP CONSTRAINT IF EXISTS bot_configuration_config_key_unique;
