' Data Sync Bot - Demonstrates new data operations keywords
' This template shows how to use MERGE, FILTER, AGGREGATE, JOIN, and other data keywords

' ============================================================================
' WEBHOOK: External systems can trigger this sync via HTTP POST
' Endpoint: /api/office/webhook/data-sync
' ============================================================================
WEBHOOK "data-sync"

TALK "Starting data synchronization..."

' ============================================================================
' EXAMPLE 1: Fetch and merge external data
' ============================================================================

' Fetch customers from external CRM API
SET_HEADER "Authorization", "Bearer " + GET_BOT_MEMORY("crm_api_key")
SET_HEADER "Content-Type", "application/json"

external_customers = GET "https://api.crm.example.com/customers"

' Merge external data with local database using email as the key
merge_result = MERGE "customers", external_customers, "email"

TALK "Customer sync complete: " + merge_result.inserted + " new, " + merge_result.updated + " updated"

CLEAR_HEADERS

' ============================================================================
' EXAMPLE 2: Data transformation with MAP and FILL
' ============================================================================

' Read raw order data
orders = FIND "orders.xlsx", "status=pending"

' Map field names to match our internal format
mapped_orders = MAP orders, "customerName->customer, orderDate->date, totalAmount->amount"

' Fill a report template with the data
report_template = #{
    "title": "Order Report for {{customer}}",
    "summary": "Order placed on {{date}} for ${{amount}}",
    "processed_at": NOW()
}

report_data = FILL mapped_orders, report_template

' ============================================================================
' EXAMPLE 3: Filtering and aggregation
' ============================================================================

' Get all sales data
sales = FIND "sales.xlsx"

' Filter for this month's sales
this_month = FILTER sales, "date>=" + FORMAT(DATEADD(TODAY(), "month", -1), "yyyy-MM-dd")

' Filter high-value transactions
high_value = FILTER this_month, "amount>1000"

' Calculate aggregates
total_sales = AGGREGATE "SUM", this_month, "amount"
average_sale = AGGREGATE "AVG", this_month, "amount"
sale_count = AGGREGATE "COUNT", this_month, "id"
max_sale = AGGREGATE "MAX", this_month, "amount"
min_sale = AGGREGATE "MIN", this_month, "amount"

TALK "This month's statistics:"
TALK "- Total sales: $" + total_sales
TALK "- Average sale: $" + average_sale
TALK "- Number of sales: " + sale_count
TALK "- Largest sale: $" + max_sale
TALK "- Smallest sale: $" + min_sale

' ============================================================================
' EXAMPLE 4: Joining datasets
' ============================================================================

' Load related data
customers = FIND "customers.xlsx"
orders = FIND "orders.xlsx"
products = FIND "products.xlsx"

' Join orders with customer information
orders_with_customers = JOIN orders, customers, "customer_id"

' Now join with product data
complete_orders = JOIN orders_with_customers, products, "product_id"

' ============================================================================
' EXAMPLE 5: Grouping and pivoting
' ============================================================================

' Group sales by salesperson
sales_by_rep = GROUP_BY sales, "salesperson"

' Create pivot table: sales by month
monthly_pivot = PIVOT sales, "month", "amount"

TALK "Monthly sales pivot created with " + UBOUND(monthly_pivot) + " rows"

' ============================================================================
' EXAMPLE 6: Database CRUD operations
' ============================================================================

' Insert a new sync log entry
log_entry = #{
    "sync_type": "full",
    "started_at": NOW(),
    "records_processed": sale_count,
    "status": "completed"
}
insert_result = INSERT "sync_logs", log_entry
TALK "Created sync log: " + insert_result.id

' Update existing records
rows_updated = UPDATE "customers", "last_sync<" + FORMAT(DATEADD(TODAY(), "day", -7), "yyyy-MM-dd"), #{
    "needs_refresh": true,
    "updated_at": NOW()
}
TALK "Marked " + rows_updated + " customers for refresh"

' Save with upsert (insert or update)
summary = #{
    "date": TODAY(),
    "total_sales": total_sales,
    "order_count": sale_count,
    "sync_status": "complete"
}
SAVE "daily_summaries", FORMAT(TODAY(), "yyyy-MM-dd"), summary

' ============================================================================
' EXAMPLE 7: File operations for reporting
' ============================================================================

' Generate a report
report_content = "Daily Sales Report - " + TODAY() + "\n\n"
report_content = report_content + "Total Sales: $" + total_sales + "\n"
report_content = report_content + "Transactions: " + sale_count + "\n"
report_content = report_content + "Average: $" + average_sale + "\n"

' Write report to file
WRITE "reports/daily/" + FORMAT(TODAY(), "yyyy-MM-dd") + ".txt", report_content

' List all reports
all_reports = LIST "reports/daily/"
TALK "Total reports in archive: " + UBOUND(all_reports)

' ============================================================================
' EXAMPLE 8: HTTP POST to external system
' ============================================================================

' Send summary to analytics platform
analytics_payload = #{
    "event": "daily_sync_complete",
    "data": #{
        "date": TODAY(),
        "total_sales": total_sales,
        "transaction_count": sale_count,
        "new_customers": merge_result.inserted
    }
}

SET_HEADER "X-API-Key", GET_BOT_MEMORY("analytics_api_key")
analytics_response = POST "https://analytics.example.com/events", analytics_payload
CLEAR_HEADERS

' ============================================================================
' EXAMPLE 9: Cleanup old data
' ============================================================================

' Delete old sync logs (older than 30 days)
cutoff_date = FORMAT(DATEADD(TODAY(), "day", -30), "yyyy-MM-dd")
deleted_logs = DELETE "sync_logs", "created_at<" + cutoff_date

IF deleted_logs > 0 THEN
    TALK "Cleaned up " + deleted_logs + " old sync log entries"
END IF

' ============================================================================
' Return webhook response
' ============================================================================

result = #{
    "status": "success",
    "timestamp": NOW(),
    "summary": #{
        "customers_synced": merge_result.inserted + merge_result.updated,
        "sales_processed": sale_count,
        "total_revenue": total_sales,
        "logs_cleaned": deleted_logs
    }
}

TALK "Data synchronization complete!"
