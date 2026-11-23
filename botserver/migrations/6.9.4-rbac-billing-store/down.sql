-- Revert #2671 admin-only billing/store rows.
DELETE FROM rbac_api_permissions
WHERE group_name = 'admin'
  AND (method, path_pattern) IN (
    ('POST',   '/api/billing%'),
    ('PUT',    '/api/billing%'),
    ('DELETE', '/api/billing%'),
    ('POST',   '/api/products%'),
    ('PUT',    '/api/products%'),
    ('DELETE', '/api/products%'),
    ('POST',   '/api/cloud/store%'),
    ('POST',   '/api/cloud/vouchers%'),
    ('PUT',    '/api/cloud/vouchers%'),
    ('DELETE', '/api/cloud/vouchers%'),
    ('POST',   '/api/cloud/organizations%'),
    ('PUT',    '/api/cloud/organizations%'),
    ('DELETE', '/api/cloud/organizations%'),
    ('POST',   '/api/ui/billing%'),
    ('PUT',    '/api/ui/billing%')
  );
