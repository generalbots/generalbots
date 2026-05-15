' ERP Database Tables Definition
' This file defines all ERP tables using the TABLE keyword
' Tables cover inventory, purchasing, manufacturing, finance, and HR modules

' === INVENTORY MANAGEMENT ===

' Items/Products master table
TABLE items
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    item_code VARCHAR(50) UNIQUE NOT NULL
    barcode VARCHAR(50) UNIQUE
    name VARCHAR(255) NOT NULL
    description TEXT
    category VARCHAR(100)
    subcategory VARCHAR(100)
    unit_of_measure VARCHAR(20) DEFAULT 'EACH'
    weight DECIMAL(10,3)
    dimensions_length DECIMAL(10,2)
    dimensions_width DECIMAL(10,2)
    dimensions_height DECIMAL(10,2)
    minimum_stock_level INTEGER DEFAULT 0
    reorder_point INTEGER
    reorder_quantity INTEGER
    lead_time_days INTEGER DEFAULT 0
    is_active BOOLEAN DEFAULT TRUE
    is_purchasable BOOLEAN DEFAULT TRUE
    is_saleable BOOLEAN DEFAULT TRUE
    is_manufactured BOOLEAN DEFAULT FALSE
    standard_cost DECIMAL(15,4)
    last_cost DECIMAL(15,4)
    average_cost DECIMAL(15,4)
    selling_price DECIMAL(15,4)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Warehouses table
TABLE warehouses
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    code VARCHAR(20) UNIQUE NOT NULL
    name VARCHAR(100) NOT NULL
    type VARCHAR(50) DEFAULT 'standard'
    address TEXT
    city VARCHAR(100)
    state VARCHAR(50)
    country VARCHAR(50)
    postal_code VARCHAR(20)
    contact_person VARCHAR(100)
    contact_phone VARCHAR(50)
    contact_email VARCHAR(100)
    capacity_units INTEGER
    current_occupancy INTEGER DEFAULT 0
    is_active BOOLEAN DEFAULT TRUE
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Inventory stock levels
TABLE inventory_stock
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    item_id UUID REFERENCES items(id)
    warehouse_id UUID REFERENCES warehouses(id)
    location_code VARCHAR(50)
    quantity_on_hand DECIMAL(15,3) DEFAULT 0
    quantity_reserved DECIMAL(15,3) DEFAULT 0
    quantity_available DECIMAL(15,3) GENERATED ALWAYS AS (quantity_on_hand - quantity_reserved) STORED
    quantity_on_order DECIMAL(15,3) DEFAULT 0
    last_counted_date DATE
    last_movement_date TIMESTAMP
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
    UNIQUE(item_id, warehouse_id, location_code)
END TABLE

' Inventory transactions
TABLE inventory_transactions
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    transaction_type VARCHAR(50) NOT NULL
    transaction_number VARCHAR(50) UNIQUE
    item_id UUID REFERENCES items(id)
    warehouse_id UUID REFERENCES warehouses(id)
    location_code VARCHAR(50)
    quantity DECIMAL(15,3) NOT NULL
    unit_cost DECIMAL(15,4)
    total_cost DECIMAL(15,2)
    reference_type VARCHAR(50)
    reference_id UUID
    notes TEXT
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' === PURCHASING MODULE ===

' Vendors/Suppliers table
TABLE vendors
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    vendor_code VARCHAR(50) UNIQUE NOT NULL
    name VARCHAR(255) NOT NULL
    legal_name VARCHAR(255)
    tax_id VARCHAR(50)
    vendor_type VARCHAR(50)
    status VARCHAR(20) DEFAULT 'active'
    rating INTEGER CHECK (rating >= 1 AND rating <= 5)
    payment_terms VARCHAR(50)
    credit_limit DECIMAL(15,2)
    currency_code VARCHAR(3) DEFAULT 'USD'
    address TEXT
    city VARCHAR(100)
    state VARCHAR(50)
    country VARCHAR(50)
    postal_code VARCHAR(20)
    phone VARCHAR(50)
    email VARCHAR(100)
    website VARCHAR(255)
    contact_person VARCHAR(100)
    bank_account_number VARCHAR(50)
    bank_name VARCHAR(100)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Purchase orders
TABLE purchase_orders
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    po_number VARCHAR(50) UNIQUE NOT NULL
    vendor_id UUID REFERENCES vendors(id)
    order_date DATE NOT NULL
    expected_date DATE
    status VARCHAR(50) DEFAULT 'draft'
    buyer_id VARCHAR(100)
    ship_to_warehouse_id UUID REFERENCES warehouses(id)
    shipping_method VARCHAR(50)
    payment_terms VARCHAR(50)
    currency_code VARCHAR(3) DEFAULT 'USD'
    exchange_rate DECIMAL(10,6) DEFAULT 1.0
    subtotal DECIMAL(15,2)
    tax_amount DECIMAL(15,2)
    shipping_cost DECIMAL(15,2)
    total_amount DECIMAL(15,2)
    notes TEXT
    approved_by VARCHAR(100)
    approved_date TIMESTAMP
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Purchase order lines
TABLE purchase_order_lines
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    po_id UUID REFERENCES purchase_orders(id) ON DELETE CASCADE
    line_number INTEGER NOT NULL
    item_id UUID REFERENCES items(id)
    description TEXT
    quantity_ordered DECIMAL(15,3) NOT NULL
    quantity_received DECIMAL(15,3) DEFAULT 0
    quantity_remaining DECIMAL(15,3) GENERATED ALWAYS AS (quantity_ordered - quantity_received) STORED
    unit_price DECIMAL(15,4) NOT NULL
    discount_percent DECIMAL(5,2) DEFAULT 0
    tax_rate DECIMAL(5,2) DEFAULT 0
    line_total DECIMAL(15,2) GENERATED ALWAYS AS (quantity_ordered * unit_price * (1 - discount_percent/100)) STORED
    expected_date DATE
    created_at TIMESTAMP DEFAULT NOW()
    UNIQUE(po_id, line_number)
END TABLE

' === SALES MODULE ===

' Customers table
TABLE customers
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    customer_code VARCHAR(50) UNIQUE NOT NULL
    name VARCHAR(255) NOT NULL
    legal_name VARCHAR(255)
    tax_id VARCHAR(50)
    customer_type VARCHAR(50)
    status VARCHAR(20) DEFAULT 'active'
    credit_rating VARCHAR(10)
    credit_limit DECIMAL(15,2)
    payment_terms VARCHAR(50)
    discount_percent DECIMAL(5,2) DEFAULT 0
    currency_code VARCHAR(3) DEFAULT 'USD'
    billing_address TEXT
    shipping_address TEXT
    city VARCHAR(100)
    state VARCHAR(50)
    country VARCHAR(50)
    postal_code VARCHAR(20)
    phone VARCHAR(50)
    email VARCHAR(100)
    website VARCHAR(255)
    sales_person_id VARCHAR(100)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Sales orders
TABLE sales_orders
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    order_number VARCHAR(50) UNIQUE NOT NULL
    customer_id UUID REFERENCES customers(id)
    order_date DATE NOT NULL
    required_date DATE
    promised_date DATE
    status VARCHAR(50) DEFAULT 'draft'
    sales_person_id VARCHAR(100)
    ship_from_warehouse_id UUID REFERENCES warehouses(id)
    shipping_method VARCHAR(50)
    payment_terms VARCHAR(50)
    payment_method VARCHAR(50)
    currency_code VARCHAR(3) DEFAULT 'USD'
    exchange_rate DECIMAL(10,6) DEFAULT 1.0
    subtotal DECIMAL(15,2)
    discount_amount DECIMAL(15,2) DEFAULT 0
    tax_amount DECIMAL(15,2)
    shipping_cost DECIMAL(15,2)
    total_amount DECIMAL(15,2)
    notes TEXT
    approved_by VARCHAR(100)
    approved_date TIMESTAMP
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Sales order lines
TABLE sales_order_lines
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    order_id UUID REFERENCES sales_orders(id) ON DELETE CASCADE
    line_number INTEGER NOT NULL
    item_id UUID REFERENCES items(id)
    description TEXT
    quantity_ordered DECIMAL(15,3) NOT NULL
    quantity_shipped DECIMAL(15,3) DEFAULT 0
    quantity_invoiced DECIMAL(15,3) DEFAULT 0
    unit_price DECIMAL(15,4) NOT NULL
    discount_percent DECIMAL(5,2) DEFAULT 0
    tax_rate DECIMAL(5,2) DEFAULT 0
    line_total DECIMAL(15,2) GENERATED ALWAYS AS (quantity_ordered * unit_price * (1 - discount_percent/100)) STORED
    cost_of_goods_sold DECIMAL(15,2)
    margin DECIMAL(15,2) GENERATED ALWAYS AS (line_total - cost_of_goods_sold) STORED
    warehouse_id UUID REFERENCES warehouses(id)
    promised_date DATE
    created_at TIMESTAMP DEFAULT NOW()
    UNIQUE(order_id, line_number)
END TABLE

' === MANUFACTURING MODULE ===

' Bill of Materials (BOM) header
TABLE bill_of_materials
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    bom_number VARCHAR(50) UNIQUE NOT NULL
    item_id UUID REFERENCES items(id)
    revision VARCHAR(20) DEFAULT 'A'
    description TEXT
    quantity_per_assembly DECIMAL(15,3) DEFAULT 1
    unit_of_measure VARCHAR(20)
    status VARCHAR(20) DEFAULT 'active'
    effective_date DATE
    expiration_date DATE
    total_cost DECIMAL(15,4)
    labor_cost DECIMAL(15,4)
    overhead_cost DECIMAL(15,4)
    created_by VARCHAR(100)
    approved_by VARCHAR(100)
    approved_date DATE
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' BOM components
TABLE bom_components
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    bom_id UUID REFERENCES bill_of_materials(id) ON DELETE CASCADE
    component_item_id UUID REFERENCES items(id)
    line_number INTEGER NOT NULL
    quantity_required DECIMAL(15,6) NOT NULL
    unit_of_measure VARCHAR(20)
    scrap_percent DECIMAL(5,2) DEFAULT 0
    total_quantity DECIMAL(15,6) GENERATED ALWAYS AS (quantity_required * (1 + scrap_percent/100)) STORED
    cost_per_unit DECIMAL(15,4)
    total_cost DECIMAL(15,4) GENERATED ALWAYS AS (total_quantity * cost_per_unit) STORED
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    UNIQUE(bom_id, line_number)
END TABLE

' Work orders
TABLE work_orders
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    wo_number VARCHAR(50) UNIQUE NOT NULL
    item_id UUID REFERENCES items(id)
    bom_id UUID REFERENCES bill_of_materials(id)
    quantity_to_produce DECIMAL(15,3) NOT NULL
    quantity_completed DECIMAL(15,3) DEFAULT 0
    quantity_scrapped DECIMAL(15,3) DEFAULT 0
    status VARCHAR(50) DEFAULT 'planned'
    priority VARCHAR(20) DEFAULT 'normal'
    planned_start_date TIMESTAMP
    planned_end_date TIMESTAMP
    actual_start_date TIMESTAMP
    actual_end_date TIMESTAMP
    warehouse_id UUID REFERENCES warehouses(id)
    work_center VARCHAR(50)
    labor_hours_estimated DECIMAL(10,2)
    labor_hours_actual DECIMAL(10,2)
    notes TEXT
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' === FINANCIAL MODULE ===

' General ledger accounts
TABLE gl_accounts
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    account_number VARCHAR(20) UNIQUE NOT NULL
    account_name VARCHAR(100) NOT NULL
    account_type VARCHAR(50) NOT NULL
    parent_account_id UUID REFERENCES gl_accounts(id)
    currency_code VARCHAR(3) DEFAULT 'USD'
    normal_balance VARCHAR(10) CHECK (normal_balance IN ('debit', 'credit'))
    is_active BOOLEAN DEFAULT TRUE
    is_control_account BOOLEAN DEFAULT FALSE
    allow_manual_entry BOOLEAN DEFAULT TRUE
    description TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Journal entries header
TABLE journal_entries
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    journal_number VARCHAR(50) UNIQUE NOT NULL
    journal_date DATE NOT NULL
    posting_date DATE NOT NULL
    period VARCHAR(20)
    journal_type VARCHAR(50)
    description TEXT
    reference_type VARCHAR(50)
    reference_number VARCHAR(50)
    status VARCHAR(20) DEFAULT 'draft'
    total_debit DECIMAL(15,2)
    total_credit DECIMAL(15,2)
    is_balanced BOOLEAN GENERATED ALWAYS AS (total_debit = total_credit) STORED
    posted_by VARCHAR(100)
    posted_at TIMESTAMP
    reversed_by_id UUID REFERENCES journal_entries(id)
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Journal entry lines
TABLE journal_entry_lines
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    journal_entry_id UUID REFERENCES journal_entries(id) ON DELETE CASCADE
    line_number INTEGER NOT NULL
    account_id UUID REFERENCES gl_accounts(id)
    debit_amount DECIMAL(15,2) DEFAULT 0
    credit_amount DECIMAL(15,2) DEFAULT 0
    description TEXT
    dimension1 VARCHAR(50)
    dimension2 VARCHAR(50)
    dimension3 VARCHAR(50)
    created_at TIMESTAMP DEFAULT NOW()
    UNIQUE(journal_entry_id, line_number)
    CHECK (debit_amount = 0 OR credit_amount = 0)
END TABLE

' Invoices
TABLE invoices
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    invoice_number VARCHAR(50) UNIQUE NOT NULL
    invoice_type VARCHAR(20) DEFAULT 'standard'
    customer_id UUID REFERENCES customers(id)
    vendor_id UUID REFERENCES vendors(id)
    order_id UUID
    invoice_date DATE NOT NULL
    due_date DATE NOT NULL
    status VARCHAR(20) DEFAULT 'draft'
    currency_code VARCHAR(3) DEFAULT 'USD'
    exchange_rate DECIMAL(10,6) DEFAULT 1.0
    subtotal DECIMAL(15,2)
    discount_amount DECIMAL(15,2) DEFAULT 0
    tax_amount DECIMAL(15,2)
    total_amount DECIMAL(15,2)
    amount_paid DECIMAL(15,2) DEFAULT 0
    balance_due DECIMAL(15,2) GENERATED ALWAYS AS (total_amount - amount_paid) STORED
    payment_terms VARCHAR(50)
    notes TEXT
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' === HUMAN RESOURCES MODULE ===

' Employees table
TABLE employees
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    employee_number VARCHAR(50) UNIQUE NOT NULL
    first_name VARCHAR(100) NOT NULL
    last_name VARCHAR(100) NOT NULL
    middle_name VARCHAR(100)
    full_name VARCHAR(255) GENERATED ALWAYS AS (first_name || ' ' || COALESCE(middle_name || ' ', '') || last_name) STORED
    email VARCHAR(100) UNIQUE
    phone VARCHAR(50)
    mobile VARCHAR(50)
    address TEXT
    city VARCHAR(100)
    state VARCHAR(50)
    country VARCHAR(50)
    postal_code VARCHAR(20)
    date_of_birth DATE
    gender VARCHAR(20)
    marital_status VARCHAR(20)
    national_id VARCHAR(50)
    passport_number VARCHAR(50)
    department_id UUID
    position_title VARCHAR(100)
    manager_id UUID REFERENCES employees(id)
    hire_date DATE NOT NULL
    employment_status VARCHAR(50) DEFAULT 'active'
    employment_type VARCHAR(50) DEFAULT 'full-time'
    salary DECIMAL(15,2)
    hourly_rate DECIMAL(10,2)
    commission_percent DECIMAL(5,2)
    bank_account_number VARCHAR(50)
    bank_name VARCHAR(100)
    emergency_contact_name VARCHAR(100)
    emergency_contact_phone VARCHAR(50)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Payroll records
TABLE payroll
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    payroll_number VARCHAR(50) UNIQUE NOT NULL
    employee_id UUID REFERENCES employees(id)
    pay_period_start DATE NOT NULL
    pay_period_end DATE NOT NULL
    payment_date DATE NOT NULL
    hours_worked DECIMAL(10,2)
    overtime_hours DECIMAL(10,2)
    regular_pay DECIMAL(15,2)
    overtime_pay DECIMAL(15,2)
    commission DECIMAL(15,2)
    bonus DECIMAL(15,2)
    gross_pay DECIMAL(15,2)
    tax_deductions DECIMAL(15,2)
    other_deductions DECIMAL(15,2)
    net_pay DECIMAL(15,2)
    payment_method VARCHAR(50)
    payment_reference VARCHAR(100)
    status VARCHAR(20) DEFAULT 'pending'
    approved_by VARCHAR(100)
    approved_date TIMESTAMP
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' === SYSTEM TABLES ===

' Audit trail
TABLE erp_audit_log
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    table_name VARCHAR(50) NOT NULL
    record_id UUID NOT NULL
    action VARCHAR(20) NOT NULL
    changed_fields JSONB
    old_values JSONB
    new_values JSONB
    user_id VARCHAR(100)
    user_ip VARCHAR(45)
    user_agent TEXT
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' System settings
TABLE erp_settings
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    module VARCHAR(50) NOT NULL
    setting_key VARCHAR(100) NOT NULL
    setting_value TEXT
    data_type VARCHAR(20)
    description TEXT
    is_encrypted BOOLEAN DEFAULT FALSE
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
    UNIQUE(module, setting_key)
END TABLE

' Create indexes for performance
CREATE INDEX idx_inventory_item_warehouse ON inventory_stock(item_id, warehouse_id)
CREATE INDEX idx_po_vendor ON purchase_orders(vendor_id)
CREATE INDEX idx_po_status ON purchase_orders(status)
CREATE INDEX idx_so_customer ON sales_orders(customer_id)
CREATE INDEX idx_so_status ON sales_orders(status)
CREATE INDEX idx_wo_status ON work_orders(status)
CREATE INDEX idx_invoice_customer ON invoices(customer_id)
CREATE INDEX idx_invoice_status ON invoices(status)
CREATE INDEX idx_employee_manager ON employees(manager_id)
CREATE INDEX idx_journal_date ON journal_entries(journal_date)
CREATE INDEX idx_audit_table_record ON erp_audit_log(table_name, record_id)
