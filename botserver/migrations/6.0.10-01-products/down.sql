DROP INDEX IF EXISTS idx_product_variants_sku;
DROP INDEX IF EXISTS idx_product_variants_product;

DROP INDEX IF EXISTS idx_inventory_movements_created;
DROP INDEX IF EXISTS idx_inventory_movements_product;
DROP INDEX IF EXISTS idx_inventory_movements_org_bot;

DROP INDEX IF EXISTS idx_price_list_items_service;
DROP INDEX IF EXISTS idx_price_list_items_product;
DROP INDEX IF EXISTS idx_price_list_items_list;

DROP INDEX IF EXISTS idx_price_lists_default;
DROP INDEX IF EXISTS idx_price_lists_active;
DROP INDEX IF EXISTS idx_price_lists_org_bot;

DROP INDEX IF EXISTS idx_product_categories_slug;
DROP INDEX IF EXISTS idx_product_categories_parent;
DROP INDEX IF EXISTS idx_product_categories_org_bot;

DROP INDEX IF EXISTS idx_services_active;
DROP INDEX IF EXISTS idx_services_category;
DROP INDEX IF EXISTS idx_services_org_bot;

DROP INDEX IF EXISTS idx_products_org_sku;
DROP INDEX IF EXISTS idx_products_sku;
DROP INDEX IF EXISTS idx_products_active;
DROP INDEX IF EXISTS idx_products_category;
DROP INDEX IF EXISTS idx_products_org_bot;

DROP TABLE IF EXISTS product_variants;
DROP TABLE IF EXISTS inventory_movements;
DROP TABLE IF EXISTS price_list_items;
DROP TABLE IF EXISTS price_lists;
DROP TABLE IF EXISTS product_categories;
DROP TABLE IF EXISTS services;
DROP TABLE IF EXISTS products;
