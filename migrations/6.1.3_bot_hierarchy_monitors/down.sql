-- Drop comments first
COMMENT ON TABLE public.user_organizations IS NULL;
COMMENT ON TABLE public.email_received_events IS NULL;
COMMENT ON TABLE public.folder_change_events IS NULL;
COMMENT ON TABLE public.folder_monitors IS NULL;
COMMENT ON TABLE public.email_monitors IS NULL;
COMMENT ON COLUMN public.bots.inherit_parent_config IS NULL;
COMMENT ON COLUMN public.bots.enabled_tabs_json IS NULL;
COMMENT ON COLUMN public.bots.parent_bot_id IS NULL;
COMMENT ON TABLE public.system_automations IS NULL;

-- Drop user organizations table
DROP INDEX IF EXISTS idx_user_orgs_default;
DROP INDEX IF EXISTS idx_user_orgs_org;
DROP INDEX IF EXISTS idx_user_orgs_user;
DROP TABLE IF EXISTS public.user_organizations;

-- Drop email received events table
DROP INDEX IF EXISTS idx_email_events_received;
DROP INDEX IF EXISTS idx_email_events_processed;
DROP INDEX IF EXISTS idx_email_events_monitor;
DROP TABLE IF EXISTS public.email_received_events;

-- Drop folder change events table
DROP INDEX IF EXISTS idx_folder_events_created;
DROP INDEX IF EXISTS idx_folder_events_processed;
DROP INDEX IF EXISTS idx_folder_events_monitor;
DROP TABLE IF EXISTS public.folder_change_events;

-- Drop folder monitors table
DROP INDEX IF EXISTS idx_folder_monitors_account_email;
DROP INDEX IF EXISTS idx_folder_monitors_active;
DROP INDEX IF EXISTS idx_folder_monitors_provider;
DROP INDEX IF EXISTS idx_folder_monitors_bot_id;
DROP TABLE IF EXISTS public.folder_monitors;

-- Drop email monitors table
DROP INDEX IF EXISTS idx_email_monitors_active;
DROP INDEX IF EXISTS idx_email_monitors_email;
DROP INDEX IF EXISTS idx_email_monitors_bot_id;
DROP TABLE IF EXISTS public.email_monitors;

-- Remove bot hierarchy columns
DROP INDEX IF EXISTS idx_bots_parent_bot_id;
ALTER TABLE public.bots DROP COLUMN IF EXISTS inherit_parent_config;
ALTER TABLE public.bots DROP COLUMN IF EXISTS enabled_tabs_json;
ALTER TABLE public.bots DROP COLUMN IF EXISTS parent_bot_id;
