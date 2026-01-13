CREATE TABLE canvases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    width INTEGER NOT NULL DEFAULT 1920,
    height INTEGER NOT NULL DEFAULT 1080,
    background_color VARCHAR(20) DEFAULT '#ffffff',
    thumbnail_url TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    is_template BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE canvas_elements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
    element_type VARCHAR(50) NOT NULL,
    x DOUBLE PRECISION NOT NULL DEFAULT 0,
    y DOUBLE PRECISION NOT NULL DEFAULT 0,
    width DOUBLE PRECISION NOT NULL DEFAULT 100,
    height DOUBLE PRECISION NOT NULL DEFAULT 100,
    rotation DOUBLE PRECISION NOT NULL DEFAULT 0,
    z_index INTEGER NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    properties JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE canvas_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    permission VARCHAR(50) NOT NULL DEFAULT 'view',
    added_by UUID,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(canvas_id, user_id)
);

CREATE TABLE canvas_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    name VARCHAR(255),
    elements_snapshot JSONB NOT NULL DEFAULT '[]',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(canvas_id, version_number)
);

CREATE TABLE canvas_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
    element_id UUID REFERENCES canvas_elements(id) ON DELETE CASCADE,
    parent_comment_id UUID REFERENCES canvas_comments(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    x_position DOUBLE PRECISION,
    y_position DOUBLE PRECISION,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_canvases_org_bot ON canvases(org_id, bot_id);
CREATE INDEX idx_canvases_created_by ON canvases(created_by);
CREATE INDEX idx_canvases_public ON canvases(is_public) WHERE is_public = TRUE;
CREATE INDEX idx_canvases_template ON canvases(is_template) WHERE is_template = TRUE;

CREATE INDEX idx_canvas_elements_canvas ON canvas_elements(canvas_id);
CREATE INDEX idx_canvas_elements_type ON canvas_elements(element_type);
CREATE INDEX idx_canvas_elements_z_index ON canvas_elements(canvas_id, z_index);

CREATE INDEX idx_canvas_collaborators_canvas ON canvas_collaborators(canvas_id);
CREATE INDEX idx_canvas_collaborators_user ON canvas_collaborators(user_id);

CREATE INDEX idx_canvas_versions_canvas ON canvas_versions(canvas_id);
CREATE INDEX idx_canvas_versions_number ON canvas_versions(canvas_id, version_number DESC);

CREATE INDEX idx_canvas_comments_canvas ON canvas_comments(canvas_id);
CREATE INDEX idx_canvas_comments_element ON canvas_comments(element_id) WHERE element_id IS NOT NULL;
CREATE INDEX idx_canvas_comments_parent ON canvas_comments(parent_comment_id) WHERE parent_comment_id IS NOT NULL;
CREATE INDEX idx_canvas_comments_unresolved ON canvas_comments(canvas_id, resolved) WHERE resolved = FALSE;
