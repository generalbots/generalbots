# DELETE HTTP

> **Deprecated:** The `DELETE HTTP` syntax is kept for backwards compatibility. Use the unified `DELETE` keyword instead, which auto-detects HTTP URLs.

---

## Redirect to DELETE

The `DELETE` keyword now automatically handles HTTP DELETE requests when given a URL:

```basic
' Preferred - unified DELETE
DELETE "https://api.example.com/resource/123"

' Also works (backwards compatibility)
DELETE HTTP "https://api.example.com/resource/123"
```

---

## See Also

- **[DELETE](keyword-delete.md)** — Unified delete keyword (recommended)

The unified `DELETE` keyword automatically detects:
- HTTP URLs → HTTP DELETE request
- Table + filter → Database delete
- File path → File delete

---

## Quick Example

```basic
' Set authentication header
SET HEADER "Authorization", "Bearer " + api_token

' Delete resource via API
DELETE "https://api.example.com/users/456"

' Clear headers
CLEAR HEADERS

TALK "User deleted"
```

---

## Migration

Replace this:
```basic
DELETE HTTP "https://api.example.com/resource/123"
```

With this:
```basic
DELETE "https://api.example.com/resource/123"
```

Both work, but the unified `DELETE` is cleaner and more intuitive.