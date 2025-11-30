# DELETE

The `DELETE` keyword removes resources using dynamic path interpretation, similar to how `GET` works. The system automatically determines the appropriate operation based on the path provided.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Syntax

```basic
DELETE path
DELETE path, options
```

## Dynamic Path Interpretation

Like `GET`, `DELETE` interprets the path and selects the appropriate engine:

| Path Pattern | Operation |
|--------------|-----------|
| `/files/document.pdf` | Delete file from storage |
| `/users/user-id` | Delete user |
| `/tasks/task-id` | Delete task |
| `/projects/project-id` | Delete project |
| `https://api.example.com/items/123` | HTTP DELETE to external API |

## Examples

### Delete a File

```basic
DELETE "/reports/old-report.pdf"
TALK "File deleted"
```

### Delete from External API

```basic
DELETE "https://api.crm.com/contacts/12345"
```

### Delete with Condition

```basic
' Delete all files older than 30 days
files = LIST "/temp/"
FOR EACH file IN files
    IF DATEDIFF("day", file.modified, NOW()) > 30 THEN
        DELETE "/temp/" + file.name
    END IF
NEXT file
```

### Delete a Task

```basic
DELETE "/tasks/" + task_id
TALK "Task removed"
```

### Delete a User

```basic
DELETE "/users/" + user_id
```

### Delete a Project

```basic
DELETE "/projects/" + project_id
```

## Options

Pass options as a second parameter for additional control:

```basic
' Soft delete (archive instead of permanent removal)
DELETE "/files/report.pdf", #{soft: true}

' Force delete (bypass confirmation)
DELETE "/files/temp/", #{force: true, recursive: true}
```

## Return Value

`DELETE` returns information about the operation:

```basic
result = DELETE "/files/document.pdf"
IF result.success THEN
    TALK "Deleted: " + result.path
ELSE
    TALK "Failed: " + result.error
END IF
```

## HTTP DELETE

When the path is a full URL, `DELETE` performs an HTTP DELETE request:

```basic
' Delete via REST API
DELETE "https://api.service.com/items/456"

' With authentication
SET HEADER "Authorization", "Bearer " + token
DELETE "https://api.service.com/items/456"
```

## Database Records

For database operations, use the `DELETE` keyword with table syntax:

```basic
' Delete specific records
DELETE "orders", "status = 'cancelled' AND created_at < '2024-01-01'"

' Delete by ID
DELETE "customers", "id = '" + customer_id + "'"
```

## Best Practices

**Verify before deleting.** Confirm the resource exists and the user has permission:

```basic
file = GET "/files/" + filename
IF file THEN
    DELETE "/files/" + filename
ELSE
    TALK "File not found"
END IF
```

**Use soft deletes for important data.** Archive rather than permanently remove:

```basic
' Move to archive instead of delete
MOVE "/active/" + filename, "/archive/" + filename
```

**Log deletions for audit trails:**

```basic
DELETE "/files/" + filename
INSERT "audit_log", #{
    action: "delete",
    path: filename,
    user: user.id,
    timestamp: NOW()
}
```

## See Also

- [GET](./keyword-get.md) - Dynamic resource retrieval
- [LIST](./keyword-list.md) - List resources before deletion
- [MOVE](./keyword-move.md) - Move instead of delete