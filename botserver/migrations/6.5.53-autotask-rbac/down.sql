DELETE FROM rbac_api_permissions
WHERE group_name = 'admin' AND path_pattern = '/api/autotask%' AND method IN ('POST', 'PUT', 'DELETE');
