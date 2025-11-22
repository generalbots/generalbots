PARAM job_name AS STRING

user_id = GET "session.user_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF job_name = "inventory_reorder" THEN
    items = FIND "items", "is_purchasable = true AND reorder_point > 0"

    reorders_created = 0

    FOR EACH item IN items DO
        stocks = FIND "inventory_stock", "item_id = '" + item.id + "'"

        total_available = 0
        FOR EACH stock IN stocks DO
            total_available = total_available + stock.quantity_available
        END FOR

        IF total_available <= item.reorder_point THEN
            po = CREATE OBJECT
            SET po.id = FORMAT GUID()
            SET po.po_number = "PO-AUTO-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(100, 999)
            SET po.status = "draft"
            SET po.order_date = current_time
            SET po.buyer_id = "system"
            SET po.created_by = "system"
            SET po.created_at = current_time

            vendor_item = FIND "vendor_items", "item_id = '" + item.id + "' AND is_preferred = true"
            IF vendor_item != NULL THEN
                po.vendor_id = vendor_item.vendor_id

                SAVE_FROM_UNSTRUCTURED "purchase_orders", FORMAT po AS JSON

                line = CREATE OBJECT
                SET line.id = FORMAT GUID()
                SET line.po_id = po.id
                SET line.line_number = 1
                SET line.item_id = item.id
                SET line.quantity_ordered = item.reorder_quantity
                SET line.unit_price = vendor_item.unit_price
                SET line.created_at = current_time

                SAVE_FROM_UNSTRUCTURED "purchase_order_lines", FORMAT line AS JSON

                reorders_created = reorders_created + 1

                CREATE_TASK "Approve reorder PO " + po.po_number + " for " + item.name, "high", "purchasing"
            END IF
        END IF
    END FOR

    IF reorders_created > 0 THEN
        notification = "Created " + reorders_created + " automatic reorder POs"
        SEND MAIL "purchasing@company.com", "Automatic Reorders", notification
    END IF
END IF

IF job_name = "low_stock_alert" THEN
    items = FIND "items", "minimum_stock_level > 0"

    low_stock_items = []
    critical_items = []

    FOR EACH item IN items DO
        stocks = FIND "inventory_stock", "item_id = '" + item.id + "'"

        total_on_hand = 0
        FOR EACH stock IN stocks DO
            total_on_hand = total_on_hand + stock.quantity_on_hand
        END FOR

        IF total_on_hand < item.minimum_stock_level THEN
            stock_ratio = total_on_hand / item.minimum_stock_level

            IF stock_ratio < 0.25 THEN
                APPEND critical_items, item.name + " (" + total_on_hand + "/" + item.minimum_stock_level + ")"
            ELSE
                APPEND low_stock_items, item.name + " (" + total_on_hand + "/" + item.minimum_stock_level + ")"
            END IF
        END IF
    END FOR

    IF LENGTH(critical_items) > 0 OR LENGTH(low_stock_items) > 0 THEN
        alert = "INVENTORY ALERT\n"
        alert = alert + "===============\n\n"

        IF LENGTH(critical_items) > 0 THEN
            alert = alert + "CRITICAL (Below 25%):\n"
            FOR EACH item_info IN critical_items DO
                alert = alert + "- " + item_info + "\n"
            END FOR
            alert = alert + "\n"
        END IF

        IF LENGTH(low_stock_items) > 0 THEN
            alert = alert + "LOW STOCK:\n"
            FOR EACH item_info IN low_stock_items DO
                alert = alert + "- " + item_info + "\n"
            END FOR
        END IF

        SEND MAIL "inventory-manager@company.com", "Low Stock Alert", alert
    END IF
END IF

IF job_name = "po_follow_up" THEN
    pos = FIND "purchase_orders", "status = 'approved'"

    FOR EACH po IN pos DO
        days_old = DAYS_BETWEEN(po.order_date, current_time)

        IF days_old > 7 THEN
            vendor = FIND "vendors", "id = '" + po.vendor_id + "'"

            notification = "PO " + po.po_number + " has been approved for " + days_old + " days without receipt"
            SEND MAIL po.buyer_id, "PO Follow-up Required", notification

            CREATE_TASK "Follow up on PO " + po.po_number + " with " + vendor.name, "medium", po.buyer_id
        END IF
    END FOR
END IF

IF job_name = "cost_analysis" THEN
    start_of_month = FORMAT NOW() AS "YYYY-MM" + "-01"

    transactions = FIND "inventory_transactions", "created_at >= '" + start_of_month + "'"

    total_receipts_value = 0
    total_shipments_value = 0
    total_adjustments_value = 0

    FOR EACH trans IN transactions DO
        IF trans.transaction_type = "receipt" THEN
            total_receipts_value = total_receipts_value + trans.total_cost
        ELSE IF trans.transaction_type = "shipment" THEN
            total_shipments_value = total_shipments_value + ABS(trans.total_cost)
        ELSE IF trans.transaction_type = "adjustment" THEN
            total_adjustments_value = total_adjustments_value + ABS(trans.total_cost)
        END IF
    END FOR

    report = "MONTHLY INVENTORY COST ANALYSIS\n"
    report = report + "================================\n"
    report = report + "Period: " + FORMAT NOW() AS "MMMM YYYY" + "\n\n"
    report = report + "Receipts Value: $" + total_receipts_value + "\n"
    report = report + "Shipments Value: $" + total_shipments_value + "\n"
    report = report + "Adjustments Value: $" + total_adjustments_value + "\n"
    report = report + "\n"
    report = report + "Gross Margin: $" + (total_shipments_value - total_receipts_value) + "\n"

    SEND MAIL "cfo@company.com", "Monthly Inventory Cost Report", report
END IF

IF job_name = "vendor_scorecard" THEN
    vendors = FIND "vendors", "status = 'active'"

    scorecard = "VENDOR SCORECARD - " + current_time + "\n"
    scorecard = scorecard + "====================================\n\n"

    FOR EACH vendor IN vendors DO
        pos = FIND "purchase_orders", "vendor_id = '" + vendor.id + "' AND created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY)"

        total_pos = 0
        on_time = 0
        total_spend = 0

        FOR EACH po IN pos DO
            total_pos = total_pos + 1
            total_spend = total_spend + po.total_amount

            IF po.status = "received" THEN
                IF po.received_date <= po.expected_date THEN
                    on_time = on_time + 1
                END IF
            END IF
        END FOR

        IF total_pos > 0 THEN
            on_time_rate = (on_time / total_pos) * 100

            scorecard = scorecard + vendor.name + "\n"
            scorecard = scorecard + "  Orders: " + total_pos + "\n"
            scorecard = scorecard + "  Spend: $" + total_spend + "\n"
            scorecard = scorecard + "  On-Time: " + on_time_rate + "%\n"

            IF on_time_rate < 80 THEN
                scorecard = scorecard + "  WARNING: Low performance\n"
            END IF

            scorecard = scorecard + "\n"
        END IF
    END FOR

    SEND MAIL "purchasing@company.com", "Vendor Scorecard", scorecard
END IF

IF job_name = "warehouse_capacity" THEN
    warehouses = FIND "warehouses", "is_active = true"

    capacity_report = "WAREHOUSE CAPACITY REPORT\n"
    capacity_report = capacity_report + "========================\n\n"

    FOR EACH warehouse IN warehouses DO
        stocks = FIND "inventory_stock", "warehouse_id = '" + warehouse.id + "'"

        total_units = 0
        FOR EACH stock IN stocks DO
            total_units = total_units + stock.quantity_on_hand
        END FOR

        utilization = 0
        IF warehouse.capacity_units > 0 THEN
            utilization = (total_units / warehouse.capacity_units) * 100
        END IF

        capacity_report = capacity_report + warehouse.name + "\n"
        capacity_report = capacity_report + "  Units: " + total_units + " / " + warehouse.capacity_units + "\n"
        capacity_report = capacity_report + "  Utilization: " + utilization + "%\n"

        IF utilization > 90 THEN
            capacity_report = capacity_report + "  WARNING: Near capacity\n"
            CREATE_TASK "Review capacity at " + warehouse.name, "high", "warehouse-manager"
        ELSE IF utilization < 20 THEN
            capacity_report = capacity_report + "  NOTE: Low utilization\n"
        END IF

        capacity_report = capacity_report + "\n"
    END FOR

    SEND MAIL "operations@company.com", "Warehouse Capacity Report", capacity_report
END IF

IF job_name = "invoice_aging" THEN
    invoices = FIND "invoices", "balance_due > 0"

    aging_30 = 0
    aging_60 = 0
    aging_90 = 0
    aging_over_90 = 0

    total_30 = 0
    total_60 = 0
    total_90 = 0
    total_over_90 = 0

    FOR EACH invoice IN invoices DO
        days_old = DAYS_BETWEEN(invoice.invoice_date, current_time)

        IF days_old <= 30 THEN
            aging_30 = aging_30 + 1
            total_30 = total_30 + invoice.balance_due
        ELSE IF days_old <= 60 THEN
            aging_60 = aging_60 + 1
            total_60 = total_60 + invoice.balance_due
        ELSE IF days_old <= 90 THEN
            aging_90 = aging_90 + 1
            total_90 = total_90 + invoice.balance_due
        ELSE
            aging_over_90 = aging_over_90 + 1
            total_over_90 = total_over_90 + invoice.balance_due

            customer = FIND "customers", "id = '" + invoice.customer_id + "'"
            IF customer != NULL THEN
                notification = "Invoice " + invoice.invoice_number + " is over 90 days past due. Amount: $" + invoice.balance_due
                CREATE_TASK "Collection: " + customer.name + " - " + invoice.invoice_number, "critical", "collections"
            END IF
        END IF
    END FOR

    aging_report = "ACCOUNTS RECEIVABLE AGING\n"
    aging_report = aging_report + "=========================\n\n"
    aging_report = aging_report + "0-30 days: " + aging_30 + " invoices, $" + total_30 + "\n"
    aging_report = aging_report + "31-60 days: " + aging_60 + " invoices, $" + total_60 + "\n"
    aging_report = aging_report + "61-90 days: " + aging_90 + " invoices, $" + total_90 + "\n"
    aging_report = aging_report + "Over 90 days: " + aging_over_90 + " invoices, $" + total_over_90 + "\n"
    aging_report = aging_report + "\n"
    aging_report = aging_report + "TOTAL OUTSTANDING: $" + (total_30 + total_60 + total_90 + total_over_90) + "\n"

    SEND MAIL "finance@company.com", "AR Aging Report", aging_report
END IF

IF job_name = "setup_schedules" THEN
    SET SCHEDULE "0 6 * * *" "erp-jobs.bas" "inventory_reorder"
    SET SCHEDULE "0 8,16 * * *" "erp-jobs.bas" "low_stock_alert"
    SET SCHEDULE "0 10 * * *" "erp-jobs.bas" "po_follow_up"
    SET SCHEDULE "0 0 1 * *" "erp-jobs.bas" "cost_analysis"
    SET SCHEDULE "0 9 * * MON" "erp-jobs.bas" "vendor_scorecard"
    SET SCHEDULE "0 7 * * *" "erp-jobs.bas" "warehouse_capacity"
    SET SCHEDULE "0 11 * * *" "erp-jobs.bas" "invoice_aging"

    TALK "All ERP schedules have been configured"
END IF
