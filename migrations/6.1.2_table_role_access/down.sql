-- Rollback: Remove role-based access control columns from dynamic tables
-- Migration: 6.1.2_table_role_access

-- Remove columns from dynamic_table_definitions
ALTER TABLE dynamic_table_definitions
    DROP COLUMN IF EXISTS read_roles,
    DROP COLUMN IF EXISTS write_roles;

-- Remove columns from dynamic_table_fields
ALTER TABLE dynamic_table_fields
    DROP COLUMN IF EXISTS read_roles,
    DROP COLUMN IF EXISTS write_roles;
