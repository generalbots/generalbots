-- #1679 — the run-quality view is an admin surface.
INSERT INTO rbac_api_permissions (method, path_pattern, group_name)
VALUES ('GET', '/api/runs/quality', 'admin')
ON CONFLICT (group_name, method, path_pattern) DO NOTHING;
