-- General Ledger
DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS gl_accounts (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        code VARCHAR(20) NOT NULL,
        name VARCHAR(200) NOT NULL,
        account_type VARCHAR(20) NOT NULL,
        parent_id UUID REFERENCES gl_accounts(id),
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_gl_accounts_bot ON gl_accounts(bot_id);
    CREATE INDEX IF NOT EXISTS idx_gl_accounts_type ON gl_accounts(account_type);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating gl_accounts table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS gl_journal_entries (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        entry_date DATE NOT NULL,
        description TEXT NOT NULL,
        reference_type VARCHAR(50),
        reference_id UUID,
        status VARCHAR(20) NOT NULL DEFAULT 'draft',
        created_by UUID,
        posted_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_gl_entries_bot ON gl_journal_entries(bot_id);
    CREATE INDEX IF NOT EXISTS idx_gl_entries_status ON gl_journal_entries(status);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating gl_journal_entries table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS gl_journal_lines (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        entry_id UUID NOT NULL REFERENCES gl_journal_entries(id),
        account_id UUID NOT NULL REFERENCES gl_accounts(id),
        debit DECIMAL(15,2) NOT NULL DEFAULT 0,
        credit DECIMAL(15,2) NOT NULL DEFAULT 0,
        description TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_gl_lines_entry ON gl_journal_lines(entry_id);
    CREATE INDEX IF NOT EXISTS idx_gl_lines_account ON gl_journal_lines(account_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating gl_journal_lines table: %', SQLERRM;
END $$;

-- Inventory Management
DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS inventory_items (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        product_id UUID,
        sku VARCHAR(50) NOT NULL,
        name VARCHAR(200) NOT NULL,
        description TEXT,
        quantity DECIMAL(15,4) NOT NULL DEFAULT 0,
        unit VARCHAR(20) NOT NULL DEFAULT 'unit',
        min_stock DECIMAL(15,4) DEFAULT 0,
        max_stock DECIMAL(15,4),
        location VARCHAR(100),
        category VARCHAR(100),
        unit_cost DECIMAL(15,2) DEFAULT 0,
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_inventory_bot ON inventory_items(bot_id);
    CREATE INDEX IF NOT EXISTS idx_inventory_sku ON inventory_items(sku);
    CREATE INDEX IF NOT EXISTS idx_inventory_category ON inventory_items(category);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating inventory_items table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS inventory_movements (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        item_id UUID NOT NULL REFERENCES inventory_items(id),
        movement_type VARCHAR(20) NOT NULL,
        quantity DECIMAL(15,4) NOT NULL,
        reference_type VARCHAR(50),
        reference_id UUID,
        notes TEXT,
        created_by UUID,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_inv_movements_item ON inventory_movements(item_id);
    CREATE INDEX IF NOT EXISTS idx_inv_movements_type ON inventory_movements(movement_type);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating inventory_movements table: %', SQLERRM;
END $$;

-- Procurement
DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS purchase_orders (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        po_number VARCHAR(50) NOT NULL,
        vendor_name VARCHAR(200) NOT NULL,
        status VARCHAR(20) NOT NULL DEFAULT 'draft',
        total_amount DECIMAL(15,2) NOT NULL DEFAULT 0,
        currency VARCHAR(3) NOT NULL DEFAULT 'BRL',
        expected_date DATE,
        notes TEXT,
        created_by UUID,
        approved_by UUID,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_po_bot ON purchase_orders(bot_id);
    CREATE INDEX IF NOT EXISTS idx_po_status ON purchase_orders(status);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating purchase_orders table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS purchase_order_items (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        po_id UUID NOT NULL REFERENCES purchase_orders(id),
        item_id UUID REFERENCES inventory_items(id),
        description TEXT NOT NULL,
        quantity DECIMAL(15,4) NOT NULL,
        unit_price DECIMAL(15,2) NOT NULL,
        total_price DECIMAL(15,2) NOT NULL,
        received_quantity DECIMAL(15,4) DEFAULT 0,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_po_items_po ON purchase_order_items(po_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating purchase_order_items table: %', SQLERRM;
END $$;
