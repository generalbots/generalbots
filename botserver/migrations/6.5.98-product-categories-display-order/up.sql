-- Catches databases up with 6.5.15.1-consolidated for product_categories.
-- Prod installs created the table before that migration existed, so the
-- botproducts category seeding failed on every signup with
-- 'column "display_order" of relation "product_categories" does not exist'.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables
                   WHERE table_schema = 'public' AND table_name = 'product_categories') THEN
        RETURN; -- nothing to do on shapes without the table
    END IF;

    ALTER TABLE product_categories ADD COLUMN IF NOT EXISTS display_order INTEGER;
    ALTER TABLE product_categories ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

    -- botproducts seed may not provide org_id/bot_id
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_schema = 'public' AND table_name = 'product_categories'
                 AND column_name = 'org_id' AND is_nullable = 'NO') THEN
        ALTER TABLE product_categories ALTER COLUMN org_id DROP NOT NULL;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_schema = 'public' AND table_name = 'product_categories'
                 AND column_name = 'bot_id' AND is_nullable = 'NO') THEN
        ALTER TABLE product_categories ALTER COLUMN bot_id DROP NOT NULL;
    END IF;
END $$;
