# Group Membership API

BotServer provides RESTful endpoints for managing user groups, team memberships, and collaborative workspaces.

## Overview

The Group Membership API enables:
- Group creation and management
- Member addition and removal
- Role assignments within groups
- Permission inheritance
- Team collaboration features
- Workspace organization

## Base URL

```
http://localhost:8080/api/v1/groups
```

## Authentication

All Group Membership API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Create Group

**POST** `/groups`

Create a new group or team.

**Request Body:**
```json
{
  "name": "Engineering Team",
  "description": "Product engineering team",
  "type": "team",
  "visibility": "private",
  "settings": {
    "allow_join_requests": true,
    "require_approval": true,
    "max_members": 50
  },
  "permissions": ["bot.use", "kb.read", "reports.view"]
}
```

**Response:**
```json
{
  "group_id": "grp_abc123",
  "name": "Engineering Team",
  "type": "team",
  "created_at": "2024-01-15T10:00:00Z",
  "created_by": "usr_admin",
  "member_count": 0
}
```

### Get Group

**GET** `/groups/{group_id}`

Retrieve group information.

**Response:**
```json
{
  "group_id": "grp_abc123",
  "name": "Engineering Team",
  "description": "Product engineering team",
  "type": "team",
  "visibility": "private",
  "member_count": 12,
  "created_at": "2024-01-15T10:00:00Z",
  "settings": {
    "allow_join_requests": true,
    "require_approval": true,
    "max_members": 50
  },
  "permissions": ["bot.use", "kb.read", "reports.view"]
}
```

### Update Group

**PATCH** `/groups/{group_id}`

Update group information.

**Request Body:**
```json
{
  "name": "Engineering & DevOps Team",
  "description": "Combined engineering and operations team",
  "settings": {
    "max_members": 75
  }
}
```

### Delete Group

**DELETE** `/groups/{group_id}`

Delete a group (requires admin permissions).

**Response:**
```json
{
  "deleted": true,
  "group_id": "grp_abc123",
  "members_removed": 12
}
```

### List Groups

**GET** `/groups`

List all groups with filtering.

**Query Parameters:**
- `type` - Filter by group type: `team`, `department`, `project`
- `visibility` - Filter by visibility: `public`, `private`
- `member` - Filter groups containing specific user
- `search` - Search in name and description
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)

**Response:**
```json
{
  "groups": [
    {
      "group_id": "grp_abc123",
      "name": "Engineering Team",
      "type": "team",
      "member_count": 12,
      "visibility": "private"
    }
  ],
  "total": 8,
  "page": 1,
  "limit": 20
}
```

## Member Management

### Add Member

**POST** `/groups/{group_id}/members`

Add a member to a group.

**Request Body:**
```json
{
  "user_id": "usr_xyz789",
  "role": "member",
  "permissions": ["read", "write"],
  "notify": true
}
```

**Response:**
```json
{
  "membership_id": "mem_123",
  "group_id": "grp_abc123",
  "user_id": "usr_xyz789",
  "role": "member",
  "joined_at": "2024-01-15T10:30:00Z"
}
```

### Bulk Add Members

**POST** `/groups/{group_id}/members/bulk`

Add multiple members at once.

**Request Body:**
```json
{
  "members": [
    {"user_id": "usr_001", "role": "admin"},
    {"user_id": "usr_002", "role": "member"},
    {"user_id": "usr_003", "role": "member"}
  ],
  "notify_all": true
}
```

**Response:**
```json
{
  "added": 3,
  "failed": 0,
  "memberships": [
    {"user_id": "usr_001", "status": "added"},
    {"user_id": "usr_002", "status": "added"},
    {"user_id": "usr_003", "status": "added"}
  ]
}
```

### List Members

**GET** `/groups/{group_id}/members`

List group members.

**Query Parameters:**
- `role` - Filter by role
- `status` - Filter by status: `active`, `pending`, `suspended`
- `search` - Search in member names
- `page` - Page number
- `limit` - Items per page

**Response:**
```json
{
  "members": [
    {
      "membership_id": "mem_123",
      "user": {
        "user_id": "usr_xyz789",
        "username": "johndoe",
        "full_name": "John Doe",
        "avatar_url": "https://example.com/avatar.jpg"
      },
      "role": "admin",
      "status": "active",
      "joined_at": "2024-01-15T10:30:00Z",
      "last_active": "2024-01-15T14:00:00Z"
    }
  ],
  "total": 12,
  "page": 1,
  "limit": 20
}
```

### Update Member Role

**PATCH** `/groups/{group_id}/members/{user_id}`

Update a member's role or permissions.

**Request Body:**
```json
{
  "role": "admin",
  "permissions": ["read", "write", "delete"]
}
```

### Remove Member

**DELETE** `/groups/{group_id}/members/{user_id}`

Remove a member from a group.

**Response:**
```json
{
  "removed": true,
  "group_id": "grp_abc123",
  "user_id": "usr_xyz789",
  "removed_at": "2024-01-15T15:00:00Z"
}
```

## Group Roles

### List Roles

**GET** `/groups/{group_id}/roles`

List available roles in a group.

**Response:**
```json
{
  "roles": [
    {
      "role_id": "owner",
      "name": "Owner",
      "permissions": ["all"],
      "member_count": 1
    },
    {
      "role_id": "admin",
      "name": "Administrator",
      "permissions": ["manage_members", "manage_settings", "read", "write"],
      "member_count": 2
    },
    {
      "role_id": "member",
      "name": "Member",
      "permissions": ["read", "write"],
      "member_count": 9
    }
  ]
}
```

### Create Custom Role

**POST** `/groups/{group_id}/roles`

Create a custom role for a group.

**Request Body:**
```json
{
  "name": "Moderator",
  "permissions": ["read", "write", "moderate"],
  "description": "Can moderate content and manage posts"
}
```

## Join Requests

### Request to Join

**POST** `/groups/{group_id}/join-requests`

Request to join a private group.

**Request Body:**
```json
{
  "message": "I would like to join the engineering team",
  "referred_by": "usr_admin"
}
```

**Response:**
```json
{
  "request_id": "req_456",
  "group_id": "grp_abc123",
  "user_id": "usr_xyz789",
  "status": "pending",
  "submitted_at": "2024-01-15T10:00:00Z"
}
```

### List Join Requests

**GET** `/groups/{group_id}/join-requests`

List pending join requests (admin only).

**Response:**
```json
{
  "requests": [
    {
      "request_id": "req_456",
      "user": {
        "user_id": "usr_xyz789",
        "username": "newuser",
        "full_name": "New User"
      },
      "message": "I would like to join the engineering team",
      "status": "pending",
      "submitted_at": "2024-01-15T10:00:00Z"
    }
  ],
  "total": 3
}
```

### Approve/Reject Request

**PATCH** `/groups/{group_id}/join-requests/{request_id}`

Process a join request.

**Request Body:**
```json
{
  "action": "approve",
  "role": "member",
  "note": "Welcome to the team!"
}
```

## Group Invitations

### Send Invitation

**POST** `/groups/{group_id}/invitations`

Invite users to join a group.

**Request Body:**
```json
{
  "emails": ["user1@example.com", "user2@example.com"],
  "role": "member",
  "message": "You're invited to join our team!",
  "expires_in_days": 7
}
```

**Response:**
```json
{
  "invitations": [
    {
      "invitation_id": "inv_789",
      "email": "user1@example.com",
      "status": "sent",
      "expires_at": "2024-01-22T10:00:00Z"
    }
  ],
  "sent": 2,
  "failed": 0
}
```

### Accept Invitation

**POST** `/invitations/{invitation_id}/accept`

Accept a group invitation.

**Response:**
```json
{
  "membership_id": "mem_999",
  "group_id": "grp_abc123",
  "joined_at": "2024-01-15T11:00:00Z"
}
```

## Group Permissions

### Get Group Permissions

**GET** `/groups/{group_id}/permissions`

List group permissions.

**Response:**
```json
{
  "permissions": [
    {
      "permission": "bot.use",
      "description": "Use bots",
      "inherited_from": null
    },
    {
      "permission": "kb.read",
      "description": "Read knowledge base",
      "inherited_from": "parent_group"
    }
  ]
}
```

### Update Permissions

**PATCH** `/groups/{group_id}/permissions`

Update group permissions.

**Request Body:**
```json
{
  "add": ["reports.create", "analytics.view"],
  "remove": ["kb.write"]
}
```

## Hierarchical Groups

### Create Subgroup

**POST** `/groups/{parent_id}/subgroups`

Create a subgroup under a parent group.

**Request Body:**
```json
{
  "name": "Frontend Team",
  "inherit_permissions": true,
  "inherit_members": false
}
```

### List Subgroups

**GET** `/groups/{group_id}/subgroups`

List all subgroups.

**Response:**
```json
{
  "subgroups": [
    {
      "group_id": "grp_sub123",
      "name": "Frontend Team",
      "member_count": 5,
      "created_at": "2024-01-15T10:00:00Z"
    }
  ],
  "total": 2
}
```

## Group Analytics

### Get Group Analytics

**GET** `/groups/{group_id}/analytics`

Get group activity analytics.

**Response:**
```json
{
  "group_id": "grp_abc123",
  "analytics": {
    "member_growth": {
      "current": 12,
      "last_month": 10,
      "growth_rate": 0.20
    },
    "activity": {
      "messages_sent": 456,
      "tasks_completed": 23,
      "avg_response_time": 3600
    },
    "engagement": {
      "active_members": 10,
      "engagement_rate": 0.83
    }
  },
  "period": "30d"
}
```

## Error Responses

### 403 Forbidden
```json
{
  "error": "permission_denied",
  "message": "You don't have permission to manage this group"
}
```

### 409 Conflict
```json
{
  "error": "member_exists",
  "message": "User is already a member of this group"
}
```

### 422 Unprocessable Entity
```json
{
  "error": "group_full",
  "message": "Group has reached maximum member limit",
  "max_members": 50,
  "current_members": 50
}
```

## Best Practices

1. **Use Descriptive Names**: Group names should clearly indicate purpose
2. **Set Member Limits**: Prevent groups from becoming too large
3. **Regular Cleanup**: Remove inactive members periodically
4. **Permission Inheritance**: Use hierarchy for easier management
5. **Document Purpose**: Always include group descriptions
6. **Review Requests**: Don't auto-approve join requests for sensitive groups

## Related APIs

- [User Security API](./user-security.md) - User management
- [Notifications API](./notifications-api.md) - Group notifications
- [Tasks API](./tasks-api.md) - Group task management