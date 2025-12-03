# LLM Tools Template (llm-tools.gbai)

A General Bots template demonstrating how to create and register custom tools (functions) that LLMs can call during conversations.

---

## Overview

The LLM Tools template shows how to extend your bot's capabilities by creating tools that the AI can invoke automatically. Tools enable the LLM to perform actions like database lookups, API calls, calculations, and more—all triggered naturally through conversation.

## Features

- **Custom Tool Registration** - Define tools the LLM can call
- **Parameter Validation** - Type-safe tool parameters with descriptions
- **Knowledge Base Integration** - Combine tools with RAG
- **Natural Interaction** - Users don't need to know tool names
- **Extensible Architecture** - Easy to add new tools

---

## Package Structure

```
llm-tools.gbai/
├── llm-tools.gbdata/       # Data files for tools
│   └── products.csv        # Product catalog
├── llm-tools.gbdialog/
│   ├── start.bas           # Main dialog with tool registration
│   └── get-price.bas       # Example tool implementation
├── llm-tools.gbkb/         # Knowledge base
│   └── products.gbkb/      # Product documentation
└── llm-tools.gbot/
    └── config.csv          # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `start.bas` | Registers tools, configures context, and sets system prompt |
| `get-price.bas` | Example tool that looks up product prices |

---

## How Tools Work

### 1. Tool Registration

In `start.bas`, tools are registered with `ADD TOOL`:

```basic
ADD TOOL "get-price"

USE KB "products.gbkb"

CLEAR SUGGESTIONS

ADD SUGGESTION "price" AS "Check product price"
ADD SUGGESTION "products" AS "View products"
ADD SUGGESTION "help" AS "How to use"

BEGIN TALK
**Product Assistant**

I can help you check product prices and information.

Just ask me about any product and I'll look it up for you.
END TALK

BEGIN SYSTEM PROMPT
You are a product assistant with access to internal tools.

When get-price returns -1, the product does not exist.
When asked about a price, use the get-price tool and return the result.

Do not expose tool names to users - just act on their requests naturally.
END SYSTEM PROMPT
```

### 2. Tool Implementation

Each tool is a separate `.bas` file with `PARAM` and `DESCRIPTION`:

```basic
PARAM product AS STRING LIKE "fax" DESCRIPTION "Name of the product to get price for"

DESCRIPTION "Get the price of a product by name from the product catalog"

productRecord = FIND "products.csv", "name = ${product}"

IF productRecord THEN
    RETURN productRecord.price
ELSE
    RETURN -1
END IF
```

### 3. LLM Invocation

When a user asks "How much is the fax machine?", the LLM:
1. Recognizes this requires price information
2. Calls `get-price` with `product="fax"`
3. Receives the price (or -1 if not found)
4. Formulates a natural response

---

## Sample Conversation

```
User: How much does the wireless mouse cost?

Bot:  [Calls get-price with product="wireless mouse"]
      The wireless mouse is $29.99.

User: What about the keyboard?

Bot:  [Calls get-price with product="keyboard"]
      The standard keyboard is $49.99. We also have a 
      mechanical keyboard for $89.99.

User: Is the laptop in stock and what's the price?

Bot:  [Calls get-price with product="laptop"]
      [Calls check-inventory with sku="LAPTOP-001"]
      The laptop is $999. Good news - we have 12 units 
      available in our main warehouse!

User: I want 3 monitors with my 20% employee discount

Bot:  [Calls get-price with product="monitor"]
      [Calls calculate-discount with price=299, percent=20]
      Each monitor is $299, but with your 20% employee 
      discount, you'll pay $239.20 each. 
      For 3 monitors: $717.60 (saving $179.40)!
```

---

## Creating Custom Tools

### Tool Anatomy

Every tool needs:
- **PARAM declarations** - Input parameters with types and examples
- **DESCRIPTION** - What the tool does (for LLM understanding)
- **Implementation** - The actual logic
- **RETURN** - The output value

### Parameter Types

| Type | Description | Example |
|------|-------------|---------|
| `STRING` | Text input | `PARAM name AS STRING LIKE "John"` |
| `NUMBER` | Numeric input | `PARAM quantity AS NUMBER LIKE 5` |
| `INTEGER` | Whole numbers | `PARAM count AS INTEGER LIKE 10` |
| `BOOLEAN` | True/false | `PARAM active AS BOOLEAN` |
| `DATE` | Date values | `PARAM start AS DATE LIKE "2024-01-15"` |
| `EMAIL` | Email addresses | `PARAM email AS EMAIL` |
| `PHONE` | Phone numbers | `PARAM phone AS PHONE` |
| `OBJECT` | JSON objects | `PARAM data AS OBJECT` |

---

## Example Tools

### Database Lookup Tool

```basic
' lookup-customer.bas
PARAM customer_id AS STRING LIKE "CUST-001" DESCRIPTION "Customer ID to look up"

DESCRIPTION "Retrieve customer information by ID"

customer = FIND "customers.csv", "id = ${customer_id}"

IF customer THEN
    WITH result
        name = customer.name
        email = customer.email
        status = customer.status
        since = customer.created_at
    END WITH
    RETURN result
ELSE
    RETURN NULL
END IF
```

### Calculation Tool

```basic
' calculate-discount.bas
PARAM original_price AS NUMBER LIKE 100 DESCRIPTION "Original product price"
PARAM discount_percent AS NUMBER LIKE 15 DESCRIPTION "Discount percentage"

DESCRIPTION "Calculate the final price after applying a discount"

discount_amount = original_price * (discount_percent / 100)
final_price = original_price - discount_amount

WITH result
    original = original_price
    discount = discount_amount
    final = final_price
    savings = discount_percent + "% off"
END WITH

RETURN result
```

### API Integration Tool

```basic
' check-inventory.bas
PARAM sku AS STRING LIKE "SKU-12345" DESCRIPTION "Product SKU to check"
PARAM warehouse AS STRING LIKE "main" DESCRIPTION "Warehouse location" OPTIONAL

DESCRIPTION "Check real-time inventory levels for a product"

IF NOT warehouse THEN
    warehouse = "main"
END IF

SET HEADER "Authorization" AS "Bearer " + GET ENV "INVENTORY_API_KEY"
response = GET "https://api.inventory.com/stock/" + sku + "?warehouse=" + warehouse

IF response.error THEN
    RETURN {"available": false, "error": response.error}
END IF

WITH result
    sku = sku
    available = response.quantity > 0
    quantity = response.quantity
    warehouse = warehouse
    last_updated = response.timestamp
END WITH

RETURN result
```

### Email Sending Tool

```basic
' send-notification.bas
PARAM recipient AS EMAIL LIKE "user@example.com" DESCRIPTION "Email recipient"
PARAM subject AS STRING LIKE "Order Confirmation" DESCRIPTION "Email subject"
PARAM message AS STRING DESCRIPTION "Email body content"

DESCRIPTION "Send an email notification to a customer"

SEND EMAIL recipient, subject, message

WITH result
    sent = true
    recipient = recipient
    timestamp = NOW()
END WITH

RETURN result
```

---

## Tool Registration Patterns

### Single Tool

```basic
ADD TOOL "get-price"
```

### Multiple Tools

```basic
ADD TOOL "get-price"
ADD TOOL "check-inventory"
ADD TOOL "lookup-customer"
ADD TOOL "calculate-discount"
ADD TOOL "send-notification"
```

### Conditional Tools

```basic
user_role = GET SESSION "user_role"

ADD TOOL "get-price"
ADD TOOL "check-inventory"

IF user_role = "admin" THEN
    ADD TOOL "update-price"
    ADD TOOL "delete-product"
END IF
```

---

## System Prompt Best Practices

Guide the LLM on when and how to use tools:

```basic
BEGIN SYSTEM PROMPT
You are a helpful product assistant with access to the following capabilities:

**Available Tools:**
- get-price: Look up product prices by name
- check-inventory: Check stock availability
- calculate-discount: Calculate prices with discounts

**Guidelines:**
1. When users ask about prices, use the get-price tool
2. When asked about availability, use check-inventory
3. If a tool returns an error, explain politely that the item wasn't found
4. Never mention tool names to users - just provide the information naturally
5. Combine multiple tool results when needed to answer complex questions

**Error Handling:**
- If get-price returns -1, the product doesn't exist
- If check-inventory shows quantity 0, inform the user it's out of stock
END SYSTEM PROMPT
```

---

## Configuration

Configure in `llm-tools.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `LLM Provider` | AI provider | `openai` |
| `LLM Model` | Model for tool calls | `gpt-4` |
| `Tool Timeout` | Max tool execution time | `30` |
| `Max Tool Calls` | Limit per conversation | `10` |

---

## Error Handling

### In Tool Implementation

```basic
' get-price.bas with error handling
PARAM product AS STRING LIKE "laptop" DESCRIPTION "Product name"

DESCRIPTION "Get product price with error handling"

ON ERROR GOTO HandleError

productRecord = FIND "products.csv", "name LIKE '%" + product + "%'"

IF productRecord THEN
    RETURN productRecord.price
ELSE
    RETURN {"error": "not_found", "message": "Product not in catalog"}
END IF

HandleError:
    RETURN {"error": "system_error", "message": "Unable to look up price"}
```

### In System Prompt

```basic
BEGIN SYSTEM PROMPT
When tools return errors:
- "not_found": Apologize and suggest similar products
- "out_of_stock": Offer to notify when back in stock
- "system_error": Ask user to try again later
END SYSTEM PROMPT
```

---

## Testing Tools

### Manual Testing

```basic
' test-tools.bas
result = CALL "get-price", {"product": "laptop"}
TALK "Price result: " + JSON(result)

result = CALL "check-inventory", {"sku": "LAPTOP-001"}
TALK "Inventory result: " + JSON(result)
```

### Conversation Testing

Test various phrasings to ensure tool invocation:

- "What's the price of X?"
- "How much does X cost?"
- "Price for X please"
- "X price?"
- "Can you tell me what X costs?"

---

## Best Practices

1. **Clear descriptions** - Help the LLM understand when to use each tool
2. **Good examples** - LIKE clauses guide parameter format
3. **Handle errors** - Always return meaningful error responses
4. **Validate input** - Check parameters before processing
5. **Log tool calls** - Track usage for debugging
6. **Keep tools focused** - One tool, one purpose
7. **Test thoroughly** - Various phrasings should trigger correct tools

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Tool not called | Description unclear | Improve DESCRIPTION text |
| Wrong parameters | Examples missing | Add LIKE examples |
| Tool errors | Missing validation | Add error handling |
| Slow responses | Complex tool logic | Optimize or cache |
| Tool exposed to user | System prompt issue | Add "don't mention tools" |

---

## Use Cases

- **Product Lookup** - Price, availability, specifications
- **Customer Service** - Order status, account info
- **Calculations** - Quotes, discounts, shipping
- **Integrations** - CRM, ERP, external APIs
- **Data Access** - Database queries, report generation

---

## Related Templates

- [LLM Server](./template-llm-server.md) - Headless API with LLM processing
- [CRM](./template-crm.md) - CRM with many tool examples
- [Store](./template-store.md) - E-commerce with product tools
- [Talk to Data](./template-talk-to-data.md) - Data query tools

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide