-- Migration: 6.1.2_table_role_access
-- Add role-based access control columns to dynamic table definitions and fields
--
-- Syntax in .gbdialog TABLE definitions:
--   TABLE Contatos ON maria READ BY "admin;manager"
--       Id number key
--       Nome string(150)
--       NumeroDocumento string(25) READ BY "admin"
--       Celular string(20) WRITE BY "admin;manager"
--
-- Empty roles = everyone has access (default behavior)
-- Roles are semicolon-separated and match Zitadel directory roles

-- Add role columns to dynamic_table_definitions
ALTER TABLE dynamic_table_definitions
ADD COLUMN IF NOT EXISTS read_roles TEXT DEFAULT NULL,
ADD COLUMN IF NOT EXISTS write_roles TEXT DEFAULT NULL;

-- Add role columns to dynamic_table_fields
ALTER TABLE dynamic_table_fields
ADD COLUMN IF NOT EXISTS read_roles TEXT DEFAULT NULL,
ADD COLUMN IF NOT EXISTS write_roles TEXT DEFAULT NULL;

-- Add comments for documentation
COMMENT ON COLUMN dynamic_table_definitions.read_roles IS 'Semicolon-separated roles that can read from this table (empty = everyone)';
COMMENT ON COLUMN dynamic_table_definitions.write_roles IS 'Semicolon-separated roles that can write to this table (empty = everyone)';
COMMENT ON COLUMN dynamic_table_fields.read_roles IS 'Semicolon-separated roles that can read this field (empty = everyone)';
COMMENT ON COLUMN dynamic_table_fields.write_roles IS 'Semicolon-separated roles that can write this field (empty = everyone)';
