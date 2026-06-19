# SCIM API 🟡 BETA

> **SCIM 2.0 (System for Cross-domain Identity Management) for automated user and group provisioning.**

---

## Base URL

```
/scim/v2
```

## Authentication

All endpoints require a valid Bearer token via `Authorization: Bearer <token>` header. The token must have sufficient permissions for identity provider operations.

---

## Overview

The SCIM API implements the [RFC 7644](https://datatracker.ietf.org/doc/html/rfc7644) standard for identity provisioning. It enables external identity providers (Azure AD, Okta, OneLogin, etc.) to automatically create, update, and delete users and groups.

**Supported Features:**
- User CRUD operations
- Group CRUD operations
- Filtering by `userName`
- Pagination via `startIndex` and `count`
- ServiceProviderConfig, ResourceTypes, and Schemas discovery

**Not Supported:**
- Bulk operations
- PATCH (partial update) — use PUT instead
- `GET /Me` — use specific user ID

---

## Endpoints

### Users

#### List Users

**`GET /scim/v2/Users`**

List all users with optional pagination and filtering.

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| startIndex | integer | No | 1 | 1-based index of the first result |
| count | integer | No | 20 | Maximum number of results |
| filter | string | No | — | SCIM filter expression (supports `userName eq "..."`) |

**Response:**
```json
{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
  "totalResults": 2,
  "startIndex": 1,
  "itemsPerPage": 20,
  "Resources": [
    {
      "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
      "id": "user-uuid-1",
      "externalId": "zitadel-user-id",
      "userName": "john.doe",
      "name": {
        "familyName": "Doe",
        "givenName": "John",
        "formatted": "John Doe"
      },
      "active": true,
      "emails": [
        {
          "value": "john.doe@example.com",
          "type": "work",
          "primary": true
        }
      ],
      "groups": [
        {
          "value": "group_admins",
          "$ref": "/Groups/group_admins"
        }
      ],
      "meta": {
        "resourceType": "User",
        "created": "2024-01-15T10:00:00Z",
        "lastModified": "2024-01-15T10:00:00Z"
      }
    }
  ]
}
```

**Filter Example:**
```
GET /scim/v2/Users?filter=userName eq "john.doe"
```

---

#### Create User

**`POST /scim/v2/Users`**

Create a new user in the identity provider.

**Request Body:**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "userName": "jane.smith",
  "name": {
    "familyName": "Smith",
    "givenName": "Jane",
    "formatted": "Jane Smith"
  },
  "emails": [
    {
      "value": "jane.smith@example.com",
      "type": "work",
      "primary": true
    }
  ],
  "active": true
}
```

**Response (201 Created):**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "id": "new-uuid-abc123",
  "userName": "jane.smith",
  "name": {
    "familyName": "Smith",
    "givenName": "Jane"
  },
  "active": true,
  "emails": [
    {
      "value": "jane.smith@example.com",
      "type": "work",
      "primary": true
    }
  ],
  "groups": [],
  "meta": {
    "resourceType": "User",
    "created": "2024-01-15T12:00:00Z",
    "lastModified": "2024-01-15T12:00:00Z",
    "location": "/Users/new-uuid-abc123"
  }
}
```

---

#### Get User

**`GET /scim/v2/Users/{user_id}`**

Retrieve a specific user by ID.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user_id | string | Yes | User UUID or external ID |

**Response:** Same as Create User response, including group memberships.

---

#### Update User

**`PUT /scim/v2/Users/{user_id}`**

Replace the entire user resource. All fields are required.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user_id | string | Yes | User UUID |

**Request Body:** Full SCIM User resource (same as Create).

**Response:** Updated user resource.

---

#### Delete User

**`DELETE /scim/v2/Users/{user_id}`**

Delete a user from the identity provider.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user_id | string | Yes | User UUID |

**Response:** `204 No Content`

---

#### Replace User

**`POST /scim/v2/Users/{user_id}/replace`**

Replace a user resource. Functions identically to `PUT /Users/{user_id}`.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user_id | string | Yes | User UUID |

**Request Body:** Full SCIM User resource.

**Response:** Updated user resource.

---

### Groups

#### List Groups

**`GET /scim/v2/Groups`**

List all groups with optional pagination.

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| startIndex | integer | No | 1 | 1-based index of the first result |
| count | integer | No | 20 | Maximum number of results |

**Response:**
```json
{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
  "totalResults": 1,
  "startIndex": 1,
  "itemsPerPage": 20,
  "Resources": [
    {
      "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group"],
      "id": "group_admins",
      "displayName": "Administrators",
      "members": [
        {
          "value": "user-uuid-1",
          "$ref": "/Users/user-uuid-1",
          "type": "User"
        }
      ],
      "meta": {
        "resourceType": "Group",
        "created": "2024-01-15T10:00:00Z",
        "location": "/Groups/group_admins"
      }
    }
  ]
}
```

---

#### Create Group

**`POST /scim/v2/Groups`**

Create a new group.

**Request Body:**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group"],
  "displayName": "Engineering Team",
  "members": [
    {
      "value": "user-uuid-1",
      "type": "User"
    },
    {
      "value": "user-uuid-2",
      "type": "User"
    }
  ]
}
```

**Response (201 Created):** Created group resource with generated `id`.

---

#### Get Group

**`GET /scim/v2/Groups/{group_id}`**

Retrieve a specific group by ID.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| group_id | string | Yes | Group identifier |

**Response:** Group resource with member details.

---

#### Update Group

**`PUT /scim/v2/Groups/{group_id}`**

Replace the entire group resource. All fields are required.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| group_id | string | Yes | Group identifier |

**Request Body:** Full SCIM Group resource.

**Response:** Updated group resource.

---

#### Delete Group

**`DELETE /scim/v2/Groups/{group_id}`**

Delete a group.

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| group_id | string | Yes | Group identifier |

**Response:** `204 No Content`

---

#### Replace Group

**`POST /scim/v2/Groups/{group_id}/replace`**

Replace a group resource. Functions identically to `PUT /Groups/{group_id}`.

---

### Get Current User

**`GET /scim/v2/Me`**

Returns a `501 Not Implemented` error. Use `GET /Users/{user_id}` with a specific user ID instead.

**Response (501):**
```json
{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:Error"],
  "detail": "GET /Me not supported - use specific user ID",
  "status": 501
}
```

---

### Service Discovery

#### Service Provider Configuration

**`GET /scim/v2/ServiceProviderConfig`**

Returns the SCIM server's capabilities and supported features.

**Response:**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"],
  "patch": { "supported": true },
  "bulk": { "supported": false, "maxOperations": 0, "maxPayloadSize": 0 },
  "filter": { "supported": true, "maxResults": 100 },
  "changePassword": { "supported": false },
  "sort": { "supported": false },
  "etag": { "supported": false },
  "authenticationSchemes": [
    {
      "type": "oauthbearertoken",
      "name": "OAuth Bearer Token",
      "description": "Authentication scheme using the OAuth Bearer Token Standard",
      "specUri": "https://www.rfc-editor.org/info/rfc6750",
      "primary": true
    }
  ]
}
```

---

#### Resource Types

**`GET /scim/v2/ResourceTypes`**

Returns the list of supported SCIM resource types.

**Response:**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
  "resources": [
    {
      "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
      "id": "User",
      "name": "User",
      "endpoint": "/Users",
      "description": "User account",
      "schema": "urn:ietf:params:scim:schemas:core:2.0:User"
    },
    {
      "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
      "id": "Group",
      "name": "Group",
      "endpoint": "/Groups",
      "description": "Group of users",
      "schema": "urn:ietf:params:scim:schemas:core:2.0:Group"
    }
  ]
}
```

---

#### Schemas

**`GET /scim/v2/Schemas`**

Returns the list of supported SCIM schemas and their attribute definitions.

**Response:**
```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Schema"],
  "resources": [
    {
      "id": "urn:ietf:params:scim:schemas:core:2.0:User",
      "name": "User",
      "description": "Core User Schema",
      "attributes": [
        {
          "name": "userName",
          "type": "string",
          "required": true,
          "multiValued": false,
          "description": "Unique identifier for the User"
        }
      ]
    },
    {
      "id": "urn:ietf:params:scim:schemas:core:2.0:Group",
      "name": "Group",
      "description": "Core Group Schema",
      "attributes": [
        {
          "name": "displayName",
          "type": "string",
          "required": true,
          "multiValued": false,
          "description": "A human-readable name for the Group"
        }
      ]
    }
  ]
}
```

---

## Examples

### List Users with Pagination

```bash
curl -X GET "http://localhost:8080/scim/v2/Users?startIndex=1&count=10" \
  -H "Authorization: Bearer $TOKEN"
```

### Filter Users by Username

```bash
curl -X GET 'http://localhost:8080/scim/v2/Users?filter=userName eq "john.doe"' \
  -H "Authorization: Bearer $TOKEN"
```

### Create a User

```bash
curl -X POST http://localhost:8080/scim/v2/Users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
    "userName": "alice.jones",
    "name": {
      "familyName": "Jones",
      "givenName": "Alice"
    },
    "emails": [{"value": "alice@example.com", "type": "work", "primary": true}],
    "active": true
  }'
```

### Get a Group

```bash
curl -X GET http://localhost:8080/scim/v2/Groups/group_admins \
  -H "Authorization: Bearer $TOKEN"
```

### Discover Server Capabilities

```bash
curl -X GET http://localhost:8080/scim/v2/ServiceProviderConfig
```

---

## SCIM Error Format

All errors follow the SCIM 2.0 error schema:

```json
{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:Error"],
  "scimType": "invalidValue",
  "detail": "User not found: ...",
  "status": 404
}
```

| Status | scimType | Description |
|--------|----------|-------------|
| 400 | — | Invalid request or malformed JSON |
| 404 | invalidValue | Resource not found |
| 500 | — | Internal server error |
| 501 | — | Not implemented (e.g., `GET /Me`) |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful deletion) |
| 400 | Bad Request (invalid SCIM payload) |
| 401 | Unauthorized |
| 404 | Not Found |
| 500 | Internal Server Error |
| 501 | Not Implemented |

---

## See Also

- [Organizations API](./organizations-api.md) — Organization management and Office 365 migration
- [Groups API](./groups-api.md) — BotServer native group management
- [User Security](./user-security.md) — User roles and permissions
