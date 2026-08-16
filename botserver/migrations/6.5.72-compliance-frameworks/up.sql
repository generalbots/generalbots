-- Compliance framework configuration (enterprise-grade release)
--
-- Persists user-defined compliance frameworks (LGPD, GDPR, SOC 2, ISO 27001,
-- PCI-DSS or custom) with their control catalogs and the evidence attached to
-- each control. Coverage is computed from evidence + automated scan results,
-- and every mutation is written to the existing `compliance_audit_log`.

CREATE TABLE IF NOT EXISTS compliance_frameworks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    description TEXT,
    framework_key VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_compliance_frameworks_name_version
    ON compliance_frameworks (branch_id, name, version);

CREATE TABLE IF NOT EXISTS compliance_controls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL,
    framework_id UUID NOT NULL REFERENCES compliance_frameworks(id) ON DELETE CASCADE,
    control_id VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    is_mandatory BOOLEAN NOT NULL DEFAULT TRUE,
    version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (framework_id, control_id)
);

CREATE INDEX IF NOT EXISTS idx_compliance_controls_framework
    ON compliance_controls (framework_id);
CREATE INDEX IF NOT EXISTS idx_compliance_controls_branch
    ON compliance_controls (branch_id);

CREATE TABLE IF NOT EXISTS compliance_control_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL,
    control_id UUID NOT NULL REFERENCES compliance_controls(id) ON DELETE CASCADE,
    file_path VARCHAR(500) NOT NULL,
    description TEXT,
    evidence_type VARCHAR(50) NOT NULL DEFAULT 'artifact',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    owner_id UUID,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_control
    ON compliance_control_evidence (control_id);
CREATE INDEX IF NOT EXISTS idx_compliance_evidence_branch
    ON compliance_control_evidence (branch_id);
