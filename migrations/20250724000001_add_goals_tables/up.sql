CREATE TABLE okr_objectives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    parent_id UUID REFERENCES okr_objectives(id) ON DELETE SET NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    period VARCHAR(50) NOT NULL,
    period_start DATE,
    period_end DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    progress DECIMAL(5,2) NOT NULL DEFAULT 0,
    visibility VARCHAR(50) NOT NULL DEFAULT 'team',
    weight DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE okr_key_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    objective_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    metric_type VARCHAR(50) NOT NULL,
    start_value DECIMAL(15,2) NOT NULL DEFAULT 0,
    target_value DECIMAL(15,2) NOT NULL,
    current_value DECIMAL(15,2) NOT NULL DEFAULT 0,
    unit VARCHAR(50),
    weight DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    status VARCHAR(50) NOT NULL DEFAULT 'not_started',
    due_date DATE,
    scoring_type VARCHAR(50) NOT NULL DEFAULT 'linear',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE okr_checkins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    key_result_id UUID NOT NULL REFERENCES okr_key_results(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    previous_value DECIMAL(15,2),
    new_value DECIMAL(15,2) NOT NULL,
    note TEXT,
    confidence VARCHAR(50),
    blockers TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE okr_alignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    child_objective_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    parent_objective_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    alignment_type VARCHAR(50) NOT NULL DEFAULT 'supports',
    weight DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(child_objective_id, parent_objective_id)
);

CREATE TABLE okr_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    objective_template JSONB NOT NULL DEFAULT '{}',
    key_result_templates JSONB NOT NULL DEFAULT '[]',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE okr_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    objective_id UUID REFERENCES okr_objectives(id) ON DELETE CASCADE,
    key_result_id UUID REFERENCES okr_key_results(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    content TEXT NOT NULL,
    parent_comment_id UUID REFERENCES okr_comments(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT okr_comments_target_check CHECK (
        (objective_id IS NOT NULL AND key_result_id IS NULL) OR
        (objective_id IS NULL AND key_result_id IS NOT NULL)
    )
);

CREATE TABLE okr_activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    objective_id UUID REFERENCES okr_objectives(id) ON DELETE CASCADE,
    key_result_id UUID REFERENCES okr_key_results(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    old_value TEXT,
    new_value TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_okr_objectives_org_bot ON okr_objectives(org_id, bot_id);
CREATE INDEX idx_okr_objectives_owner ON okr_objectives(owner_id);
CREATE INDEX idx_okr_objectives_parent ON okr_objectives(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_okr_objectives_period ON okr_objectives(period, period_start, period_end);
CREATE INDEX idx_okr_objectives_status ON okr_objectives(status);

CREATE INDEX idx_okr_key_results_org_bot ON okr_key_results(org_id, bot_id);
CREATE INDEX idx_okr_key_results_objective ON okr_key_results(objective_id);
CREATE INDEX idx_okr_key_results_owner ON okr_key_results(owner_id);
CREATE INDEX idx_okr_key_results_status ON okr_key_results(status);
CREATE INDEX idx_okr_key_results_due_date ON okr_key_results(due_date) WHERE due_date IS NOT NULL;

CREATE INDEX idx_okr_checkins_org_bot ON okr_checkins(org_id, bot_id);
CREATE INDEX idx_okr_checkins_key_result ON okr_checkins(key_result_id);
CREATE INDEX idx_okr_checkins_user ON okr_checkins(user_id);
CREATE INDEX idx_okr_checkins_created ON okr_checkins(created_at DESC);

CREATE INDEX idx_okr_alignments_org_bot ON okr_alignments(org_id, bot_id);
CREATE INDEX idx_okr_alignments_child ON okr_alignments(child_objective_id);
CREATE INDEX idx_okr_alignments_parent ON okr_alignments(parent_objective_id);

CREATE INDEX idx_okr_templates_org_bot ON okr_templates(org_id, bot_id);
CREATE INDEX idx_okr_templates_category ON okr_templates(category);
CREATE INDEX idx_okr_templates_system ON okr_templates(is_system) WHERE is_system = TRUE;

CREATE INDEX idx_okr_comments_org_bot ON okr_comments(org_id, bot_id);
CREATE INDEX idx_okr_comments_objective ON okr_comments(objective_id) WHERE objective_id IS NOT NULL;
CREATE INDEX idx_okr_comments_key_result ON okr_comments(key_result_id) WHERE key_result_id IS NOT NULL;
CREATE INDEX idx_okr_comments_parent ON okr_comments(parent_comment_id) WHERE parent_comment_id IS NOT NULL;

CREATE INDEX idx_okr_activity_org_bot ON okr_activity_log(org_id, bot_id);
CREATE INDEX idx_okr_activity_objective ON okr_activity_log(objective_id) WHERE objective_id IS NOT NULL;
CREATE INDEX idx_okr_activity_key_result ON okr_activity_log(key_result_id) WHERE key_result_id IS NOT NULL;
CREATE INDEX idx_okr_activity_user ON okr_activity_log(user_id);
CREATE INDEX idx_okr_activity_created ON okr_activity_log(created_at DESC);
