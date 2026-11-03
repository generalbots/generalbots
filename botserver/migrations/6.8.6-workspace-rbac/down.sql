DELETE FROM rbac_api_permissions
WHERE group_name = 'admin' AND method = 'GET' AND path_pattern = '/api/runs/quality';
