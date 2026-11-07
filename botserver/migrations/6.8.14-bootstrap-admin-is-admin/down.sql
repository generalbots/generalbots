-- Data backfill only. Demoting the same rows on rollback could remove real
-- administrators (especially the RBAC-derived ones), so leave the flags intact.
SELECT 1;
