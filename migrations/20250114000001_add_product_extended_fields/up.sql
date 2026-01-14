-- Extended product fields for e-commerce and payment integrations

-- Tax and fiscal identification fields
ALTER TABLE products ADD COLUMN IF NOT EXISTS tax_code VARCHAR(10);
ALTER TABLE products ADD COLUMN IF NOT EXISTS tax_class VARCHAR(50);
ALTER TABLE products ADD COLUMN IF NOT EXISTS fiscal_code VARCHAR(10);
ALTER TABLE products ADD COLUMN IF NOT EXISTS origin_code INTEGER DEFAULT 0;
ALTER TABLE products ADD COLUMN IF NOT EXISTS global_trade_number VARCHAR(14);
ALTER TABLE products ADD COLUMN IF NOT EXISTS tax_unit_code VARCHAR(14);

-- Detailed dimensions (for shipping calculation)
ALTER TABLE products ADD COLUMN IF NOT EXISTS net_weight DECIMAL(10,3);
ALTER TABLE products ADD COLUMN IF NOT EXISTS gross_weight DECIMAL(10,3);
ALTER TABLE products ADD COLUMN IF NOT EXISTS width DECIMAL(10,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS height DECIMAL(10,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS length DECIMAL(10,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS package_count INTEGER DEFAULT 1;

-- Tax rates by type
ALTER TABLE products ADD COLUMN IF NOT EXISTS sales_tax_code VARCHAR(3);
ALTER TABLE products ADD COLUMN IF NOT EXISTS sales_tax_rate DECIMAL(5,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS excise_tax_code VARCHAR(2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS excise_tax_rate DECIMAL(5,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS vat_code VARCHAR(2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS vat_rate DECIMAL(5,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS service_tax_code VARCHAR(2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS service_tax_rate DECIMAL(5,2);

-- Marketplace and e-commerce fields
ALTER TABLE products ADD COLUMN IF NOT EXISTS brand VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS model VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS color VARCHAR(50);
ALTER TABLE products ADD COLUMN IF NOT EXISTS size VARCHAR(20);
ALTER TABLE products ADD COLUMN IF NOT EXISTS material VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS gender VARCHAR(20);

-- Advanced inventory control
ALTER TABLE products ADD COLUMN IF NOT EXISTS warehouse_location VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS batch_number VARCHAR(50);
ALTER TABLE products ADD COLUMN IF NOT EXISTS expiration_date DATE;
ALTER TABLE products ADD COLUMN IF NOT EXISTS manufacture_date DATE;
ALTER TABLE products ADD COLUMN IF NOT EXISTS min_stock INTEGER DEFAULT 0;
ALTER TABLE products ADD COLUMN IF NOT EXISTS max_stock INTEGER;
ALTER TABLE products ADD COLUMN IF NOT EXISTS reorder_point INTEGER;

-- Detailed pricing
ALTER TABLE products ADD COLUMN IF NOT EXISTS sale_price DECIMAL(15,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS sale_start TIMESTAMPTZ;
ALTER TABLE products ADD COLUMN IF NOT EXISTS sale_end TIMESTAMPTZ;
ALTER TABLE products ADD COLUMN IF NOT EXISTS shipping_cost DECIMAL(15,2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS profit_margin DECIMAL(5,2);

-- Payment gateway integration
ALTER TABLE products ADD COLUMN IF NOT EXISTS external_id VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS external_category_id VARCHAR(100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS external_metadata JSONB DEFAULT '{}';

-- SEO and search
ALTER TABLE products ADD COLUMN IF NOT EXISTS slug VARCHAR(255);
ALTER TABLE products ADD COLUMN IF NOT EXISTS meta_title VARCHAR(255);
ALTER TABLE products ADD COLUMN IF NOT EXISTS meta_description TEXT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS tags TEXT[];

-- Indexes for new fields
CREATE INDEX IF NOT EXISTS idx_products_tax_code ON products(tax_code);
CREATE INDEX IF NOT EXISTS idx_products_global_trade_number ON products(global_trade_number);
CREATE INDEX IF NOT EXISTS idx_products_brand ON products(brand);
CREATE INDEX IF NOT EXISTS idx_products_slug ON products(slug);
CREATE INDEX IF NOT EXISTS idx_products_expiration ON products(expiration_date);
CREATE INDEX IF NOT EXISTS idx_products_external_id ON products(external_id);

-- Add similar fields to product variants
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS global_trade_number VARCHAR(14);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS net_weight DECIMAL(10,3);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS gross_weight DECIMAL(10,3);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS width DECIMAL(10,2);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS height DECIMAL(10,2);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS length DECIMAL(10,2);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS color VARCHAR(50);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS size VARCHAR(20);
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS images JSONB DEFAULT '[]';

CREATE INDEX IF NOT EXISTS idx_product_variants_global_trade_number ON product_variants(global_trade_number);
