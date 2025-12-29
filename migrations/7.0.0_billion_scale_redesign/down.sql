-- Migration: 7.0.0 Billion Scale Redesign - ROLLBACK
-- Description: Drops the gb schema and all its objects
-- WARNING: This is a DESTRUCTIVE operation - all data will be lost

-- Drop the entire schema (CASCADE drops all objects within)
DROP SCHEMA IF EXISTS gb CASCADE;

-- Note: This migration completely removes the v7 schema.
-- To restore previous schema, run migrations 6.x.x in order.
