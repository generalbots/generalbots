-- RBAC (Role-Based Access Control) Tables for Enterprise Suite
-- Designed as a free alternative to Office 365 / Google Workspace
-- Comprehensive permissions for all suite applications

-- Roles table: defines available roles in the system
CREATE TABLE IF NOT EXISTS rbac_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Groups table: organizes users into logical groups (like AD Groups / Google Groups)
CREATE TABLE IF NOT EXISTS rbac_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_group_id UUID REFERENCES rbac_groups(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Permissions table: defines granular permissions for all suite apps
CREATE TABLE IF NOT EXISTS rbac_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    category VARCHAR(100) NOT NULL DEFAULT 'general',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(resource_type, action)
);

-- Role-Permission junction table
CREATE TABLE IF NOT EXISTS rbac_role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES rbac_permissions(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, permission_id)
);

-- User-Role junction table (direct role assignment)
CREATE TABLE IF NOT EXISTS rbac_user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(user_id, role_id)
);

-- User-Group junction table
CREATE TABLE IF NOT EXISTS rbac_user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    added_by UUID REFERENCES users(id) ON DELETE SET NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);

-- Group-Role junction table (role inheritance through groups)
CREATE TABLE IF NOT EXISTS rbac_group_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(group_id, role_id)
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_rbac_roles_name ON rbac_roles(name);
CREATE INDEX IF NOT EXISTS idx_rbac_roles_is_active ON rbac_roles(is_active);
CREATE INDEX IF NOT EXISTS idx_rbac_groups_name ON rbac_groups(name);
CREATE INDEX IF NOT EXISTS idx_rbac_groups_parent ON rbac_groups(parent_group_id);
CREATE INDEX IF NOT EXISTS idx_rbac_groups_is_active ON rbac_groups(is_active);
CREATE INDEX IF NOT EXISTS idx_rbac_permissions_resource ON rbac_permissions(resource_type);
CREATE INDEX IF NOT EXISTS idx_rbac_permissions_category ON rbac_permissions(category);
CREATE INDEX IF NOT EXISTS idx_rbac_role_permissions_role ON rbac_role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_rbac_role_permissions_permission ON rbac_role_permissions(permission_id);
CREATE INDEX IF NOT EXISTS idx_rbac_user_roles_user ON rbac_user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_rbac_user_roles_role ON rbac_user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_rbac_user_roles_expires ON rbac_user_roles(expires_at);
CREATE INDEX IF NOT EXISTS idx_rbac_user_groups_user ON rbac_user_groups(user_id);
CREATE INDEX IF NOT EXISTS idx_rbac_user_groups_group ON rbac_user_groups(group_id);
CREATE INDEX IF NOT EXISTS idx_rbac_group_roles_group ON rbac_group_roles(group_id);
CREATE INDEX IF NOT EXISTS idx_rbac_group_roles_role ON rbac_group_roles(role_id);

-- ============================================================================
-- DEFAULT SYSTEM ROLES (Similar to Office 365 / Google Workspace roles)
-- ============================================================================

INSERT INTO rbac_roles (name, display_name, description, is_system) VALUES
    -- Global Admin Roles
    ('global_admin', 'Global Administrator', 'Full control over all organization settings, users, and resources. Similar to Office 365 Global Admin.', TRUE),
    ('billing_admin', 'Billing Administrator', 'Manages subscriptions, billing, and payment methods.', TRUE),
    ('compliance_admin', 'Compliance Administrator', 'Manages compliance features, audit logs, and data governance.', TRUE),
    ('security_admin', 'Security Administrator', 'Manages security settings, threat protection, and access policies.', TRUE),

    -- User Management Roles
    ('user_admin', 'User Administrator', 'Creates and manages users, resets passwords, manages groups.', TRUE),
    ('groups_admin', 'Groups Administrator', 'Creates and manages all groups in the organization.', TRUE),
    ('helpdesk_admin', 'Helpdesk Administrator', 'Resets passwords and manages support tickets for non-admin users.', TRUE),
    ('password_admin', 'Password Administrator', 'Can reset passwords for non-privileged users.', TRUE),

    -- Service-Specific Admin Roles
    ('exchange_admin', 'Mail Administrator', 'Full control over mail settings, mailboxes, and mail flow rules.', TRUE),
    ('sharepoint_admin', 'Drive Administrator', 'Manages file storage, sharing settings, and site collections.', TRUE),
    ('teams_admin', 'Meet & Chat Administrator', 'Manages video meetings, chat settings, and collaboration features.', TRUE),

    -- Content Roles
    ('knowledge_admin', 'Knowledge Administrator', 'Manages knowledge bases, document libraries, and search settings.', TRUE),
    ('reports_reader', 'Reports Reader', 'Can view usage reports, analytics, and dashboards.', TRUE),

    -- Standard User Roles
    ('power_user', 'Power User', 'Advanced user with additional permissions for automation and integrations.', TRUE),
    ('standard_user', 'Standard User', 'Default role for regular employees with access to productivity apps.', TRUE),
    ('guest_user', 'Guest User', 'Limited external access for collaboration with partners.', TRUE),
    ('viewer', 'Viewer', 'Read-only access across applications.', TRUE)
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- COMPREHENSIVE PERMISSIONS BY APPLICATION
-- ============================================================================

-- === ADMIN & SYSTEM PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    -- Organization Management
    ('org.read', 'View Organization', 'View organization settings and information', 'organization', 'read', 'admin', TRUE),
    ('org.write', 'Manage Organization', 'Modify organization settings', 'organization', 'write', 'admin', TRUE),
    ('org.delete', 'Delete Organization', 'Delete organization data', 'organization', 'delete', 'admin', TRUE),
    ('org.billing', 'Manage Billing', 'Access billing and subscription management', 'organization', 'billing', 'admin', TRUE),

    -- User Management (like Azure AD / Google Admin)
    ('users.read', 'View Users', 'View user profiles and directory', 'users', 'read', 'admin', TRUE),
    ('users.create', 'Create Users', 'Create new user accounts', 'users', 'create', 'admin', TRUE),
    ('users.write', 'Edit Users', 'Modify user profiles and settings', 'users', 'write', 'admin', TRUE),
    ('users.delete', 'Delete Users', 'Delete user accounts', 'users', 'delete', 'admin', TRUE),
    ('users.password_reset', 'Reset Passwords', 'Reset user passwords', 'users', 'password_reset', 'admin', TRUE),
    ('users.mfa_manage', 'Manage MFA', 'Enable/disable multi-factor authentication', 'users', 'mfa_manage', 'admin', TRUE),
    ('users.impersonate', 'Impersonate Users', 'Sign in as another user for troubleshooting', 'users', 'impersonate', 'admin', TRUE),
    ('users.export', 'Export Users', 'Export user data and directory', 'users', 'export', 'admin', TRUE),
    ('users.import', 'Import Users', 'Bulk import users from CSV/LDAP', 'users', 'import', 'admin', TRUE),

    -- Group Management
    ('groups.read', 'View Groups', 'View groups and memberships', 'groups', 'read', 'admin', TRUE),
    ('groups.create', 'Create Groups', 'Create new groups', 'groups', 'create', 'admin', TRUE),
    ('groups.write', 'Edit Groups', 'Modify group settings and membership', 'groups', 'write', 'admin', TRUE),
    ('groups.delete', 'Delete Groups', 'Delete groups', 'groups', 'delete', 'admin', TRUE),
    ('groups.manage_members', 'Manage Members', 'Add/remove group members', 'groups', 'manage_members', 'admin', TRUE),
    ('groups.manage_owners', 'Manage Owners', 'Assign group owners', 'groups', 'manage_owners', 'admin', TRUE),

    -- Role & Permission Management
    ('roles.read', 'View Roles', 'View role definitions', 'roles', 'read', 'admin', TRUE),
    ('roles.create', 'Create Roles', 'Create custom roles', 'roles', 'create', 'admin', TRUE),
    ('roles.write', 'Edit Roles', 'Modify role permissions', 'roles', 'write', 'admin', TRUE),
    ('roles.delete', 'Delete Roles', 'Delete custom roles', 'roles', 'delete', 'admin', TRUE),
    ('roles.assign', 'Assign Roles', 'Assign roles to users and groups', 'roles', 'assign', 'admin', TRUE),

    -- DNS & Domain Management
    ('dns.read', 'View DNS', 'View DNS records and domain settings', 'dns', 'read', 'admin', TRUE),
    ('dns.write', 'Manage DNS', 'Add/modify DNS records', 'dns', 'write', 'admin', TRUE),
    ('domains.verify', 'Verify Domains', 'Verify domain ownership', 'domains', 'verify', 'admin', TRUE),

    -- Audit & Compliance
    ('audit.read', 'View Audit Logs', 'Access audit and activity logs', 'audit', 'read', 'compliance', TRUE),
    ('audit.export', 'Export Audit Logs', 'Export audit data for compliance', 'audit', 'export', 'compliance', TRUE),
    ('compliance.read', 'View Compliance', 'View compliance dashboard and reports', 'compliance', 'read', 'compliance', TRUE),
    ('compliance.write', 'Manage Compliance', 'Configure compliance policies', 'compliance', 'write', 'compliance', TRUE),
    ('dlp.read', 'View DLP Policies', 'View data loss prevention rules', 'dlp', 'read', 'compliance', TRUE),
    ('dlp.write', 'Manage DLP', 'Create and modify DLP policies', 'dlp', 'write', 'compliance', TRUE),
    ('retention.read', 'View Retention', 'View data retention policies', 'retention', 'read', 'compliance', TRUE),
    ('retention.write', 'Manage Retention', 'Configure retention policies', 'retention', 'write', 'compliance', TRUE),
    ('ediscovery.access', 'eDiscovery Access', 'Access eDiscovery tools and holds', 'ediscovery', 'access', 'compliance', TRUE),

    -- Security
    ('security.read', 'View Security', 'View security dashboard and alerts', 'security', 'read', 'security', TRUE),
    ('security.write', 'Manage Security', 'Configure security settings', 'security', 'write', 'security', TRUE),
    ('threats.read', 'View Threats', 'View threat detection and incidents', 'threats', 'read', 'security', TRUE),
    ('threats.respond', 'Respond to Threats', 'Take action on security incidents', 'threats', 'respond', 'security', TRUE),
    ('secrets.read', 'View Secrets', 'View API keys and secrets', 'secrets', 'read', 'security', TRUE),
    ('secrets.write', 'Manage Secrets', 'Create and rotate secrets', 'secrets', 'write', 'security', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === MAIL PERMISSIONS (Like Outlook / Gmail) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('mail.read', 'Read Mail', 'Read own mailbox and messages', 'mail', 'read', 'mail', TRUE),
    ('mail.send', 'Send Mail', 'Send emails', 'mail', 'send', 'mail', TRUE),
    ('mail.delete', 'Delete Mail', 'Delete emails', 'mail', 'delete', 'mail', TRUE),
    ('mail.organize', 'Organize Mail', 'Create folders, apply labels, set rules', 'mail', 'organize', 'mail', TRUE),
    ('mail.delegate', 'Mail Delegation', 'Grant mailbox access to others', 'mail', 'delegate', 'mail', TRUE),
    ('mail.shared_read', 'Read Shared Mailbox', 'Access shared mailboxes', 'mail', 'shared_read', 'mail', TRUE),
    ('mail.shared_send', 'Send from Shared', 'Send as shared mailbox', 'mail', 'shared_send', 'mail', TRUE),
    ('mail.admin', 'Mail Admin', 'Administer mail settings globally', 'mail', 'admin', 'mail', TRUE),
    ('mail.rules_global', 'Global Mail Rules', 'Create organization-wide mail rules', 'mail', 'rules_global', 'mail', TRUE),
    ('mail.signatures_global', 'Global Signatures', 'Manage organization email signatures', 'mail', 'signatures_global', 'mail', TRUE),
    ('mail.distribution_lists', 'Distribution Lists', 'Manage distribution lists', 'mail', 'distribution_lists', 'mail', TRUE),
    ('mail.encryption', 'Mail Encryption', 'Send encrypted messages', 'mail', 'encryption', 'mail', TRUE),
    ('mail.archive', 'Mail Archive', 'Access mail archive', 'mail', 'archive', 'mail', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === CALENDAR PERMISSIONS (Like Outlook Calendar / Google Calendar) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('calendar.read', 'View Calendar', 'View own calendar and events', 'calendar', 'read', 'calendar', TRUE),
    ('calendar.write', 'Manage Calendar', 'Create, edit, delete events', 'calendar', 'write', 'calendar', TRUE),
    ('calendar.share', 'Share Calendar', 'Share calendar with others', 'calendar', 'share', 'calendar', TRUE),
    ('calendar.delegate', 'Calendar Delegation', 'Allow others to manage calendar', 'calendar', 'delegate', 'calendar', TRUE),
    ('calendar.free_busy', 'View Free/Busy', 'View availability of others', 'calendar', 'free_busy', 'calendar', TRUE),
    ('calendar.rooms', 'Book Rooms', 'Reserve meeting rooms and resources', 'calendar', 'rooms', 'calendar', TRUE),
    ('calendar.rooms_admin', 'Manage Rooms', 'Administer room resources', 'calendar', 'rooms_admin', 'calendar', TRUE),
    ('calendar.shared_read', 'Read Shared Calendars', 'View shared team calendars', 'calendar', 'shared_read', 'calendar', TRUE),
    ('calendar.shared_write', 'Edit Shared Calendars', 'Modify shared team calendars', 'calendar', 'shared_write', 'calendar', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === DRIVE PERMISSIONS (Like OneDrive / SharePoint / Google Drive) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('drive.read', 'View Files', 'View own files and folders', 'drive', 'read', 'drive', TRUE),
    ('drive.write', 'Upload Files', 'Upload and create files', 'drive', 'write', 'drive', TRUE),
    ('drive.delete', 'Delete Files', 'Delete own files', 'drive', 'delete', 'drive', TRUE),
    ('drive.share', 'Share Files', 'Share files with others', 'drive', 'share', 'drive', TRUE),
    ('drive.share_external', 'External Sharing', 'Share files externally', 'drive', 'share_external', 'drive', TRUE),
    ('drive.download', 'Download Files', 'Download files locally', 'drive', 'download', 'drive', TRUE),
    ('drive.sync', 'Sync Files', 'Use desktop sync client', 'drive', 'sync', 'drive', TRUE),
    ('drive.version_history', 'Version History', 'View and restore file versions', 'drive', 'version_history', 'drive', TRUE),
    ('drive.shared_read', 'Read Shared Drives', 'Access team shared drives', 'drive', 'shared_read', 'drive', TRUE),
    ('drive.shared_write', 'Write Shared Drives', 'Modify files in shared drives', 'drive', 'shared_write', 'drive', TRUE),
    ('drive.shared_admin', 'Manage Shared Drives', 'Administer shared drive settings', 'drive', 'shared_admin', 'drive', TRUE),
    ('drive.trash', 'Manage Trash', 'View and restore deleted items', 'drive', 'trash', 'drive', TRUE),
    ('drive.quota', 'View Storage Quota', 'View storage usage', 'drive', 'quota', 'drive', TRUE),
    ('drive.admin', 'Drive Admin', 'Full administrative access to all drives', 'drive', 'admin', 'drive', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === DOCS PERMISSIONS (Like Word Online / Google Docs) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('docs.read', 'View Documents', 'View documents', 'docs', 'read', 'docs', TRUE),
    ('docs.write', 'Edit Documents', 'Create and edit documents', 'docs', 'write', 'docs', TRUE),
    ('docs.comment', 'Comment on Documents', 'Add comments and suggestions', 'docs', 'comment', 'docs', TRUE),
    ('docs.share', 'Share Documents', 'Share documents with others', 'docs', 'share', 'docs', TRUE),
    ('docs.export', 'Export Documents', 'Export to PDF, Word, etc.', 'docs', 'export', 'docs', TRUE),
    ('docs.templates', 'Use Templates', 'Access document templates', 'docs', 'templates', 'docs', TRUE),
    ('docs.templates_manage', 'Manage Templates', 'Create organization templates', 'docs', 'templates_manage', 'docs', TRUE),
    ('docs.collaborate', 'Real-time Collaboration', 'Co-author documents in real-time', 'docs', 'collaborate', 'docs', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === SHEET PERMISSIONS (Like Excel Online / Google Sheets) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('sheet.read', 'View Spreadsheets', 'View spreadsheets', 'sheet', 'read', 'sheet', TRUE),
    ('sheet.write', 'Edit Spreadsheets', 'Create and edit spreadsheets', 'sheet', 'write', 'sheet', TRUE),
    ('sheet.share', 'Share Spreadsheets', 'Share spreadsheets with others', 'sheet', 'share', 'sheet', TRUE),
    ('sheet.export', 'Export Spreadsheets', 'Export to Excel, CSV, etc.', 'sheet', 'export', 'sheet', TRUE),
    ('sheet.import', 'Import Data', 'Import data from external sources', 'sheet', 'import', 'sheet', TRUE),
    ('sheet.macros', 'Run Macros', 'Execute spreadsheet macros', 'sheet', 'macros', 'sheet', TRUE),
    ('sheet.connections', 'Data Connections', 'Create database connections', 'sheet', 'connections', 'sheet', TRUE),
    ('sheet.pivot', 'Pivot Tables', 'Create pivot tables and charts', 'sheet', 'pivot', 'sheet', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === SLIDES PERMISSIONS (Like PowerPoint Online / Google Slides) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('slides.read', 'View Presentations', 'View presentations', 'slides', 'read', 'slides', TRUE),
    ('slides.write', 'Edit Presentations', 'Create and edit presentations', 'slides', 'write', 'slides', TRUE),
    ('slides.share', 'Share Presentations', 'Share presentations with others', 'slides', 'share', 'slides', TRUE),
    ('slides.present', 'Present Live', 'Start live presentations', 'slides', 'present', 'slides', TRUE),
    ('slides.export', 'Export Presentations', 'Export to PDF, PowerPoint', 'slides', 'export', 'slides', TRUE),
    ('slides.templates', 'Slide Templates', 'Access presentation templates', 'slides', 'templates', 'slides', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === MEET PERMISSIONS (Like Teams / Zoom / Google Meet) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('meet.join', 'Join Meetings', 'Join video meetings', 'meet', 'join', 'meet', TRUE),
    ('meet.create', 'Create Meetings', 'Schedule and create meetings', 'meet', 'create', 'meet', TRUE),
    ('meet.host', 'Host Meetings', 'Full host controls in meetings', 'meet', 'host', 'meet', TRUE),
    ('meet.record', 'Record Meetings', 'Record meeting sessions', 'meet', 'record', 'meet', TRUE),
    ('meet.transcript', 'Meeting Transcripts', 'Access meeting transcriptions', 'meet', 'transcript', 'meet', TRUE),
    ('meet.screen_share', 'Screen Share', 'Share screen in meetings', 'meet', 'screen_share', 'meet', TRUE),
    ('meet.breakout', 'Breakout Rooms', 'Create and manage breakout rooms', 'meet', 'breakout', 'meet', TRUE),
    ('meet.webinar', 'Host Webinars', 'Host large webinar events', 'meet', 'webinar', 'meet', TRUE),
    ('meet.admin', 'Meet Admin', 'Administer meeting settings globally', 'meet', 'admin', 'meet', TRUE),
    ('meet.external', 'External Meetings', 'Meet with external participants', 'meet', 'external', 'meet', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === CHAT PERMISSIONS (Like Teams Chat / Slack / Google Chat) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('chat.read', 'Read Messages', 'Read chat messages', 'chat', 'read', 'chat', TRUE),
    ('chat.write', 'Send Messages', 'Send chat messages', 'chat', 'write', 'chat', TRUE),
    ('chat.delete', 'Delete Messages', 'Delete own messages', 'chat', 'delete', 'chat', TRUE),
    ('chat.edit', 'Edit Messages', 'Edit sent messages', 'chat', 'edit', 'chat', TRUE),
    ('chat.files', 'Share Files in Chat', 'Share files in conversations', 'chat', 'files', 'chat', TRUE),
    ('chat.channels_create', 'Create Channels', 'Create chat channels', 'chat', 'channels_create', 'chat', TRUE),
    ('chat.channels_manage', 'Manage Channels', 'Manage channel settings', 'chat', 'channels_manage', 'chat', TRUE),
    ('chat.external', 'External Chat', 'Chat with external users', 'chat', 'external', 'chat', TRUE),
    ('chat.reactions', 'Reactions', 'Add reactions to messages', 'chat', 'reactions', 'chat', TRUE),
    ('chat.threads', 'Thread Replies', 'Reply in threads', 'chat', 'threads', 'chat', TRUE),
    ('chat.mentions', 'Mentions', 'Mention users and groups', 'chat', 'mentions', 'chat', TRUE),
    ('chat.admin', 'Chat Admin', 'Administer chat settings globally', 'chat', 'admin', 'chat', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === TASKS PERMISSIONS (Like Planner / Asana / Google Tasks) ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('tasks.read', 'View Tasks', 'View own and assigned tasks', 'tasks', 'read', 'tasks', TRUE),
    ('tasks.write', 'Manage Tasks', 'Create and edit tasks', 'tasks', 'write', 'tasks', TRUE),
    ('tasks.delete', 'Delete Tasks', 'Delete tasks', 'tasks', 'delete', 'tasks', TRUE),
    ('tasks.assign', 'Assign Tasks', 'Assign tasks to others', 'tasks', 'assign', 'tasks', TRUE),
    ('tasks.projects_create', 'Create Projects', 'Create task projects/boards', 'tasks', 'projects_create', 'tasks', TRUE),
    ('tasks.projects_manage', 'Manage Projects', 'Administer project settings', 'tasks', 'projects_manage', 'tasks', TRUE),
    ('tasks.time_track', 'Time Tracking', 'Log time against tasks', 'tasks', 'time_track', 'tasks', TRUE),
    ('tasks.reports', 'Task Reports', 'View task analytics and reports', 'tasks', 'reports', 'tasks', TRUE),
    ('tasks.automation', 'Task Automation', 'Create task automation rules', 'tasks', 'automation', 'tasks', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === BOT & AI PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('bots.read', 'View Bots', 'View bot configurations', 'bots', 'read', 'ai', TRUE),
    ('bots.create', 'Create Bots', 'Create new bots', 'bots', 'create', 'ai', TRUE),
    ('bots.write', 'Edit Bots', 'Modify bot settings', 'bots', 'write', 'ai', TRUE),
    ('bots.delete', 'Delete Bots', 'Delete bots', 'bots', 'delete', 'ai', TRUE),
    ('bots.publish', 'Publish Bots', 'Publish bots to channels', 'bots', 'publish', 'ai', TRUE),
    ('bots.channels', 'Manage Channels', 'Configure bot communication channels', 'bots', 'channels', 'ai', TRUE),

    -- AI Assistant / Copilot
    ('ai.chat', 'AI Chat', 'Use AI chat assistant', 'ai', 'chat', 'ai', TRUE),
    ('ai.summarize', 'AI Summarize', 'Use AI to summarize content', 'ai', 'summarize', 'ai', TRUE),
    ('ai.compose', 'AI Compose', 'Use AI to draft content', 'ai', 'compose', 'ai', TRUE),
    ('ai.translate', 'AI Translate', 'Use AI translation', 'ai', 'translate', 'ai', TRUE),
    ('ai.analyze', 'AI Analyze', 'Use AI for data analysis', 'ai', 'analyze', 'ai', TRUE),
    ('ai.advanced', 'Advanced AI', 'Access advanced AI features', 'ai', 'advanced', 'ai', TRUE),

    -- Knowledge Base
    ('kb.read', 'View Knowledge Base', 'Access knowledge base documents', 'kb', 'read', 'ai', TRUE),
    ('kb.write', 'Edit Knowledge Base', 'Add/edit knowledge base content', 'kb', 'write', 'ai', TRUE),
    ('kb.admin', 'KB Admin', 'Administer knowledge base settings', 'kb', 'admin', 'ai', TRUE),

    -- Conversations / Attendant
    ('conversations.read', 'View Conversations', 'View bot conversations', 'conversations', 'read', 'ai', TRUE),
    ('conversations.write', 'Manage Conversations', 'Intervene in conversations', 'conversations', 'write', 'ai', TRUE),
    ('conversations.transfer', 'Transfer Conversations', 'Transfer to human agent', 'conversations', 'transfer', 'ai', TRUE),
    ('conversations.history', 'Conversation History', 'Access conversation history', 'conversations', 'history', 'ai', TRUE),
    ('attendant.access', 'Attendant Access', 'Access human attendant queue', 'attendant', 'access', 'ai', TRUE),
    ('attendant.respond', 'Attendant Respond', 'Respond to queued conversations', 'attendant', 'respond', 'ai', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === ANALYTICS & REPORTING PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('analytics.read', 'View Analytics', 'View usage analytics and dashboards', 'analytics', 'read', 'analytics', TRUE),
    ('analytics.export', 'Export Analytics', 'Export analytics data', 'analytics', 'export', 'analytics', TRUE),
    ('analytics.custom', 'Custom Reports', 'Create custom reports and dashboards', 'analytics', 'custom', 'analytics', TRUE),
    ('analytics.realtime', 'Real-time Analytics', 'Access real-time analytics', 'analytics', 'realtime', 'analytics', TRUE),
    ('reports.read', 'View Reports', 'Access standard reports', 'reports', 'read', 'analytics', TRUE),
    ('reports.schedule', 'Schedule Reports', 'Schedule automated report delivery', 'reports', 'schedule', 'analytics', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === MONITORING & SYSTEM PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('monitoring.read', 'View Monitoring', 'View system health and metrics', 'monitoring', 'read', 'system', TRUE),
    ('monitoring.alerts', 'Manage Alerts', 'Configure monitoring alerts', 'monitoring', 'alerts', 'system', TRUE),
    ('logs.read', 'View Logs', 'Access system and application logs', 'logs', 'read', 'system', TRUE),
    ('logs.export', 'Export Logs', 'Export log data', 'logs', 'export', 'system', TRUE),
    ('services.read', 'View Services', 'View service status', 'services', 'read', 'system', TRUE),
    ('services.manage', 'Manage Services', 'Start/stop/restart services', 'services', 'manage', 'system', TRUE),
    ('resources.read', 'View Resources', 'View resource usage', 'resources', 'read', 'system', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === PAPER & RESEARCH (AI Writing) PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('paper.read', 'View Papers', 'View AI-generated papers and notes', 'paper', 'read', 'paper', TRUE),
    ('paper.write', 'Create Papers', 'Create and edit AI-assisted documents', 'paper', 'write', 'paper', TRUE),
    ('paper.publish', 'Publish Papers', 'Publish papers to knowledge base', 'paper', 'publish', 'paper', TRUE),
    ('research.read', 'View Research', 'Access AI research results', 'research', 'read', 'research', TRUE),
    ('research.create', 'Create Research', 'Start AI research queries', 'research', 'create', 'research', TRUE),
    ('research.deep', 'Deep Research', 'Access deep research features', 'research', 'deep', 'research', TRUE),
    ('quicknote.access', 'Quick Notes', 'Access quick note feature', 'quicknote', 'access', 'paper', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === SOURCES & INTEGRATIONS PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('sources.read', 'View Sources', 'View configured data sources', 'sources', 'read', 'integrations', TRUE),
    ('sources.create', 'Create Sources', 'Add new data sources', 'sources', 'create', 'integrations', TRUE),
    ('sources.write', 'Edit Sources', 'Modify data source configurations', 'sources', 'write', 'integrations', TRUE),
    ('sources.delete', 'Delete Sources', 'Remove data sources', 'sources', 'delete', 'integrations', TRUE),
    ('webhooks.read', 'View Webhooks', 'View webhook configurations', 'webhooks', 'read', 'integrations', TRUE),
    ('webhooks.write', 'Manage Webhooks', 'Create and edit webhooks', 'webhooks', 'write', 'integrations', TRUE),
    ('api.access', 'API Access', 'Access REST API endpoints', 'api', 'access', 'integrations', TRUE),
    ('api.keys', 'API Key Management', 'Create and manage API keys', 'api', 'keys', 'integrations', TRUE),
    ('integrations.read', 'View Integrations', 'View third-party integrations', 'integrations', 'read', 'integrations', TRUE),
    ('integrations.write', 'Manage Integrations', 'Configure third-party integrations', 'integrations', 'write', 'integrations', TRUE),
    ('mcp.access', 'MCP Access', 'Access Model Context Protocol tools', 'mcp', 'access', 'integrations', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === AUTOTASK / AUTOMATION PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('autotask.read', 'View AutoTasks', 'View automated task definitions', 'autotask', 'read', 'automation', TRUE),
    ('autotask.create', 'Create AutoTasks', 'Create new automated tasks', 'autotask', 'create', 'automation', TRUE),
    ('autotask.write', 'Edit AutoTasks', 'Modify automated task settings', 'autotask', 'write', 'automation', TRUE),
    ('autotask.delete', 'Delete AutoTasks', 'Remove automated tasks', 'autotask', 'delete', 'automation', TRUE),
    ('autotask.execute', 'Execute AutoTasks', 'Run automated tasks manually', 'autotask', 'execute', 'automation', TRUE),
    ('autotask.schedule', 'Schedule AutoTasks', 'Schedule task automation', 'autotask', 'schedule', 'automation', TRUE),
    ('workflows.read', 'View Workflows', 'View workflow definitions', 'workflows', 'read', 'automation', TRUE),
    ('workflows.write', 'Manage Workflows', 'Create and edit workflows', 'workflows', 'write', 'automation', TRUE),
    ('intents.read', 'View Intents', 'View AI intent definitions', 'intents', 'read', 'automation', TRUE),
    ('intents.write', 'Manage Intents', 'Create and edit intents', 'intents', 'write', 'automation', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === DESIGNER PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('designer.access', 'Access Designer', 'Open visual designer tool', 'designer', 'access', 'designer', TRUE),
    ('designer.create', 'Create Designs', 'Create new UI designs', 'designer', 'create', 'designer', TRUE),
    ('designer.edit', 'Edit Designs', 'Modify existing designs', 'designer', 'edit', 'designer', TRUE),
    ('designer.publish', 'Publish Designs', 'Publish designs to production', 'designer', 'publish', 'designer', TRUE),
    ('designer.templates', 'Design Templates', 'Access and create design templates', 'designer', 'templates', 'designer', TRUE)
ON CONFLICT (name) DO NOTHING;

-- === SETTINGS PERMISSIONS ===
INSERT INTO rbac_permissions (name, display_name, description, resource_type, action, category, is_system) VALUES
    ('settings.personal', 'Personal Settings', 'Manage own user settings', 'settings', 'personal', 'settings', TRUE),
    ('settings.organization', 'Organization Settings', 'Manage organization settings', 'settings', 'organization', 'settings', TRUE),
    ('settings.security', 'Security Settings', 'Manage security configuration', 'settings', 'security', 'settings', TRUE),
    ('settings.notifications', 'Notification Settings', 'Manage notification preferences', 'settings', 'notifications', 'settings', TRUE),
    ('settings.appearance', 'Appearance Settings', 'Customize appearance and themes', 'settings', 'appearance', 'settings', TRUE),
    ('settings.language', 'Language Settings', 'Set language and locale', 'settings', 'language', 'settings', TRUE),
    ('settings.backup', 'Backup Settings', 'Configure backup and export', 'settings', 'backup', 'settings', TRUE)
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- DEFAULT GROUPS (Like AD / Google Groups)
-- ============================================================================

INSERT INTO rbac_groups (name, display_name, description) VALUES
    ('all_users', 'All Users', 'Default group containing all organization users'),
    ('executives', 'Executives', 'Executive leadership team'),
    ('managers', 'Managers', 'Department managers and team leads'),
    ('it_department', 'IT Department', 'Information technology staff'),
    ('hr_department', 'HR Department', 'Human resources staff'),
    ('finance_department', 'Finance Department', 'Finance and accounting staff'),
    ('sales_team', 'Sales Team', 'Sales representatives and account managers'),
    ('support_team', 'Support Team', 'Customer support and helpdesk staff'),
    ('marketing_team', 'Marketing Team', 'Marketing and communications staff'),
    ('developers', 'Developers', 'Software development team'),
    ('external_contractors', 'External Contractors', 'External consultants and contractors'),
    ('guests', 'Guests', 'External guest users')
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- ROLE-PERMISSION ASSIGNMENTS
-- ============================================================================

-- Global Admin gets ALL permissions
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'global_admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Billing Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'billing_admin' AND p.name IN (
    'org.read', 'org.billing', 'users.read', 'reports.read', 'analytics.read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Compliance Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'compliance_admin' AND p.name IN (
    'org.read', 'users.read', 'groups.read', 'audit.read', 'audit.export',
    'compliance.read', 'compliance.write', 'dlp.read', 'dlp.write',
    'retention.read', 'retention.write', 'ediscovery.access',
    'analytics.read', 'reports.read', 'logs.read', 'logs.export'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Security Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'security_admin' AND p.name IN (
    'org.read', 'users.read', 'users.mfa_manage', 'groups.read',
    'security.read', 'security.write', 'threats.read', 'threats.respond',
    'secrets.read', 'secrets.write', 'audit.read', 'logs.read',
    'monitoring.read', 'monitoring.alerts'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- User Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'user_admin' AND p.name IN (
    'users.read', 'users.create', 'users.write', 'users.delete',
    'users.password_reset', 'users.mfa_manage', 'users.export', 'users.import',
    'groups.read', 'groups.create', 'groups.write', 'groups.manage_members',
    'roles.read', 'roles.assign', 'audit.read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Groups Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'groups_admin' AND p.name IN (
    'users.read', 'groups.read', 'groups.create', 'groups.write', 'groups.delete',
    'groups.manage_members', 'groups.manage_owners'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Helpdesk Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'helpdesk_admin' AND p.name IN (
    'users.read', 'users.password_reset', 'groups.read',
    'attendant.access', 'attendant.respond', 'conversations.read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Password Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'password_admin' AND p.name IN (
    'users.read', 'users.password_reset'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Mail Admin (Exchange Admin)
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'exchange_admin' AND p.name LIKE 'mail.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Drive Admin (SharePoint Admin)
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'sharepoint_admin' AND p.name LIKE 'drive.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Meet & Chat Admin (Teams Admin)
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'teams_admin' AND (p.name LIKE 'meet.%' OR p.name LIKE 'chat.%')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Knowledge Admin
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'knowledge_admin' AND p.name IN (
    'kb.read', 'kb.write', 'kb.admin', 'docs.read', 'docs.write',
    'docs.templates_manage', 'paper.read', 'paper.write', 'paper.publish',
    'research.read', 'research.create', 'research.deep'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Reports Reader
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'reports_reader' AND p.name IN (
    'analytics.read', 'reports.read', 'monitoring.read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Power User
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'power_user' AND p.name IN (
    -- All standard user permissions plus extras
    'mail.read', 'mail.send', 'mail.delete', 'mail.organize', 'mail.delegate',
    'calendar.read', 'calendar.write', 'calendar.share', 'calendar.delegate', 'calendar.free_busy', 'calendar.rooms',
    'drive.read', 'drive.write', 'drive.delete', 'drive.share', 'drive.share_external', 'drive.download', 'drive.sync', 'drive.version_history', 'drive.shared_read', 'drive.shared_write',
    'docs.read', 'docs.write', 'docs.comment', 'docs.share', 'docs.export', 'docs.templates', 'docs.collaborate',
    'sheet.read', 'sheet.write', 'sheet.share', 'sheet.export', 'sheet.import', 'sheet.macros', 'sheet.connections', 'sheet.pivot',
    'slides.read', 'slides.write', 'slides.share', 'slides.present', 'slides.export', 'slides.templates',
    'meet.join', 'meet.create', 'meet.host', 'meet.record', 'meet.transcript', 'meet.screen_share', 'meet.breakout', 'meet.external',
    'chat.read', 'chat.write', 'chat.delete', 'chat.edit', 'chat.files', 'chat.channels_create', 'chat.reactions', 'chat.threads', 'chat.mentions',
    'tasks.read', 'tasks.write', 'tasks.delete', 'tasks.assign', 'tasks.projects_create', 'tasks.time_track', 'tasks.automation',
    'ai.chat', 'ai.summarize', 'ai.compose', 'ai.translate', 'ai.analyze', 'ai.advanced',
    'kb.read', 'kb.write',
    'paper.read', 'paper.write', 'quicknote.access',
    'research.read', 'research.create', 'research.deep',
    'autotask.read', 'autotask.create', 'autotask.execute',
    'sources.read', 'sources.create',
    'webhooks.read', 'webhooks.write',
    'api.access', 'api.keys',
    'designer.access', 'designer.create', 'designer.edit',
    'settings.personal', 'settings.notifications', 'settings.appearance', 'settings.language',
    'analytics.read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Standard User (Default for employees)
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'standard_user' AND p.name IN (
    -- Mail
    'mail.read', 'mail.send', 'mail.delete', 'mail.organize',
    -- Calendar
    'calendar.read', 'calendar.write', 'calendar.share', 'calendar.free_busy', 'calendar.rooms', 'calendar.shared_read',
    -- Drive
    'drive.read', 'drive.write', 'drive.delete', 'drive.share', 'drive.download', 'drive.sync', 'drive.version_history', 'drive.shared_read', 'drive.shared_write',
    -- Docs
    'docs.read', 'docs.write', 'docs.comment', 'docs.share', 'docs.export', 'docs.templates', 'docs.collaborate',
    -- Sheet
    'sheet.read', 'sheet.write', 'sheet.share', 'sheet.export', 'sheet.import', 'sheet.pivot',
    -- Slides
    'slides.read', 'slides.write', 'slides.share', 'slides.present', 'slides.export', 'slides.templates',
    -- Meet
    'meet.join', 'meet.create', 'meet.host', 'meet.screen_share', 'meet.external',
    -- Chat
    'chat.read', 'chat.write', 'chat.delete', 'chat.edit', 'chat.files', 'chat.reactions', 'chat.threads', 'chat.mentions',
    -- Tasks
    'tasks.read', 'tasks.write', 'tasks.delete', 'tasks.assign', 'tasks.time_track',
    -- AI
    'ai.chat', 'ai.summarize', 'ai.compose', 'ai.translate',
    -- KB
    'kb.read',
    -- Paper & Research
    'paper.read', 'paper.write', 'quicknote.access',
    'research.read', 'research.create',
    -- Settings
    'settings.personal', 'settings.notifications', 'settings.appearance', 'settings.language',
    -- API
    'api.access'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Guest User (External collaboration)
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'guest_user' AND p.name IN (
    -- Limited mail
    'mail.read', 'mail.send',
    -- Limited calendar
    'calendar.read', 'calendar.free_busy',
    -- Limited drive (shared only)
    'drive.read', 'drive.download', 'drive.shared_read',
    -- Limited docs
    'docs.read', 'docs.comment',
    -- Limited meet
    'meet.join', 'meet.screen_share',
    -- Limited chat
    'chat.read', 'chat.write', 'chat.reactions',
    -- Limited tasks
    'tasks.read',
    -- Settings
    'settings.personal', 'settings.language'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Viewer (Read-only access)
INSERT INTO rbac_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'viewer' AND p.name IN (
    'mail.read', 'calendar.read', 'calendar.free_busy',
    'drive.read', 'drive.shared_read', 'docs.read', 'sheet.read', 'slides.read',
    'meet.join', 'chat.read', 'tasks.read', 'kb.read',
    'analytics.read', 'settings.personal'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ============================================================================
-- GROUP-ROLE ASSIGNMENTS (Common patterns)
-- ============================================================================

-- IT Department gets helpdesk + security reader
INSERT INTO rbac_group_roles (group_id, role_id)
SELECT g.id, r.id FROM rbac_groups g, rbac_roles r
WHERE g.name = 'it_department' AND r.name IN ('helpdesk_admin', 'reports_reader')
ON CONFLICT (group_id, role_id) DO NOTHING;

-- Developers get power user role
INSERT INTO rbac_group_roles (group_id, role_id)
SELECT g.id, r.id FROM rbac_groups g, rbac_roles r
WHERE g.name = 'developers' AND r.name = 'power_user'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- All Users get standard user role
INSERT INTO rbac_group_roles (group_id, role_id)
SELECT g.id, r.id FROM rbac_groups g, rbac_roles r
WHERE g.name = 'all_users' AND r.name = 'standard_user'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- External Contractors get guest role
INSERT INTO rbac_group_roles (group_id, role_id)
SELECT g.id, r.id FROM rbac_groups g, rbac_roles r
WHERE g.name = 'external_contractors' AND r.name = 'guest_user'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- Guests get guest role
INSERT INTO rbac_group_roles (group_id, role_id)
SELECT g.id, r.id FROM rbac_groups g, rbac_roles r
WHERE g.name = 'guests' AND r.name = 'guest_user'
ON CONFLICT (group_id, role_id) DO NOTHING;

-- Managers get reports reader
INSERT INTO rbac_group_roles (group_id, role_id)
SELECT g.id, r.id FROM rbac_groups g, rbac_roles r
WHERE g.name = 'managers' AND r.name = 'reports_reader'
ON CONFLICT (group_id, role_id) DO NOTHING;
