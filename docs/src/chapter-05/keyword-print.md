# PRINT Keyword

**Syntax**

```
PRINT $expression
```

**Parameters**

- `$expression` – Any valid BASIC expression whose evaluated value will be printed to the server log.

**Description**

`PRINT` evaluates the given expression and writes its string representation to the application log (using the `log::info!` macro). It does not send any output back to the user; it is primarily a debugging aid for developers to inspect variable values or intermediate results during script execution.

**Example**

```basic
SET total = 42
PRINT total          ; logs "42"
PRINT "User: " + user_name   ; logs "User: Alice"
```

When the script runs, the values are written to the server’s log file and can be viewed in the console or log aggregation system.

**Implementation Notes**

- The keyword always returns `UNIT` (no value) to the script.
- It runs synchronously and does not affect the dialog flow.
