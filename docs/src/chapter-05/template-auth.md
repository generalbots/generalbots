# auth.bas

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

This template demonstrates a basic password check with a limited number of attempts. It uses the `HEAR`, `TALK`, `SET`, `IF`, `ELSE`, `GOTO`, and `EXIT` keywords to manage the dialog flow.
