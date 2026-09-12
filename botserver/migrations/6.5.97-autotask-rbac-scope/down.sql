-- Restore the blanket admin-only rule for mutating AutoTask endpoints (#1362).
DELETE FROM rbac_api_permissions
 WHERE group_name = 'admin'
   AND method = 'GET'
   AND path_pattern IN ('/api/autotask/tasks', '/api/autotask/stats');

INSERT INTO rbac_api_permissions (method, path_pattern, group_name) VALUES
    ('POST', '/api/autotask%', 'admin'),
    ('PUT', '/api/autotask%', 'admin'),
    ('DELETE', '/api/autotask%', 'admin')
ON CONFLICT (group_name, method, path_pattern) DO NOTHING;
