# Tool Format

botserver generates OpenAI-compatible function definitions from BASIC scripts, enabling integration with OpenAI's function calling API.

## Overview

OpenAI's function calling format allows GPT models to:
- Discover available functions
- Understand parameter requirements
- Generate structured function calls
- Process function results

## Function Structure

An OpenAI-compatible function definition contains:

```json
{
  "name": "function_name",
  "description": "Function description",
  "parameters": {
    "type": "object",
    "properties": {
      "param1": {
        "type": "string",
        "description": "Parameter description"
      },
      "param2": {
        "type": "number",
        "description": "Another parameter"
      }
    },
    "required": ["param1", "param2"]
  }
}
```

## Conversion from BASIC

### Source BASIC Script

```basic
PARAM product_id AS string LIKE "SKU-12345" DESCRIPTION "Product identifier"
PARAM quantity AS number LIKE 10 DESCRIPTION "Quantity to order"
PARAM urgent AS boolean LIKE true DESCRIPTION "Rush delivery needed"

DESCRIPTION "Create a purchase order for inventory"

# Script implementation
let order_id = CREATE_ORDER(product_id, quantity, urgent)
TALK "Order created: " + order_id
```

### Generated Function

```json
{
  "name": "create_purchase_order",
  "description": "Create a purchase order for inventory",
  "parameters": {
    "type": "object",
    "properties": {
      "product_id": {
        "type": "string",
        "description": "Product identifier"
      },
      "quantity": {
        "type": "number",
        "description": "Quantity to order"
      },
      "urgent": {
        "type": "boolean",
        "description": "Rush delivery needed"
      }
    },
    "required": ["product_id", "quantity", "urgent"]
  }
}
```

## Integration with OpenAI API

When using OpenAI's API, the functions are passed in the request:

```json
{
  "model": "gpt-4o",
  "messages": [...],
  "functions": [
    {
      "name": "create_purchase_order",
      "description": "Create a purchase order for inventory",
      "parameters": {...}
    }
  ],
  "function_call": "auto"
}
```

## Parameter Type Mapping

| BASIC Type | OpenAI Type | Description |
|------------|-------------|-------------|
| string | "string" | Text values |
| number | "number" | Numeric values (integer or float) |
| boolean | "boolean" | True/false values |

## Function Calling Flow

1. **User Query**: User asks to perform an action
2. **Function Discovery**: GPT identifies relevant function
3. **Parameter Extraction**: GPT extracts parameters from context
4. **Function Call**: GPT generates structured function call
5. **Execution**: botserver executes the BASIC script
6. **Result Processing**: Output returned to GPT for response

## Example Function Calls

### Customer Service Function

```json
{
  "name": "check_order_status",
  "description": "Check the status of a customer order",
  "parameters": {
    "type": "object",
    "properties": {
      "order_id": {
        "type": "string",
        "description": "Order reference number"
      },
      "customer_email": {
        "type": "string",
        "description": "Customer email for verification"
      }
    },
    "required": ["order_id", "customer_email"]
  }
}
```

### Data Analysis Function

```json
{
  "name": "generate_sales_report",
  "description": "Generate sales report for specified period",
  "parameters": {
    "type": "object",
    "properties": {
      "start_date": {
        "type": "string",
        "description": "Report start date (YYYY-MM-DD)"
      },
      "end_date": {
        "type": "string",
        "description": "Report end date (YYYY-MM-DD)"
      },
      "region": {
        "type": "string",
        "description": "Sales region to analyze"
      }
    },
    "required": ["start_date", "end_date"]
  }
}
```

## Function Response Handling

When a function is executed:

1. **Script Execution**: BASIC script runs with provided parameters
2. **Output Collection**: TALK statements and return values collected
3. **Response Format**: Results formatted for OpenAI API
4. **Context Update**: Function result added to conversation

## Differences from MCP Format

| Aspect | OpenAI Format | MCP Format |
|--------|---------------|------------|
| Schema Location | `parameters` | `input_schema` |
| Example Values | Not included | Included in schema |
| Metadata | Minimal | Extended metadata |
| Compatibility | OpenAI models only | Multiple providers |

## Error Handling

Function errors are handled gracefully:
- Missing parameters return error message
- Type mismatches caught before execution
- Script errors logged and returned
- Timeout protection for long-running scripts

## Best Practices

1. **Descriptive Names**: Use clear function names
2. **Comprehensive Descriptions**: Explain what the function does
3. **Parameter Clarity**: Each parameter needs clear description
4. **Error Messages**: Provide helpful error feedback
5. **Idempotency**: Design functions to be safely retryable

## Limitations

Current OpenAI format limitations in botserver:
- No nested objects in parameters
- No array parameters
- No enum constraints
- All parameters marked as required
- No custom validation rules

## Storage

OpenAI function definitions are stored alongside MCP definitions:
- Stored in `basic_tools` table
- Generated during compilation
- Cached for performance
- Updated when script changes

## Usage in Conversations

When a user message triggers function calling:

```
User: "Order 50 units of SKU-12345 urgently"

System: [Identifies create_purchase_order function]
        [Extracts: product_id="SKU-12345", quantity=50, urgent=true]
        [Executes function]

Bot: "Order created: ORD-2024-001. Rush delivery confirmed for 50 units of SKU-12345."
```

## Performance Considerations

- Functions cached after compilation
- Parallel function execution supported
- Rate limiting applied per session
- Timeout protection (30 seconds default)

## Debugging

To debug OpenAI function calls:
1. Enable debug logging
2. Check function registration
3. Verify parameter extraction
4. Review execution logs
5. Test with manual invocation

## Summary

The OpenAI function format enables seamless integration between BASIC scripts and OpenAI's GPT models. By automatically generating compatible function definitions, botserver allows natural language interactions to trigger complex business logic implementations.