CREATE TABLE legal_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    slug VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    document_type VARCHAR(50) NOT NULL,
    version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    effective_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    requires_acceptance BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, bot_id, slug, version)
);

CREATE TABLE legal_document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES legal_documents(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    change_summary TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE cookie_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID,
    session_id VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    country_code VARCHAR(2),
    consent_necessary BOOLEAN NOT NULL DEFAULT TRUE,
    consent_analytics BOOLEAN NOT NULL DEFAULT FALSE,
    consent_marketing BOOLEAN NOT NULL DEFAULT FALSE,
    consent_preferences BOOLEAN NOT NULL DEFAULT FALSE,
    consent_functional BOOLEAN NOT NULL DEFAULT FALSE,
    consent_version VARCHAR(50) NOT NULL,
    consent_given_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    consent_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    consent_withdrawn_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE consent_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consent_id UUID NOT NULL REFERENCES cookie_consents(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    previous_consents JSONB NOT NULL DEFAULT '{}',
    new_consents JSONB NOT NULL DEFAULT '{}',
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE legal_acceptances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES legal_documents(id) ON DELETE CASCADE,
    document_version VARCHAR(50) NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT,
    UNIQUE(user_id, document_id, document_version)
);

CREATE TABLE data_deletion_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    request_type VARCHAR(50) NOT NULL DEFAULT 'full',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    reason TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scheduled_for TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    confirmation_token VARCHAR(255) NOT NULL,
    confirmed_at TIMESTAMPTZ,
    processed_by UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE data_export_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    format VARCHAR(50) NOT NULL DEFAULT 'json',
    include_sections JSONB NOT NULL DEFAULT '["all"]',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    file_url TEXT,
    file_size INTEGER,
    expires_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_legal_documents_org_bot ON legal_documents(org_id, bot_id);
CREATE INDEX idx_legal_documents_slug ON legal_documents(org_id, bot_id, slug);
CREATE INDEX idx_legal_documents_type ON legal_documents(document_type);
CREATE INDEX idx_legal_documents_active ON legal_documents(is_active) WHERE is_active = TRUE;

CREATE INDEX idx_legal_document_versions_document ON legal_document_versions(document_id);
CREATE INDEX idx_legal_document_versions_version ON legal_document_versions(document_id, version);

CREATE INDEX idx_cookie_consents_org_bot ON cookie_consents(org_id, bot_id);
CREATE INDEX idx_cookie_consents_user ON cookie_consents(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_cookie_consents_session ON cookie_consents(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX idx_cookie_consents_given ON cookie_consents(consent_given_at DESC);

CREATE INDEX idx_consent_history_consent ON consent_history(consent_id);
CREATE INDEX idx_consent_history_action ON consent_history(action);
CREATE INDEX idx_consent_history_created ON consent_history(created_at DESC);

CREATE INDEX idx_legal_acceptances_org_bot ON legal_acceptances(org_id, bot_id);
CREATE INDEX idx_legal_acceptances_user ON legal_acceptances(user_id);
CREATE INDEX idx_legal_acceptances_document ON legal_acceptances(document_id);

CREATE INDEX idx_data_deletion_requests_org_bot ON data_deletion_requests(org_id, bot_id);
CREATE INDEX idx_data_deletion_requests_user ON data_deletion_requests(user_id);
CREATE INDEX idx_data_deletion_requests_status ON data_deletion_requests(status);
CREATE INDEX idx_data_deletion_requests_token ON data_deletion_requests(confirmation_token);

CREATE INDEX idx_data_export_requests_org_bot ON data_export_requests(org_id, bot_id);
CREATE INDEX idx_data_export_requests_user ON data_export_requests(user_id);
CREATE INDEX idx_data_export_requests_status ON data_export_requests(status);
