-- Rollback extended product fields

-- Remove variant indexes
DROP INDEX IF EXISTS idx_product_variants_global_trade_number;

-- Remove variant fields
ALTER TABLE product_variants DROP COLUMN IF EXISTS global_trade_number;
ALTER TABLE product_variants DROP COLUMN IF EXISTS net_weight;
ALTER TABLE product_variants DROP COLUMN IF EXISTS gross_weight;
ALTER TABLE product_variants DROP COLUMN IF EXISTS width;
ALTER TABLE product_variants DROP COLUMN IF EXISTS height;
ALTER TABLE product_variants DROP COLUMN IF EXISTS length;
ALTER TABLE product_variants DROP COLUMN IF EXISTS color;
ALTER TABLE product_variants DROP COLUMN IF EXISTS size;
ALTER TABLE product_variants DROP COLUMN IF EXISTS images;

-- Remove product indexes
DROP INDEX IF EXISTS idx_products_tax_code;
DROP INDEX IF EXISTS idx_products_global_trade_number;
DROP INDEX IF EXISTS idx_products_brand;
DROP INDEX IF EXISTS idx_products_slug;
DROP INDEX IF EXISTS idx_products_expiration;
DROP INDEX IF EXISTS idx_products_external_id;

-- Remove SEO and search
ALTER TABLE products DROP COLUMN IF EXISTS slug;
ALTER TABLE products DROP COLUMN IF EXISTS meta_title;
ALTER TABLE products DROP COLUMN IF EXISTS meta_description;
ALTER TABLE products DROP COLUMN IF EXISTS tags;

-- Remove payment gateway integration
ALTER TABLE products DROP COLUMN IF EXISTS external_id;
ALTER TABLE products DROP COLUMN IF EXISTS external_category_id;
ALTER TABLE products DROP COLUMN IF EXISTS external_metadata;

-- Remove detailed pricing
ALTER TABLE products DROP COLUMN IF EXISTS sale_price;
ALTER TABLE products DROP COLUMN IF EXISTS sale_start;
ALTER TABLE products DROP COLUMN IF EXISTS sale_end;
ALTER TABLE products DROP COLUMN IF EXISTS shipping_cost;
ALTER TABLE products DROP COLUMN IF EXISTS profit_margin;

-- Remove advanced inventory control
ALTER TABLE products DROP COLUMN IF EXISTS warehouse_location;
ALTER TABLE products DROP COLUMN IF EXISTS batch_number;
ALTER TABLE products DROP COLUMN IF EXISTS expiration_date;
ALTER TABLE products DROP COLUMN IF EXISTS manufacture_date;
ALTER TABLE products DROP COLUMN IF EXISTS min_stock;
ALTER TABLE products DROP COLUMN IF EXISTS max_stock;
ALTER TABLE products DROP COLUMN IF EXISTS reorder_point;

-- Remove marketplace and e-commerce fields
ALTER TABLE products DROP COLUMN IF EXISTS brand;
ALTER TABLE products DROP COLUMN IF EXISTS model;
ALTER TABLE products DROP COLUMN IF EXISTS color;
ALTER TABLE products DROP COLUMN IF EXISTS size;
ALTER TABLE products DROP COLUMN IF EXISTS material;
ALTER TABLE products DROP COLUMN IF EXISTS gender;

-- Remove tax rates by type
ALTER TABLE products DROP COLUMN IF EXISTS sales_tax_code;
ALTER TABLE products DROP COLUMN IF EXISTS sales_tax_rate;
ALTER TABLE products DROP COLUMN IF EXISTS excise_tax_code;
ALTER TABLE products DROP COLUMN IF EXISTS excise_tax_rate;
ALTER TABLE products DROP COLUMN IF EXISTS vat_code;
ALTER TABLE products DROP COLUMN IF EXISTS vat_rate;
ALTER TABLE products DROP COLUMN IF EXISTS service_tax_code;
ALTER TABLE products DROP COLUMN IF EXISTS service_tax_rate;

-- Remove detailed dimensions
ALTER TABLE products DROP COLUMN IF EXISTS net_weight;
ALTER TABLE products DROP COLUMN IF EXISTS gross_weight;
ALTER TABLE products DROP COLUMN IF EXISTS width;
ALTER TABLE products DROP COLUMN IF EXISTS height;
ALTER TABLE products DROP COLUMN IF EXISTS length;
ALTER TABLE products DROP COLUMN IF EXISTS package_count;

-- Remove tax and fiscal identification fields
ALTER TABLE products DROP COLUMN IF EXISTS tax_code;
ALTER TABLE products DROP COLUMN IF EXISTS tax_class;
ALTER TABLE products DROP COLUMN IF EXISTS fiscal_code;
ALTER TABLE products DROP COLUMN IF EXISTS origin_code;
ALTER TABLE products DROP COLUMN IF EXISTS global_trade_number;
ALTER TABLE products DROP COLUMN IF EXISTS tax_unit_code;
