pub fn create_contacts_tables_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS contacts (
        id UUID PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
        owner_id UUID REFERENCES users(id),
        first_name TEXT NOT NULL,
        last_name TEXT,
        email TEXT,
        phone TEXT,
        mobile TEXT,
        company TEXT,
        job_title TEXT,
        department TEXT,
        address_line1 TEXT,
        address_line2 TEXT,
        city TEXT,
        state TEXT,
        postal_code TEXT,
        country TEXT,
        website TEXT,
        linkedin TEXT,
        twitter TEXT,
        notes TEXT,
        tags JSONB DEFAULT '[]',
        custom_fields JSONB DEFAULT '{}',
        source TEXT,
        status TEXT NOT NULL DEFAULT 'active',
        is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
        last_contacted_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_contacts_org ON contacts(organization_id);
    CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email);
    CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company);
    CREATE INDEX IF NOT EXISTS idx_contacts_status ON contacts(status);

    CREATE TABLE IF NOT EXISTS contact_groups (
        id UUID PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        description TEXT,
        color TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS contact_group_members (
        contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
        group_id UUID NOT NULL REFERENCES contact_groups(id) ON DELETE CASCADE,
        PRIMARY KEY (contact_id, group_id)
    );

    CREATE TABLE IF NOT EXISTS contact_activities (
        id UUID PRIMARY KEY,
        contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
        activity_type TEXT NOT NULL,
        title TEXT NOT NULL,
        description TEXT,
        related_id UUID,
        related_type TEXT,
        performed_by UUID REFERENCES users(id),
        occurred_at TIMESTAMPTZ NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_contact_activities_contact ON contact_activities(contact_id);
    "#
}
