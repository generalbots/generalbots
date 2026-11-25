-- Reverting drops only the policy column: tokens, roles, expiries and owners
-- stay intact, so every pre-existing link keeps its original behaviour.

ALTER TABLE resource_links
    DROP COLUMN IF EXISTS allow_download;
