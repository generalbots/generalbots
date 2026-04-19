CREATE TABLE compliance_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    framework VARCHAR(50) NOT NULL,
    control_id VARCHAR(100) NOT NULL,
    control_name VARCHAR(500) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    score NUMERIC(5,2) NOT NULL DEFAULT 0,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checked_by UUID,
    evidence JSONB NOT NULL DEFAULT '[]',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    check_id UUID REFERENCES compliance_checks(id) ON DELETE CASCADE,
    severity VARCHAR(50) NOT NULL DEFAULT 'medium',
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    remediation TEXT,
    due_date TIMESTAMPTZ,
    assigned_to UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    user_id UUID,
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    result VARCHAR(50) NOT NULL DEFAULT 'success',
    ip_address VARCHAR(45),
    user_agent TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    check_id UUID REFERENCES compliance_checks(id) ON DELETE CASCADE,
    issue_id UUID REFERENCES compliance_issues(id) ON DELETE CASCADE,
    evidence_type VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    file_url TEXT,
    file_name VARCHAR(255),
    file_size INTEGER,
    mime_type VARCHAR(100),
    metadata JSONB NOT NULL DEFAULT '{}',
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    collected_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    title VARCHAR(500) NOT NULL,
    assessor_id UUID NOT NULL,
    methodology VARCHAR(100) NOT NULL DEFAULT 'qualitative',
    overall_risk_score NUMERIC(5,2) NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    next_review_date DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id UUID NOT NULL REFERENCES compliance_risk_assessments(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL DEFAULT 'operational',
    likelihood_score INTEGER NOT NULL DEFAULT 1 CHECK (likelihood_score BETWEEN 1 AND 5),
    impact_score INTEGER NOT NULL DEFAULT 1 CHECK (impact_score BETWEEN 1 AND 5),
    risk_score INTEGER NOT NULL DEFAULT 1,
    risk_level VARCHAR(50) NOT NULL DEFAULT 'low',
    current_controls JSONB NOT NULL DEFAULT '[]',
    treatment_strategy VARCHAR(50) NOT NULL DEFAULT 'mitigate',
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    owner_id UUID,
    due_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    training_type VARCHAR(100) NOT NULL,
    training_name VARCHAR(500) NOT NULL,
    provider VARCHAR(255),
    score INTEGER,
    passed BOOLEAN NOT NULL DEFAULT FALSE,
    completion_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    certificate_url TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE compliance_access_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    reviewer_id UUID NOT NULL,
    review_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    permissions_reviewed JSONB NOT NULL DEFAULT '[]',
    anomalies JSONB NOT NULL DEFAULT '[]',
    recommendations JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    approved_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_compliance_checks_org_bot ON compliance_checks(org_id, bot_id);
CREATE INDEX idx_compliance_checks_framework ON compliance_checks(framework);
CREATE INDEX idx_compliance_checks_status ON compliance_checks(status);
CREATE INDEX idx_compliance_checks_checked_at ON compliance_checks(checked_at DESC);

CREATE INDEX idx_compliance_issues_org_bot ON compliance_issues(org_id, bot_id);
CREATE INDEX idx_compliance_issues_check ON compliance_issues(check_id);
CREATE INDEX idx_compliance_issues_severity ON compliance_issues(severity);
CREATE INDEX idx_compliance_issues_status ON compliance_issues(status);
CREATE INDEX idx_compliance_issues_assigned ON compliance_issues(assigned_to) WHERE assigned_to IS NOT NULL;
CREATE INDEX idx_compliance_issues_due ON compliance_issues(due_date) WHERE due_date IS NOT NULL;

CREATE INDEX idx_compliance_audit_log_org_bot ON compliance_audit_log(org_id, bot_id);
CREATE INDEX idx_compliance_audit_log_event ON compliance_audit_log(event_type);
CREATE INDEX idx_compliance_audit_log_user ON compliance_audit_log(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_compliance_audit_log_resource ON compliance_audit_log(resource_type, resource_id);
CREATE INDEX idx_compliance_audit_log_created ON compliance_audit_log(created_at DESC);

CREATE INDEX idx_compliance_evidence_org_bot ON compliance_evidence(org_id, bot_id);
CREATE INDEX idx_compliance_evidence_check ON compliance_evidence(check_id) WHERE check_id IS NOT NULL;
CREATE INDEX idx_compliance_evidence_issue ON compliance_evidence(issue_id) WHERE issue_id IS NOT NULL;
CREATE INDEX idx_compliance_evidence_type ON compliance_evidence(evidence_type);

CREATE INDEX idx_compliance_risk_assessments_org_bot ON compliance_risk_assessments(org_id, bot_id);
CREATE INDEX idx_compliance_risk_assessments_status ON compliance_risk_assessments(status);
CREATE INDEX idx_compliance_risk_assessments_assessor ON compliance_risk_assessments(assessor_id);

CREATE INDEX idx_compliance_risks_assessment ON compliance_risks(assessment_id);
CREATE INDEX idx_compliance_risks_category ON compliance_risks(category);
CREATE INDEX idx_compliance_risks_level ON compliance_risks(risk_level);
CREATE INDEX idx_compliance_risks_status ON compliance_risks(status);

CREATE INDEX idx_compliance_training_org_bot ON compliance_training_records(org_id, bot_id);
CREATE INDEX idx_compliance_training_user ON compliance_training_records(user_id);
CREATE INDEX idx_compliance_training_type ON compliance_training_records(training_type);
CREATE INDEX idx_compliance_training_valid ON compliance_training_records(valid_until) WHERE valid_until IS NOT NULL;

CREATE INDEX idx_compliance_access_reviews_org_bot ON compliance_access_reviews(org_id, bot_id);
CREATE INDEX idx_compliance_access_reviews_user ON compliance_access_reviews(user_id);
CREATE INDEX idx_compliance_access_reviews_reviewer ON compliance_access_reviews(reviewer_id);
CREATE INDEX idx_compliance_access_reviews_status ON compliance_access_reviews(status);
