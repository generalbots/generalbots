# ADD MEMBER Keywords

Manage team and group membership within bots.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `ADD_MEMBER` | Add user to a group with role |
| `REMOVE_MEMBER` | Remove user from group |
| `CREATE_TEAM` | Create a new team |
| `LIST_MEMBERS` | List group members |

## ADD_MEMBER

```basic
result = ADD_MEMBER group_id, user_email, role
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `group_id` | String | Team or group identifier |
| `user_email` | String | Email of user to add |
| `role` | String | Role: "admin", "member", "viewer" |

### Example

```basic
result = ADD_MEMBER "team-sales", "john@company.com", "member"
TALK "Added user: " + result
```

## REMOVE_MEMBER

```basic
result = REMOVE_MEMBER "team-sales", "john@company.com"
```

## CREATE_TEAM

```basic
members = ["alice@company.com", "bob@company.com"]
result = CREATE_TEAM "Project Alpha", "Development team", members
```

## LIST_MEMBERS

```basic
members = LIST_MEMBERS "team-sales"
FOR EACH member IN members
    TALK member.email + " - " + member.role
NEXT
```

## Roles

| Role | Permissions |
|------|-------------|
| `admin` | Full control, manage members |
| `member` | Standard access |
| `viewer` | Read-only access |

## See Also

- [ADD BOT](./keyword-add-bot.md)
- [User Session Handling](../chapter-10-features/user-sessions.md)