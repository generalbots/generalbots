# IS NUMERIC

The `IS NUMERIC` function tests whether a string value can be converted to a number. This is essential for input validation before performing mathematical operations.

## Syntax

```basic
result = IS NUMERIC(value)
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `value` | string | The value to test for numeric content |

## Return Value

- Returns `true` if the value can be parsed as a number
- Returns `false` if the value contains non-numeric characters

## Description

`IS NUMERIC` examines a string to determine if it represents a valid numeric value. It recognizes:
- Integers: `42`, `-17`, `0`
- Decimals: `3.14`, `-0.5`, `.25`
- Scientific notation: `1e10`, `2.5E-3`

Empty strings and strings containing letters or special characters (except `-`, `.`, `e`, `E`) return `false`.

## Examples

### Basic Validation

```basic
input = HEAR "Enter a number:"
IF IS NUMERIC(input) THEN
    TALK "You entered: " + input
ELSE
    TALK "That's not a valid number"
END IF
```

### Bot Memory with Default Value

```basic
max_items = GET BOT MEMORY "max_items"
IF max_items = "" OR NOT IS NUMERIC(max_items) THEN
    max_items = "10"
END IF

TALK "Processing up to " + max_items + " items"
```

### Input Loop Until Valid

```basic
valid = false
DO WHILE NOT valid
    age = HEAR "Enter your age:"
    IF IS NUMERIC(age) THEN
        valid = true
    ELSE
        TALK "Please enter a number"
    END IF
LOOP
TALK "Your age is " + age
```

### Combined Conditions with OR NOT

```basic
quantity = HEAR "How many items?"
IF quantity = "" OR NOT IS NUMERIC(quantity) THEN
    TALK "Invalid quantity, using default of 1"
    quantity = "1"
END IF
```

### Validating Multiple Fields

```basic
price = HEAR "Enter price:"
quantity = HEAR "Enter quantity:"

IF IS NUMERIC(price) AND IS NUMERIC(quantity) THEN
    total = price * quantity
    TALK "Total: $" + total
ELSE
    IF NOT IS NUMERIC(price) THEN
        TALK "Price must be a number"
    END IF
    IF NOT IS NUMERIC(quantity) THEN
        TALK "Quantity must be a number"
    END IF
END IF
```

### Configuration Validation

```basic
' Load timeout from config, validate it's numeric
timeout = GET BOT MEMORY "api_timeout"
IF NOT IS NUMERIC(timeout) THEN
    timeout = "30"
    SET BOT MEMORY "api_timeout", timeout
    TALK "Set default timeout to 30 seconds"
END IF
```

### Range Checking After Validation

```basic
rating = HEAR "Rate from 1-5:"

IF NOT IS NUMERIC(rating) THEN
    TALK "Please enter a number"
ELSE IF rating < 1 OR rating > 5 THEN
    TALK "Rating must be between 1 and 5"
ELSE
    TALK "Thank you for your rating of " + rating
    SET BOT MEMORY "last_rating", rating
END IF
```

## What IS NUMERIC Accepts

| Input | Result | Notes |
|-------|--------|-------|
| `"42"` | `true` | Integer |
| `"-17"` | `true` | Negative integer |
| `"3.14"` | `true` | Decimal |
| `".5"` | `true` | Leading decimal |
| `"1e10"` | `true` | Scientific notation |
| `"2.5E-3"` | `true` | Scientific with decimal |
| `""` | `false` | Empty string |
| `"abc"` | `false` | Letters |
| `"12abc"` | `false` | Mixed content |
| `"$100"` | `false` | Currency symbol |
| `"1,000"` | `false` | Thousands separator |
| `"  42  "` | `true` | Whitespace trimmed |

## Common Patterns

### Default Value Pattern

```basic
value = GET BOT MEMORY key
IF value = "" OR NOT IS NUMERIC(value) THEN
    value = default_value
END IF
```

### Safe Division

```basic
divisor = HEAR "Enter divisor:"
IF NOT IS NUMERIC(divisor) THEN
    TALK "Must be a number"
ELSE IF divisor = 0 THEN
    TALK "Cannot divide by zero"
ELSE
    result = 100 / divisor
    TALK "Result: " + result
END IF
```

### Percentage Validation

```basic
percent = HEAR "Enter percentage (0-100):"
IF IS NUMERIC(percent) THEN
    IF percent >= 0 AND percent <= 100 THEN
        TALK "Discount: " + percent + "%"
    ELSE
        TALK "Must be between 0 and 100"
    END IF
ELSE
    TALK "Enter a number without %"
END IF
```

## Notes

- **Whitespace**: Leading and trailing spaces are trimmed before checking
- **Empty strings**: Always return `false`
- **Locale**: Uses period (.) as decimal separator
- **Currency**: Does not recognize currency symbols ($, €, etc.)
- **Separators**: Does not recognize thousands separators (commas)

## Error Prevention

Using `IS NUMERIC` prevents runtime errors when converting strings to numbers:

```basic
' Without validation - could cause error
value = HEAR "Enter number:"
result = value * 2  ' Error if value is "abc"

' With validation - safe
value = HEAR "Enter number:"
IF IS NUMERIC(value) THEN
    result = value * 2
ELSE
    TALK "Invalid input"
END IF
```

## See Also

- [GET BOT MEMORY](./keyword-get-bot-memory.md) - Retrieve stored values
- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Store values
- [INSTR](./keyword-instr.md) - Find substring position
- [FORMAT](./keyword-format.md) - Format numbers as strings