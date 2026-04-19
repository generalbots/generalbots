# Creating an LLM REST Server

General Bots offers an incredibly simple way to transform a Large Language Model (LLM) into a fully functional REST API server. With just a few lines of our proprietary BASIC-like syntax, you can create sophisticated AI-powered applications.

## Overview

By defining PARAM declarations and a DESCRIPTION in your `.bas` file, General Bots automatically:

1. Creates REST API endpoints callable by the LLM as tools
2. Generates OpenAI-compatible function calling schemas
3. Generates MCP (Model Context Protocol) tool definitions
4. Handles conversation state and context management

## Basic Structure

Every LLM-callable tool follows this structure:

```basic
PARAM parameter_name AS type LIKE "example" DESCRIPTION "What this parameter is for"

DESCRIPTION "What this tool does. Called when user wants to [action]."

' Your business logic here
```

## Example: Store Chatbot

Here's how easy it is to create a chatbot for a store:

```basic
PARAM operator AS number LIKE 12312312
DESCRIPTION "Operator code."

DESCRIPTION It is a WebService of GB.

products = FIND "products.csv"

BEGIN SYSTEM PROMPT
  You must act as a chatbot that will assist a store attendant by 
  following these rules: Whenever the attendant places an order, it must 
  include the table and the customer's name. Example: A 400ml Pineapple 
  Caipirinha for Rafael at table 10. Orders are based on the products and 
  sides from this product menu: ${JSON.stringify(products)}.

  For each order placed, return a JSON containing the product name, the 
  table, and a list of sides with their respective ids.
END SYSTEM PROMPT
```

That's it! With just this simple BASIC code, you've created a fully functional LLM-powered chatbot that can handle complex order processing.

## REST API Endpoints

The system automatically generates REST API endpoints for your dialogs.

### Starting a Conversation

```
GET http://localhost:1111/llm-server/dialogs/start?operator=123&userSystemId=999
```

This returns a **Process ID (PID)**, a number like `24795078551392`. This PID should be passed within the call chain for maintaining conversation context.

### Talking to the Bot

Once you have the PID, you can interact with the LLM:

```
GET http://localhost:1111/llm-server/dk/talk?pid=4893749837&text=add%20soda
```

This call acts like talking to the LLM, but it can be used for anything that General Bots can do in a robotic conversation between systems mediated by LLM. The return will be JSON (or any format specified in your BEGIN SYSTEM PROMPT).

## Example: Enrollment Process API

Creating a REST API server for any business process is equally straightforward:

```basic
PARAM name AS string LIKE "João Silva"
DESCRIPTION "Required full name of the individual."

PARAM birthday AS date LIKE "23/09/2001"
DESCRIPTION "Required birth date of the individual in DD/MM/YYYY format."

PARAM email AS string LIKE "joao.silva@example.com"
DESCRIPTION "Required email address for contact purposes."

PARAM personalid AS integer LIKE "12345678900"
DESCRIPTION "Required Personal ID number of the individual (only numbers)."

PARAM address AS string LIKE "Rua das Flores, 123, São Paulo, SP"
DESCRIPTION "Required full address of the individual."

DESCRIPTION "This is the enrollment process, called when the user wants to enroll. Once all information is collected, confirm the details and inform them that their enrollment request has been successfully submitted. Provide a polite and professional tone throughout the interaction."

SAVE "enrollments.csv", id, name, birthday, email, cpf, rg, address
```

This creates a full-fledged enrollment system with:
- Data validation
- User interaction
- Data storage
- Automatic REST API endpoint

The system automatically generates a REST API endpoint that is called by LLM as a tool:

```
GET http://api.pragmatismo.cloud/llm-server/dialogs/enrollment?birthday=...&name=...
```

## Generated Tool Schemas

### MCP Format

For each tool, General Bots generates MCP-compatible schemas:

```json
{
  "name": "enrollment",
  "description": "This is the enrollment process...",
  "input_schema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Required full name of the individual.",
        "example": "João Silva"
      },
      "birthday": {
        "type": "string",
        "description": "Required birth date...",
        "example": "23/09/2001"
      }
    },
    "required": ["name", "birthday", "email", "personalid", "address"]
  }
}
```

### OpenAI Format

Also generates OpenAI function calling format:

```json
{
  "type": "function",
  "function": {
    "name": "enrollment",
    "description": "This is the enrollment process...",
    "parameters": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Required full name of the individual."
        }
      },
      "required": ["name", "birthday", "email", "personalid", "address"]
    }
  }
}
```

## Parameter Types

| Type | Description | Example |
|------|-------------|---------|
| `string` | Text values | `"John Smith"` |
| `number` | Numeric values | `42`, `3.14` |
| `integer` | Whole numbers | `100` |
| `date` | Date values | `"2024-01-15"` |
| `boolean` | True/false | `true` |

## Advanced: External API Integration

You can combine LLM tools with external API calls:

```basic
PARAM location AS string LIKE "Seattle"
DESCRIPTION "City for weather lookup"

DESCRIPTION "Gets current weather for a city"

let api_key = GET BOT MEMORY "openweather_key"
let url = "https://api.openweathermap.org/data/2.5/weather?q=" + location + "&appid=" + api_key

let response = GET url
let weather = LLM "Describe the weather based on: " + response
TALK weather
```

## Best Practices

1. **Clear Descriptions**: Write detailed DESCRIPTION text - this is what the LLM uses to decide when to call your tool.

2. **Good Examples**: The LIKE clause provides examples that help both the LLM and API consumers understand expected values.

3. **Validation**: Add validation logic to handle edge cases:

```basic
PARAM email AS string LIKE "user@example.com"
DESCRIPTION "Email address"

IF NOT INSTR(email, "@") > 0 THEN
    TALK "Please provide a valid email address."
    RETURN
END IF
```

4. **Error Handling**: Always handle potential errors gracefully:

```basic
result = GET "https://api.example.com/data"
IF result.error THEN
    TALK "Unable to fetch data. Please try again."
    RETURN
END IF
```

5. **Secure Credentials**: Use BOT MEMORY for API keys:

```basic
api_key = GET BOT MEMORY "my_api_key"
```

## Deployment

Once your `.bas` file is saved in the `.gbdialog` folder, General Bots automatically:

1. Compiles the tool definition
2. Generates the REST endpoints
3. Makes it available to the LLM as a callable tool
4. Updates when you modify the file

No additional configuration or deployment steps are required!

## See Also

- [PARAM Declaration](./param-declaration.md) - Detailed PARAM syntax
- [Tool Definition](./tool-definition.md) - Complete tool definition reference
- [MCP Format](./mcp-format.md) - MCP schema details
- [OpenAI Format](./openai-format.md) - OpenAI function calling format
- [External APIs](./external-apis.md) - Integrating external services