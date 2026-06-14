CREATE TABLE aiworkspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon_type VARCHAR(20) DEFAULT 'emoji',
    icon_value VARCHAR(100),
    cover_image TEXT,
    settings JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE aiworkspace_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES aiworkspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    invited_by UUID,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, user_id)
);

CREATE TABLE aiworkspace_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES aiworkspaces(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES aiworkspace_pages(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    icon_type VARCHAR(20),
    icon_value VARCHAR(100),
    cover_image TEXT,
    content JSONB NOT NULL DEFAULT '[]',
    properties JSONB NOT NULL DEFAULT '{}',
    is_template BOOLEAN NOT NULL DEFAULT FALSE,
    template_id UUID REFERENCES aiworkspace_pages(id) ON DELETE SET NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    public_edit BOOLEAN NOT NULL DEFAULT FALSE,
    position INTEGER NOT NULL DEFAULT 0,
    created_by UUID NOT NULL,
    last_edited_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE aiworkspace_page_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id UUID NOT NULL REFERENCES aiworkspace_pages(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    title VARCHAR(500) NOT NULL,
    content JSONB NOT NULL DEFAULT '[]',
    change_summary TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(page_id, version_number)
);

CREATE TABLE aiworkspace_page_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id UUID NOT NULL REFERENCES aiworkspace_pages(id) ON DELETE CASCADE,
    user_id UUID,
    role VARCHAR(50),
    permission VARCHAR(50) NOT NULL DEFAULT 'view',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(page_id, user_id),
    UNIQUE(page_id, role)
);

CREATE TABLE aiworkspace_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES aiworkspaces(id) ON DELETE CASCADE,
    page_id UUID NOT NULL REFERENCES aiworkspace_pages(id) ON DELETE CASCADE,
    block_id UUID,
    parent_comment_id UUID REFERENCES aiworkspace_comments(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE aiworkspace_comment_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id UUID NOT NULL REFERENCES aiworkspace_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    emoji VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(comment_id, user_id, emoji)
);

CREATE TABLE aiworkspace_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    icon_type VARCHAR(20),
    icon_value VARCHAR(100),
    cover_image TEXT,
    content JSONB NOT NULL DEFAULT '[]',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_aiworkspaces_org_bot ON aiworkspaces(org_id, bot_id);
CREATE INDEX idx_aiworkspaces_created_by ON aiworkspaces(created_by);

CREATE INDEX idx_aiworkspace_members_workspace ON aiworkspace_members(workspace_id);
CREATE INDEX idx_aiworkspace_members_user ON aiworkspace_members(user_id);
CREATE INDEX idx_aiworkspace_members_role ON aiworkspace_members(role);

CREATE INDEX idx_aiworkspace_pages_workspace ON aiworkspace_pages(workspace_id);
CREATE INDEX idx_aiworkspace_pages_parent ON aiworkspace_pages(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_aiworkspace_pages_template ON aiworkspace_pages(is_template) WHERE is_template = TRUE;
CREATE INDEX idx_aiworkspace_pages_public ON aiworkspace_pages(is_public) WHERE is_public = TRUE;
CREATE INDEX idx_aiworkspace_pages_position ON aiworkspace_pages(workspace_id, parent_id, position);

CREATE INDEX idx_aiworkspace_page_versions_page ON aiworkspace_page_versions(page_id);
CREATE INDEX idx_aiworkspace_page_versions_number ON aiworkspace_page_versions(page_id, version_number DESC);

CREATE INDEX idx_aiworkspace_page_permissions_page ON aiworkspace_page_permissions(page_id);
CREATE INDEX idx_aiworkspace_page_permissions_user ON aiworkspace_page_permissions(user_id) WHERE user_id IS NOT NULL;

CREATE INDEX idx_aiworkspace_comments_workspace ON aiworkspace_comments(workspace_id);
CREATE INDEX idx_aiworkspace_comments_page ON aiworkspace_comments(page_id);
CREATE INDEX idx_aiworkspace_comments_block ON aiworkspace_comments(block_id) WHERE block_id IS NOT NULL;
CREATE INDEX idx_aiworkspace_comments_parent ON aiworkspace_comments(parent_comment_id) WHERE parent_comment_id IS NOT NULL;
CREATE INDEX idx_aiworkspace_comments_unresolved ON aiworkspace_comments(page_id, resolved) WHERE resolved = FALSE;

CREATE INDEX idx_aiworkspace_comment_reactions_comment ON aiworkspace_comment_reactions(comment_id);

CREATE INDEX idx_aiworkspace_templates_org_bot ON aiworkspace_templates(org_id, bot_id);
CREATE INDEX idx_aiworkspace_templates_category ON aiworkspace_templates(category);
CREATE INDEX idx_aiworkspace_templates_system ON aiworkspace_templates(is_system) WHERE is_system = TRUE;
