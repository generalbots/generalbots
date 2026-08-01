DO $$
BEGIN
    DROP INDEX IF EXISTS idx_cloud_workspaces_branch;
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error dropping idx_cloud_workspaces_branch: %', SQLERRM;
END $$;
