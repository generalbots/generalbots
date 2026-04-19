# ON ERROR RESUME NEXT

The `ON ERROR RESUME NEXT` keyword enables BASIC error handling, allowing scripts to continue execution when errors occur instead of terminating immediately.

---

## Syntax

```basic
ON ERROR RESUME NEXT    ' Enable error trapping
ON ERROR GOTO 0         ' Disable error trapping
```

---

## Description

`ON ERROR RESUME NEXT` enables error trapping mode where execution continues to the next statement after an error occurs, rather than halting the script. This pattern is borrowed from BASIC and provides a simple way to handle errors gracefully in BASIC scripts.

When error trapping is enabled:
- Errors are captured but don't stop execution
- The `ERROR` function returns `TRUE` if an error occurred
- The `ERROR MESSAGE` function returns the error description
- The `ERR` variable contains the error number

Use `ON ERROR GOTO 0` to disable error trapping and restore normal error behavior.

---

## Error State Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `ON ERROR RESUME NEXT` | Enable error trapping | `ON ERROR RESUME NEXT` |
| `ON ERROR GOTO 0` | Disable error trapping | `ON ERROR GOTO 0` |
| `ERROR` | Returns TRUE if error occurred | `IF ERROR THEN` |
| `ERROR MESSAGE` | Get last error message | `msg = ERROR MESSAGE` |
| `ERR` | Get error number | `code = ERR` |
| `CLEAR ERROR` | Clear error state | `CLEAR ERROR` |
| `THROW` | Raise a custom error | `THROW "Invalid input"` |
| `ASSERT` | Assert a condition | `ASSERT count > 0, "Count must be positive"` |

---

## Examples

### Basic Error Handling

```basic
' Enable error trapping
ON ERROR RESUME NEXT

' Attempt a risky operation
result = GET "https://api.example.com/data"

' Check if it failed
IF ERROR THEN
    TALK "Sorry, I couldn't fetch the data: " + ERROR MESSAGE
ELSE
    TALK "Data retrieved successfully!"
END IF
```

### Multiple Operations with Error Checking

```basic
ON ERROR RESUME NEXT

' Try to read a file
content = READ "config.csv"
IF ERROR THEN
    TALK "Config file not found, using defaults"
    CLEAR ERROR
    content = "name,value\ndefault,true"
END IF

' Try to parse it
data = PARSE_CSV(content)
IF ERROR THEN
    TALK "Failed to parse config: " + ERROR MESSAGE
    CLEAR ERROR
END IF

' Continue with whatever data we have
TALK "Loaded " + LEN(data) + " configuration entries"
```

### Database Operations

```basic
ON ERROR RESUME NEXT

' Try to insert a record
INSERT "customers", #{
    "name": customer_name,
    "email": customer_email,
    "created_at": NOW()
}

IF ERROR THEN
    IF INSTR(ERROR MESSAGE, "duplicate") > 0 THEN
        TALK "This customer already exists in our system."
    ELSE IF INSTR(ERROR MESSAGE, "connection") > 0 THEN
        TALK "Database is temporarily unavailable. Please try again."
    ELSE
        TALK "Could not save customer: " + ERROR MESSAGE
    END IF
ELSE
    TALK "Customer saved successfully!"
END IF

ON ERROR GOTO 0  ' Disable error trapping
```

### HTTP Request with Fallback

```basic
ON ERROR RESUME NEXT

' Try primary API
data = GET "https://primary-api.example.com/endpoint"

IF ERROR THEN
    PRINT "Primary API failed: " + ERROR MESSAGE
    CLEAR ERROR
    
    ' Try fallback API
    data = GET "https://fallback-api.example.com/endpoint"
    
    IF ERROR THEN
        TALK "Both APIs are unavailable. Please try again later."
        RETURN
    END IF
END IF

' Process the data
TALK "Retrieved: " + data.name
```

### File Operations with Cleanup

```basic
ON ERROR RESUME NEXT

' Generate a report
report = GENERATE PDF "templates/report.html", report_data, "temp/report.pdf"

IF ERROR THEN
    TALK "Failed to generate report: " + ERROR MESSAGE
    CLEAR ERROR
ELSE
    ' Try to send it
    SEND MAIL recipient, "Your Report", "Please find your report attached.", [report.localName]
    
    IF ERROR THEN
        TALK "Report generated but email failed: " + ERROR MESSAGE
        DOWNLOAD report.url AS "report.pdf"
        TALK "You can download it directly instead."
    ELSE
        TALK "Report sent successfully!"
    END IF
    
    ' Clean up temp file
    DELETE "temp/report.pdf"
    ' Ignore cleanup errors
    CLEAR ERROR
END IF

ON ERROR GOTO 0
```

### Validation with ASSERT

```basic
ON ERROR RESUME NEXT

' Validate input
ASSERT LEN(email) > 0, "Email is required"
IF ERROR THEN
    TALK ERROR MESSAGE
    RETURN
END IF

ASSERT INSTR(email, "@") > 0, "Invalid email format"
IF ERROR THEN
    TALK ERROR MESSAGE
    RETURN
END IF

ASSERT amount > 0, "Amount must be positive"
IF ERROR THEN
    TALK ERROR MESSAGE
    RETURN
END IF

' All validations passed
TALK "Processing your request..."
```

### Custom Error with THROW

```basic
ON ERROR RESUME NEXT

' Check business rules
IF quantity > inventory_count THEN
    THROW "Insufficient inventory: only " + inventory_count + " items available"
END IF

IF customer_credit < total_price THEN
    THROW "Insufficient credit: need $" + FORMAT(total_price - customer_credit, "#,##0.00") + " more"
END IF

' Check for errors from THROW
IF ERROR THEN
    TALK "Cannot process order: " + ERROR MESSAGE
    RETURN
END IF

' Continue with order processing
TALK "Order confirmed!"
```

### Loop with Error Recovery

```basic
ON ERROR RESUME NEXT

successful = 0
failed = 0

FOR EACH item IN items_to_process
    ' Process each item
    result = POST "https://api.example.com/process", item
    
    IF ERROR THEN
        PRINT "Failed to process item " + item.id + ": " + ERROR MESSAGE
        failed = failed + 1
        CLEAR ERROR  ' Clear error to continue loop
    ELSE
        successful = successful + 1
    END IF
NEXT

ON ERROR GOTO 0

TALK "Processing complete: " + successful + " succeeded, " + failed + " failed"
```

### Nested Error Handling

```basic
ON ERROR RESUME NEXT

' Outer operation
connection = CONNECT "database"

IF ERROR THEN
    TALK "Database connection failed"
    RETURN
END IF

' Inner operation
ON ERROR RESUME NEXT
result = QUERY connection, "SELECT * FROM users"

IF ERROR THEN
    TALK "Query failed, trying alternative..."
    CLEAR ERROR
    result = QUERY connection, "SELECT * FROM users_backup"
END IF

IF ERROR THEN
    TALK "All queries failed: " + ERROR MESSAGE
ELSE
    TALK "Found " + LEN(result) + " users"
END IF

ON ERROR GOTO 0
```

---

## Error Codes

Common error codes returned by `ERR`:

| Code | Category | Description |
|------|----------|-------------|
| 1xxx | Network | HTTP and connection errors |
| 2xxx | Database | SQL and data errors |
| 3xxx | File | File system errors |
| 4xxx | Validation | Input validation errors |
| 5xxx | Auth | Authentication/authorization errors |
| 9xxx | System | Internal system errors |

### Checking Error Codes

```basic
ON ERROR RESUME NEXT

result = GET url

IF ERROR THEN
    SELECT CASE ERR
        CASE 1001
            TALK "Network timeout - please try again"
        CASE 1002
            TALK "Server not found"
        CASE 1003
            TALK "Connection refused"
        CASE 1404
            TALK "Resource not found"
        CASE 1500
            TALK "Server error - please try later"
        CASE ELSE
            TALK "Error " + ERR + ": " + ERROR MESSAGE
    END SELECT
END IF
```

---

## Best Practices

### Always Clear Errors When Continuing

```basic
ON ERROR RESUME NEXT

result = RISKY_OPERATION()
IF ERROR THEN
    ' Handle the error
    TALK "Operation failed"
    CLEAR ERROR  ' Clear before next operation
END IF

' Next operation won't see stale error
result2 = ANOTHER_OPERATION()
```

### Disable Error Trapping When Done

```basic
ON ERROR RESUME NEXT

' Protected code section
result = RISKY_OPERATION()
IF ERROR THEN
    TALK "Handled error: " + ERROR MESSAGE
END IF

ON ERROR GOTO 0  ' Re-enable normal error behavior

' Errors here will halt execution as normal
```

### Log Errors for Debugging

```basic
ON ERROR RESUME NEXT

result = COMPLEX_OPERATION()

IF ERROR THEN
    ' Log full details for debugging
    PRINT "ERROR at " + FORMAT(NOW(), "YYYY-MM-DD HH:mm:ss")
    PRINT "  Code: " + ERR
    PRINT "  Message: " + ERROR MESSAGE
    PRINT "  Context: processing " + current_item.id
    
    ' User-friendly message
    TALK "Something went wrong. Our team has been notified."
    CLEAR ERROR
END IF
```

### Don't Ignore Errors Silently

```basic
' BAD - Errors are silently ignored
ON ERROR RESUME NEXT
data = GET url
TALK "Got data: " + data  ' May fail if GET failed

' GOOD - Check and handle errors
ON ERROR RESUME NEXT
data = GET url
IF ERROR THEN
    TALK "Could not fetch data"
    data = DEFAULT_DATA
END IF
TALK "Got data: " + data
```

### Use Specific Error Checks

```basic
ON ERROR RESUME NEXT

result = DATABASE_OPERATION()

IF ERROR THEN
    error_msg = ERROR MESSAGE
    
    ' Handle specific errors differently
    IF INSTR(error_msg, "timeout") > 0 THEN
        WAIT 2
        result = DATABASE_OPERATION()  ' Retry
    ELSE IF INSTR(error_msg, "duplicate") > 0 THEN
        TALK "This record already exists"
    ELSE IF INSTR(error_msg, "permission") > 0 THEN
        TALK "You don't have permission for this action"
    ELSE
        TALK "Database error: " + error_msg
    END IF
END IF
```

---

## Comparison with TRY...CATCH

For more complex error handling, you can use structured `TRY...CATCH` blocks:

```basic
' ON ERROR style (simpler)
ON ERROR RESUME NEXT
result = RISKY()
IF ERROR THEN
    TALK "Error: " + ERROR MESSAGE
END IF
ON ERROR GOTO 0

' TRY...CATCH style (more structured)
TRY
    result = RISKY()
CATCH e
    TALK "Error: " + e.message
END TRY
```

Use `ON ERROR RESUME NEXT` for:
- Simple scripts with few error points
- Quick prototyping
- BASIC-familiar developers

Use `TRY...CATCH` for:
- Complex error handling logic
- Multiple specific catch blocks needed
- Modern structured programming style

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/errors/on_error.rs`
- Error state is thread-local per script execution
- Error trapping scope is global to the script (not block-scoped)
- `CLEAR ERROR` resets both `ERROR` flag and `ERROR MESSAGE`
- Maximum error message length: 4096 characters

---

## Related Keywords

- [THROW](keyword-throw.md) — Raise custom errors
- [ASSERT](keyword-assert.md) — Assert conditions
- [PRINT](keyword-print.md) — Debug output for error logging
- [WAIT](keyword-wait.md) — Delay before retry

---

## Summary

`ON ERROR RESUME NEXT` provides BASIC error handling for BASIC scripts, allowing graceful handling of errors without script termination. Always check `ERROR` after risky operations, use `CLEAR ERROR` before subsequent operations, and disable error trapping with `ON ERROR GOTO 0` when protection is no longer needed. For user-facing errors, provide clear messages while logging technical details for debugging.