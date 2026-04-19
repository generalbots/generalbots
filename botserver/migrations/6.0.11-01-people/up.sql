CREATE TABLE people (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    job_title VARCHAR(255),
    department VARCHAR(255),
    manager_id UUID REFERENCES people(id) ON DELETE SET NULL,
    office_location VARCHAR(255),
    hire_date DATE,
    birthday DATE,
    avatar_url TEXT,
    bio TEXT,
    skills TEXT[] NOT NULL DEFAULT '{}',
    social_links JSONB NOT NULL DEFAULT '{}',
    custom_fields JSONB NOT NULL DEFAULT '{}',
    timezone VARCHAR(50),
    locale VARCHAR(10),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE people_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    leader_id UUID REFERENCES people(id) ON DELETE SET NULL,
    parent_team_id UUID REFERENCES people_teams(id) ON DELETE SET NULL,
    color VARCHAR(20),
    icon VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE people_team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES people_teams(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role VARCHAR(100),
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, person_id)
);

CREATE TABLE people_org_chart (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    reports_to_id UUID REFERENCES people(id) ON DELETE SET NULL,
    position_title VARCHAR(255),
    position_level INTEGER NOT NULL DEFAULT 0,
    position_order INTEGER NOT NULL DEFAULT 0,
    effective_from DATE,
    effective_until DATE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, person_id, effective_from)
);

CREATE TABLE people_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    code VARCHAR(50),
    parent_id UUID REFERENCES people_departments(id) ON DELETE SET NULL,
    head_id UUID REFERENCES people(id) ON DELETE SET NULL,
    cost_center VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE people_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE people_person_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    skill_id UUID NOT NULL REFERENCES people_skills(id) ON DELETE CASCADE,
    proficiency_level INTEGER NOT NULL DEFAULT 1,
    years_experience DECIMAL(4,1),
    verified_by UUID REFERENCES people(id) ON DELETE SET NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(person_id, skill_id)
);

CREATE TABLE people_time_off (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    time_off_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    hours_requested DECIMAL(5,1),
    reason TEXT,
    approved_by UUID REFERENCES people(id) ON DELETE SET NULL,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_people_org_bot ON people(org_id, bot_id);
CREATE INDEX idx_people_email ON people(email);
CREATE INDEX idx_people_department ON people(department);
CREATE INDEX idx_people_manager ON people(manager_id);
CREATE INDEX idx_people_active ON people(is_active);
CREATE INDEX idx_people_user ON people(user_id);
CREATE UNIQUE INDEX idx_people_org_email ON people(org_id, email) WHERE email IS NOT NULL;

CREATE INDEX idx_people_teams_org_bot ON people_teams(org_id, bot_id);
CREATE INDEX idx_people_teams_parent ON people_teams(parent_team_id);
CREATE INDEX idx_people_teams_leader ON people_teams(leader_id);

CREATE INDEX idx_people_team_members_team ON people_team_members(team_id);
CREATE INDEX idx_people_team_members_person ON people_team_members(person_id);

CREATE INDEX idx_people_org_chart_org ON people_org_chart(org_id, bot_id);
CREATE INDEX idx_people_org_chart_person ON people_org_chart(person_id);
CREATE INDEX idx_people_org_chart_reports_to ON people_org_chart(reports_to_id);

CREATE INDEX idx_people_departments_org_bot ON people_departments(org_id, bot_id);
CREATE INDEX idx_people_departments_parent ON people_departments(parent_id);
CREATE INDEX idx_people_departments_head ON people_departments(head_id);
CREATE UNIQUE INDEX idx_people_departments_org_code ON people_departments(org_id, code) WHERE code IS NOT NULL;

CREATE INDEX idx_people_skills_org_bot ON people_skills(org_id, bot_id);
CREATE INDEX idx_people_skills_category ON people_skills(category);

CREATE INDEX idx_people_person_skills_person ON people_person_skills(person_id);
CREATE INDEX idx_people_person_skills_skill ON people_person_skills(skill_id);

CREATE INDEX idx_people_time_off_org_bot ON people_time_off(org_id, bot_id);
CREATE INDEX idx_people_time_off_person ON people_time_off(person_id);
CREATE INDEX idx_people_time_off_dates ON people_time_off(start_date, end_date);
CREATE INDEX idx_people_time_off_status ON people_time_off(status);
