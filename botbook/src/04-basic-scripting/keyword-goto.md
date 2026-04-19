# GOTO Keyword

> ⚠️ **WARNING: GOTO is supported but NOT RECOMMENDED**
>
> While GOTO works for backward compatibility, you should use **event-driven patterns with the ON keyword** instead. GOTO makes code harder to maintain and debug.

## Syntax

```basic
label:
    ' statements
GOTO label

IF condition THEN GOTO label
```

## Description

`GOTO` transfers program execution to a labeled line. Labels are identifiers followed by a colon (`:`) at the start of a line.

**This keyword exists for backward compatibility only.** Modern General Bots code should use:

- `ON INSERT/UPDATE/DELETE OF` for event-driven programming
- `WHILE ... WEND` for loops
- `FOR EACH ... NEXT` for iteration
- `SUB` / `FUNCTION` for code organization

---

## ❌ OLD WAY vs ✅ NEW WAY

### Polling Loop (Don't Do This)

```basic
' ❌ BAD: GOTO-based polling loop
mainLoop:
    leads = FIND "leads", "processed = false"
    FOR EACH lead IN leads
        CALL processLead(lead)
    NEXT lead
    WAIT 5
GOTO mainLoop
```

### Event-Driven (Do This Instead)

```basic
' ✅ GOOD: Event-driven with ON keyword
ON INSERT OF "leads"
    lead = GET LAST "leads"
    
    score = SCORE LEAD lead
    
    IF score.status = "hot" THEN
        TALK TO "whatsapp:" + sales_phone, "🔥 Hot lead: " + lead.name
    END IF
END ON
```

---

## Why ON is Better Than GOTO

| Aspect | GOTO Loop | ON Event |
|--------|-----------|----------|
| **Efficiency** | Polls constantly, wastes CPU | Triggers only when data changes |
| **Code clarity** | Spaghetti code, hard to follow | Clear cause-and-effect |
| **Resource usage** | Always running | Idle until triggered |
| **Scalability** | Degrades with more data | Handles scale naturally |
| **Debugging** | Hard to trace execution | Clear event flow |
| **LLM integration** | Poor | Works well with TOOLs |

---

## If You Must Use GOTO

For legacy code migration or specific use cases, GOTO is supported:

### Simple Loop

```basic
counter:
    TALK "Count: " + x
    x = x + 1
    IF x < 5 THEN GOTO counter
    TALK "Done!"
```

### Multiple Labels

```basic
start:
    TALK "Starting..."
    GOTO process

error:
    TALK "An error occurred"
    GOTO cleanup

process:
    result = DO_SOMETHING
    IF result = "error" THEN GOTO error
    TALK "Success!"
    GOTO cleanup

cleanup:
    TALK "Cleaning up..."
```

### Conditional GOTO

```basic
check:
    IF temperature > 30 THEN GOTO too_hot
    IF temperature < 10 THEN GOTO too_cold
    TALK "Temperature is comfortable"
    GOTO done

too_hot:
    TALK "Warning: Too hot!"
    GPIO SET fan_pin, HIGH
    GOTO done

too_cold:
    TALK "Warning: Too cold!"
    GPIO SET heater_pin, HIGH
    GOTO done

done:
    TALK "Check complete"
```

---

## How GOTO Works Internally

GOTO is **not native to the Rhai engine**. General Bots transforms GOTO-based code into a state machine:

```basic
' Your code:
loop:
    x = x + 1
    IF x < 3 THEN GOTO loop
    TALK "Done"
```

```basic
' Transformed internally to:
let __goto_label = "loop"
while __goto_label != "__exit" {
    if __goto_label == "loop" {
        x = x + 1
        if x < 3 { __goto_label = "loop"; continue; }
        TALK "Done"
        __goto_label = "__exit"
    }
}
```

This transformation:
- Adds overhead compared to native loops
- Has a safety limit of 1,000,000 iterations to prevent infinite loops
- Emits warnings in the server logs recommending ON patterns

---

## Limitations

| Limitation | Description |
|------------|-------------|
| **No computed GOTO** | `GOTO variable` is not supported |
| **No GOSUB** | Use `SUB` / `CALL` instead |
| **No line numbers** | Only named labels are supported |
| **Performance** | Slower than native WHILE loops |
| **Iteration limit** | Maximum 1,000,000 iterations per GOTO loop |

---

## Migration Guide

### From GOTO Loop to WHILE

```basic
' Before (GOTO):
start:
    TALK "Hello"
    x = x + 1
    IF x < 10 THEN GOTO start

' After (WHILE):
WHILE x < 10
    TALK "Hello"
    x = x + 1
WEND
```

### From GOTO Loop to ON Event

```basic
' Before (GOTO polling):
checkOrders:
    orders = FIND "orders", "status = 'new'"
    FOR EACH order IN orders
        CALL processOrder(order)
    NEXT order
    WAIT 10
GOTO checkOrders

' After (ON event):
ON INSERT OF "orders"
    order = GET LAST "orders"
    IF order.status = "new" THEN
        CALL processOrder(order)
    END IF
END ON
```

### From GOTO Error Handling to ON ERROR

```basic
' Before (GOTO):
    result = RISKY_OPERATION
    IF result = "error" THEN GOTO handleError
    TALK "Success"
    GOTO done
handleError:
    TALK "Failed!"
done:

' After (ON ERROR):
ON ERROR RESUME NEXT
    result = RISKY_OPERATION
    IF ERROR THEN
        TALK "Failed!"
    ELSE
        TALK "Success"
    END IF
ON ERROR GOTO 0
```

---

## Best Practices

1. **Don't use GOTO for new code** - Use WHILE, FOR EACH, ON, or SUB/FUNCTION
2. **Migrate existing GOTO code** - Refactor to event-driven patterns when possible
3. **If you must use GOTO** - Keep it simple, avoid deep nesting
4. **Add comments** - Explain why GOTO is necessary
5. **Set reasonable limits** - Don't rely on the 1M iteration safety limit

---

## See Also

- [ON Keyword](./keyword-on.md) - **Recommended**: Event-driven programming
- [WHILE ... WEND](./keyword-while.md) - Loop construct
- [FOR EACH ... NEXT](./keyword-for-each.md) - Iteration
- [SUB / FUNCTION](./keyword-sub.md) - Code organization
- [ON ERROR](./keyword-on-error.md) - Error handling

---

## Summary

| Use Case | Instead of GOTO, Use |
|----------|---------------------|
| Polling for changes | `ON INSERT/UPDATE/DELETE OF` |
| Repeating code | `WHILE ... WEND` |
| Iterating collections | `FOR EACH ... NEXT` |
| Reusable code blocks | `SUB` / `FUNCTION` + `CALL` |
| Error handling | `ON ERROR RESUME NEXT` |
| Conditional branching | `IF ... ELSEIF ... ELSE ... END IF` |
| Multiple conditions | `SWITCH ... CASE ... END SWITCH` |

**The ON keyword is the future. GOTO is the past.**