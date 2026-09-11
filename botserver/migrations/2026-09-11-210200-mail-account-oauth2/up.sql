-- OAuth2 authentication mode for mail accounts.
--
-- Microsoft 365, Outlook.com and Gmail no longer accept a static password over
-- IMAP/SMTP, so an account may authenticate with an OAuth2 access token instead
-- (issue #1333). `password_encrypted` remains populated for password accounts
-- and is unused when `auth_mode` is 'oauth2'.
ALTER TABLE user_email_accounts
    ADD COLUMN IF NOT EXISTS auth_mode VARCHAR(20) NOT NULL DEFAULT 'password',
    ADD COLUMN IF NOT EXISTS oauth_provider VARCHAR(40),
    ADD COLUMN IF NOT EXISTS refresh_token_encrypted TEXT,
    ADD COLUMN IF NOT EXISTS access_token_encrypted TEXT,
    ADD COLUMN IF NOT EXISTS token_expires_at TIMESTAMPTZ;

ALTER TABLE user_email_accounts
    DROP CONSTRAINT IF EXISTS user_email_accounts_auth_mode_check;

ALTER TABLE user_email_accounts
    ADD CONSTRAINT user_email_accounts_auth_mode_check
    CHECK (auth_mode IN ('password', 'oauth2'));

CREATE INDEX IF NOT EXISTS idx_user_email_accounts_auth_mode
    ON user_email_accounts (auth_mode);
