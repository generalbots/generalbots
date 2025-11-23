# MCP Format

Model Context Protocol (MCP) is a standardized format for defining tools that language models can discover and invoke. BotServer generates MCP-compatible tool definitions from BASIC scripts.

## Overview

MCP provides a structured way to describe:
- Tool name and purpose
- Input parameters and types
- Parameter descriptions and examples
- Output format expectations

## MCP Tool Structure

A compiled MCP tool definition contains:

```json
{
  "name": "tool_name",
  "description": "Tool description from DESCRIPTION statement",
  "input_schema": {
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

## From BASIC to MCP

### Source BASIC Script

```basic
PARAM customer_name AS string LIKE "John Doe" DESCRIPTION "Customer's full name"
PARAM order_amount AS number LIKE 99.99 DESCRIPTION "Total order amount"
PARAM shipping_address AS string LIKE "123 Main St" DESCRIPTION "Delivery address"

DESCRIPTION "Process a new customer order"

# Script logic here
TALK "Processing order for " + customer_name
# ...
```

### Generated MCP Definition

```json
{
  "name": "process_order",
  "description": "Process a new customer order",
  "input_schema": {
    "type": "object",
    "properties": {
      "customer_name": {
        "type": "string",
        "description": "Customer's full name",
        "example": "John Doe"
      },
      "order_amount": {
        "type": "number",
        "description": "Total order amount",
        "example": 99.99
      },
      "shipping_address": {
        "type": "string",
        "description": "Delivery address",
        "example": "123 Main St"
      }
    },
    "required": ["customer_name", "order_amount", "shipping_address"]
  }
}
```

## Parameter Types

MCP supports these parameter types in BotServer:

| BASIC Type | MCP Type | JSON Schema Type |
|------------|----------|------------------|
| string | string | "type": "string" |
| number | number | "type": "number" |
| boolean | boolean | "type": "boolean" |

## Input Schema

The `input_schema` follows JSON Schema specification:

### Required Fields
- `type`: Always "object" for tool parameters
- `properties`: Object containing parameter definitions
- `required`: Array of required parameter names

### Parameter Properties
- `type`: Data type of the parameter
- `description`: Human-readable description
- `example`: Example value from LIKE clause

## Tool Discovery

MCP tools are discoverable through:

1. **Tool Listing**: LLM can query available tools
2. **Parameter Inspection**: LLM examines input schema
3. **Description Matching**: LLM matches user intent to tool description

## Integration with LLM

When the LLM invokes an MCP tool:

1. **Parameter Collection**: LLM extracts values from context
2. **Schema Validation**: Parameters validated against input_schema
3. **Tool Execution**: BASIC script runs with provided parameters
4. **Result Return**: Output returned to LLM for processing

## Example Use Cases

### Form Processing Tool

```json
{
  "name": "submit_application",
  "description": "Submit a job application form",
  "input_schema": {
    "type": "object",
    "properties": {
      "applicant_name": {
        "type": "string",
        "description": "Full name of applicant"
      },
      "position": {
        "type": "string",
        "description": "Position applying for"
      },
      "experience_years": {
        "type": "number",
        "description": "Years of relevant experience"
      }
    },
    "required": ["applicant_name", "position", "experience_years"]
  }
}
```

### Data Query Tool

```json
{
  "name": "search_inventory",
  "description": "Search product inventory",
  "input_schema": {
    "type": "object",
    "properties": {
      "product_name": {
        "type": "string",
        "description": "Product to search for"
      },
      "min_quantity": {
        "type": "number",
        "description": "Minimum quantity available"
      }
    },
    "required": ["product_name"]
  }
}
```

## Storage and Retrieval

MCP definitions are stored in the `basic_tools` table:
- Tool metadata serialized as JSON
- Indexed for fast retrieval
- Associated with bot ID
- Cached for performance

## Advantages of MCP Format

1. **Standardized**: Compatible with multiple LLM providers
2. **Self-Documenting**: Contains all necessary metadata
3. **Type-Safe**: Schema validation prevents errors
4. **Discoverable**: LLMs can understand tool capabilities
5. **Extensible**: Can add custom properties as needed

## Limitations in BotServer

Current MCP implementation limitations:
- No nested object parameters
- No array parameters
- All parameters are required (no optional)
- No enum/choice constraints
- No pattern validation

## Best Practices

1. **Clear Descriptions**: Make tool purpose obvious
2. **Meaningful Names**: Use descriptive parameter names
3. **Provide Examples**: LIKE values help LLM understand expected input
4. **Type Accuracy**: Use correct types (string vs number)
5. **Complete Documentation**: Every parameter needs description

## Validation

MCP tools are validated during compilation:
- Parameter names must be valid identifiers
- Types must be supported
- Descriptions cannot be empty
- Tool name must be unique per bot

## Summary

The MCP format provides a structured way to expose BASIC scripts as callable tools for LLMs. By generating MCP-compatible definitions, BotServer enables seamless tool discovery and invocation within conversational flows.