# People API 🟡 BETA

> **Human resources management for people profiles, teams, departments, skills, and time-off tracking.**

---

## Base URL

```
/api/people
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## People Management

### List People

**`GET /api/people/`**

Returns all people records with optional filtering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `departmentId` | string | No | Filter by department |
| `teamId` | string | No | Filter by team |
| `search` | string | No | Search by name or email |
| `status` | string | No | `active`, `inactive`, `on_leave` |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 50) |

**Request:**

```
GET /api/people/?departmentId=dept-eng&limit=10
```

**Response:**

```json
{
  "people": [
    {
      "id": "person-001",
      "name": "Maria Santos",
      "email": "maria@example.com",
      "role": "Senior Developer",
      "departmentId": "dept-eng",
      "teamIds": ["team-backend"],
      "status": "active",
      "hireDate": "2024-03-15",
      "avatar": "https://cdn.example.com/avatars/maria.jpg"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 35,
    "totalPages": 4
  }
}
```

---

### Create Person

**`POST /api/people/`**

Creates a new person record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Full name |
| `email` | string | Yes | Email address |
| `role` | string | Yes | Job role/title |
| `departmentId` | string | Yes | Department identifier |
| `hireDate` | string | No | Hire date (ISO 8601) |
| `status` | string | No | `active`, `inactive` (default: `active`) |
| `phone` | string | No | Phone number |
| `location` | string | No | Work location |

**Request:**

```json
{
  "name": "João Silva",
  "email": "joao@example.com",
  "role": "Backend Developer",
  "departmentId": "dept-eng",
  "hireDate": "2026-06-15",
  "phone": "+55 11 99999-0000",
  "location": "São Paulo"
}
```

**Response:**

```json
{
  "id": "person-002",
  "name": "João Silva",
  "email": "joao@example.com",
  "role": "Backend Developer",
  "departmentId": "dept-eng",
  "teamIds": [],
  "status": "active",
  "hireDate": "2026-06-15",
  "phone": "+55 11 99999-0000",
  "location": "São Paulo",
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

### Get Person

**`GET /api/people/:id`**

Returns full details of a specific person.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Person identifier (path param) |

**Request:**

```
GET /api/people/person-001
```

**Response:**

```json
{
  "id": "person-001",
  "name": "Maria Santos",
  "email": "maria@example.com",
  "role": "Senior Developer",
  "departmentId": "dept-eng",
  "departmentName": "Engineering",
  "teamIds": ["team-backend"],
  "teamNames": ["Backend Team"],
  "status": "active",
  "hireDate": "2024-03-15",
  "phone": "+55 11 88888-0000",
  "location": "São Paulo",
  "skills": ["rust", "postgresql", "redis"],
  "avatar": "https://cdn.example.com/avatars/maria.jpg",
  "createdAt": "2024-03-15T10:00:00Z",
  "updatedAt": "2026-05-20T14:00:00Z"
}
```

---

### Update Person

**`PUT /api/people/:id`**

Updates a person's record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Person identifier (path param) |
| `name` | string | No | Full name |
| `role` | string | No | Job role/title |
| `departmentId` | string | No | Department |
| `status` | string | No | `active`, `inactive`, `on_leave` |
| `phone` | string | No | Phone number |
| `location` | string | No | Work location |

**Request:**

```json
{
  "role": "Lead Developer",
  "location": "Remote"
}
```

**Response:**

```json
{
  "id": "person-001",
  "name": "Maria Santos",
  "role": "Lead Developer",
  "location": "Remote",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Delete Person

**`DELETE /api/people/:id`**

Soft-deletes a person (sets status to `inactive`).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Person identifier (path param) |

**Request:**

```
DELETE /api/people/person-002
```

**Response:**

```json
{
  "deleted": true,
  "id": "person-002"
}
```

---

## Reports

### Get Person Reports

**`GET /api/people/:id/reports`**

Returns direct reports and reporting hierarchy for a person.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Person identifier (path param) |

**Request:**

```
GET /api/people/person-001/reports
```

**Response:**

```json
{
  "personId": "person-001",
  "name": "Maria Santos",
  "role": "Lead Developer",
  "directReports": [
    {
      "id": "person-003",
      "name": "Ana Costa",
      "role": "Junior Developer",
      "hireDate": "2025-08-01"
    },
    {
      "id": "person-004",
      "name": "Carlos Lima",
      "role": "Developer",
      "hireDate": "2025-11-15"
    }
  ],
  "reportCount": 2
}
```

---

## Skills

### Add Skill

**`POST /api/people/:id/skills`**

Adds a skill to a person's profile.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Person identifier (path param) |
| `skillId` | string | Yes | Skill identifier |
| `level` | string | No | `beginner`, `intermediate`, `advanced`, `expert` (default: `intermediate`) |

**Request:**

```json
{
  "skillId": "skill-rust",
  "level": "advanced"
}
```

**Response:**

```json
{
  "personId": "person-001",
  "skillId": "skill-rust",
  "skillName": "Rust",
  "level": "advanced",
  "addedAt": "2026-06-04T12:00:00Z"
}
```

---

## Teams

### List Teams

**`GET /api/people/teams`**

Returns all teams.

**Response:**

```json
[
  {
    "id": "team-backend",
    "name": "Backend Team",
    "departmentId": "dept-eng",
    "leadId": "person-001",
    "memberCount": 8,
    "description": "Core API and infrastructure development"
  },
  {
    "id": "team-frontend",
    "name": "Frontend Team",
    "departmentId": "dept-eng",
    "leadId": "person-005",
    "memberCount": 6,
    "description": "UI/UX development and HTMX apps"
  }
]
```

---

### Create Team

**`POST /api/people/teams`**

Creates a new team.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Team name |
| `departmentId` | string | Yes | Department identifier |
| `leadId` | string | No | Team lead person ID |
| `description` | string | No | Team description |

**Request:**

```json
{
  "name": "DevOps",
  "departmentId": "dept-eng",
  "leadId": "person-006",
  "description": "Infrastructure, CI/CD, and deployment automation"
}
```

**Response:**

```json
{
  "id": "team-devops",
  "name": "DevOps",
  "departmentId": "dept-eng",
  "leadId": "person-006",
  "description": "Infrastructure, CI/CD, and deployment automation",
  "memberCount": 0,
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

### Get Team

**`GET /api/people/teams/:id`**

Returns full team details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Team identifier (path param) |

**Request:**

```
GET /api/people/teams/team-backend
```

**Response:**

```json
{
  "id": "team-backend",
  "name": "Backend Team",
  "departmentId": "dept-eng",
  "departmentName": "Engineering",
  "leadId": "person-001",
  "leadName": "Maria Santos",
  "description": "Core API and infrastructure development",
  "memberCount": 8,
  "members": [
    { "id": "person-001", "name": "Maria Santos", "role": "Lead Developer" },
    { "id": "person-003", "name": "Ana Costa", "role": "Junior Developer" },
    { "id": "person-004", "name": "Carlos Lima", "role": "Developer" }
  ],
  "createdAt": "2024-01-10T08:00:00Z"
}
```

---

### Delete Team

**`DELETE /api/people/teams/:id`**

Deletes a team. Members are not removed from the system.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Team identifier (path param) |

**Request:**

```
DELETE /api/people/teams/team-devops
```

**Response:**

```json
{
  "deleted": true,
  "id": "team-devops"
}
```

---

### Add Team Member

**`POST /api/people/teams/:id/members`**

Adds a person to a team.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Team identifier (path param) |
| `personId` | string | Yes | Person to add |
| `role` | string | No | Role within the team |

**Request:**

```json
{
  "personId": "person-004",
  "role": "Backend Developer"
}
```

**Response:**

```json
{
  "teamId": "team-backend",
  "personId": "person-004",
  "name": "Carlos Lima",
  "role": "Backend Developer",
  "addedAt": "2026-06-04T12:00:00Z"
}
```

---

### Remove Team Member

**`DELETE /api/people/teams/:team_id/members/:person_id`**

Removes a person from a team.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `team_id` | string | Yes | Team identifier (path param) |
| `person_id` | string | Yes | Person identifier (path param) |

**Request:**

```
DELETE /api/people/teams/team-backend/members/person-004
```

**Response:**

```json
{
  "removed": true,
  "teamId": "team-backend",
  "personId": "person-004"
}
```

---

## Departments

### List Departments

**`GET /api/people/departments`**

Returns all departments.

**Response:**

```json
[
  {
    "id": "dept-eng",
    "name": "Engineering",
    "headId": "person-001",
    "headName": "Maria Santos",
    "teamCount": 3,
    "personCount": 15
  },
  {
    "id": "dept-sales",
    "name": "Sales",
    "headId": "person-010",
    "headName": "Pedro Almeida",
    "teamCount": 2,
    "personCount": 8
  }
]
```

---

### Create Department

**`POST /api/people/departments`**

Creates a new department.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Department name |
| `headId` | string | No | Department head person ID |
| `description` | string | No | Department description |

**Request:**

```json
{
  "name": "Quality Assurance",
  "description": "Software testing and quality control"
}
```

**Response:**

```json
{
  "id": "dept-qa",
  "name": "Quality Assurance",
  "description": "Software testing and quality control",
  "teamCount": 0,
  "personCount": 0,
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

## Skills (Global)

### List Skills

**`GET /api/people/skills`**

Returns all available skills in the system.

**Response:**

```json
[
  { "id": "skill-rust", "name": "Rust", "category": "Programming", "personCount": 4 },
  { "id": "skill-react", "name": "React", "category": "Frontend", "personCount": 6 },
  { "id": "skill-postgres", "name": "PostgreSQL", "category": "Database", "personCount": 8 }
]
```

---

### Create Skill

**`POST /api/people/skills`**

Creates a new skill definition.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Skill name |
| `category` | string | No | Skill category |

**Request:**

```json
{
  "name": "Kubernetes",
  "category": "Infrastructure"
}
```

**Response:**

```json
{
  "id": "skill-k8s",
  "name": "Kubernetes",
  "category": "Infrastructure",
  "personCount": 0,
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

## Time Off

### List Time Off Requests

**`GET /api/people/time-off`**

Returns time off requests with optional filtering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `personId` | string | No | Filter by person |
| `status` | string | No | `pending`, `approved`, `rejected` |
| `startDate` | string | No | From date (ISO 8601) |
| `endDate` | string | No | To date (ISO 8601) |

**Request:**

```
GET /api/people/time-off?status=pending
```

**Response:**

```json
[
  {
    "id": "to-001",
    "personId": "person-003",
    "personName": "Ana Costa",
    "type": "vacation",
    "startDate": "2026-07-01",
    "endDate": "2026-07-14",
    "days": 10,
    "status": "pending",
    "requestedAt": "2026-06-01T10:00:00Z"
  }
]
```

---

### Create Time Off Request

**`POST /api/people/time-off`**

Creates a new time off request.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `personId` | string | Yes | Person requesting time off |
| `type` | string | Yes | `vacation`, `sick`, `personal`, `other` |
| `startDate` | string | Yes | Start date (ISO 8601) |
| `endDate` | string | Yes | End date (ISO 8601) |
| `reason` | string | No | Optional reason |

**Request:**

```json
{
  "personId": "person-001",
  "type": "vacation",
  "startDate": "2026-08-01",
  "endDate": "2026-08-15",
  "reason": "Summer vacation"
}
```

**Response:**

```json
{
  "id": "to-002",
  "personId": "person-001",
  "personName": "Maria Santos",
  "type": "vacation",
  "startDate": "2026-08-01",
  "endDate": "2026-08-15",
  "days": 11,
  "reason": "Summer vacation",
  "status": "pending",
  "requestedAt": "2026-06-04T12:00:00Z"
}
```

---

### Approve Time Off

**`PUT /api/people/time-off/:id/approve`**

Approves or rejects a time off request.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Time off request identifier (path param) |
| `approved` | boolean | Yes | Approval decision |
| `comment` | string | No | Comment |

**Request:**

```json
{
  "approved": true,
  "comment": "Approved. Ensure handover to team lead before departure."
}
```

**Response:**

```json
{
  "id": "to-001",
  "personId": "person-003",
  "status": "approved",
  "approvedBy": "person-001",
  "approvedAt": "2026-06-04T14:00:00Z",
  "comment": "Approved. Ensure handover to team lead before departure."
}
```

---

## Statistics

### Get People Stats

**`GET /api/people/stats`**

Returns aggregate people statistics.

**Response:**

```json
{
  "totalPeople": 58,
  "activePeople": 52,
  "byDepartment": [
    { "departmentId": "dept-eng", "name": "Engineering", "count": 25 },
    { "departmentId": "dept-sales", "name": "Sales", "count": 12 },
    { "departmentId": "dept-hr", "name": "Human Resources", "count": 5 },
    { "departmentId": "dept-finance", "name": "Finance", "count": 6 }
  ],
  "pendingTimeOffRequests": 3,
  "newHiresThisMonth": 2,
  "lastUpdated": "2026-06-04T12:00:00Z"
}
```

---

## See Also

- [Users API](../08-rest-api-tools/users-api.md) — User account and authentication management
- [Groups API](../08-rest-api-tools/groups-api.md) — User group management
- [Calendar API](../08-rest-api-tools/calendar-api.md) — Event and scheduling integration
- [Tasks API](../08-rest-api-tools/tasks-api.md) — Task assignment to people
