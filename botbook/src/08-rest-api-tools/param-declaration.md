# PARAM Declaration

The `PARAM` keyword defines input parameters for tools, enabling type checking, validation, and documentation.

## Syntax
```
PARAM parameter_name AS type LIKE "example" DESCRIPTION "description text"
```

## Components

- `parameter_name`: The name used to reference the parameter in the script
- `AS type`: The data type (string, integer, number, boolean, date, etc.)
- `LIKE "example"`: An example value showing expected format
- `DESCRIPTION "text"`: Explanation of what the parameter represents

## Supported Types

- **string**: Text values (default if no type specified)
- **integer**: Whole numbers
- **number**: Decimal numbers  
- **boolean**: True/false values
- **date**: Date values
- **datetime**: Date and time values
- **array**: Lists of values
- **object**: Structured data

## Examples

### Basic Parameter
```basic
PARAM username AS string LIKE "john_doe" DESCRIPTION "User's unique identifier"
```

### Multiple Parameters
```basic
PARAM first_name AS string LIKE "John" DESCRIPTION "User's first name"
PARAM last_name AS string LIKE "Doe" DESCRIPTION "User's last name" 
PARAM age AS integer LIKE "25" DESCRIPTION "User's age in years"
PARAM email AS string LIKE "john@example.com" DESCRIPTION "User's email address"
```

### Complex Types
```basic
PARAM preferences AS object LIKE "{"theme": "dark", "notifications": true}" DESCRIPTION "User preference settings"
PARAM tags AS array LIKE "["urgent", "follow-up"]" DESCRIPTION "Item categorization tags"
```

## Type Validation

Parameters are validated when tools are called:
- **string**: Any text value accepted
- **integer**: Must be a whole number
- **number**: Must be a valid number
- **boolean**: Converted from "true"/"false" or 1/0
- **date**: Parsed according to locale format

## Usage in Tools

Parameters become available as variables in the tool script:

```basic
PARAM product_id AS integer LIKE "12345" DESCRIPTION "Product identifier"

REM product_id variable is now available
TALK "Fetching details for product " + product_id
```

## Documentation Generation

Parameter declarations are used to automatically generate:
- Tool documentation
- API schemas (OpenAI tools format)
- MCP (Model Context Protocol) definitions
- User interface forms

## Required vs Optional

All parameters are required by default. For optional parameters, check for empty values:

```basic
PARAM phone AS string LIKE "+1-555-0123" DESCRIPTION "Optional phone number"

IF phone != "" THEN
    TALK "We'll contact you at " + phone
ELSE
    TALK "No phone number provided"
END IF
```

Parameter declarations make tools self-documenting and enable rich integration with AI systems that can understand and use the defined interfaces.
