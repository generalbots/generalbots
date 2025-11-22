PARAM action AS STRING
PARAM purchase_data AS OBJECT

user_id = GET "session.user_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF action = "create_po" THEN
    vendor_code = GET "purchase_data.vendor_code"

    IF vendor_code = "" THEN
        TALK "Enter vendor code:"
        vendor_code = HEAR
    END IF

    vendor = FIND "vendors", "vendor_code = '" + vendor_code + "'"

    IF vendor = NULL THEN
        TALK "Vendor not found."
        EXIT
    END IF

    po_number = "PO-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

    po = CREATE OBJECT
    SET po.id = FORMAT GUID()
    SET po.po_number = po_number
    SET po.vendor_id = vendor.id
    SET po.order_date = current_time
    SET po.status = "draft"
    SET po.buyer_id = user_id
    SET po.created_by = user_id
    SET po.created_at = current_time
    SET po.subtotal = 0

    SAVE_FROM_UNSTRUCTURED "purchase_orders", FORMAT po AS JSON

    SET "session.po_id" = po.id
    REMEMBER "po_" + po.id = po

    TALK "Purchase Order " + po_number + " created for " + vendor.name

    adding_items = true
    line_number = 1
    total = 0

    WHILE adding_items = true DO
        TALK "Enter item code (or 'done' to finish):"
        item_code = HEAR

        IF item_code = "done" THEN
            adding_items = false
        ELSE
            item = FIND "items", "item_code = '" + item_code + "'"

            IF item = NULL THEN
                TALK "Item not found. Try again."
            ELSE
                TALK "Quantity to order:"
                quantity = HEAR

                TALK "Unit price (or press Enter for last cost: " + item.last_cost + "):"
                price_input = HEAR

                IF price_input = "" THEN
                    unit_price = item.last_cost
                ELSE
                    unit_price = price_input
                END IF

                line = CREATE OBJECT
                SET line.id = FORMAT GUID()
                SET line.po_id = po.id
                SET line.line_number = line_number
                SET line.item_id = item.id
                SET line.description = item.name
                SET line.quantity_ordered = quantity
                SET line.unit_price = unit_price
                SET line.line_total = quantity * unit_price
                SET line.created_at = current_time

                SAVE_FROM_UNSTRUCTURED "purchase_order_lines", FORMAT line AS JSON

                total = total + line.line_total
                line_number = line_number + 1

                TALK "Added: " + item.name + " x " + quantity + " @ $" + unit_price
            END IF
        END IF
    END WHILE

    po.subtotal = total
    po.total_amount = total
    SAVE_FROM_UNSTRUCTURED "purchase_orders", FORMAT po AS JSON

    TALK "Purchase Order " + po_number + " total: $" + total

END IF

IF action = "approve_po" THEN
    po_number = GET "purchase_data.po_number"

    IF po_number = "" THEN
        TALK "Enter PO number to approve:"
        po_number = HEAR
    END IF

    po = FIND "purchase_orders", "po_number = '" + po_number + "'"

    IF po = NULL THEN
        TALK "Purchase order not found."
        EXIT
    END IF

    IF po.status != "draft" THEN
        TALK "PO status is " + po.status + ". Can only approve draft POs."
        EXIT
    END IF

    po_lines = FIND "purchase_order_lines", "po_id = '" + po.id + "'"

    TALK "PO Summary:"
    TALK "Vendor: " + po.vendor_id
    TALK "Total: $" + po.total_amount
    TALK "Items:"

    FOR EACH line IN po_lines DO
        TALK "  - " + line.description + " x " + line.quantity_ordered + " @ $" + line.unit_price
    END FOR

    TALK "Approve this PO? (yes/no)"
    approval = HEAR

    IF approval = "yes" OR approval = "YES" OR approval = "Yes" THEN
        po.status = "approved"
        po.approved_by = user_id
        po.approved_date = current_time
        SAVE_FROM_UNSTRUCTURED "purchase_orders", FORMAT po AS JSON

        vendor = FIND "vendors", "id = '" + po.vendor_id + "'"

        IF vendor.email != "" THEN
            message = "Purchase Order " + po_number + " has been approved. Total: $" + po.total_amount
            SEND MAIL vendor.email, "PO Approved: " + po_number, message
        END IF

        TALK "PO " + po_number + " approved successfully."

        CREATE_TASK "Process PO " + po_number, "high", user_id

    ELSE
        TALK "PO not approved."
    END IF

END IF

IF action = "vendor_performance" THEN
    vendor_code = GET "purchase_data.vendor_code"

    IF vendor_code = "" THEN
        TALK "Enter vendor code:"
        vendor_code = HEAR
    END IF

    vendor = FIND "vendors", "vendor_code = '" + vendor_code + "'"

    IF vendor = NULL THEN
        TALK "Vendor not found."
        EXIT
    END IF

    pos = FIND "purchase_orders", "vendor_id = '" + vendor.id + "'"

    total_pos = 0
    on_time = 0
    late = 0
    total_spend = 0

    FOR EACH po IN pos DO
        total_pos = total_pos + 1
        total_spend = total_spend + po.total_amount

        IF po.status = "received" THEN
            IF po.received_date <= po.expected_date THEN
                on_time = on_time + 1
            ELSE
                late = late + 1
            END IF
        END IF
    END FOR

    on_time_percentage = 0
    IF total_pos > 0 THEN
        on_time_percentage = (on_time / total_pos) * 100
    END IF

    TALK "VENDOR PERFORMANCE: " + vendor.name
    TALK "================================"
    TALK "Total Purchase Orders: " + total_pos
    TALK "Total Spend: $" + total_spend
    TALK "On-Time Delivery: " + on_time_percentage + "%"
    TALK "Late Deliveries: " + late

    IF on_time_percentage < 80 THEN
        TALK "WARNING: Low on-time delivery rate"
        CREATE_TASK "Review vendor " + vendor.name + " performance", "medium", user_id
    END IF

END IF

IF action = "reorder_check" THEN
    items = FIND "items", "is_purchasable = true"

    reorder_needed = 0

    FOR EACH item IN items DO
        IF item.reorder_point > 0 THEN
            stocks = FIND "inventory_stock", "item_id = '" + item.id + "'"

            total_available = 0
            FOR EACH stock IN stocks DO
                total_available = total_available + stock.quantity_available
            END FOR

            IF total_available <= item.reorder_point THEN
                reorder_needed = reorder_needed + 1

                TALK "REORDER: " + item.name
                TALK "  Current stock: " + total_available
                TALK "  Reorder point: " + item.reorder_point
                TALK "  Suggested qty: " + item.reorder_quantity

                preferred_vendor = FIND "vendor_items", "item_id = '" + item.id + "' AND is_preferred = true"

                IF preferred_vendor != NULL THEN
                    vendor = FIND "vendors", "id = '" + preferred_vendor.vendor_id + "'"
                    TALK "  Preferred vendor: " + vendor.name
                END IF

                CREATE_TASK "Reorder " + item.name + " (qty: " + item.reorder_quantity + ")", "high", user_id
            END IF
        END IF
    END FOR

    IF reorder_needed = 0 THEN
        TALK "No items need reordering."
    ELSE
        TALK "Total items needing reorder: " + reorder_needed
    END IF

END IF

IF action = "requisition" THEN
    req_number = "REQ-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

    TALK "Creating requisition " + req_number

    req = CREATE OBJECT
    SET req.id = FORMAT GUID()
    SET req.req_number = req_number
    SET req.requester = user_id
    SET req.status = "pending"
    SET req.created_at = current_time
    SET req.items = []

    adding = true

    WHILE adding = true DO
        TALK "Enter item description (or 'done'):"
        item_desc = HEAR

        IF item_desc = "done" THEN
            adding = false
        ELSE
            TALK "Quantity needed:"
            quantity = HEAR

            TALK "Reason/Project:"
            reason = HEAR

            req_item = CREATE OBJECT
            SET req_item.description = item_desc
            SET req_item.quantity = quantity
            SET req_item.reason = reason

            APPEND req.items, req_item

            TALK "Added to requisition."
        END IF
    END WHILE

    SAVE_FROM_UNSTRUCTURED "requisitions", FORMAT req AS JSON

    TALK "Requisition " + req_number + " created with " + LENGTH(req.items) + " items."

    notification = "New requisition " + req_number + " from " + user_id + " needs approval"
    SEND MAIL "purchasing@company.com", "New Requisition", notification

    CREATE_TASK "Review requisition " + req_number, "medium", "purchasing"

END IF

IF action = "price_comparison" THEN
    item_code = GET "purchase_data.item_code"

    IF item_code = "" THEN
        TALK "Enter item code:"
        item_code = HEAR
    END IF

    item = FIND "items", "item_code = '" + item_code + "'"

    IF item = NULL THEN
        TALK "Item not found."
        EXIT
    END IF

    vendor_items = FIND "vendor_items", "item_id = '" + item.id + "'"

    IF vendor_items = NULL THEN
        TALK "No vendor pricing found for this item."
        EXIT
    END IF

    TALK "PRICE COMPARISON: " + item.name
    TALK "================================"

    best_price = 999999
    best_vendor = ""

    FOR EACH vi IN vendor_items DO
        vendor = FIND "vendors", "id = '" + vi.vendor_id + "'"

        TALK vendor.name + ":"
        TALK "  Unit price: $" + vi.unit_price
        TALK "  Min order: " + vi.min_order_qty
        TALK "  Lead time: " + vi.lead_time_days + " days"

        IF vi.unit_price < best_price THEN
            best_price = vi.unit_price
            best_vendor = vendor.name
        END IF
    END FOR

    TALK ""
    TALK "Best price: $" + best_price + " from " + best_vendor

END IF
