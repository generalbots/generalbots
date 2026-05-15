PARAM action AS STRING LIKE "check_stock" DESCRIPTION "Action: receive_inventory, ship_inventory, check_stock, transfer_stock, cycle_count"
PARAM item_data AS OBJECT LIKE "{po_number: 'PO-123'}" DESCRIPTION "Data object with action-specific parameters"

DESCRIPTION "Manage inventory operations - receive, ship, check stock, transfer between warehouses, and cycle counts"

user_id = GET "session.user_id"
warehouse_id = GET "session.warehouse_id"

IF action = "receive_inventory" THEN
    po_number = item_data.po_number
    po = FIND "purchase_orders", "po_number = '" + po_number + "'"

    IF NOT po THEN
        TALK "Purchase order not found."
        RETURN NULL
    END IF

    IF po.status = "received" THEN
        TALK "This PO has already been received."
        RETURN NULL
    END IF

    po_lines = FIND "purchase_order_lines", "po_id = '" + po.id + "'"

    FOR EACH line IN po_lines
        item = FIND "items", "id = '" + line.item_id + "'"

        TALK "Receiving " + item.name + " - Ordered: " + line.quantity_ordered
        TALK "Enter quantity received:"
        HEAR qty_received AS INTEGER

        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"

        IF NOT stock THEN
            WITH newStock
                id = FORMAT(GUID())
                item_id = item.id
                warehouse_id = warehouse_id
                quantity_on_hand = qty_received
                last_movement_date = NOW()
            END WITH
            SAVE "inventory_stock", newStock
        ELSE
            new_qty = stock.quantity_on_hand + qty_received
            UPDATE "inventory_stock" SET quantity_on_hand = new_qty, last_movement_date = NOW() WHERE id = stock.id
        END IF

        WITH transaction
            id = FORMAT(GUID())
            transaction_type = "receipt"
            transaction_number = "REC-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
            item_id = item.id
            warehouse_id = warehouse_id
            quantity = qty_received
            unit_cost = line.unit_price
            total_cost = qty_received * line.unit_price
            reference_type = "purchase_order"
            reference_id = po.id
            created_by = user_id
            created_at = NOW()
        END WITH

        SAVE "inventory_transactions", transaction
        UPDATE "purchase_order_lines" SET quantity_received = line.quantity_received + qty_received WHERE id = line.id
        UPDATE "items" SET last_cost = line.unit_price WHERE id = item.id
    NEXT

    UPDATE "purchase_orders" SET status = "received" WHERE id = po.id

    TALK "Purchase order " + po_number + " received."
    SEND EMAIL po.buyer_id, "PO Received", "PO " + po_number + " received at warehouse " + warehouse_id

    RETURN po_number
END IF

IF action = "ship_inventory" THEN
    so_number = item_data.so_number
    so = FIND "sales_orders", "order_number = '" + so_number + "'"

    IF NOT so THEN
        TALK "Sales order not found."
        RETURN NULL
    END IF

    so_lines = FIND "sales_order_lines", "order_id = '" + so.id + "'"
    can_ship = true

    FOR EACH line IN so_lines
        item = FIND "items", "id = '" + line.item_id + "'"
        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"

        IF NOT stock OR stock.quantity_available < line.quantity_ordered THEN
            TALK "Insufficient stock for " + item.name
            can_ship = false
        END IF
    NEXT

    IF NOT can_ship THEN
        TALK "Cannot ship order due to insufficient inventory."
        RETURN NULL
    END IF

    shipment_number = "SHIP-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))

    FOR EACH line IN so_lines
        item = FIND "items", "id = '" + line.item_id + "'"
        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"

        new_qty = stock.quantity_on_hand - line.quantity_ordered
        UPDATE "inventory_stock" SET quantity_on_hand = new_qty, last_movement_date = NOW() WHERE id = stock.id

        WITH transaction
            id = FORMAT(GUID())
            transaction_type = "shipment"
            transaction_number = shipment_number
            item_id = item.id
            warehouse_id = warehouse_id
            quantity = 0 - line.quantity_ordered
            unit_cost = item.average_cost
            total_cost = line.quantity_ordered * item.average_cost
            reference_type = "sales_order"
            reference_id = so.id
            created_by = user_id
            created_at = NOW()
        END WITH

        SAVE "inventory_transactions", transaction
        UPDATE "sales_order_lines" SET quantity_shipped = line.quantity_ordered, cost_of_goods_sold = transaction.total_cost WHERE id = line.id
    NEXT

    UPDATE "sales_orders" SET status = "shipped" WHERE id = so.id

    TALK "Order " + so_number + " shipped. Tracking: " + shipment_number

    customer = FIND "customers", "id = '" + so.customer_id + "'"
    IF customer AND customer.email THEN
        SEND EMAIL customer.email, "Order Shipped", "Your order " + so_number + " has been shipped. Tracking: " + shipment_number
    END IF

    RETURN shipment_number
END IF

IF action = "check_stock" THEN
    item_search = item_data.item_search
    items = FIND "items", "name LIKE '%" + item_search + "%' OR item_code = '" + item_search + "'"

    IF NOT items THEN
        TALK "No items found."
        RETURN NULL
    END IF

    FOR EACH item IN items
        TALK "Item: " + item.name + " (" + item.item_code + ")"

        stocks = FIND "inventory_stock", "item_id = '" + item.id + "'"

        total_on_hand = 0
        total_available = 0

        FOR EACH stock IN stocks
            warehouse = FIND "warehouses", "id = '" + stock.warehouse_id + "'"
            TALK "  " + warehouse.name + ": " + stock.quantity_on_hand + " on hand"
            total_on_hand = total_on_hand + stock.quantity_on_hand
            total_available = total_available + stock.quantity_available
        NEXT

        TALK "  TOTAL: " + total_on_hand + " on hand, " + total_available + " available"

        IF total_available < item.minimum_stock_level THEN
            TALK "  WARNING: Below minimum stock level (" + item.minimum_stock_level + ")"
            IF item.reorder_point > 0 AND total_available <= item.reorder_point THEN
                TALK "  REORDER NEEDED! Qty: " + item.reorder_quantity
                CREATE_TASK "Reorder " + item.name, "high", user_id
            END IF
        END IF
    NEXT

    RETURN items
END IF

IF action = "transfer_stock" THEN
    TALK "Enter item code:"
    HEAR item_code AS STRING

    item = FIND "items", "item_code = '" + item_code + "'"

    IF NOT item THEN
        TALK "Item not found."
        RETURN NULL
    END IF

    TALK "From warehouse code:"
    HEAR from_warehouse_code AS STRING

    from_warehouse = FIND "warehouses", "code = '" + from_warehouse_code + "'"

    IF NOT from_warehouse THEN
        TALK "Source warehouse not found."
        RETURN NULL
    END IF

    from_stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + from_warehouse.id + "'"

    IF NOT from_stock THEN
        TALK "No stock in source warehouse."
        RETURN NULL
    END IF

    TALK "Available: " + from_stock.quantity_available
    TALK "Transfer quantity:"
    HEAR transfer_qty AS INTEGER

    IF transfer_qty > from_stock.quantity_available THEN
        TALK "Insufficient available quantity."
        RETURN NULL
    END IF

    TALK "To warehouse code:"
    HEAR to_warehouse_code AS STRING

    to_warehouse = FIND "warehouses", "code = '" + to_warehouse_code + "'"

    IF NOT to_warehouse THEN
        TALK "Destination warehouse not found."
        RETURN NULL
    END IF

    transfer_number = "TRAN-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))

    new_from_qty = from_stock.quantity_on_hand - transfer_qty
    UPDATE "inventory_stock" SET quantity_on_hand = new_from_qty, last_movement_date = NOW() WHERE id = from_stock.id

    WITH from_transaction
        id = FORMAT(GUID())
        transaction_type = "transfer_out"
        transaction_number = transfer_number
        item_id = item.id
        warehouse_id = from_warehouse.id
        quantity = 0 - transfer_qty
        unit_cost = item.average_cost
        created_by = user_id
        created_at = NOW()
    END WITH

    SAVE "inventory_transactions", from_transaction

    to_stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + to_warehouse.id + "'"

    IF NOT to_stock THEN
        WITH newToStock
            id = FORMAT(GUID())
            item_id = item.id
            warehouse_id = to_warehouse.id
            quantity_on_hand = transfer_qty
            last_movement_date = NOW()
        END WITH
        SAVE "inventory_stock", newToStock
    ELSE
        new_to_qty = to_stock.quantity_on_hand + transfer_qty
        UPDATE "inventory_stock" SET quantity_on_hand = new_to_qty, last_movement_date = NOW() WHERE id = to_stock.id
    END IF

    WITH to_transaction
        id = FORMAT(GUID())
        transaction_type = "transfer_in"
        transaction_number = transfer_number
        item_id = item.id
        warehouse_id = to_warehouse.id
        quantity = transfer_qty
        unit_cost = item.average_cost
        created_by = user_id
        created_at = NOW()
    END WITH

    SAVE "inventory_transactions", to_transaction

    TALK "Transfer " + transfer_number + " completed: " + transfer_qty + " units from " + from_warehouse.name + " to " + to_warehouse.name

    RETURN transfer_number
END IF

IF action = "cycle_count" THEN
    TALK "Enter warehouse code:"
    HEAR warehouse_code AS STRING

    warehouse = FIND "warehouses", "code = '" + warehouse_code + "'"

    IF NOT warehouse THEN
        TALK "Warehouse not found."
        RETURN NULL
    END IF

    stocks = FIND "inventory_stock", "warehouse_id = '" + warehouse.id + "'"

    count_number = "COUNT-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
    adjustments = 0

    FOR EACH stock IN stocks
        item = FIND "items", "id = '" + stock.item_id + "'"

        TALK "Item: " + item.name + " (" + item.item_code + ")"
        TALK "System quantity: " + stock.quantity_on_hand
        TALK "Enter physical count:"
        HEAR physical_count AS INTEGER

        IF physical_count <> stock.quantity_on_hand THEN
            variance = physical_count - stock.quantity_on_hand

            WITH adjustment
                id = FORMAT(GUID())
                transaction_type = "adjustment"
                transaction_number = count_number
                item_id = item.id
                warehouse_id = warehouse.id
                quantity = variance
                notes = "Cycle count adjustment"
                created_by = user_id
                created_at = NOW()
            END WITH

            SAVE "inventory_transactions", adjustment

            UPDATE "inventory_stock" SET quantity_on_hand = physical_count, last_counted_date = NOW(), last_movement_date = NOW() WHERE id = stock.id

            adjustments = adjustments + 1
            TALK "  Adjusted by " + variance + " units"
        ELSE
            UPDATE "inventory_stock" SET last_counted_date = NOW() WHERE id = stock.id
            TALK "  Count confirmed"
        END IF
    NEXT

    TALK "Cycle count " + count_number + " completed with " + adjustments + " adjustments"

    IF adjustments > 0 THEN
        SEND EMAIL "inventory-manager@company.com", "Cycle Count Results", "Cycle count " + count_number + " at " + warehouse.name + " with " + adjustments + " adjustments"
    END IF

    RETURN count_number
END IF

TALK "Unknown action: " + action
RETURN NULL
