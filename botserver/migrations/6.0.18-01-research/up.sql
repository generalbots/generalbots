CREATE TABLE research_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    owner_id UUID NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE research_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    source_type VARCHAR(50) NOT NULL,
    name VARCHAR(500) NOT NULL,
    url TEXT,
    content TEXT,
    summary TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    credibility_score INTEGER,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    added_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE research_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    source_id UUID REFERENCES research_sources(id) ON DELETE SET NULL,
    title VARCHAR(500),
    content TEXT NOT NULL,
    note_type VARCHAR(50) NOT NULL DEFAULT 'general',
    tags TEXT[] NOT NULL DEFAULT '{}',
    highlight_text TEXT,
    highlight_position JSONB,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE research_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    finding_type VARCHAR(50) NOT NULL DEFAULT 'insight',
    confidence_level VARCHAR(50),
    supporting_sources JSONB NOT NULL DEFAULT '[]',
    related_findings JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE research_citations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES research_sources(id) ON DELETE CASCADE,
    citation_style VARCHAR(50) NOT NULL DEFAULT 'apa',
    formatted_citation TEXT NOT NULL,
    bibtex TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE research_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'viewer',
    invited_by UUID,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, user_id)
);

CREATE TABLE research_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    export_type VARCHAR(50) NOT NULL,
    format VARCHAR(50) NOT NULL,
    file_url TEXT,
    file_size INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_research_projects_org_bot ON research_projects(org_id, bot_id);
CREATE INDEX idx_research_projects_owner ON research_projects(owner_id);
CREATE INDEX idx_research_projects_status ON research_projects(status);
CREATE INDEX idx_research_projects_tags ON research_projects USING GIN(tags);

CREATE INDEX idx_research_sources_project ON research_sources(project_id);
CREATE INDEX idx_research_sources_type ON research_sources(source_type);
CREATE INDEX idx_research_sources_verified ON research_sources(is_verified) WHERE is_verified = TRUE;

CREATE INDEX idx_research_notes_project ON research_notes(project_id);
CREATE INDEX idx_research_notes_source ON research_notes(source_id) WHERE source_id IS NOT NULL;
CREATE INDEX idx_research_notes_type ON research_notes(note_type);
CREATE INDEX idx_research_notes_tags ON research_notes USING GIN(tags);

CREATE INDEX idx_research_findings_project ON research_findings(project_id);
CREATE INDEX idx_research_findings_type ON research_findings(finding_type);
CREATE INDEX idx_research_findings_status ON research_findings(status);

CREATE INDEX idx_research_citations_source ON research_citations(source_id);
CREATE INDEX idx_research_citations_style ON research_citations(citation_style);

CREATE INDEX idx_research_collaborators_project ON research_collaborators(project_id);
CREATE INDEX idx_research_collaborators_user ON research_collaborators(user_id);

CREATE INDEX idx_research_exports_project ON research_exports(project_id);
CREATE INDEX idx_research_exports_status ON research_exports(status);
