CREATE TABLE IF NOT EXISTS skill_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(80) NOT NULL UNIQUE,
    name VARCHAR(160) NOT NULL,
    description TEXT,
    latest_version VARCHAR(32),
    publisher_org_id UUID,
    publisher_name VARCHAR(160),
    visibility VARCHAR(16) NOT NULL DEFAULT 'public',
    review_status VARCHAR(16) NOT NULL DEFAULT 'auto',
    downloads BIGINT NOT NULL DEFAULT 0,
    icon_glyph VARCHAR(8),
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS skill_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES skill_packages(id) ON DELETE CASCADE,
    version VARCHAR(32) NOT NULL,
    manifest JSONB NOT NULL,
    object_key TEXT NOT NULL,
    changelog TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (package_id, version)
);

CREATE TABLE IF NOT EXISTS skill_installs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES skill_packages(id) ON DELETE CASCADE,
    version_id UUID NOT NULL REFERENCES skill_versions(id),
    org_id UUID,
    branch_id UUID,
    bot_id UUID NOT NULL,
    installed_by UUID,
    status VARCHAR(24) NOT NULL DEFAULT 'installed',
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_skill_installs_bot ON skill_installs(bot_id);
