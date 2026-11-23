-- #2671 — the default-deny RBAC matrix did not cover state-changing
-- billing / store / voucher / organization / product endpoints. They relied on
-- the coarse global bearer check, so the LLM command catalog could propose and
-- execute them for ordinary users. Writes are admin-only; reads keep their
-- existing tenant-level access.
INSERT INTO rbac_api_permissions (method, path_pattern, group_name) VALUES
    ('POST',   '/api/billing%',             'admin'),
    ('PUT',    '/api/billing%',             'admin'),
    ('DELETE', '/api/billing%',             'admin'),
    ('POST',   '/api/products%',            'admin'),
    ('PUT',    '/api/products%',            'admin'),
    ('DELETE', '/api/products%',            'admin'),
    ('POST',   '/api/cloud/store%',         'admin'),
    ('POST',   '/api/cloud/vouchers%',      'admin'),
    ('PUT',    '/api/cloud/vouchers%',      'admin'),
    ('DELETE', '/api/cloud/vouchers%',      'admin'),
    ('POST',   '/api/cloud/organizations%', 'admin'),
    ('PUT',    '/api/cloud/organizations%', 'admin'),
    ('DELETE', '/api/cloud/organizations%', 'admin'),
    ('POST',   '/api/ui/billing%',          'admin'),
    ('PUT',    '/api/ui/billing%',          'admin')
ON CONFLICT (group_name, method, path_pattern) DO NOTHING;
