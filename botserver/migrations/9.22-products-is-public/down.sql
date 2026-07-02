DROP INDEX IF EXISTS idx_products_is_public;
ALTER TABLE products DROP COLUMN IF EXISTS is_public;
