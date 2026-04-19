' ERP Database Tables Definition
' This file defines all ERP tables using the TABLE keyword
' Tables cover inventory, purchasing, manufacturing, finance, and HR modules

' === INVENTORY MANAGEMENT ===

' Items/Products master table
TABLE items
    id UUID KEY
    item_code STRING(50)
    barcode STRING(50)
    name STRING(255)
    description TEXT
    category STRING(100)
    subcategory STRING(100)
    unit_of_measure STRING(20)
    weight NUMBER(10,3)
    dimensions_length NUMBER(10,2)
    dimensions_width NUMBER(10,2)
    dimensions_height NUMBER(10,2)
    minimum_stock_level INTEGER
    reorder_point INTEGER
    reorder_quantity INTEGER
    lead_time_days INTEGER
    is_active BOOLEAN
    is_purchasable BOOLEAN
    is_saleable BOOLEAN
    is_manufactured BOOLEAN
    standard_cost NUMBER(15,4)
    last_cost NUMBER(15,4)
    average_cost NUMBER(15,4)
    selling_price NUMBER(15,4)
    created_at TIMESTAMP
    updated_at TIMESTAMP
END TABLE

' Warehouses table
TABLE warehouses
    id UUID KEY
    code STRING(20)
    name STRING(100)
    type STRING(50)
    address TEXT
    city STRING(100)
    state STRING(50)
    country STRING(50)
    postal_code STRING(20)
    contact_person STRING(100)
    contact_phone STRING(50)
    contact_email STRING(100)
    capacity_units INTEGER
    current_occupancy INTEGER
    is_active BOOLEAN DEFAULT TRUE
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Inventory stock levels
TABLE inventory_stock
    id UUID KEY
    item_id UUID
    warehouse_id UUID
    location_code STRING(50)
    quantity_on_hand NUMBER(15,3)
    quantity_reserved NUMBER(15,3)
    quantity_available NUMBER(15,3)
    quantity_on_order NUMBER(15,3)
    last_counted_date DATE
    last_movement_date TIMESTAMP
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
   (item_id, warehouse_id, location_code)
END TABLE

' Inventory transactions
TABLE inventory_transactions
    id UUID KEY
    transaction_type STRING(50)
    transaction_number STRING(50)
    item_id UUID
    warehouse_id UUID
    location_code STRING(50)
    quantity NUMBER(15,3)
    unit_cost NUMBER(15,4)
    total_cost NUMBER(15,2)
    reference_type STRING(50)
    reference_id UUID
    notes TEXT
    created_by STRING(100)
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' === PURCHASING MODULE ===

' Vendors/Suppliers table
TABLE vendors
    id UUID KEY
    vendor_code STRING(50)
    name STRING(255)
    legal_name STRING(255)
    tax_id STRING(50)
    vendor_type STRING(50)
    status STRING(20)
    rating INTEGER
    payment_terms STRING(50)
    credit_limit NUMBER(15,2)
    currency_code STRING(3)
    address TEXT
    city STRING(100)
    state STRING(50)
    country STRING(50)
    postal_code STRING(20)
    phone STRING(50)
    email STRING(100)
    website STRING(255)
    contact_person STRING(100)
    bank_account_number STRING(50)
    bank_name STRING(100)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Purchase orders
TABLE purchase_orders
    id UUID KEY
    po_number STRING(50)
    vendor_id UUID
    order_date DATE
    expected_date DATE
    status STRING(50)
    buyer_id STRING(100)
    ship_to_warehouse_id UUID
    shipping_method STRING(50)
    payment_terms STRING(50)
    currency_code STRING(3)
    exchange_rate NUMBER(10,6)
    subtotal NUMBER(15,2)
    tax_amount NUMBER(15,2)
    shipping_cost NUMBER(15,2)
    total_amount NUMBER(15,2)
    notes TEXT
    approved_by STRING(100)
    approved_date TIMESTAMP
    created_by STRING(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Purchase order lines
TABLE purchase_order_lines
    id UUID KEY
    po_id UUID
    line_number INTEGER
    item_id UUID
    description TEXT
    quantity_ordered NUMBER(15,3)
    quantity_received NUMBER(15,3)
    quantity_remaining NUMBER(15,3)
    unit_price NUMBER(15,4)
    discount_percent NUMBER(5,2)
    tax_rate NUMBER(5,2)
    line_total NUMBER(15,2)
    expected_date DATE
    created_at TIMESTAMP DEFAULT NOW()
   (po_id, line_number)
END TABLE

' === SALES MODULE ===

' Customers table
TABLE customers
    id UUID KEY
    customer_code STRING(50)
    name STRING(255)
    legal_name STRING(255)
    tax_id STRING(50)
    customer_type STRING(50)
    status STRING(20)
    credit_rating STRING(10)
    credit_limit NUMBER(15,2)
    payment_terms STRING(50)
    discount_percent NUMBER(5,2)
    currency_code STRING(3)
    billing_address TEXT
    shipping_address TEXT
    city STRING(100)
    state STRING(50)
    country STRING(50)
    postal_code STRING(20)
    phone STRING(50)
    email STRING(100)
    website STRING(255)
    sales_person_id STRING(100)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Sales orders
TABLE sales_orders
    id UUID KEY
    order_number STRING(50)
    customer_id UUID
    order_date DATE
    required_date DATE
    promised_date DATE
    status STRING(50)
    sales_person_id STRING(100)
    ship_from_warehouse_id UUID
    shipping_method STRING(50)
    payment_terms STRING(50)
    payment_method STRING(50)
    currency_code STRING(3)
    exchange_rate NUMBER(10,6)
    subtotal NUMBER(15,2)
    discount_amount NUMBER(15,2)
    tax_amount NUMBER(15,2)
    shipping_cost NUMBER(15,2)
    total_amount NUMBER(15,2)
    notes TEXT
    approved_by STRING(100)
    approved_date TIMESTAMP
    created_by STRING(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Sales order lines
TABLE sales_order_lines
    id UUID KEY
    order_id UUID
    line_number INTEGER
    item_id UUID
    description TEXT
    quantity_ordered NUMBER(15,3)
    quantity_shipped NUMBER(15,3)
    quantity_invoiced NUMBER(15,3)
    unit_price NUMBER(15,4)
    discount_percent NUMBER(5,2)
    tax_rate NUMBER(5,2)
    line_total NUMBER(15,2)
    cost_of_goods_sold NUMBER(15,2)
    margin NUMBER(15,2)
    warehouse_id UUID
    promised_date DATE
    created_at TIMESTAMP DEFAULT NOW()
   (order_id, line_number)
END TABLE

' === MANUFACTURING MODULE ===

' Bill of Materials (BOM) header
TABLE bill_of_materials
    id UUID KEY
    bom_number STRING(50)
    item_id UUID
    revision STRING(20)
    description TEXT
    quantity_per_assembly NUMBER(15,3)
    unit_of_measure STRING(20)
    status STRING(20)
    effective_date DATE
    expiration_date DATE
    total_cost NUMBER(15,4)
    labor_cost NUMBER(15,4)
    overhead_cost NUMBER(15,4)
    created_by STRING(100)
    approved_by STRING(100)
    approved_date DATE
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' BOM components
TABLE bom_components
    id UUID KEY
    bom_id UUID
    component_item_id UUID
    line_number INTEGER
    quantity_required NUMBER(15,6)
    unit_of_measure STRING(20)
    scrap_percent NUMBER(5,2)
    total_quantity NUMBER(15,6)
    cost_per_unit NUMBER(15,4)
    total_cost NUMBER(15,4)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
   (bom_id, line_number)
END TABLE

' Work orders
TABLE work_orders
    id UUID KEY
    wo_number STRING(50)
    item_id UUID
    bom_id UUID
    quantity_to_produce NUMBER(15,3)
    quantity_completed NUMBER(15,3)
    quantity_scrapped NUMBER(15,3)
    status STRING(50)
    priority STRING(20)
    planned_start_date TIMESTAMP
    planned_end_date TIMESTAMP
    actual_start_date TIMESTAMP
    actual_end_date TIMESTAMP
    warehouse_id UUID
    work_center STRING(50)
    labor_hours_estimated NUMBER(10,2)
    labor_hours_actual NUMBER(10,2)
    notes TEXT
    created_by STRING(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' === FINANCIAL MODULE ===

' General ledger accounts
TABLE gl_accounts
    id UUID KEY
    account_number STRING(20)
    account_name STRING(100)
    account_type STRING(50)
    parent_account_id UUID
    currency_code STRING(3)
    normal_balance STRING(10)
    is_active BOOLEAN DEFAULT TRUE
    is_control_account BOOLEAN DEFAULT FALSE
    allow_manual_entry BOOLEAN DEFAULT TRUE
    description TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Journal entries header
TABLE journal_entries
    id UUID KEY
    journal_number STRING(50)
    journal_date DATE
    posting_date DATE
    period STRING(20)
    journal_type STRING(50)
    description TEXT
    reference_type STRING(50)
    reference_number STRING(50)
    status STRING(20)
    total_debit NUMBER(15,2)
    total_credit NUMBER(15,2)
    is_balanced BOOLEAN
    posted_by STRING(100)
    posted_at TIMESTAMP
    reversed_by_id UUID
    created_by STRING(100)
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Journal entry lines
TABLE journal_entry_lines
    id UUID KEY
    journal_entry_id UUID
    line_number INTEGER
    account_id UUID
    debit_amount NUMBER(15,2)
    credit_amount NUMBER(15,2)
    description TEXT
    dimension1 STRING(50)
    dimension2 STRING(50)
    dimension3 STRING(50)
    created_at TIMESTAMP DEFAULT NOW()
   (journal_entry_id, line_number)
   
END TABLE

' Invoices
TABLE invoices
    id UUID KEY
    invoice_number STRING(50)
    invoice_type STRING(20)
    customer_id UUID
    vendor_id UUID
    order_id UUID
    invoice_date DATE
    due_date DATE
    status STRING(20)
    currency_code STRING(3)
    exchange_rate NUMBER(10,6)
    subtotal NUMBER(15,2)
    discount_amount NUMBER(15,2)
    tax_amount NUMBER(15,2)
    total_amount NUMBER(15,2)
    amount_paid NUMBER(15,2)
    balance_due NUMBER(15,2)
    payment_terms STRING(50)
    notes TEXT
    created_by STRING(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' === HUMAN RESOURCES MODULE ===

' Employees table
TABLE employees
    id UUID KEY
    employee_number STRING(50)
    first_name STRING(100)
    last_name STRING(100)
    middle_name STRING(100)
    full_name STRING(255)
    email STRING(100)
    phone STRING(50)
    mobile STRING(50)
    address TEXT
    city STRING(100)
    state STRING(50)
    country STRING(50)
    postal_code STRING(20)
    date_of_birth DATE
    gender STRING(20)
    marital_status STRING(20)
    national_id STRING(50)
    passport_number STRING(50)
    department_id UUID
    position_title STRING(100)
    manager_id UUID
    hire_date DATE
    employment_status STRING(50)
    employment_type STRING(50)'full-time'
    salary NUMBER(15,2)
    hourly_rate NUMBER(10,2)
    commission_percent NUMBER(5,2)
    bank_account_number STRING(50)
    bank_name STRING(100)
    emergency_contact_name STRING(100)
    emergency_contact_phone STRING(50)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Payroll records
TABLE payroll
    id UUID KEY
    payroll_number STRING(50)
    employee_id UUID
    pay_period_start DATE
    pay_period_end DATE
    payment_date DATE
    hours_worked NUMBER(10,2)
    overtime_hours NUMBER(10,2)
    regular_pay NUMBER(15,2)
    overtime_pay NUMBER(15,2)
    commission NUMBER(15,2)
    bonus NUMBER(15,2)
    gross_pay NUMBER(15,2)
    tax_deductions NUMBER(15,2)
    other_deductions NUMBER(15,2)
    net_pay NUMBER(15,2)
    payment_method STRING(50)
    payment_reference STRING(100)
    status STRING(20)
    approved_by STRING(100)
    approved_date TIMESTAMP
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' === SYSTEM TABLES ===

' Audit trail
TABLE erp_audit_log
    id UUID KEY
    table_name STRING(50)
    record_id UUID
    action STRING(20)
    changed_fieldsTEXT
    old_valuesTEXT
    new_valuesTEXT
    user_id STRING(100)
    user_ip STRING(45)
    user_agent TEXT
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' System settings
TABLE erp_settings
    id UUID KEY
    module STRING(50)
    setting_key STRING(100)
    setting_value TEXT
    data_type STRING(20)
    description TEXT
    is_encrypted BOOLEAN DEFAULT FALSE
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
   (module, setting_key)
END TABLE

' Create indexes for performance
'CREATE INDEX idx_inventory_item_warehouse ON inventory_stock(item_id, warehouse_id)
'CREATE INDEX idx_po_vendor ON purchase_orders(vendor_id)
'CREATE INDEX idx_po_status ON purchase_orders(status)
'CREATE INDEX idx_so_customer ON sales_orders(customer_id)
'CREATE INDEX idx_so_status ON sales_orders(status)
'CREATE INDEX idx_wo_status ON work_orders(status)
'CREATE INDEX idx_invoice_customer ON invoices(customer_id)
'CREATE INDEX idx_invoice_status ON invoices(status)
'CREATE INDEX idx_employee_manager ON employees(manager_id)
'CREATE INDEX idx_journal_date ON journal_entries(journal_date)
'CREATE INDEX idx_audit_table_record ON erp_audit_log(table_name, record_id)
