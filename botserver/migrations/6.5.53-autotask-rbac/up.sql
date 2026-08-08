INSERT INTO rbac_api_permissions (method, path_pattern, group_name) VALUES
    ('POST', '/api/autotask%', 'admin'),
    ('PUT', '/api/autotask%', 'admin'),
    ('DELETE', '/api/autotask%', 'admin')
ON CONFLICT (group_name, method, path_pattern) DO NOTHING;
