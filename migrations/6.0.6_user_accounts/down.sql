-- Drop login tokens table
DROP TABLE IF EXISTS public.user_login_tokens;

-- Drop user preferences table
DROP TABLE IF EXISTS public.user_preferences;

-- Remove session enhancement
ALTER TABLE public.user_sessions
DROP CONSTRAINT IF EXISTS user_sessions_email_account_id_fkey,
DROP COLUMN IF EXISTS active_email_account_id;

-- Drop email folders table
DROP TABLE IF EXISTS public.email_folders;

-- Drop email drafts table
DROP TABLE IF EXISTS public.email_drafts;

-- Drop user email accounts table
DROP TABLE IF EXISTS public.user_email_accounts;
