-- Add user_email_accounts table for storing user email credentials
CREATE TABLE public.user_email_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    email varchar(255) NOT NULL,
    display_name varchar(255) NULL,
    imap_server varchar(255) NOT NULL,
    imap_port int4 DEFAULT 993 NOT NULL,
    smtp_server varchar(255) NOT NULL,
    smtp_port int4 DEFAULT 587 NOT NULL,
    username varchar(255) NOT NULL,
    password_encrypted text NOT NULL,
    is_primary bool DEFAULT false NOT NULL,
    is_active bool DEFAULT true NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT user_email_accounts_pkey PRIMARY KEY (id),
    CONSTRAINT user_email_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT user_email_accounts_user_email_key UNIQUE (user_id, email)
);

CREATE INDEX idx_user_email_accounts_user_id ON public.user_email_accounts USING btree (user_id);
CREATE INDEX idx_user_email_accounts_active ON public.user_email_accounts USING btree (is_active) WHERE is_active;

-- Add email drafts table
CREATE TABLE public.email_drafts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    account_id uuid NOT NULL,
    to_address text NOT NULL,
    cc_address text NULL,
    bcc_address text NULL,
    subject varchar(500) NULL,
    body text NULL,
    attachments jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT email_drafts_pkey PRIMARY KEY (id),
    CONSTRAINT email_drafts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT email_drafts_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.user_email_accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_email_drafts_user_id ON public.email_drafts USING btree (user_id);
CREATE INDEX idx_email_drafts_account_id ON public.email_drafts USING btree (account_id);

-- Add email folders metadata table (for caching and custom folders)
CREATE TABLE public.email_folders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    account_id uuid NOT NULL,
    folder_name varchar(255) NOT NULL,
    folder_path varchar(500) NOT NULL,
    unread_count int4 DEFAULT 0 NOT NULL,
    total_count int4 DEFAULT 0 NOT NULL,
    last_synced timestamptz NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT email_folders_pkey PRIMARY KEY (id),
    CONSTRAINT email_folders_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.user_email_accounts(id) ON DELETE CASCADE,
    CONSTRAINT email_folders_account_path_key UNIQUE (account_id, folder_path)
);

CREATE INDEX idx_email_folders_account_id ON public.email_folders USING btree (account_id);

-- Add sessions table enhancement for storing current email account
ALTER TABLE public.user_sessions
ADD COLUMN IF NOT EXISTS active_email_account_id uuid NULL,
ADD CONSTRAINT user_sessions_email_account_id_fkey
FOREIGN KEY (active_email_account_id) REFERENCES public.user_email_accounts(id) ON DELETE SET NULL;

-- Add user preferences table
CREATE TABLE public.user_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    preference_key varchar(100) NOT NULL,
    preference_value jsonb NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT user_preferences_pkey PRIMARY KEY (id),
    CONSTRAINT user_preferences_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT user_preferences_user_key_unique UNIQUE (user_id, preference_key)
);

CREATE INDEX idx_user_preferences_user_id ON public.user_preferences USING btree (user_id);

-- Add login tokens table for session management
CREATE TABLE public.user_login_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    token_hash varchar(255) NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    last_used timestamptz DEFAULT now() NOT NULL,
    user_agent text NULL,
    ip_address varchar(50) NULL,
    is_active bool DEFAULT true NOT NULL,
    CONSTRAINT user_login_tokens_pkey PRIMARY KEY (id),
    CONSTRAINT user_login_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT user_login_tokens_token_hash_key UNIQUE (token_hash)
);

CREATE INDEX idx_user_login_tokens_user_id ON public.user_login_tokens USING btree (user_id);
CREATE INDEX idx_user_login_tokens_expires ON public.user_login_tokens USING btree (expires_at) WHERE is_active;
