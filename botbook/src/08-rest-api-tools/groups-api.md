# Groups API

The Groups API provides endpoints for managing groups and organizations through Zitadel integration.

## Overview

Groups in botserver represent organizations in Zitadel. They provide multi-tenant support and user grouping capabilities.

## Endpoints

### Create Group

**POST** `/groups/create`

Creates a new group/organization.

**Request:**
```json
{
  "name": "Engineering Team",
  "description": "Software engineering department",
  "domain": "engineering.example.com"
}
```

**Response:**
```json
{
  "id": "org-123",
  "name": "Engineering Team",
  "created_at": "2024-01-20T10:00:00Z"
}
```

### Update Group

**PUT** `/groups/:id/update`

Updates group information.

**Request:**
```json
{
  "name": "Updated Name",
  "description": "Updated description"
}
```

**Response:**
```json
{
  "id": "org-123",
  "name": "Updated Name",
  "updated_at": "2024-01-20T11:00:00Z"
}
```

### Delete Group

**DELETE** `/groups/:id/delete`

Deletes a group/organization.

**Response:**
```json
{
  "success": true,
  "message": "Group deleted successfully"
}
```

### List Groups

**GET** `/groups/list`

Lists all groups accessible to the user.

**Query Parameters:**
- `limit` - Maximum number of results (default: 20)
- `offset` - Pagination offset

**Response:**
```json
{
  "groups": [
    {
      "id": "org-123",
      "name": "Engineering Team",
      "member_count": 25,
      "created_at": "2024-01-20T10:00:00Z"
    }
  ],
  "total": 1
}
```

### Get Group Members

**GET** `/groups/:id/members`

Retrieves members of a specific group.

**Response:**
```json
{
  "members": [
    {
      "user_id": "user-456",
      "username": "john_doe",
      "email": "john@example.com",
      "role": "member",
      "joined_at": "2024-01-15T09:00:00Z"
    }
  ],
  "total": 1
}
```

### Add Group Member

**POST** `/groups/:id/members/add`

Adds a user to a group.

**Request:**
```json
{
  "user_id": "user-789",
  "role": "member"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Member added successfully"
}
```

### Remove Group Member

**DELETE** `/groups/:id/members/remove`

Removes a user from a group.

**Request:**
```json
{
  "user_id": "user-789"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Member removed successfully"
}
```

## Implementation Details

### Zitadel Integration

All group operations are proxied to Zitadel:
- Groups map to Zitadel organizations
- Members are managed through Zitadel's org API
- Permissions inherited from Zitadel roles

### Data Model

Groups are not stored in botserver's database. All data comes from Zitadel:
- Group metadata from Zitadel orgs
- Membership from Zitadel org members
- Permissions from Zitadel policies

## Error Responses

All endpoints may return standard error responses:

```json
{
  "error": "Group not found",
  "code": "GROUP_NOT_FOUND",
  "status": 404
}
```

Common error codes:
- `GROUP_NOT_FOUND` - Group doesn't exist
- `UNAUTHORIZED` - User lacks permission
- `MEMBER_EXISTS` - User already in group
- `MEMBER_NOT_FOUND` - User not in group
- `ZITADEL_ERROR` - Upstream service error

## Permissions

Group operations require appropriate Zitadel permissions:
- **Create**: Organization admin
- **Update**: Organization owner or admin
- **Delete**: Organization owner
- **List**: Authenticated user
- **View Members**: Group member
- **Add/Remove Members**: Group admin

## Rate Limiting

Group endpoints are rate-limited:
- 100 requests per minute for read operations
- 20 requests per minute for write operations

## Best Practices

1. **Cache Group Data**: Groups change infrequently
2. **Batch Operations**: Use bulk endpoints when available
3. **Handle Zitadel Errors**: Gracefully handle upstream failures
4. **Validate Permissions**: Check user has required role
5. **Audit Changes**: Log all group modifications