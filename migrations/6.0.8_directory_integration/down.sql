-- Drop triggers
DROP TRIGGER IF EXISTS update_directory_users_updated_at ON public.directory_users;
DROP TRIGGER IF EXISTS update_oauth_applications_updated_at ON public.oauth_applications;

-- Drop function if no other triggers use it
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS public.bot_access CASCADE;
DROP TABLE IF EXISTS public.oauth_applications CASCADE;
DROP TABLE IF EXISTS public.directory_users CASCADE;

-- Drop indexes
DROP INDEX IF EXISTS idx_bots_org_id;

-- Remove columns from bots table
ALTER TABLE public.bots
DROP CONSTRAINT IF EXISTS bots_org_id_fkey,
DROP COLUMN IF EXISTS org_id,
DROP COLUMN IF EXISTS is_default;

-- Note: We don't delete the default organization or bot data as they may have other relationships
-- The application should handle orphaned data appropriately
