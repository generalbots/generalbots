' API Integration Bot - Demonstrates HTTP & API operations keywords
' This template shows how to use POST, PUT, PATCH, DELETE_HTTP, GRAPHQL, SOAP, and SET_HEADER

' ============================================================================
' WEBHOOK: External systems can trigger API operations via HTTP POST
' Endpoint: /api/office/webhook/api-gateway
' ============================================================================
WEBHOOK "api-gateway"

TALK "API Integration Bot initialized..."

' ============================================================================
' EXAMPLE 1: Basic REST API calls with authentication
' ============================================================================

' Set up authentication headers (reused for subsequent requests)
api_key = GET_BOT_MEMORY("external_api_key")
SET_HEADER "Authorization", "Bearer " + api_key
SET_HEADER "Content-Type", "application/json"
SET_HEADER "X-Client-ID", "office-bot"

' GET request (using existing GET keyword)
users = GET "https://api.example.com/users"
TALK "Retrieved " + UBOUND(users) + " users from API"

' ============================================================================
' EXAMPLE 2: POST - Create new resources
' ============================================================================

' Create a new customer
new_customer = #{
    "name": "John Doe",
    "email": "john.doe@example.com",
    "phone": "+1-555-0123",
    "company": "Acme Corp",
    "tier": "enterprise"
}

create_response = POST "https://api.example.com/customers", new_customer

IF create_response.status = 201 THEN
    TALK "Customer created successfully with ID: " + create_response.data.id
    SET_BOT_MEMORY "last_customer_id", create_response.data.id
ELSE
    TALK "Failed to create customer: " + create_response.data.error
END IF

' ============================================================================
' EXAMPLE 3: PUT - Full resource update
' ============================================================================

' Update entire customer record
customer_id = GET_BOT_MEMORY("last_customer_id")

updated_customer = #{
    "name": "John Doe",
    "email": "john.doe@newdomain.com",
    "phone": "+1-555-9999",
    "company": "Acme Corp International",
    "tier": "enterprise",
    "status": "active",
    "updated_at": NOW()
}

put_response = PUT "https://api.example.com/customers/" + customer_id, updated_customer

IF put_response.status = 200 THEN
    TALK "Customer fully updated"
END IF

' ============================================================================
' EXAMPLE 4: PATCH - Partial resource update
' ============================================================================

' Update only specific fields
partial_update = #{
    "tier": "premium",
    "notes": "Upgraded from enterprise plan"
}

patch_response = PATCH "https://api.example.com/customers/" + customer_id, partial_update

IF patch_response.status = 200 THEN
    TALK "Customer tier upgraded to premium"
END IF

' ============================================================================
' EXAMPLE 5: DELETE_HTTP - Remove resources
' ============================================================================

' Delete a temporary resource
temp_resource_id = "temp-12345"
delete_response = DELETE_HTTP "https://api.example.com/temp-files/" + temp_resource_id

IF delete_response.status = 204 OR delete_response.status = 200 THEN
    TALK "Temporary resource deleted"
END IF

' ============================================================================
' EXAMPLE 6: Working with multiple APIs
' ============================================================================

' Clear headers and set new ones for different API
CLEAR_HEADERS

' Stripe-style API authentication
SET_HEADER "Authorization", "Basic " + GET_BOT_MEMORY("stripe_api_key")
SET_HEADER "Content-Type", "application/x-www-form-urlencoded"

' Create a payment intent
payment_data = #{
    "amount": 2999,
    "currency": "usd",
    "customer": "cus_abc123",
    "description": "Order #12345"
}

payment_response = POST "https://api.stripe.com/v1/payment_intents", payment_data

IF payment_response.status = 200 THEN
    payment_intent_id = payment_response.data.id
    client_secret = payment_response.data.client_secret
    TALK "Payment intent created: " + payment_intent_id
END IF

CLEAR_HEADERS

' ============================================================================
' EXAMPLE 7: GraphQL API calls
' ============================================================================

' Query users with GraphQL
graphql_query = "query GetUsers($limit: Int!, $status: String) { users(first: $limit, status: $status) { id name email role createdAt } }"

graphql_vars = #{
    "limit": 10,
    "status": "active"
}

graphql_response = GRAPHQL "https://api.example.com/graphql", graphql_query, graphql_vars

IF graphql_response.status = 200 THEN
    users = graphql_response.data.data.users
    FOR EACH user IN users
        TALK "User: " + user.name + " (" + user.email + ")"
    NEXT user
END IF

' GraphQL mutation example
mutation_query = "mutation CreateTask($input: TaskInput!) { createTask(input: $input) { id title status assignee { name } } }"

mutation_vars = #{
    "input": #{
        "title": "Review quarterly report",
        "description": "Review and approve Q4 financial report",
        "assigneeId": "user-456",
        "dueDate": FORMAT(DATEADD(TODAY(), "day", 7), "yyyy-MM-dd"),
        "priority": "high"
    }
}

mutation_response = GRAPHQL "https://api.example.com/graphql", mutation_query, mutation_vars

IF mutation_response.status = 200 THEN
    task = mutation_response.data.data.createTask
    TALK "Task created: " + task.title + " (assigned to " + task.assignee.name + ")"
END IF

' ============================================================================
' EXAMPLE 8: SOAP API calls (Legacy system integration)
' ============================================================================

' Call a SOAP web service for legacy ERP integration
soap_params = #{
    "customerNumber": "CUST-001",
    "orderDate": FORMAT(TODAY(), "yyyy-MM-dd"),
    "productCode": "PRD-12345",
    "quantity": 5
}

soap_response = SOAP "https://erp.legacy.example.com/OrderService.wsdl", "CreateOrder", soap_params

IF soap_response.status = 200 THEN
    order_number = soap_response.data.raw
    TALK "Legacy order created: " + order_number
END IF

' Another SOAP call - Get inventory status
inventory_params = #{
    "warehouseCode": "WH-01",
    "productCode": "PRD-12345"
}

inventory_response = SOAP "https://erp.legacy.example.com/InventoryService.wsdl", "GetStock", inventory_params
TALK "Current stock level retrieved from legacy system"

' ============================================================================
' EXAMPLE 9: Chained API calls (workflow)
' ============================================================================

TALK "Starting order fulfillment workflow..."

' Step 1: Create order in main system
SET_HEADER "Authorization", "Bearer " + api_key

order_data = #{
    "customer_id": customer_id,
    "items": [
        #{ "sku": "WIDGET-001", "quantity": 2, "price": 29.99 },
        #{ "sku": "GADGET-002", "quantity": 1, "price": 49.99 }
    ],
    "shipping_address": #{
        "street": "123 Main St",
        "city": "Anytown",
        "state": "CA",
        "zip": "90210"
    }
}

order_response = POST "https://api.example.com/orders", order_data
order_id = order_response.data.id

' Step 2: Request shipping quote
shipping_request = #{
    "origin_zip": "10001",
    "destination_zip": "90210",
    "weight_lbs": 5,
    "dimensions": #{ "length": 12, "width": 8, "height": 6 }
}

shipping_response = POST "https://api.shipping.com/quotes", shipping_request
shipping_cost = shipping_response.data.rate

' Step 3: Update order with shipping info
shipping_update = #{
    "shipping_method": shipping_response.data.service,
    "shipping_cost": shipping_cost,
    "estimated_delivery": shipping_response.data.estimated_delivery
}

PATCH "https://api.example.com/orders/" + order_id, shipping_update

' Step 4: Notify warehouse
warehouse_notification = #{
    "order_id": order_id,
    "priority": "standard",
    "ship_by": FORMAT(DATEADD(TODAY(), "day", 2), "yyyy-MM-dd")
}

POST "https://api.warehouse.example.com/pick-requests", warehouse_notification

TALK "Order fulfillment workflow complete for order: " + order_id

CLEAR_HEADERS

' ============================================================================
' EXAMPLE 10: Error handling and retries
' ============================================================================

' Attempt API call with error handling
max_retries = 3
retry_count = 0
success = false

WHILE retry_count < max_retries AND success = false
    SET_HEADER "Authorization", "Bearer " + api_key

    health_check = GET "https://api.example.com/health"

    IF health_check.status = 200 THEN
        success = true
        TALK "API health check passed"
    ELSE
        retry_count = retry_count + 1
        TALK "API check failed, attempt " + retry_count + " of " + max_retries
        WAIT 2
    END IF

    CLEAR_HEADERS
WEND

IF success = false THEN
    TALK "API is currently unavailable after " + max_retries + " attempts"
END IF

' ============================================================================
' Return webhook response with summary
' ============================================================================

result = #{
    "status": "success",
    "timestamp": NOW(),
    "operations_completed": #{
        "customers_created": 1,
        "orders_processed": 1,
        "api_calls_made": 12
    },
    "integrations_tested": ["REST", "GraphQL", "SOAP"]
}

TALK "API Integration examples completed!"
