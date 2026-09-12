-- #1362 — AutoTask RBAC was blanket admin-only for every mutating endpoint
-- (`POST|PUT|DELETE /api/autotask%`), which blocked per-user private tasks:
-- a regular bot user could not classify, create or execute an AutoTask against
-- their own bot/session (the #1328 media-filing use case).
--
-- Those mutating endpoints are session-scoped — the caller acts on their own
-- bot/session — so they are opened to any authenticated user. Only the
-- genuinely cross-tenant reads (the global task list and the global counters)
-- remain admin-only for the LLM api-command catalog.
--
-- Matching note: `is_admin_only_endpoint` matches by prefix
-- (`path.starts_with(pattern_without_trailing_%)`), so the patterns below are
-- deliberately exact GET paths; they never collide with the POST
-- `/api/autotask/tasks/:id/{approve,cancel}` routes.
DELETE FROM rbac_api_permissions
 WHERE group_name = 'admin'
   AND path_pattern = '/api/autotask%';

INSERT INTO rbac_api_permissions (method, path_pattern, group_name) VALUES
    ('GET', '/api/autotask/tasks', 'admin'),
    ('GET', '/api/autotask/stats', 'admin')
ON CONFLICT (group_name, method, path_pattern) DO NOTHING;
