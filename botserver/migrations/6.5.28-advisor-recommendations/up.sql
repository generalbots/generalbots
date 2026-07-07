CREATE TABLE IF NOT EXISTS advisor_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    campaign_id UUID,
    recommendation TEXT NOT NULL,
    reason TEXT,
    check_name VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    message TEXT NOT NULL,
    details TEXT,
    dismissed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_advisor_rec_branch ON advisor_recommendations (branch_id);
CREATE INDEX IF NOT EXISTS idx_advisor_rec_severity ON advisor_recommendations (severity);
