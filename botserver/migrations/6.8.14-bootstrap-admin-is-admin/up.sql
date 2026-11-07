-- #1839 — the first-run wizard administrator was persisted with
-- `users.is_admin = false`. The desktop login fallback mints a session from
-- `conf/directory/admin-credentials.json`; account-creation paths (AI accounts)
-- then inserted a bare `users` row without the admin flag.
--
-- Backfill the rows we can identify without the credentials file (the server
-- also reconciles the file at boot and on local-admin login). Keep this
-- idempotent: it may run on databases where the rows are already correct.
UPDATE users
SET is_admin = TRUE,
    is_active = TRUE,
    updated_at = NOW()
WHERE is_admin = FALSE
  AND (
      lower(email) IN ('admin@localhost', 'admin@default')
      OR (lower(username) = 'admin' AND lower(email) LIKE '%@localhost')
  );

-- The legacy column is a directory-view mirror of the RBAC groups. If a user
-- is already an active admin through the RBAC tables, keep the mirror honest.
UPDATE users u
SET is_admin = TRUE,
    updated_at = NOW()
WHERE u.is_admin = FALSE
  AND EXISTS (
      SELECT 1
      FROM rbac_user_groups ug
      JOIN rbac_groups g ON g.id = ug.group_id
      WHERE ug.user_id = u.id
        AND g.is_active = TRUE
        AND lower(g.name) LIKE '%admin%'
  );
