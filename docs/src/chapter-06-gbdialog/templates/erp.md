# ERP Template

The ERP (Enterprise Resource Planning) template provides comprehensive inventory management, purchasing, and warehouse operations through a conversational AI interface.

## Topic: Enterprise Resource Planning & Inventory

This template is perfect for:
- Warehouse management
- Inventory tracking
- Purchase order processing
- Stock transfers
- Cycle counting and audits

## The Code

```basic
ADD TOOL "inventory-management"
ADD TOOL "purchasing"
ADD TOOL "erp-jobs"

SET CONTEXT "erp" AS "You are an ERP assistant helping with inventory management, purchasing, and warehouse operations. Help users receive inventory, ship orders, check stock levels, transfer between warehouses, and conduct cycle counts."

CLEAR SUGGESTIONS

ADD SUGGESTION "receive" AS "Receive inventory"
ADD SUGGESTION "ship" AS "Ship order"
ADD SUGGESTION "stock" AS "Check stock"
ADD SUGGESTION "transfer" AS "Transfer stock"
ADD SUGGESTION "count" AS "Cycle count"
ADD SUGGESTION "purchase" AS "Create PO"

BEGIN TALK
**ERP Inventory Manager**

I can help you with:
• Receive inventory from purchase orders
• Ship orders to customers
• Check stock levels across warehouses
• Transfer stock between locations
• Conduct cycle counts
• Create and manage purchase orders

What would you like to do?
END TALK

BEGIN SYSTEM PROMPT
You are an ERP inventory management assistant.

Key operations:
- receive_inventory: Process incoming goods from POs
- ship_inventory: Process outgoing shipments for sales orders
- check_stock: Query inventory levels
- transfer_stock: Move inventory between warehouses
- cycle_count: Physical inventory verification

Always confirm quantities before processing.
Log all transactions for audit trail.
Alert on low stock and reorder points.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the ERP template works in real-world scenarios.

### Dialog 1: Receiving Inventory

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">ERP Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">ERP Manager</div>
      <p><strong>ERP Inventory Manager</strong></p>
      <p>I can help you with:</p>
      <p>• Receive inventory from purchase orders<br>• Ship orders to customers<br>• Check stock levels across warehouses</p>
      <p>What would you like to do?</p>
      <div class="wa-time">08:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need to receive PO-2025-0123</p>
      <div class="wa-time">08:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Purchase Order PO-2025-0123</strong></p>
      <p>Vendor: Acme Supplies</p>
      <p>Expected items:</p>
      <p></p>
      <p><strong>1. Widget A (SKU-001)</strong></p>
      <p>   Ordered: 100 units | Unit cost: $25.00</p>
      <p>   How many did you receive?</p>
      <div class="wa-time">08:01</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>100</p>
      <div class="wa-time">08:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✓ Widget A: 100 units received</p>
      <p></p>
      <p><strong>2. Widget B (SKU-002)</strong></p>
      <p>   Ordered: 50 units | Unit cost: $45.00</p>
      <p>   How many did you receive?</p>
      <div class="wa-time">08:02</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>48</p>
      <div class="wa-time">08:03 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>⚠️ Widget B: 48 of 50 received (2 short)</p>
      <p></p>
      <p>✅ <strong>Receipt Complete</strong></p>
      <p>📄 Receipt #: REC-2025-0115-4521</p>
      <p></p>
      <p><strong>Summary:</strong></p>
      <p>• Widget A: 100 ✓</p>
      <p>• Widget B: 48 (short 2)</p>
      <p></p>
      <p>📊 Stock updated in Main Warehouse</p>
      <p>📧 Buyer notified of shortage</p>
      <div class="wa-time">08:03</div>
    </div>
  </div>
</div>

### Dialog 2: Check Stock Levels

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">ERP Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Check stock for Widget A</p>
      <div class="wa-time">10:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 <strong>Stock Status: Widget A (SKU-001)</strong></p>
      <p></p>
      <p><strong>Main Warehouse:</strong> 250 on hand</p>
      <p><strong>East Distribution:</strong> 75 on hand</p>
      <p><strong>West Distribution:</strong> 120 on hand</p>
      <p></p>
      <p><strong>TOTAL:</strong> 445 on hand, 420 available</p>
      <p>(25 reserved for pending orders)</p>
      <p></p>
      <p>✅ Above minimum stock level (100)</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>
</div>

### Dialog 3: Ship an Order

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">ERP Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Ship sales order SO-2025-0456</p>
      <div class="wa-time">14:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📦 <strong>Sales Order SO-2025-0456</strong></p>
      <p>Customer: ABC Corp</p>
      <p></p>
      <p>Checking stock availability...</p>
      <p></p>
      <p>✅ Widget A: 50 units - Available</p>
      <p>✅ Widget C: 25 units - Available</p>
      <p></p>
      <p>All items available. Proceed with shipment?</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Yes</p>
      <div class="wa-time">14:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Shipment Created</strong></p>
      <p>🚚 Tracking #: SHIP-2025-0115-7832</p>
      <p></p>
      <p>• Stock deducted from Main Warehouse</p>
      <p>• Packing slip generated</p>
      <p>• Customer notified via email</p>
      <p></p>
      <p>📋 <strong>Cost of Goods Sold:</strong> $2,375.00</p>
      <div class="wa-time">14:01</div>
    </div>
  </div>
</div>

### Dialog 4: Transfer Stock

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">ERP Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Transfer 50 Widget A from Main to East warehouse</p>
      <div class="wa-time">15:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔄 <strong>Stock Transfer Request</strong></p>
      <p></p>
      <p><strong>Item:</strong> Widget A (SKU-001)</p>
      <p><strong>Quantity:</strong> 50 units</p>
      <p><strong>From:</strong> Main Warehouse (250 available)</p>
      <p><strong>To:</strong> East Distribution</p>
      <p></p>
      <p>Confirm transfer?</p>
      <div class="wa-time">15:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Confirm</p>
      <div class="wa-time">15:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Transfer Complete</strong></p>
      <p>📄 Transfer #: TRAN-2025-0115-3341</p>
      <p></p>
      <p><strong>Updated Stock:</strong></p>
      <p>• Main Warehouse: 200 units</p>
      <p>• East Distribution: 125 units</p>
      <div class="wa-time">15:31</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register ERP operation tools |
| `SET CONTEXT` | Define ERP assistant behavior |
| `FIND` | Query inventory and orders |
| `SAVE` | Record transactions |
| `UPDATE` | Modify stock levels |
| `SEND MAIL` | Notify stakeholders |

## Template Structure

```
erp.gbai/
├── erp.gbdialog/
│   ├── inventory-management.bas  # Stock operations
│   ├── purchasing.bas            # PO management
│   ├── erp-jobs.bas             # Scheduled tasks
│   └── tables.bas               # Data structures
└── erp.gbot/
    └── config.csv               # Configuration
```

## Data Tables

### Items Table
| Field | Description |
|-------|-------------|
| id | Unique item identifier |
| item_code | SKU/product code |
| name | Item description |
| category | Product category |
| unit_of_measure | UOM (each, case, etc.) |
| minimum_stock_level | Reorder threshold |
| reorder_point | When to reorder |
| reorder_quantity | How much to order |
| average_cost | Weighted average cost |
| last_cost | Most recent purchase cost |

### Inventory Stock Table
| Field | Description |
|-------|-------------|
| item_id | Reference to item |
| warehouse_id | Location |
| quantity_on_hand | Physical count |
| quantity_reserved | Allocated to orders |
| quantity_available | On hand minus reserved |
| last_movement_date | Last transaction |
| last_counted_date | Last physical count |

### Inventory Transactions Table
| Field | Description |
|-------|-------------|
| transaction_type | receipt, shipment, transfer, adjustment |
| transaction_number | Unique reference |
| item_id | Item affected |
| warehouse_id | Location |
| quantity | Amount (+/-) |
| unit_cost | Cost per unit |
| reference_type | PO, SO, Transfer |
| reference_id | Source document |

## Inventory Management Tool

```basic
PARAM action AS STRING LIKE "check_stock" DESCRIPTION "Action: receive_inventory, ship_inventory, check_stock, transfer_stock, cycle_count"
PARAM item_data AS OBJECT LIKE "{po_number: 'PO-123'}" DESCRIPTION "Data object with action-specific parameters"

DESCRIPTION "Manage inventory operations"

user_id = GET "session.user_id"
warehouse_id = GET "session.warehouse_id"

IF action = "receive_inventory" THEN
    po_number = item_data.po_number
    po = FIND "purchase_orders", "po_number = '" + po_number + "'"
    
    IF NOT po THEN
        TALK "Purchase order not found."
        RETURN NULL
    END IF
    
    po_lines = FIND "purchase_order_lines", "po_id = '" + po.id + "'"
    
    FOR EACH line IN po_lines
        item = FIND "items", "id = '" + line.item_id + "'"
        
        TALK "Receiving " + item.name + " - Ordered: " + line.quantity_ordered
        TALK "Enter quantity received:"
        HEAR qty_received AS INTEGER
        
        ' Update stock
        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"
        
        IF NOT stock THEN
            WITH newStock
                item_id = item.id
                warehouse_id = warehouse_id
                quantity_on_hand = qty_received
            END WITH
            SAVE "inventory_stock", newStock
        ELSE
            new_qty = stock.quantity_on_hand + qty_received
            UPDATE "inventory_stock" SET quantity_on_hand = new_qty WHERE id = stock.id
        END IF
        
        ' Create transaction record
        WITH transaction
            transaction_type = "receipt"
            item_id = item.id
            warehouse_id = warehouse_id
            quantity = qty_received
            unit_cost = line.unit_price
            reference_type = "purchase_order"
            reference_id = po.id
            created_at = NOW()
        END WITH
        
        SAVE "inventory_transactions", transaction
    NEXT
    
    UPDATE "purchase_orders" SET status = "received" WHERE id = po.id
    TALK "Purchase order " + po_number + " received."
END IF

IF action = "check_stock" THEN
    item_search = item_data.item_search
    items = FIND "items", "name LIKE '%" + item_search + "%'"
    
    FOR EACH item IN items
        TALK "📦 " + item.name + " (" + item.item_code + ")"
        
        stocks = FIND "inventory_stock", "item_id = '" + item.id + "'"
        total = 0
        
        FOR EACH stock IN stocks
            warehouse = FIND "warehouses", "id = '" + stock.warehouse_id + "'"
            TALK "  " + warehouse.name + ": " + stock.quantity_on_hand
            total = total + stock.quantity_on_hand
        NEXT
        
        TALK "  **TOTAL:** " + total
        
        IF total < item.minimum_stock_level THEN
            TALK "  ⚠️ Below minimum (" + item.minimum_stock_level + ")"
        END IF
    NEXT
END IF
```

## Scheduled Jobs: erp-jobs.bas

```basic
PARAM jobname AS STRING DESCRIPTION "Job to execute"

IF jobname = "low stock alert" THEN
    SET SCHEDULE "0 8 * * *"  ' Daily at 8 AM
    
    ' Find items below reorder point
    low_items = SQL "SELECT i.*, s.quantity_on_hand 
                     FROM items i 
                     JOIN inventory_stock s ON i.id = s.item_id 
                     WHERE s.quantity_on_hand <= i.reorder_point"
    
    IF UBOUND(low_items) > 0 THEN
        report = "Low Stock Alert\n\n"
        FOR EACH item IN low_items
            report = report + item.name + ": " + item.quantity_on_hand + " (reorder at " + item.reorder_point + ")\n"
        NEXT
        
        SEND MAIL "purchasing@company.com", "Daily Low Stock Alert", report
    END IF
END IF

IF jobname = "pending shipments" THEN
    SET SCHEDULE "0 7 * * *"  ' Daily at 7 AM
    
    pending = FIND "sales_orders", "status = 'ready_to_ship'"
    
    TALK "📦 " + UBOUND(pending) + " orders ready to ship today."
    
    SEND MAIL "warehouse@company.com", "Pending Shipments", 
        UBOUND(pending) + " orders need to be shipped today."
END IF
```

## Best Practices

1. **Always Verify Quantities**: Confirm counts before processing
2. **Maintain Audit Trail**: Log all inventory movements
3. **Regular Cycle Counts**: Schedule periodic physical inventory
4. **Monitor Reorder Points**: Act on low stock alerts promptly
5. **Validate PO/SO Numbers**: Check document existence before processing
6. **Cost Tracking**: Maintain accurate cost records for COGS

## Related Templates

- [store.bas](./store.md) - E-commerce integration
- [talk-to-data.bas](./talk-to-data.md) - Inventory analytics
- [backup.bas](./backup.md) - Data backup procedures

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>