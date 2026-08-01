DO $$
BEGIN
    -- One cloud workspace per Drive branch (.gbai prefix) — drive_monitor upserts
    -- these rows so the cloud UI mirrors Drive exactly.
    CREATE UNIQUE INDEX IF NOT EXISTS idx_cloud_workspaces_branch
        ON cloud_workspaces(branch_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating unique index on cloud_workspaces(branch_id): %', SQLERRM;
END $$;
