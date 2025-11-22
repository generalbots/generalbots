-- Add organization relationship to bots
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS org_id UUID,
ADD COLUMN IF NOT EXISTS is_default BOOLEAN DEFAULT false;

-- Add foreign key constraint to organizations
ALTER TABLE public.bots
ADD CONSTRAINT bots_org_id_fkey
FOREIGN KEY (org_id) REFERENCES public.organizations(org_id) ON DELETE CASCADE;

-- Create index for org_id lookups
CREATE INDEX IF NOT EXISTS idx_bots_org_id ON public.bots(org_id);

-- Create directory_users table to map directory (Zitadel) users to our system
CREATE TABLE IF NOT EXISTS public.directory_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    directory_id VARCHAR(255) NOT NULL UNIQUE, -- Zitadel user ID
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES public.organizations(org_id) ON DELETE CASCADE,
    bot_id UUID REFERENCES public.bots(id) ON DELETE SET NULL,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    is_admin BOOLEAN DEFAULT false,
    is_bot_user BOOLEAN DEFAULT false, -- true for bot service accounts
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create indexes for directory_users
CREATE INDEX IF NOT EXISTS idx_directory_users_org_id ON public.directory_users(org_id);
CREATE INDEX IF NOT EXISTS idx_directory_users_bot_id ON public.directory_users(bot_id);
CREATE INDEX IF NOT EXISTS idx_directory_users_email ON public.directory_users(email);
CREATE INDEX IF NOT EXISTS idx_directory_users_directory_id ON public.directory_users(directory_id);

-- Create bot_access table to manage which users can access which bots
CREATE TABLE IF NOT EXISTS public.bot_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES public.bots(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES public.directory_users(id) ON DELETE CASCADE,
    access_level VARCHAR(50) NOT NULL DEFAULT 'user', -- 'owner', 'admin', 'user', 'viewer'
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    granted_by UUID REFERENCES public.directory_users(id),
    UNIQUE(bot_id, user_id)
);

-- Create indexes for bot_access
CREATE INDEX IF NOT EXISTS idx_bot_access_bot_id ON public.bot_access(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_access_user_id ON public.bot_access(user_id);

-- Create OAuth application registry for directory integrations
CREATE TABLE IF NOT EXISTS public.oauth_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES public.organizations(org_id) ON DELETE CASCADE,
    project_id VARCHAR(255),
    client_id VARCHAR(255) NOT NULL UNIQUE,
    client_secret_encrypted TEXT NOT NULL, -- Store encrypted
    redirect_uris TEXT[] NOT NULL DEFAULT '{}',
    application_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create index for OAuth applications
CREATE INDEX IF NOT EXISTS idx_oauth_applications_org_id ON public.oauth_applications(org_id);
CREATE INDEX IF NOT EXISTS idx_oauth_applications_client_id ON public.oauth_applications(client_id);

-- Insert default organization if it doesn't exist
INSERT INTO public.organizations (org_id, name, slug, created_at, updated_at)
VALUES (
    'f47ac10b-58cc-4372-a567-0e02b2c3d479'::uuid, -- Fixed UUID for default org
    'Default Organization',
    'default',
    NOW(),
    NOW()
) ON CONFLICT (slug) DO NOTHING;

-- Insert default bot for the default organization
DO $$
DECLARE
    v_org_id UUID;
    v_bot_id UUID;
BEGIN
    -- Get the default organization ID
    SELECT org_id INTO v_org_id FROM public.organizations WHERE slug = 'default';

    -- Generate or use fixed UUID for default bot
    v_bot_id := 'f47ac10b-58cc-4372-a567-0e02b2c3d480'::uuid;

    -- Insert default bot if it doesn't exist
    INSERT INTO public.bots (
        id,
        org_id,
        name,
        description,
        llm_provider,
        llm_config,
        context_provider,
        context_config,
        is_default,
        is_active,
        created_at,
        updated_at
    )
    VALUES (
        v_bot_id,
        v_org_id,
        'Default Bot',
        'Default bot for the default organization',
        'openai',
        '{"model": "gpt-4", "temperature": 0.7}'::jsonb,
        'none',
        '{}'::jsonb,
        true,
        true,
        NOW(),
        NOW()
    ) ON CONFLICT (id) DO UPDATE
    SET org_id = EXCLUDED.org_id,
        is_default = true,
        updated_at = NOW();

    -- Insert default admin user (admin@default)
    INSERT INTO public.directory_users (
        directory_id,
        username,
        email,
        org_id,
        bot_id,
        first_name,
        last_name,
        is_admin,
        is_bot_user,
        created_at,
        updated_at
    )
    VALUES (
        'admin-default-001', -- Will be replaced with actual Zitadel ID
        'admin',
        'admin@default',
        v_org_id,
        v_bot_id,
        'Admin',
        'Default',
        true,
        false,
        NOW(),
        NOW()
    ) ON CONFLICT (email) DO UPDATE
    SET org_id = EXCLUDED.org_id,
        bot_id = EXCLUDED.bot_id,
        is_admin = true,
        updated_at = NOW();

    -- Insert default regular user (user@default)
    INSERT INTO public.directory_users (
        directory_id,
        username,
        email,
        org_id,
        bot_id,
        first_name,
        last_name,
        is_admin,
        is_bot_user,
        created_at,
        updated_at
    )
    VALUES (
        'user-default-001', -- Will be replaced with actual Zitadel ID
        'user',
        'user@default',
        v_org_id,
        v_bot_id,
        'User',
        'Default',
        false,
        false,
        NOW(),
        NOW()
    ) ON CONFLICT (email) DO UPDATE
    SET org_id = EXCLUDED.org_id,
        bot_id = EXCLUDED.bot_id,
        is_admin = false,
        updated_at = NOW();

    -- Grant bot access to admin user
    INSERT INTO public.bot_access (bot_id, user_id, access_level, granted_at)
    SELECT
        v_bot_id,
        id,
        'owner',
        NOW()
    FROM public.directory_users
    WHERE email = 'admin@default'
    ON CONFLICT (bot_id, user_id) DO UPDATE
    SET access_level = 'owner',
        granted_at = NOW();

    -- Grant bot access to regular user
    INSERT INTO public.bot_access (bot_id, user_id, access_level, granted_at)
    SELECT
        v_bot_id,
        id,
        'user',
        NOW()
    FROM public.directory_users
    WHERE email = 'user@default'
    ON CONFLICT (bot_id, user_id) DO UPDATE
    SET access_level = 'user',
        granted_at = NOW();

END $$;

-- Create function to update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for updated_at columns if they don't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_directory_users_updated_at') THEN
        CREATE TRIGGER update_directory_users_updated_at
        BEFORE UPDATE ON public.directory_users
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_oauth_applications_updated_at') THEN
        CREATE TRIGGER update_oauth_applications_updated_at
        BEFORE UPDATE ON public.oauth_applications
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;

-- Add comment documentation
COMMENT ON TABLE public.directory_users IS 'Maps directory (Zitadel) users to the system and their associated bots';
COMMENT ON TABLE public.bot_access IS 'Controls which users have access to which bots and their permission levels';
COMMENT ON TABLE public.oauth_applications IS 'OAuth application configurations for directory integration';
COMMENT ON COLUMN public.bots.is_default IS 'Indicates if this is the default bot for an organization';
COMMENT ON COLUMN public.directory_users.is_bot_user IS 'True if this user is a service account for bot operations';
COMMENT ON COLUMN public.bot_access.access_level IS 'Access level: owner (full control), admin (manage), user (use), viewer (read-only)';
