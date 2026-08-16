-- Cross-app resource sharing / access control (Docs, Slides, Drive, …).
--
-- Resources are addressed generically (resource_type, resource_id) so every
-- collaboration app shares one permission model. Ownership is represented as
-- a 'user' grant with role 'owner'; on first grant for a resource with no
-- owner, the granting user becomes owner. Roles:
--   viewer     — read-only
--   commenter  — can comment/react but not edit content
--   editor     — full edit
--   owner      — everything, including transfer of ownership
--
-- grantee_type: 'user' (email/uuid) | 'group' (group id) | 'domain' (@corp.com)

CREATE TABLE IF NOT EXISTS resource_permissions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    grantee_type  VARCHAR(16)  NOT NULL,
    grantee_id    VARCHAR(255) NOT NULL,
    role          VARCHAR(16)  NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (resource_type, resource_id, grantee_type, grantee_id)
);

CREATE INDEX IF NOT EXISTS idx_resource_permissions_resource
    ON resource_permissions (resource_type, resource_id);

CREATE INDEX IF NOT EXISTS idx_resource_permissions_grantee
    ON resource_permissions (grantee_type, grantee_id);

-- Public link sharing: a bearer token that grants a role to anyone with the
-- link, optionally expiring. Mirrors the Drive public-link pattern.

CREATE TABLE IF NOT EXISTS resource_links (
    token         VARCHAR(64) PRIMARY KEY,
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    role          VARCHAR(16)  NOT NULL,
    expires_at    TIMESTAMPTZ,
    created_by    VARCHAR(255) NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_resource_links_resource
    ON resource_links (resource_type, resource_id);
