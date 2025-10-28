# auth.bas (Template)

This template demonstrates a simple authentication flow using the BASIC dialog language.

```basic
REM Simple authentication flow
SET attempts = 0
LABEL auth_loop
HEAR password
IF password = "secret123" THEN
    TALK "Authentication successful."
ELSE
    SET attempts = attempts + 1
    IF attempts >= 3 THEN
        TALK "Too many attempts. Goodbye."
        EXIT
    ENDIF
    TALK "Incorrect password. Try again."
    GOTO auth_loop
ENDIF
```

**Purpose**

- Shows how to collect a password with `HEAR`.
- Limits the number of attempts to three.
- Uses `TALK` to give feedback and `EXIT` to end the dialog after too many failures.

**Keywords used:** `SET`, `HEAR`, `IF`, `ELSE`, `GOTO`, `EXIT`, `TALK`.

---
