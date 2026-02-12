pub fn create_webinar_tables_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS webinars (
        id UUID PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
        meeting_id UUID NOT NULL,
        title TEXT NOT NULL,
        description TEXT,
        scheduled_start TIMESTAMPTZ NOT NULL,
        scheduled_end TIMESTAMPTZ,
        actual_start TIMESTAMPTZ,
        actual_end TIMESTAMPTZ,
        status TEXT NOT NULL DEFAULT 'scheduled',
        settings_json TEXT NOT NULL DEFAULT '{}',
        registration_required BOOLEAN NOT NULL DEFAULT FALSE,
        registration_url TEXT,
        host_id UUID NOT NULL REFERENCES users(id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS webinar_participants (
        id UUID PRIMARY KEY,
        webinar_id UUID NOT NULL REFERENCES webinars(id) ON DELETE CASCADE,
        user_id UUID REFERENCES users(id),
        name TEXT NOT NULL,
        email TEXT,
        role TEXT NOT NULL DEFAULT 'attendee',
        status TEXT NOT NULL DEFAULT 'registered',
        hand_raised BOOLEAN NOT NULL DEFAULT FALSE,
        hand_raised_at TIMESTAMPTZ,
        is_speaking BOOLEAN NOT NULL DEFAULT FALSE,
        video_enabled BOOLEAN NOT NULL DEFAULT FALSE,
        audio_enabled BOOLEAN NOT NULL DEFAULT FALSE,
        screen_sharing BOOLEAN NOT NULL DEFAULT FALSE,
        joined_at TIMESTAMPTZ,
        left_at TIMESTAMPTZ,
        registration_data TEXT
    );

    CREATE TABLE IF NOT EXISTS webinar_registrations (
        id UUID PRIMARY KEY,
        webinar_id UUID NOT NULL REFERENCES webinars(id) ON DELETE CASCADE,
        email TEXT NOT NULL,
        name TEXT NOT NULL,
        custom_fields TEXT DEFAULT '{}',
        status TEXT NOT NULL DEFAULT 'pending',
        join_link TEXT NOT NULL,
        registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        confirmed_at TIMESTAMPTZ,
        cancelled_at TIMESTAMPTZ,
        UNIQUE(webinar_id, email)
    );

    CREATE TABLE IF NOT EXISTS webinar_questions (
        id UUID PRIMARY KEY,
        webinar_id UUID NOT NULL REFERENCES webinars(id) ON DELETE CASCADE,
        asker_id UUID REFERENCES users(id),
        asker_name TEXT NOT NULL,
        is_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
        question TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        upvotes INTEGER NOT NULL DEFAULT 0,
        upvoted_by TEXT,
        answer TEXT,
        answered_by UUID REFERENCES users(id),
        answered_at TIMESTAMPTZ,
        is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
        is_highlighted BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_webinars_org ON webinars(organization_id);
    CREATE INDEX IF NOT EXISTS idx_webinar_participants_webinar ON webinar_participants(webinar_id);
    CREATE INDEX IF NOT EXISTS idx_webinar_questions_webinar ON webinar_questions(webinar_id);
    "#
}
