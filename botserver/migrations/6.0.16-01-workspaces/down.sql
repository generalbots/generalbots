DROP INDEX IF EXISTS idx_workspace_templates_system;
DROP INDEX IF EXISTS idx_workspace_templates_category;
DROP INDEX IF EXISTS idx_workspace_templates_org_bot;

DROP INDEX IF EXISTS idx_workspace_comment_reactions_comment;

DROP INDEX IF EXISTS idx_workspace_comments_unresolved;
DROP INDEX IF EXISTS idx_workspace_comments_parent;
DROP INDEX IF EXISTS idx_workspace_comments_block;
DROP INDEX IF EXISTS idx_workspace_comments_page;
DROP INDEX IF EXISTS idx_workspace_comments_workspace;

DROP INDEX IF EXISTS idx_workspace_page_permissions_user;
DROP INDEX IF EXISTS idx_workspace_page_permissions_page;

DROP INDEX IF EXISTS idx_workspace_page_versions_number;
DROP INDEX IF EXISTS idx_workspace_page_versions_page;

DROP INDEX IF EXISTS idx_workspace_pages_position;
DROP INDEX IF EXISTS idx_workspace_pages_public;
DROP INDEX IF EXISTS idx_workspace_pages_template;
DROP INDEX IF EXISTS idx_workspace_pages_parent;
DROP INDEX IF EXISTS idx_workspace_pages_workspace;

DROP INDEX IF EXISTS idx_workspace_members_role;
DROP INDEX IF EXISTS idx_workspace_members_user;
DROP INDEX IF EXISTS idx_workspace_members_workspace;

DROP INDEX IF EXISTS idx_workspaces_created_by;
DROP INDEX IF EXISTS idx_workspaces_org_bot;

DROP TABLE IF EXISTS workspace_templates;
DROP TABLE IF EXISTS workspace_comment_reactions;
DROP TABLE IF EXISTS workspace_comments;
DROP TABLE IF EXISTS workspace_page_permissions;
DROP TABLE IF EXISTS workspace_page_versions;
DROP TABLE IF EXISTS workspace_pages;
DROP TABLE IF EXISTS workspace_members;
DROP TABLE IF EXISTS workspaces;
