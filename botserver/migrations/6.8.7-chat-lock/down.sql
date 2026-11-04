ALTER TABLE user_sessions DROP COLUMN IF EXISTS lock_pin_hash;
ALTER TABLE user_sessions DROP COLUMN IF EXISTS locked;
