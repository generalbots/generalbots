CREATE TABLE dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    layout JSONB NOT NULL DEFAULT '{"columns": 12, "row_height": 80, "gap": 16}',
    refresh_interval INTEGER,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    is_template BOOLEAN NOT NULL DEFAULT FALSE,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    widget_type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    position_x INTEGER NOT NULL DEFAULT 0,
    position_y INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 4,
    height INTEGER NOT NULL DEFAULT 3,
    config JSONB NOT NULL DEFAULT '{}',
    data_query JSONB,
    style JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE dashboard_data_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    source_type VARCHAR(50) NOT NULL,
    connection JSONB NOT NULL DEFAULT '{}',
    schema_definition JSONB NOT NULL DEFAULT '{}',
    refresh_schedule VARCHAR(100),
    last_sync TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE dashboard_filters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    field VARCHAR(255) NOT NULL,
    filter_type VARCHAR(50) NOT NULL,
    default_value JSONB,
    options JSONB NOT NULL DEFAULT '[]',
    linked_widgets JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE dashboard_widget_data_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    widget_id UUID NOT NULL REFERENCES dashboard_widgets(id) ON DELETE CASCADE,
    data_source_id UUID NOT NULL REFERENCES dashboard_data_sources(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(widget_id, data_source_id)
);

CREATE TABLE conversational_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    dashboard_id UUID REFERENCES dashboards(id) ON DELETE SET NULL,
    user_id UUID NOT NULL,
    natural_language TEXT NOT NULL,
    generated_query TEXT,
    result_widget_config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_dashboards_org_bot ON dashboards(org_id, bot_id);
CREATE INDEX idx_dashboards_owner ON dashboards(owner_id);
CREATE INDEX idx_dashboards_is_public ON dashboards(is_public) WHERE is_public = TRUE;
CREATE INDEX idx_dashboards_is_template ON dashboards(is_template) WHERE is_template = TRUE;
CREATE INDEX idx_dashboards_tags ON dashboards USING GIN(tags);
CREATE INDEX idx_dashboards_created ON dashboards(created_at DESC);

CREATE INDEX idx_dashboard_widgets_dashboard ON dashboard_widgets(dashboard_id);
CREATE INDEX idx_dashboard_widgets_type ON dashboard_widgets(widget_type);

CREATE INDEX idx_dashboard_data_sources_org_bot ON dashboard_data_sources(org_id, bot_id);
CREATE INDEX idx_dashboard_data_sources_type ON dashboard_data_sources(source_type);
CREATE INDEX idx_dashboard_data_sources_status ON dashboard_data_sources(status);

CREATE INDEX idx_dashboard_filters_dashboard ON dashboard_filters(dashboard_id);

CREATE INDEX idx_conversational_queries_org_bot ON conversational_queries(org_id, bot_id);
CREATE INDEX idx_conversational_queries_dashboard ON conversational_queries(dashboard_id) WHERE dashboard_id IS NOT NULL;
CREATE INDEX idx_conversational_queries_user ON conversational_queries(user_id);
CREATE INDEX idx_conversational_queries_created ON conversational_queries(created_at DESC);
