-- Reverting drops only the optional address column; the audit rows and their
-- indexes stay intact, so the rollback is lossless for every pre-existing row.

ALTER TABLE collab_activity
    DROP COLUMN IF EXISTS ip;
