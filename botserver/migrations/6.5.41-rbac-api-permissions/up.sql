-- RBAC: API permission matrix (role vs API capabilities).
-- Rows describe endpoints that are DENIED to non-admin users; admins can
-- call everything. The LLM api-command catalog reads this table so it only
-- proposes actions the current user is allowed to execute, and the executor
-- enforces the same rule server-side.
CREATE TABLE IF NOT EXISTS rbac_api_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    method VARCHAR(16) NOT NULL DEFAULT '*',
    path_pattern VARCHAR(255) NOT NULL,
    group_name VARCHAR(100) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (group_name, method, path_pattern)
);

-- Well-known admin-only surfaces (regular users are denied these).
INSERT INTO rbac_api_permissions (method, path_pattern, group_name) VALUES
    ('*', '/api/admin%',          'admin'),
    ('*', '/api/rbac%',           'admin'),
    ('*', '/api/security%',       'admin'),
    ('*', '/api/governance%',     'admin'),
    ('*', '/api/files/bots%',     'admin'),
    ('*', '/api/cloud/domains%',  'admin'),
    ('*', '/api/directory%',      'admin'),
    ('*', '/api/monitoring%',     'admin'),
    ('*', '/api/dashboards%',     'admin')
ON CONFLICT (group_name, method, path_pattern) DO NOTHING;
