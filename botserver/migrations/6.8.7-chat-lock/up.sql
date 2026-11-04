-- #1682 — locked conversations: hidden from history/search and gated behind
-- a PIN hash (Argon2, stored per session). Content history is already
-- encrypted at rest by message_history.
ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS locked BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS lock_pin_hash TEXT;
