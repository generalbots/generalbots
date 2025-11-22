PARAM action AS STRING
PARAM item_data AS OBJECT

user_id = GET "session.user_id"
warehouse_id = GET "session.warehouse_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF action = "receive_inventory" THEN
    po_number = GET "item_data.po_number"

    IF po_number = "" THEN
        TALK "Enter Purchase Order number:"
        po_number = HEAR
    END IF

    po = FIND "purchase_orders", "po_number = '" + po_number + "'"

    IF po = NULL THEN
        TALK "Purchase order not found."
        EXIT
    END IF

    IF po.status = "received" THEN
        TALK "This PO has already been received."
        EXIT
    END IF

    po_lines = FIND "purchase_order_lines", "po_id = '" + po.id + "'"

    FOR EACH line IN po_lines DO
        item = FIND "items", "id = '" + line.item_id + "'"

        TALK "Receiving " + item.name + " - Ordered: " + line.quantity_ordered
        TALK "Enter quantity received:"
        qty_received = HEAR

        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"

        IF stock = NULL THEN
            stock = CREATE OBJECT
            SET stock.id = FORMAT GUID()
            SET stock.item_id = item.id
            SET stock.warehouse_id = warehouse_id
            SET stock.quantity_on_hand = qty_received
            SET stock.last_movement_date = current_time

            SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT stock AS JSON
        ELSE
            stock.quantity_on_hand = stock.quantity_on_hand + qty_received
            stock.last_movement_date = current_time

            SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT stock AS JSON
        END IF

        transaction = CREATE OBJECT
        SET transaction.id = FORMAT GUID()
        SET transaction.transaction_type = "receipt"
        SET transaction.transaction_number = "REC-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)
        SET transaction.item_id = item.id
        SET transaction.warehouse_id = warehouse_id
        SET transaction.quantity = qty_received
        SET transaction.unit_cost = line.unit_price
        SET transaction.total_cost = qty_received * line.unit_price
        SET transaction.reference_type = "purchase_order"
        SET transaction.reference_id = po.id
        SET transaction.created_by = user_id
        SET transaction.created_at = current_time

        SAVE_FROM_UNSTRUCTURED "inventory_transactions", FORMAT transaction AS JSON

        line.quantity_received = line.quantity_received + qty_received
        SAVE_FROM_UNSTRUCTURED "purchase_order_lines", FORMAT line AS JSON

        item.last_cost = line.unit_price
        item.average_cost = ((item.average_cost * stock.quantity_on_hand) + (qty_received * line.unit_price)) / (stock.quantity_on_hand + qty_received)
        SAVE_FROM_UNSTRUCTURED "items", FORMAT item AS JSON
    END FOR

    po.status = "received"
    SAVE_FROM_UNSTRUCTURED "purchase_orders", FORMAT po AS JSON

    TALK "Purchase order " + po_number + " received successfully."

    notification = "PO " + po_number + " received at warehouse " + warehouse_id
    SEND MAIL po.buyer_id, "PO Received", notification

END IF

IF action = "ship_inventory" THEN
    so_number = GET "item_data.so_number"

    IF so_number = "" THEN
        TALK "Enter Sales Order number:"
        so_number = HEAR
    END IF

    so = FIND "sales_orders", "order_number = '" + so_number + "'"

    IF so = NULL THEN
        TALK "Sales order not found."
        EXIT
    END IF

    so_lines = FIND "sales_order_lines", "order_id = '" + so.id + "'"

    can_ship = true

    FOR EACH line IN so_lines DO
        item = FIND "items", "id = '" + line.item_id + "'"
        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"

        IF stock = NULL OR stock.quantity_available < line.quantity_ordered THEN
            TALK "Insufficient stock for " + item.name + ". Available: " + stock.quantity_available + ", Needed: " + line.quantity_ordered
            can_ship = false
        END IF
    END FOR

    IF can_ship = false THEN
        TALK "Cannot ship order due to insufficient inventory."
        EXIT
    END IF

    shipment_number = "SHIP-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

    FOR EACH line IN so_lines DO
        item = FIND "items", "id = '" + line.item_id + "'"
        stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + warehouse_id + "'"

        stock.quantity_on_hand = stock.quantity_on_hand - line.quantity_ordered
        stock.last_movement_date = current_time
        SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT stock AS JSON

        transaction = CREATE OBJECT
        SET transaction.id = FORMAT GUID()
        SET transaction.transaction_type = "shipment"
        SET transaction.transaction_number = shipment_number
        SET transaction.item_id = item.id
        SET transaction.warehouse_id = warehouse_id
        SET transaction.quantity = 0 - line.quantity_ordered
        SET transaction.unit_cost = item.average_cost
        SET transaction.total_cost = line.quantity_ordered * item.average_cost
        SET transaction.reference_type = "sales_order"
        SET transaction.reference_id = so.id
        SET transaction.created_by = user_id
        SET transaction.created_at = current_time

        SAVE_FROM_UNSTRUCTURED "inventory_transactions", FORMAT transaction AS JSON

        line.quantity_shipped = line.quantity_ordered
        line.cost_of_goods_sold = line.quantity_ordered * item.average_cost
        SAVE_FROM_UNSTRUCTURED "sales_order_lines", FORMAT line AS JSON
    END FOR

    so.status = "shipped"
    SAVE_FROM_UNSTRUCTURED "sales_orders", FORMAT so AS JSON

    TALK "Order " + so_number + " shipped. Tracking: " + shipment_number

    customer = FIND "customers", "id = '" + so.customer_id + "'"
    IF customer != NULL AND customer.email != "" THEN
        message = "Your order " + so_number + " has been shipped. Tracking: " + shipment_number
        SEND MAIL customer.email, "Order Shipped", message
    END IF

END IF

IF action = "check_stock" THEN
    item_search = GET "item_data.item_search"

    IF item_search = "" THEN
        TALK "Enter item name or code:"
        item_search = HEAR
    END IF

    items = FIND "items", "name LIKE '%" + item_search + "%' OR item_code = '" + item_search + "'"

    IF items = NULL THEN
        TALK "No items found."
        EXIT
    END IF

    FOR EACH item IN items DO
        TALK "Item: " + item.name + " (" + item.item_code + ")"

        stocks = FIND "inventory_stock", "item_id = '" + item.id + "'"

        total_on_hand = 0
        total_available = 0
        total_reserved = 0

        FOR EACH stock IN stocks DO
            warehouse = FIND "warehouses", "id = '" + stock.warehouse_id + "'"
            TALK "  " + warehouse.name + ": " + stock.quantity_on_hand + " on hand, " + stock.quantity_available + " available"

            total_on_hand = total_on_hand + stock.quantity_on_hand
            total_available = total_available + stock.quantity_available
            total_reserved = total_reserved + stock.quantity_reserved
        END FOR

        TALK "  TOTAL: " + total_on_hand + " on hand, " + total_available + " available, " + total_reserved + " reserved"

        IF total_available < item.minimum_stock_level THEN
            TALK "  WARNING: Below minimum stock level (" + item.minimum_stock_level + ")"

            IF item.reorder_point > 0 AND total_available <= item.reorder_point THEN
                TALK "  REORDER NEEDED! Reorder quantity: " + item.reorder_quantity
                CREATE_TASK "Reorder " + item.name, "high", user_id
            END IF
        END IF
    END FOR

END IF

IF action = "transfer_stock" THEN
    TALK "Enter item code:"
    item_code = HEAR

    item = FIND "items", "item_code = '" + item_code + "'"

    IF item = NULL THEN
        TALK "Item not found."
        EXIT
    END IF

    TALK "From warehouse code:"
    from_warehouse_code = HEAR

    from_warehouse = FIND "warehouses", "code = '" + from_warehouse_code + "'"

    IF from_warehouse = NULL THEN
        TALK "Source warehouse not found."
        EXIT
    END IF

    from_stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + from_warehouse.id + "'"

    IF from_stock = NULL THEN
        TALK "No stock in source warehouse."
        EXIT
    END IF

    TALK "Available: " + from_stock.quantity_available
    TALK "Transfer quantity:"
    transfer_qty = HEAR

    IF transfer_qty > from_stock.quantity_available THEN
        TALK "Insufficient available quantity."
        EXIT
    END IF

    TALK "To warehouse code:"
    to_warehouse_code = HEAR

    to_warehouse = FIND "warehouses", "code = '" + to_warehouse_code + "'"

    IF to_warehouse = NULL THEN
        TALK "Destination warehouse not found."
        EXIT
    END IF

    transfer_number = "TRAN-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

    from_stock.quantity_on_hand = from_stock.quantity_on_hand - transfer_qty
    from_stock.last_movement_date = current_time
    SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT from_stock AS JSON

    from_transaction = CREATE OBJECT
    SET from_transaction.id = FORMAT GUID()
    SET from_transaction.transaction_type = "transfer_out"
    SET from_transaction.transaction_number = transfer_number
    SET from_transaction.item_id = item.id
    SET from_transaction.warehouse_id = from_warehouse.id
    SET from_transaction.quantity = 0 - transfer_qty
    SET from_transaction.unit_cost = item.average_cost
    SET from_transaction.created_by = user_id
    SET from_transaction.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "inventory_transactions", FORMAT from_transaction AS JSON

    to_stock = FIND "inventory_stock", "item_id = '" + item.id + "' AND warehouse_id = '" + to_warehouse.id + "'"

    IF to_stock = NULL THEN
        to_stock = CREATE OBJECT
        SET to_stock.id = FORMAT GUID()
        SET to_stock.item_id = item.id
        SET to_stock.warehouse_id = to_warehouse.id
        SET to_stock.quantity_on_hand = transfer_qty
        SET to_stock.last_movement_date = current_time

        SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT to_stock AS JSON
    ELSE
        to_stock.quantity_on_hand = to_stock.quantity_on_hand + transfer_qty
        to_stock.last_movement_date = current_time

        SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT to_stock AS JSON
    END IF

    to_transaction = CREATE OBJECT
    SET to_transaction.id = FORMAT GUID()
    SET to_transaction.transaction_type = "transfer_in"
    SET to_transaction.transaction_number = transfer_number
    SET to_transaction.item_id = item.id
    SET to_transaction.warehouse_id = to_warehouse.id
    SET to_transaction.quantity = transfer_qty
    SET to_transaction.unit_cost = item.average_cost
    SET to_transaction.created_by = user_id
    SET to_transaction.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "inventory_transactions", FORMAT to_transaction AS JSON

    TALK "Transfer " + transfer_number + " completed: " + transfer_qty + " units from " + from_warehouse.name + " to " + to_warehouse.name

END IF

IF action = "cycle_count" THEN
    TALK "Enter warehouse code:"
    warehouse_code = HEAR

    warehouse = FIND "warehouses", "code = '" + warehouse_code + "'"

    IF warehouse = NULL THEN
        TALK "Warehouse not found."
        EXIT
    END IF

    stocks = FIND "inventory_stock", "warehouse_id = '" + warehouse.id + "'"

    count_number = "COUNT-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)
    adjustments = 0

    FOR EACH stock IN stocks DO
        item = FIND "items", "id = '" + stock.item_id + "'"

        TALK "Item: " + item.name + " (" + item.item_code + ")"
        TALK "System quantity: " + stock.quantity_on_hand
        TALK "Enter physical count:"
        physical_count = HEAR

        IF physical_count != stock.quantity_on_hand THEN
            variance = physical_count - stock.quantity_on_hand

            adjustment = CREATE OBJECT
            SET adjustment.id = FORMAT GUID()
            SET adjustment.transaction_type = "adjustment"
            SET adjustment.transaction_number = count_number
            SET adjustment.item_id = item.id
            SET adjustment.warehouse_id = warehouse.id
            SET adjustment.quantity = variance
            SET adjustment.notes = "Cycle count adjustment"
            SET adjustment.created_by = user_id
            SET adjustment.created_at = current_time

            SAVE_FROM_UNSTRUCTURED "inventory_transactions", FORMAT adjustment AS JSON

            stock.quantity_on_hand = physical_count
            stock.last_counted_date = current_time
            stock.last_movement_date = current_time
            SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT stock AS JSON

            adjustments = adjustments + 1

            TALK "  Adjusted by " + variance + " units"
        ELSE
            stock.last_counted_date = current_time
            SAVE_FROM_UNSTRUCTURED "inventory_stock", FORMAT stock AS JSON

            TALK "  Count confirmed"
        END IF
    END FOR

    TALK "Cycle count " + count_number + " completed with " + adjustments + " adjustments"

    IF adjustments > 0 THEN
        notification = "Cycle count " + count_number + " completed at " + warehouse.name + " with " + adjustments + " adjustments"
        SEND MAIL "inventory-manager@company.com", "Cycle Count Results", notification
    END IF

END IF
