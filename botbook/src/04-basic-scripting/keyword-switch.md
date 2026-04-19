# SWITCH

The `SWITCH` statement provides multi-way branching based on a value, allowing clean handling of multiple conditions without nested IF statements.

## Syntax

```basic
SWITCH expression
  CASE value1
    ' statements for value1
  CASE value2
    ' statements for value2
  CASE value3, value4
    ' statements for value3 or value4
  DEFAULT
    ' statements if no case matches
END SWITCH
```

## Parameters

| Element | Description |
|---------|-------------|
| `expression` | The value to evaluate |
| `CASE value` | A specific value to match |
| `CASE value1, value2` | Multiple values for the same case |
| `DEFAULT` | Optional fallback when no case matches |

## Description

`SWITCH` evaluates an expression once and compares it against multiple `CASE` values. When a match is found, the corresponding statements execute. Unlike some languages, General Bots BASIC does not require explicit `BREAK` statements - execution automatically stops after the matched case.

If no case matches and a `DEFAULT` block exists, those statements execute. If no case matches and there's no `DEFAULT`, execution continues after `END SWITCH`.

## Examples

### Role-Based Knowledge Base Selection

```basic
role = GET role

SWITCH role
  CASE "manager"
    USE KB "management"
    USE KB "reports"
  CASE "developer"
    USE KB "documentation"
    USE KB "apis"
  CASE "customer"
    USE KB "products"
    USE KB "support"
  DEFAULT
    USE KB "general"
END SWITCH
```

### Menu Navigation

```basic
TALK "Select an option:"
TALK "1. Check balance"
TALK "2. Transfer funds"
TALK "3. View history"
TALK "4. Exit"

choice = HEAR "Enter your choice:"

SWITCH choice
  CASE "1"
    balance = GET BOT MEMORY "balance"
    TALK "Your balance is: $" + balance
  CASE "2"
    TALK "Transfer initiated..."
    ' Transfer logic here
  CASE "3"
    history = FIND "recent transactions"
    TALK history
  CASE "4"
    TALK "Goodbye!"
  DEFAULT
    TALK "Invalid option. Please choose 1-4."
END SWITCH
```

### Multiple Values Per Case

```basic
day = GET day_of_week

SWITCH day
  CASE "monday", "tuesday", "wednesday", "thursday", "friday"
    TALK "It's a weekday. Office hours: 9am-5pm"
  CASE "saturday", "sunday"
    TALK "It's the weekend. We're closed."
  DEFAULT
    TALK "Unknown day"
END SWITCH
```

### Language Selection

```basic
lang = GET user_language

SWITCH lang
  CASE "en"
    TALK "Hello! How can I help you today?"
  CASE "es"
    TALK "¡Hola! ¿Cómo puedo ayudarte hoy?"
  CASE "pt"
    TALK "Olá! Como posso ajudá-lo hoje?"
  CASE "fr"
    TALK "Bonjour! Comment puis-je vous aider?"
  DEFAULT
    TALK "Hello! How can I help you today?"
END SWITCH
```

### Department Routing

```basic
department = HEAR "Which department? (sales, support, billing)"

SWITCH department
  CASE "sales"
    SET CONTEXT "You are a sales assistant. Focus on products and pricing."
    USE KB "products"
    USE KB "pricing"
  CASE "support"
    SET CONTEXT "You are a technical support agent. Help resolve issues."
    USE KB "troubleshooting"
    USE KB "faq"
  CASE "billing"
    SET CONTEXT "You are a billing specialist. Handle payment questions."
    USE KB "invoices"
    USE KB "payment_methods"
  DEFAULT
    TALK "I'll connect you with general assistance."
    USE KB "general"
END SWITCH
```

### Status Code Handling

```basic
status = GET api_response_status

SWITCH status
  CASE "200"
    TALK "Request successful!"
  CASE "400"
    TALK "Bad request. Please check your input."
  CASE "401", "403"
    TALK "Authentication error. Please log in again."
  CASE "404"
    TALK "Resource not found."
  CASE "500", "502", "503"
    TALK "Server error. Please try again later."
  DEFAULT
    TALK "Unexpected status: " + status
END SWITCH
```

### Numeric Ranges (Using Categories)

```basic
score = GET test_score
grade = ""

' Convert score to grade category
IF score >= 90 THEN
    grade = "A"
ELSE IF score >= 80 THEN
    grade = "B"
ELSE IF score >= 70 THEN
    grade = "C"
ELSE IF score >= 60 THEN
    grade = "D"
ELSE
    grade = "F"
END IF

SWITCH grade
  CASE "A"
    TALK "Excellent work!"
    SET BOT MEMORY "achievement", "honor_roll"
  CASE "B"
    TALK "Good job!"
  CASE "C"
    TALK "Satisfactory performance."
  CASE "D"
    TALK "You passed, but could improve."
  CASE "F"
    TALK "Please see a tutor for help."
END SWITCH
```

## Comparison with IF-ELSE

### Using IF-ELSE (Verbose)

```basic
IF color = "red" THEN
    TALK "Stop"
ELSE IF color = "yellow" THEN
    TALK "Caution"
ELSE IF color = "green" THEN
    TALK "Go"
ELSE
    TALK "Unknown signal"
END IF
```

### Using SWITCH (Cleaner)

```basic
SWITCH color
  CASE "red"
    TALK "Stop"
  CASE "yellow"
    TALK "Caution"
  CASE "green"
    TALK "Go"
  DEFAULT
    TALK "Unknown signal"
END SWITCH
```

## Notes

- **No fall-through**: Each CASE is isolated; no BREAK needed
- **Case sensitivity**: String comparisons are case-sensitive
- **Expression evaluated once**: The switch expression is evaluated only once
- **DEFAULT is optional**: Without DEFAULT, unmatched values skip the block
- **Multiple values**: Use commas to match multiple values in one CASE

## Best Practices

1. **Always include DEFAULT** for robust error handling
2. **Use meaningful case values** that are self-documenting
3. **Order cases logically** - most common first or alphabetically
4. **Keep case blocks concise** - extract complex logic to separate scripts

## See Also

- [SET CONTEXT](./keyword-set-context.md) - Set conversation context
- [USE KB](./keyword-use-kb.md) - Load knowledge base
- [GET](./keyword-get.md) - Get variable values
- [IF/THEN/ELSE](./keyword-if.md) - Conditional branching