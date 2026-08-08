-- 6.5.51-project-members
-- #768: per-project RBAC. Members can be individual users or groups
-- (group grants are resolved through the rbac_user_groups/rbac_groups
-- tables at request time). A project member row is exactly one of:
-- user_id XOR group_name. Roles rank: owner > admin > developer > viewer.

CREATE TABLE IF NOT EXISTS project_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES vibe_projects(id) ON DELETE CASCADE,
    user_id UUID,
    group_name VARCHAR(255),
    role VARCHAR(32) NOT NULL DEFAULT 'viewer',
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT project_members_user_xor_group
        CHECK ((user_id IS NOT NULL)::int + (group_name IS NOT NULL)::int = 1),
    CONSTRAINT project_members_role_valid
        CHECK (role IN ('owner', 'admin', 'developer', 'viewer'))
);

-- One row per (project, user); one row per (project, group).
CREATE UNIQUE INDEX IF NOT EXISTS uq_project_members_user
    ON project_members(project_id, user_id)
    WHERE user_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_project_members_group
    ON project_members(project_id, group_name)
    WHERE group_name IS NOT NULL;

-- Owner lookup is hot on every mutation; back the transfer query too.
CREATE INDEX IF NOT EXISTS idx_project_members_project_role
    ON project_members(project_id, role);
CREATE INDEX IF NOT EXISTS idx_project_members_user
    ON project_members(user_id)
    WHERE user_id IS NOT NULL;
