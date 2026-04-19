# Data Sync Tools

This document provides a collection of specialized data synchronization tools. Instead of one monolithic sync system, these modular tools can be combined as needed.

## Overview

Data synchronization is split into focused, reusable tools:

| Tool | Purpose | File |
|------|---------|------|
| `crm-sync.bas` | CRM to/from internal database | Bidirectional customer data |
| `erp-sync.bas` | ERP system integration | Orders, inventory, accounting |
| `inventory-sync.bas` | Real-time inventory updates | Stock levels across systems |
| `user-sync.bas` | User/employee directory sync | HR systems, Active Directory |
| `conflict-resolver.bas` | Handle sync conflicts | Automated or manual resolution |
| `sync-monitor.bas` | Monitor sync health | Alerts and dashboards |

## Tool 1: CRM Sync

Bidirectional synchronization with CRM systems (Salesforce, HubSpot, etc.).

```basic
' crm-sync.bas
' Bidirectional CRM synchronization tool

WEBHOOK "crm-inbound"

event = body.event
record_type = body.type
record_id = body.id
data = body.data
source_timestamp = body.timestamp

' Validate webhook signature
signature = headers.x_webhook_signature
secret = GET BOT MEMORY "crm_webhook_secret"
IF NOT VERIFY_SIGNATURE(body, signature, secret) THEN
    WITH result = NEW OBJECT
        .status = 401
        .error = "Invalid signature"
    END WITH
    EXIT
END IF

' Log incoming sync event
WITH sync_event = NEW OBJECT
    .direction = "inbound"
    .source = "crm"
    .event = event
    .record_type = record_type
    .record_id = record_id
    .timestamp = source_timestamp
    .received_at = NOW()
END WITH
INSERT "sync_events", sync_event

' Check for conflicts before processing
existing = FIND "local_data", "external_id=" + record_id
IF existing.updated_at > source_timestamp THEN
    ' Local data is newer - create conflict record
    WITH conflict = NEW OBJECT
        .record_id = record_id
        .local_timestamp = existing.updated_at
        .remote_timestamp = source_timestamp
        .local_data = existing
        .remote_data = data
        .status = "pending"
        .created_at = NOW()
    END WITH
    INSERT "sync_conflicts", conflict
    
    WITH result = NEW OBJECT
        .status = "conflict"
        .conflict_id = conflict.id
    END WITH
    EXIT
END IF

' Transform CRM data to local format
SELECT CASE record_type
    CASE "contact"
        WITH local_record = NEW OBJECT
            .external_id = record_id
            .external_source = "crm"
            .first_name = data.firstName
            .last_name = data.lastName
            .email = data.email
            .phone = data.phone
            .company = data.account.name
            .title = data.title
            .source = data.leadSource
            .status = data.status
            .updated_at = NOW()
            .synced_at = NOW()
        END WITH
        table_name = "contacts"
        
    CASE "account"
        WITH local_record = NEW OBJECT
            .external_id = record_id
            .external_source = "crm"
            .company_name = data.name
            .industry = data.industry
            .website = data.website
            .annual_revenue = data.annualRevenue
            .employee_count = data.numberOfEmployees
            .billing_address = data.billingAddress
            .updated_at = NOW()
            .synced_at = NOW()
        END WITH
        table_name = "accounts"
        
    CASE "opportunity"
        WITH local_record = NEW OBJECT
            .external_id = record_id
            .external_source = "crm"
            .name = data.name
            .account_id = data.accountId
            .amount = data.amount
            .stage = data.stageName
            .probability = data.probability
            .close_date = data.closeDate
            .updated_at = NOW()
            .synced_at = NOW()
        END WITH
        table_name = "opportunities"
        
    CASE ELSE
        WITH result = NEW OBJECT
            .status = 400
            .error = "Unknown record type: " + record_type
        END WITH
        EXIT
END SELECT

' Apply changes based on event type
SELECT CASE event
    CASE "created"
        INSERT table_name, local_record
        
    CASE "updated"
        UPDATE table_name, "external_id=" + record_id, local_record
        
    CASE "deleted"
        WITH soft_delete = NEW OBJECT
            .deleted_at = NOW()
            .deleted_from = "crm"
        END WITH
        UPDATE table_name, "external_id=" + record_id, soft_delete
END SELECT

' Update sync status
WITH sync_status = NEW OBJECT
    .record_id = record_id
    .record_type = record_type
    .last_sync = NOW()
    .sync_direction = "inbound"
    .status = "success"
END WITH
SAVE "sync_status", record_type + "_" + record_id, sync_status

WITH result = NEW OBJECT
    .status = "synced"
    .record_id = record_id
    .direction = "inbound"
END WITH
```

### CRM Outbound Sync

```basic
' crm-outbound.bas
' Push local changes to CRM

ON "contacts", "INSERT,UPDATE"

record = trigger.new_data
old_record = trigger.old_data

' Skip if this update came from CRM (prevent loops)
IF record.external_source = "crm" AND record.synced_at = record.updated_at THEN
    EXIT
END IF

' Check if record exists in CRM
IF record.external_id = "" THEN
    ' New record - create in CRM
    operation = "create"
    endpoint = "/api/contacts"
ELSE
    ' Existing record - update in CRM
    operation = "update"
    endpoint = "/api/contacts/" + record.external_id
END IF

' Transform to CRM format
WITH crm_data = NEW OBJECT
    .firstName = record.first_name
    .lastName = record.last_name
    .email = record.email
    .phone = record.phone
    .title = record.title
    .leadSource = record.source
END WITH

' Send to CRM
crm_api_key = GET BOT MEMORY "crm_api_key"
SET HEADER "Authorization", "Bearer " + crm_api_key
SET HEADER "Content-Type", "application/json"

IF operation = "create" THEN
    response = POST "https://api.crm.com" + endpoint, crm_data
    
    ' Store external ID
    WITH id_update = NEW OBJECT
        .external_id = response.id
        .external_source = "crm"
        .synced_at = NOW()
    END WITH
    UPDATE "contacts", "id=" + record.id, id_update
ELSE
    response = PUT "https://api.crm.com" + endpoint, crm_data
    
    WITH sync_update = NEW OBJECT
        .synced_at = NOW()
    END WITH
    UPDATE "contacts", "id=" + record.id, sync_update
END IF

' Log outbound sync
WITH sync_event = NEW OBJECT
    .direction = "outbound"
    .destination = "crm"
    .event = operation
    .record_type = "contact"
    .record_id = record.id
    .external_id = record.external_id
    .timestamp = NOW()
    .response_status = response.status
END WITH
INSERT "sync_events", sync_event
```

## Tool 2: ERP Sync

Integration with ERP systems for orders, inventory, and accounting.

```basic
' erp-sync.bas
' ERP system synchronization tool

WEBHOOK "erp-webhook"

event_type = body.eventType
entity = body.entity
entity_id = body.entityId
payload = body.payload

' Authenticate request
api_key = headers.x_api_key
expected_key = GET BOT MEMORY "erp_webhook_key"
IF api_key <> expected_key THEN
    WITH result = NEW OBJECT
        .status = 401
        .error = "Unauthorized"
    END WITH
    EXIT
END IF

' Route to appropriate handler
SELECT CASE entity
    CASE "salesOrder"
        CALL process_sales_order(event_type, entity_id, payload)
        
    CASE "purchaseOrder"
        CALL process_purchase_order(event_type, entity_id, payload)
        
    CASE "invoice"
        CALL process_invoice(event_type, entity_id, payload)
        
    CASE "inventory"
        CALL process_inventory_update(entity_id, payload)
        
    CASE "shipment"
        CALL process_shipment(event_type, entity_id, payload)
END SELECT

WITH result = NEW OBJECT
    .status = "processed"
    .entity = entity
    .entity_id = entity_id
END WITH

' --- Sub-procedures ---

SUB process_sales_order(event_type, order_id, data)
    WITH order = NEW OBJECT
        .erp_order_id = order_id
        .order_number = data.orderNumber
        .customer_id = data.customerId
        .order_date = data.orderDate
        .ship_date = data.requestedShipDate
        .status = data.status
        .subtotal = data.subtotal
        .tax = data.taxAmount
        .shipping = data.shippingAmount
        .total = data.total
        .currency = data.currency
        .updated_at = NOW()
    END WITH
    
    IF event_type = "created" THEN
        INSERT "orders", order
        
        ' Create line items
        FOR EACH item IN data.lineItems
            WITH line = NEW OBJECT
                .order_id = order_id
                .sku = item.sku
                .description = item.description
                .quantity = item.quantity
                .unit_price = item.unitPrice
                .discount = item.discount
                .total = item.lineTotal
            END WITH
            INSERT "order_lines", line
        NEXT item
        
        ' Notify sales team
        SEND MAIL "sales@company.com", "New Order: " + data.orderNumber, "Order total: $" + data.total
        
    ELSE IF event_type = "updated" THEN
        UPDATE "orders", "erp_order_id=" + order_id, order
        
        ' Check for status changes
        old_order = FIND "orders", "erp_order_id=" + order_id
        IF old_order.status <> data.status THEN
            ' Notify customer of status change
            customer = FIND "customers", "id=" + data.customerId
            SEND MAIL customer.email, "Order Update: " + data.orderNumber, "Your order status is now: " + data.status
        END IF
    END IF
END SUB

SUB process_inventory_update(sku, data)
    WITH inventory = NEW OBJECT
        .sku = sku
        .quantity_on_hand = data.qtyOnHand
        .quantity_available = data.qtyAvailable
        .quantity_reserved = data.qtyReserved
        .quantity_on_order = data.qtyOnOrder
        .warehouse = data.warehouse
        .bin_location = data.binLocation
        .last_count_date = data.lastCountDate
        .updated_at = NOW()
    END WITH
    
    SAVE "inventory", sku, inventory
    
    ' Check for low stock alert
    product = FIND "products", "sku=" + sku
    IF data.qtyAvailable < product.reorder_point THEN
        WITH alert = NEW OBJECT
            .sku = sku
            .product_name = product.name
            .current_qty = data.qtyAvailable
            .reorder_point = product.reorder_point
            .reorder_qty = product.reorder_quantity
            .created_at = NOW()
        END WITH
        INSERT "stock_alerts", alert
        
        SEND MAIL "purchasing@company.com", "Low Stock Alert: " + sku, "Product " + product.name + " is below reorder point. Current: " + data.qtyAvailable + ", Reorder at: " + product.reorder_point
    END IF
END SUB

SUB process_shipment(event_type, shipment_id, data)
    WITH shipment = NEW OBJECT
        .erp_shipment_id = shipment_id
        .order_id = data.orderId
        .carrier = data.carrier
        .tracking_number = data.trackingNumber
        .ship_date = data.shipDate
        .estimated_delivery = data.estimatedDelivery
        .status = data.status
        .updated_at = NOW()
    END WITH
    
    IF event_type = "created" THEN
        INSERT "shipments", shipment
        
        ' Notify customer
        order = FIND "orders", "erp_order_id=" + data.orderId
        customer = FIND "customers", "id=" + order.customer_id
        
        tracking_url = "https://track.carrier.com/" + data.trackingNumber
        
        SEND MAIL customer.email, "Your Order Has Shipped!", "Good news! Your order " + order.order_number + " has shipped.\n\nTracking: " + data.trackingNumber + "\nCarrier: " + data.carrier + "\nEstimated Delivery: " + data.estimatedDelivery + "\n\nTrack your package: " + tracking_url
        
    ELSE IF event_type = "updated" THEN
        UPDATE "shipments", "erp_shipment_id=" + shipment_id, shipment
        
        IF data.status = "delivered" THEN
            ' Update order status
            WITH order_update = NEW OBJECT
                .status = "delivered"
                .delivered_at = NOW()
            END WITH
            UPDATE "orders", "erp_order_id=" + data.orderId, order_update
        END IF
    END IF
END SUB
```

## Tool 3: Inventory Sync

Real-time inventory synchronization across multiple systems.

```basic
' inventory-sync.bas
' Real-time inventory synchronization

WEBHOOK "inventory-update"

source = body.source
sku = body.sku
warehouse = body.warehouse
adjustment_type = body.type
quantity = body.quantity
reason = body.reason
reference = body.reference

' Get current inventory
current = FIND "inventory", "sku=" + sku + " AND warehouse=" + warehouse

' Calculate new quantity based on adjustment type
SELECT CASE adjustment_type
    CASE "receipt"
        new_qty = current.quantity_on_hand + quantity
        
    CASE "shipment"
        new_qty = current.quantity_on_hand - quantity
        
    CASE "adjustment"
        new_qty = quantity
        
    CASE "transfer_out"
        new_qty = current.quantity_on_hand - quantity
        
    CASE "transfer_in"
        new_qty = current.quantity_on_hand + quantity
        
    CASE "count"
        new_qty = quantity
END SELECT

' Validate quantity
IF new_qty < 0 THEN
    WITH result = NEW OBJECT
        .status = 400
        .error = "Inventory cannot be negative"
        .current_qty = current.quantity_on_hand
        .attempted_qty = new_qty
    END WITH
    EXIT
END IF

' Update local inventory
WITH inv_update = NEW OBJECT
    .quantity_on_hand = new_qty
    .updated_at = NOW()
    .last_adjustment_type = adjustment_type
    .last_adjustment_source = source
END WITH
UPDATE "inventory", "sku=" + sku + " AND warehouse=" + warehouse, inv_update

' Log the transaction
WITH transaction = NEW OBJECT
    .sku = sku
    .warehouse = warehouse
    .adjustment_type = adjustment_type
    .quantity_before = current.quantity_on_hand
    .quantity_change = quantity
    .quantity_after = new_qty
    .reason = reason
    .reference = reference
    .source = source
    .created_at = NOW()
END WITH
INSERT "inventory_transactions", transaction

' Sync to other systems based on source
systems_to_sync = ["erp", "ecommerce", "pos", "wms"]

FOR EACH system IN systems_to_sync
    IF system <> source THEN
        CALL sync_inventory_to_system(system, sku, warehouse, new_qty)
    END IF
NEXT system

' Check for alerts
product = FIND "products", "sku=" + sku
IF new_qty <= product.reorder_point AND current.quantity_on_hand > product.reorder_point THEN
    ' Just crossed below reorder point
    WITH alert_msg = NEW OBJECT
        .text = "⚠️ *Low Stock Alert*\n\nSKU: " + sku + "\nProduct: " + product.name + "\nWarehouse: " + warehouse + "\nCurrent Qty: " + new_qty + "\nReorder Point: " + product.reorder_point
    END WITH
    POST "https://hooks.slack.com/services/xxx", alert_msg
END IF

IF new_qty = 0 THEN
    ' Out of stock
    WITH alert_msg = NEW OBJECT
        .text = "🚨 *Out of Stock*\n\nSKU: " + sku + "\nProduct: " + product.name + "\nWarehouse: " + warehouse
    END WITH
    POST "https://hooks.slack.com/services/xxx", alert_msg
    
    ' Disable on e-commerce
    CALL disable_product_ecommerce(sku)
END IF

WITH result = NEW OBJECT
    .status = "synced"
    .sku = sku
    .warehouse = warehouse
    .new_quantity = new_qty
END WITH

' --- Helper procedures ---

SUB sync_inventory_to_system(system, sku, warehouse, qty)
    SELECT CASE system
        CASE "erp"
            SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "erp_api_key"
            WITH erp_payload = NEW OBJECT
                .sku = sku
                .warehouseCode = warehouse
                .qtyOnHand = qty
            END WITH
            PUT "https://erp.company.com/api/inventory/" + sku, erp_payload
            
        CASE "ecommerce"
            SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "ecom_api_key"
            WITH ecom_payload = NEW OBJECT
                .inventory_quantity = qty
            END WITH
            PUT "https://api.shopify.com/products/" + sku + "/inventory", ecom_payload
            
        CASE "pos"
            SET HEADER "X-API-Key", GET BOT MEMORY "pos_api_key"
            WITH pos_payload = NEW OBJECT
                .item_id = sku
                .quantity = qty
                .location_id = warehouse
            END WITH
            POST "https://api.pos.com/inventory/update", pos_payload
            
        CASE "wms"
            SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "wms_api_key"
            WITH wms_payload = NEW OBJECT
                .sku = sku
                .location = warehouse
                .qty = qty
            END WITH
            PUT "https://wms.company.com/api/inventory", wms_payload
    END SELECT
    
    ' Log sync
    WITH sync_log = NEW OBJECT
        .system = system
        .sku = sku
        .warehouse = warehouse
        .quantity = qty
        .synced_at = NOW()
    END WITH
    INSERT "inventory_sync_log", sync_log
END SUB

SUB disable_product_ecommerce(sku)
    SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "ecom_api_key"
    WITH update_payload = NEW OBJECT
        .available = FALSE
        .inventory_policy = "deny"
    END WITH
    PUT "https://api.shopify.com/products/" + sku, update_payload
END SUB
```

## Tool 4: Conflict Resolver

Handle and resolve synchronization conflicts.

```basic
' conflict-resolver.bas
' Automated and manual sync conflict resolution

' Scheduled job to process conflicts
SET SCHEDULE "resolve-conflicts", "*/15 * * * *"

' Get pending conflicts
conflicts = FIND "sync_conflicts", "status=pending ORDER BY created_at ASC LIMIT 50"

FOR EACH conflict IN conflicts
    resolution = CALL attempt_auto_resolve(conflict)
    
    IF resolution.resolved THEN
        ' Apply the resolution
        CALL apply_resolution(conflict, resolution)
        
        ' Update conflict status
        WITH status_update = NEW OBJECT
            .status = "resolved"
            .resolution_type = "automatic"
            .resolution_details = resolution.details
            .resolved_at = NOW()
        END WITH
        UPDATE "sync_conflicts", "id=" + conflict.id, status_update
    ELSE
        ' Escalate for manual review
        IF conflict.escalated_at = "" THEN
            CALL escalate_conflict(conflict)
            
            WITH escalate_update = NEW OBJECT
                .status = "escalated"
                .escalated_at = NOW()
            END WITH
            UPDATE "sync_conflicts", "id=" + conflict.id, escalate_update
        END IF
    END IF
NEXT conflict

' --- Functions ---

FUNCTION attempt_auto_resolve(conflict)
    WITH result = NEW OBJECT
        .resolved = FALSE
        .winner = ""
        .details = ""
    END WITH
    
    ' Rule 1: Timestamp-based (most recent wins)
    time_diff = DATEDIFF(conflict.local_timestamp, conflict.remote_timestamp, "second")
    IF ABS(time_diff) > 60 THEN
        ' Clear winner by timestamp
        IF conflict.local_timestamp > conflict.remote_timestamp THEN
            result.resolved = TRUE
            result.winner = "local"
            result.details = "Local data is " + ABS(time_diff) + " seconds newer"
        ELSE
            result.resolved = TRUE
            result.winner = "remote"
            result.details = "Remote data is " + ABS(time_diff) + " seconds newer"
        END IF
        RETURN result
    END IF
    
    ' Rule 2: Field-level merge (non-conflicting changes)
    local_changes = CALL get_changed_fields(conflict.original_data, conflict.local_data)
    remote_changes = CALL get_changed_fields(conflict.original_data, conflict.remote_data)
    
    ' Check if changes affect different fields
    overlap = FALSE
    FOR EACH field IN local_changes
        IF INSTR(remote_changes, field) > 0 THEN
            overlap = TRUE
            EXIT FOR
        END IF
    NEXT field
    
    IF NOT overlap THEN
        ' Can merge without conflict
        result.resolved = TRUE
        result.winner = "merge"
        result.details = "Field-level merge: local changed [" + local_changes + "], remote changed [" + remote_changes + "]"
        RETURN result
    END IF
    
    ' Rule 3: Source priority
    priority_source = GET BOT MEMORY "sync_priority_source"
    IF priority_source <> "" THEN
        IF conflict.source = priority_source THEN
            result.resolved = TRUE
            result.winner = "remote"
            result.details = "Priority source rule: " + priority_source + " wins"
        ELSE
            result.resolved = TRUE
            result.winner = "local"
            result.details = "Non-priority source: local wins"
        END IF
        RETURN result
    END IF
    
    ' Cannot auto-resolve
    result.details = "Manual resolution required: same fields modified within 60 seconds"
    RETURN result
END FUNCTION

SUB apply_resolution(conflict, resolution)
    SELECT CASE resolution.winner
        CASE "local"
            ' Push local data to remote
            CALL sync_to_remote(conflict.record_type, conflict.record_id, conflict.local_data)
            
        CASE "remote"
            ' Apply remote data locally
            UPDATE conflict.record_type, "id=" + conflict.record_id, conflict.remote_data
            
        CASE "merge"
            ' Merge both changes
            merged_data = CALL merge_records(conflict.original_data, conflict.local_data, conflict.remote_data)
            UPDATE conflict.record_type, "id=" + conflict.record_id, merged_data
            CALL sync_to_remote(conflict.record_type, conflict.record_id, merged_data)
    END SELECT
END SUB

SUB escalate_conflict(conflict)
    ' Send notification to data admin
    WITH notification = NEW OBJECT
        .conflict_id = conflict.id
        .record_type = conflict.record_type
        .record_id = conflict.record_id
        .local_timestamp = conflict.local_timestamp
        .remote_timestamp = conflict.remote_timestamp
        .local_summary = CALL summarize_data(conflict.local_data)
        .remote_summary = CALL summarize_data(conflict.remote_data)
    END WITH
    
    email_body = "A sync conflict requires manual resolution.\n\n"
    email_body = email_body + "Record: " + conflict.record_type + " #" + conflict.record_id + "\n"
    email_body = email_body + "Local changes: " + notification.local_summary + "\n"
    email_body = email_body + "Remote changes: " + notification.remote_summary + "\n\n"
    email_body = email_body + "Please review at: https://admin.company.com/conflicts/" + conflict.id
    
    SEND MAIL "data-admin@company.com", "Sync Conflict: " + conflict.record_type + " #" + conflict.record_id, email_body
    
    ' Also post to Slack
    WITH slack_msg = NEW OBJECT
        .text = "⚠️ *Sync Conflict Requires Review*\n\nRecord: " + conflict.record_type + " #" + conflict.record_id + "\n<https://admin.company.com/conflicts/" + conflict.id + "|Review Now>"
    END WITH
    POST "https://hooks.slack.com/services/xxx", slack_msg
END SUB
```

## Tool 5: Sync Monitor

Monitor sync health and generate alerts.

```basic
' sync-monitor.bas
' Data sync health monitoring

SET SCHEDULE "sync-health-check", "*/5 * * * *"

' Check sync lag for each integration
integrations = ["crm", "erp", "ecommerce", "wms"]

WITH health_report = NEW OBJECT
    .timestamp = NOW()
    .status = "healthy"
    .issues = []
END WITH

FOR EACH integration IN integrations
    ' Get latest sync event
    latest = FIND "sync_events", "source=" + integration + " OR destination=" + integration + " ORDER BY timestamp DESC LIMIT 1"
    
    lag_minutes = DATEDIFF(latest.timestamp, NOW(), "minute")
    
    WITH integration_status = NEW OBJECT
        .name = integration
        .last_sync = latest.timestamp
        .lag_minutes = lag_minutes
        .status = "ok"
    END WITH
    
    ' Check for concerning lag
    max_lag = GET BOT MEMORY "max_sync_lag_" + integration
    IF max_lag = "" THEN max_lag = 30 END IF
    
    IF lag_minutes > max_lag THEN
        integration_status.status = "warning"
        health_report.status = "degraded"
        
        WITH issue = NEW OBJECT
            .integration = integration
            .type = "sync_lag"
            .message = integration + " sync lag: " + lag_minutes + " minutes (max: " + max_lag + ")"
        END WITH
        health_report.issues.ADD(issue)
    END IF
    
    ' Check for recent errors
    recent_errors = FIND "sync_events", "source=" + integration + " AND status='error' AND timestamp > DATEADD(NOW(), -1, 'hour')"
    error_count = UBOUND(recent_errors)
    
    IF error_count > 5 THEN
        integration_status.status = "error"
        health_report.status = "unhealthy"
        
        WITH issue = NEW OBJECT
            .integration = integration
            .type = "high_error_rate"
            .message = integration + " has " + error_count + " errors in the last hour"
        END WITH
        health_report.issues.ADD(issue)
    END IF
    
    integration_status.error_count_1h = error_count
NEXT integration

' Check pending conflicts
pending_conflicts = AGGREGATE "COUNT", "sync_conflicts", "status='pending'"
escalated_conflicts = AGGREGATE "COUNT", "sync_conflicts", "status='escalated'"

IF pending_conflicts > 100 THEN
    health_report.status = "degraded"
    WITH issue = NEW OBJECT
        .type = "pending_conflicts"
        .message = pending_conflicts + " sync conflicts pending resolution"
    END WITH
    health_report.issues.ADD(issue)
END IF

' Check queue depth
queue_depth = AGGREGATE "COUNT", "sync_queue", "status='pending'"
IF queue_depth > 1000 THEN
    health_report.status = "degraded"
    WITH issue = NEW OBJECT
        .type = "queue_backlog"
        .message = "Sync queue backlog: " + queue_depth + " items"
    END WITH
    health_report.issues.ADD(issue)
END IF

' Store health report
INSERT "sync_health_reports", health_report

' Alert if unhealthy
IF health_report.status = "unhealthy" THEN
    alert_message = "🚨 *Data Sync Unhealthy*\n\n"
    FOR EACH issue IN health_report.issues
        alert_message = alert_message + "• " + issue.message + "\n"
    NEXT issue
    
    ' Slack alert
    WITH slack_alert = NEW OBJECT
        .text = alert_message
        .channel = "#ops-alerts"
    END WITH
    POST "https://hooks.slack.com/services/xxx", slack_alert
    
    ' PagerDuty for critical
    WITH pagerduty = NEW OBJECT
        .routing_key = GET BOT MEMORY "pagerduty_key"
        .event_action = "trigger"
        .payload.summary = "Data sync system unhealthy"
        .payload.severity = "critical"
        .payload.source = "sync-monitor"
    END WITH
    POST "https://events.pagerduty.com/v2/enqueue", pagerduty
    
ELSE IF health_report.status = "degraded" THEN
    alert_message = "⚠️ *Data Sync Degraded*\n\n"
    FOR EACH issue IN health_report.issues
        alert_message = alert_message + "• " + issue.message + "\n"
    NEXT issue
    
    WITH slack_alert = NEW OBJECT
        .text = alert_message
        .channel = "#ops-alerts"
    END WITH
    POST "https://hooks.slack.com/services/xxx", slack_alert
END IF

' Generate dashboard data
WITH dashboard = NEW OBJECT
    .timestamp = NOW()
    .overall_status = health_report.status
    .integrations = integration_statuses
    .pending_conflicts = pending_conflicts
    .escalated_conflicts = escalated_conflicts
    .queue_depth = queue_depth
    .events_last_hour = AGGREGATE "COUNT", "sync_events", "timestamp > DATEADD(NOW(), -1, 'hour')"
    .errors_last_hour = AGGREGATE "COUNT", "sync_events", "status='error' AND timestamp > DATEADD(NOW(), -1, 'hour')"
END WITH

SAVE "sync_dashboard", "current", dashboard
```

## Tool 6: Bulk Sync

Initial data load and bulk synchronization.

```basic
' bulk-sync.bas
' Bulk data synchronization tool

WEBHOOK "bulk-sync"

source_system = body.source
target_system = body.target
entity_type = body.entity
batch_size = body.batch_size
offset = body.offset

IF batch_size = "" THEN batch_size = 100 END IF
IF offset = "" THEN offset = 0 END IF

' Create sync job
job_id = "SYNC-" + FORMAT(NOW(), "YYYYMMDDHHmmss")

WITH job = NEW OBJECT
    .id = job_id
    .source = source_system
    .target = target_system
    .entity_type = entity_type
    .status = "running"
    .total_records = 0
    .processed_records = 0
    .error_count = 0
    .started_at = NOW()
END WITH
INSERT "sync_jobs", job

' Fetch data from source
SET HEADER "Authorization", "Bearer " + GET BOT MEMORY source_system + "_api_key"

has_more = TRUE
total_processed = 0
total_errors = 0

WHILE has_more
    source_url = CALL build_source_url(source_system, entity_type, batch_size, offset)
    response = GET source_url
    
    records = response.data
    has_more = response.has_more
    
    IF UBOUND(records) = 0 THEN
        has_more = FALSE
    ELSE
        FOR EACH record IN records
            ' Transform record
            transformed = CALL transform_record(record, source_system, target_system, entity_type)
            
            ' Send to target
            success = CALL send_to_target(target_system, entity_type, transformed)
            
            IF success THEN
                total_processed = total_processed + 1
            ELSE
                total_errors = total_errors + 1
                
                ' Log error
                WITH error_log = NEW OBJECT
                    .job_id = job_id
                    .record_id = record.id
                    .error = "Failed to sync to " + target_system
                    .created_at = NOW()
                END WITH
                INSERT "sync_errors", error_log
            END IF
            
            ' Update progress every 100 records
            IF (total_processed + total_errors) MOD 100 = 0 THEN
                WITH progress = NEW OBJECT
                    .processed_records = total_processed
                    .error_count = total_errors
                    .updated_at = NOW()
                END WITH
                UPDATE "sync_jobs", "id=" + job_id, progress
            END IF
        NEXT record
        
        offset = offset + batch_size
    END IF
WEND

' Finalize job
WITH final_update = NEW OBJECT
    .status = "completed"
    .total_records = total_processed + total_errors
    .processed_records = total_processed
    .error_count = total_errors
    .completed_at = NOW()
END WITH
UPDATE "sync_jobs", "id=" + job_id, final_update

' Send completion notification
completion_msg = "Bulk sync completed\n\n"
completion_msg = completion_msg + "Job ID: " + job_id + "\n"
completion_msg = completion_msg + "Source: " + source_system + "\n"
completion_msg = completion_msg + "Target: " + target_system + "\n"
completion_msg = completion_msg + "Entity: " + entity_type + "\n"
completion_msg = completion_msg + "Processed: " + total_processed + "\n"
completion_msg = completion_msg + "Errors: " + total_errors

SEND MAIL "data-admin@company.com", "Bulk Sync Complete: " + job_id, completion_msg

WITH result = NEW OBJECT
    .status = "completed"
    .job_id = job_id
    .processed = total_processed
    .errors = total_errors
END WITH
```

## Configuration

Store sync configuration in bot memory:

```basic
' Configure sync settings
SET BOT MEMORY "crm_api_key", "your-crm-api-key"
SET BOT MEMORY "erp_api_key", "your-erp-api-key"
SET BOT MEMORY "ecom_api_key", "your-ecommerce-api-key"
SET BOT MEMORY "max_sync_lag_crm", "30"
SET BOT MEMORY "max_sync_lag_erp", "15"
SET BOT MEMORY "sync_priority_source", "erp"
```

## See Also

- [WEBHOOK](./keyword-webhook.md) - Creating webhook endpoints
- [ON](./keyword-on.md) - Database trigger events
- [SET SCHEDULE](./keyword-set-schedule.md) - Scheduled tasks
- [Data Operations](./keywords.md#database--data-operations) - Database keywords
- [Consolidated Examples](./examples-consolidated.md) - More complete examples