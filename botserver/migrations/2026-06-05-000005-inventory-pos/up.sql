CREATE TABLE product_variations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    product_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    attributes JSONB NOT NULL DEFAULT '{}',
    price NUMERIC(12,2) NOT NULL,
    cost_price NUMERIC(12,2),
    barcode VARCHAR(100),
    weight NUMERIC(10,3),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_variations_bot ON product_variations(bot_id);
CREATE INDEX idx_variations_product ON product_variations(product_id);
CREATE INDEX idx_variations_sku ON product_variations(sku);

CREATE TABLE product_stock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    product_id UUID NOT NULL,
    variation_id UUID,
    branch_id UUID NOT NULL,
    quantity NUMERIC(12,3) NOT NULL DEFAULT 0,
    reserved NUMERIC(12,3) NOT NULL DEFAULT 0,
    reorder_point NUMERIC(12,3),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stock_bot ON product_stock(bot_id);
CREATE INDEX idx_stock_product ON product_stock(product_id);
CREATE INDEX idx_stock_branch ON product_stock(branch_id);

CREATE TABLE product_price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'BRL',
    is_default BOOLEAN NOT NULL DEFAULT false,
    valid_from DATE,
    valid_until DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_price_lists_bot ON product_price_lists(bot_id);

CREATE TABLE product_prices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    price_list_id UUID NOT NULL REFERENCES product_price_lists(id),
    product_id UUID NOT NULL,
    variation_id UUID,
    price NUMERIC(12,2) NOT NULL,
    min_quantity NUMERIC(12,3) NOT NULL DEFAULT 1
);

CREATE INDEX idx_prices_list ON product_prices(price_list_id);
CREATE INDEX idx_prices_product ON product_prices(product_id);

CREATE TABLE product_promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    discount_type VARCHAR(20) NOT NULL,
    discount_value NUMERIC(12,2) NOT NULL,
    product_ids JSONB,
    min_purchase NUMERIC(12,2),
    valid_from DATE NOT NULL,
    valid_until DATE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_promotions_bot ON product_promotions(bot_id);
CREATE INDEX idx_promotions_active ON product_promotions(is_active);
CREATE INDEX idx_promotions_dates ON product_promotions(valid_from, valid_until);

CREATE TABLE pos_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    operator_id UUID NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    opening_amount NUMERIC(12,2) NOT NULL DEFAULT 0,
    closing_amount NUMERIC(12,2),
    status VARCHAR(20) NOT NULL DEFAULT 'open'
);

CREATE INDEX idx_pos_sessions_bot ON pos_sessions(bot_id);
CREATE INDEX idx_pos_sessions_status ON pos_sessions(status);

CREATE TABLE pos_sales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL REFERENCES pos_sessions(id),
    sale_number INTEGER NOT NULL,
    items JSONB NOT NULL,
    subtotal NUMERIC(12,2) NOT NULL,
    discount NUMERIC(12,2),
    tax NUMERIC(12,2),
    total NUMERIC(12,2) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    nfse_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pos_sales_bot ON pos_sales(bot_id);
CREATE INDEX idx_pos_sales_session ON pos_sales(session_id);
CREATE INDEX idx_pos_sales_created ON pos_sales(created_at);
