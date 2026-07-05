-- Add branch_id to bot_configuration
ALTER TABLE bot_configuration ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;

-- Backfill branch_id from bots table
UPDATE bot_configuration bc
SET branch_id = b.branch_id
FROM bots b
WHERE bc.bot_id = b.id AND bc.branch_id IS NULL;

-- Default branch for any remaining NULLs
UPDATE bot_configuration
SET branch_id = '00000000-0000-0000-0000-000000000000'
WHERE branch_id IS NULL;

-- Make NOT NULL
ALTER TABLE bot_configuration ALTER COLUMN branch_id SET NOT NULL;

-- Drop old unique constraint, create scoped one
ALTER TABLE bot_configuration DROP CONSTRAINT IF EXISTS bot_configuration_bot_id_config_key_key;
ALTER TABLE bot_configuration DROP CONSTRAINT IF EXISTS bot_configuration_pkey CASCADE;
ALTER TABLE bot_configuration ADD PRIMARY KEY (id);
ALTER TABLE bot_configuration ADD UNIQUE (branch_id, bot_id, config_key);

-- Remove deprecated columns that are now handled differently
ALTER TABLE bot_configuration DROP COLUMN IF EXISTS config_type;
ALTER TABLE bot_configuration DROP COLUMN IF EXISTS is_encrypted;
