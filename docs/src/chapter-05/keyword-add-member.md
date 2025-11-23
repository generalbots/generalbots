# ADD MEMBER

Add users to groups or teams within the bot system.

## Syntax

```basic
ADD MEMBER group_id, user_email, role
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `group_id` | String | Unique identifier or name of the group |
| `user_email` | String | Email address of the user to add |
| `role` | String | Role to assign to the user in the group (e.g., "member", "admin", "moderator") |

## Description

The `ADD MEMBER` keyword manages group membership by adding users to specified groups with defined roles. This keyword:

- Validates user existence in the system
- Checks group permissions
- Assigns appropriate role-based access
- Sends notification to the new member
- Updates group analytics

## Examples

### Add Regular Member
```basic
ADD MEMBER "engineering-team", "developer@company.com", "member"
```

### Add Group Administrator
```basic
ADD MEMBER "project-alpha", "manager@company.com", "admin"
```

### Dynamic Group Assignment
```basic
group_name = "support-" + DEPARTMENT
user = GET_USER_EMAIL()
ADD MEMBER group_name, user, "agent"
```

### Bulk Member Addition
```basic
team_members = ["alice@example.com", "bob@example.com", "carol@example.com"]
FOR EACH member IN team_members
    ADD MEMBER "dev-team", member, "developer"
NEXT
```

## Return Value

Returns a membership object containing:
- `membership_id`: Unique identifier for the membership
- `group_id`: Group identifier
- `user_id`: User identifier
- `role`: Assigned role
- `joined_at`: Timestamp of when the user was added
- `status`: "active", "pending", or "error"

## Error Handling

- Validates email format before processing
- Checks if user already exists in the group
- Verifies current user has permission to add members
- Returns error if group doesn't exist
- Handles role validation against allowed roles

## Permissions

The user executing this command must have one of the following:
- Group admin privileges
- System administrator role
- Explicit "add_member" permission for the group

## Related Database Tables

- `users`: User information
- `groups`: Group definitions
- `group_members`: Membership associations
- `roles`: Role definitions and permissions

## Best Practices

1. **Verify Permissions**: Always check if the executing user has rights to add members
2. **Validate Roles**: Ensure the role exists and is valid for the group type
3. **Send Notifications**: Notify both the new member and group admins
4. **Audit Trail**: Log all membership changes for compliance
5. **Handle Duplicates**: Check for existing membership before adding

## See Also

- [CREATE TASK](./keyword-create-task.md) - Assign tasks to group members
- [SEND MAIL](./keyword-send-mail.md) - Send group notifications
- [SET USER](./keyword-set-user.md) - Set user context

## Implementation

Located in `src/basic/keywords/add_member.rs`

The implementation uses:
- Database transactions for atomic operations
- Email validation via regex
- Role-based access control (RBAC)
- Async notification dispatch