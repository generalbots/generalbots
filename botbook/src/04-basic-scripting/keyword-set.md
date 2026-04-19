# SET

Assign values to variables in BASIC dialogs.

## Syntax

```basic
SET variable = value
```

or simply:

```basic
variable = value
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `variable` | Identifier | Variable name to assign to |
| `value` | Any | Value to assign (string, number, boolean, array, object) |

## Description

The `SET` keyword assigns values to variables within BASIC dialog scripts. Variables are dynamically typed and can hold any type of value. The `SET` keyword is optional - you can use direct assignment with `=`.

Variables are scoped to the current dialog execution and persist throughout the conversation session until explicitly changed or the session ends.

## Examples

### Basic Assignment
```basic
SET name = "John Doe"
SET age = 25
SET is_premium = true
SET score = 98.5
```

### Direct Assignment (without SET)
```basic
name = "Jane Smith"
count = 0
message = "Welcome!"
```

### Array Assignment
```basic
SET colors = ["red", "green", "blue"]
SET numbers = [1, 2, 3, 4, 5]
SET mixed = ["text", 123, true]
```

### Object/Map Assignment
```basic
SET user = {
    "name": "Alice",
    "email": "alice@example.com",
    "age": 30,
    "active": true
}
```

### Dynamic Values
```basic
SET current_time = NOW()
SET user_input = HEAR "What's your name?"
SET calculated = price * quantity * tax_rate
SET formatted = FORMAT("Hello, {0}!", username)
```

## Variable Types

BASIC supports these variable types:
- **String**: Text values
- **Number**: Integers and decimals
- **Boolean**: true/false
- **Array**: Ordered lists
- **Object**: Key-value maps
- **Null**: Empty/undefined

## Variable Naming

Valid variable names:
- Start with letter or underscore
- Contain letters, numbers, underscores
- Case-sensitive
- No reserved keywords

Examples:
```basic
SET userName = "John"
SET user_name = "John"
SET _private = true
SET value123 = 456
SET firstName = "Jane"
```

Invalid names:
```basic
' These will cause errors
SET 123name = "error"      ' Starts with number
SET user-name = "error"    ' Contains hyphen
SET if = "error"           ' Reserved keyword
```

## Variable Scope

### Session Variables
Regular variables exist for the session:
```basic
SET session_data = "persists during conversation"
```

### Global Variables
Use special prefixes for broader scope:
```basic
SET $global_var = "accessible across dialogs"
SET @bot_var = "bot-level variable"
```

### Temporary Variables
```basic
SET _temp = "temporary use"
' Prefix with underscore for temporary/internal use
```

## Type Conversion

Variables automatically convert types when needed:
```basic
SET text = "123"
SET number = text + 0       ' Converts to number: 123
SET back_to_text = number + ""  ' Converts to string: "123"
SET boolean = number > 100  ' Converts to boolean: true
```

## Operations on Variables

### String Operations
```basic
SET full_name = first_name + " " + last_name
SET uppercase = UPPER(name)
SET length = LEN(message)
SET substring = MID(text, 1, 5)
```

### Numeric Operations
```basic
SET sum = a + b
SET difference = a - b
SET product = a * b
SET quotient = a / b
SET remainder = a MOD b
SET power = a ^ b
```

### Array Operations
```basic
SET first = colors[0]
SET last = colors[LEN(colors) - 1]
colors[1] = "yellow"  ' Modify array element
SET combined = array1 + array2  ' Concatenate
```

### Object/Map Operations
```basic
SET email = user["email"]
SET age = user.age
user["status"] = "active"
user.last_login = NOW()
```

## Conditional Assignment

```basic
SET status = IF(score >= 70, "pass", "fail")
SET discount = IF(is_member, 0.2, 0.1)
SET greeting = IF(hour < 12, "Good morning", "Good afternoon")
```

## Common Patterns

### Counter Variables
```basic
SET counter = 0
FOR i = 1 TO 10
    counter = counter + 1
NEXT
```

### Flag Variables
```basic
SET is_complete = false
' ... process ...
SET is_complete = true
```

### Accumulator Variables
```basic
SET total = 0
FOR EACH item IN cart
    total = total + item.price
NEXT
```

### State Variables
```basic
SET state = "initial"
' ... logic ...
SET state = "processing"
' ... more logic ...
SET state = "complete"
```

## Best Practices

1. **Use descriptive names**: `customer_email` instead of `e`
2. **Initialize variables**: Set initial values before use
3. **Use consistent naming**: camelCase or snake_case
4. **Avoid global pollution**: Use local variables when possible
5. **Clean up large variables**: Set to null when done
6. **Document complex variables**: Add comments
7. **Validate before use**: Check if variable exists

## Error Handling

```basic
' Check if variable exists
IF EXISTS(user_data) THEN
    SET name = user_data.name
ELSE
    SET name = "Guest"
END IF

' Safe assignment with default
SET value = GET_VALUE_OR_DEFAULT(config.setting, "default")
```

## Memory Management

```basic
' Clear large variables when done
SET big_data = LOAD_FILE("large.json")
' ... use big_data ...
SET big_data = null  ' Free memory
```

## Related Keywords

- [GET](./keyword-get.md) - Retrieve data from external sources
- [HEAR](./keyword-hear.md) - Get user input into variable
- [FORMAT](./keyword-format.md) - Format values for assignment
- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Persistent storage
## Implementation Notes

Variables are stored in the BASIC engine's scope map and persist for the duration of the dialog execution. The `SET` keyword is syntactic sugar - the parser treats both `SET x = y` and `x = y` identically.