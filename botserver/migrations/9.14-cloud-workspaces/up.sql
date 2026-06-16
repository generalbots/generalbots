CREATE TABLE cloud_workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE workspace_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES cloud_workspaces(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    store_item_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'provisioning',
    config JSONB,
    provisioned_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cloud_workspaces_org ON cloud_workspaces(org_id);
CREATE INDEX idx_workspace_resources_workspace ON workspace_resources(workspace_id);
CREATE INDEX idx_workspace_resources_org ON workspace_resources(org_id);
CREATE INDEX idx_workspace_resources_type ON workspace_resources(resource_type);
