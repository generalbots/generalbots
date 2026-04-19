-- Drop RBAC tables in reverse order of creation

DROP INDEX IF EXISTS idx_rbac_group_roles_role;
DROP INDEX IF EXISTS idx_rbac_group_roles_group;
DROP INDEX IF EXISTS idx_rbac_user_groups_group;
DROP INDEX IF EXISTS idx_rbac_user_groups_user;
DROP INDEX IF EXISTS idx_rbac_user_roles_expires;
DROP INDEX IF EXISTS idx_rbac_user_roles_role;
DROP INDEX IF EXISTS idx_rbac_user_roles_user;
DROP INDEX IF EXISTS idx_rbac_role_permissions_permission;
DROP INDEX IF EXISTS idx_rbac_role_permissions_role;
DROP INDEX IF EXISTS idx_rbac_permissions_resource;
DROP INDEX IF EXISTS idx_rbac_groups_is_active;
DROP INDEX IF EXISTS idx_rbac_groups_parent;
DROP INDEX IF EXISTS idx_rbac_groups_name;
DROP INDEX IF EXISTS idx_rbac_roles_is_active;
DROP INDEX IF EXISTS idx_rbac_roles_name;

DROP TABLE IF EXISTS rbac_group_roles;
DROP TABLE IF EXISTS rbac_user_groups;
DROP TABLE IF EXISTS rbac_user_roles;
DROP TABLE IF EXISTS rbac_role_permissions;
DROP TABLE IF EXISTS rbac_permissions;
DROP TABLE IF EXISTS rbac_groups;
DROP TABLE IF EXISTS rbac_roles;
