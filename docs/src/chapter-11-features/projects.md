# Projects

Projects organize work and enable team collaboration within General Bots. A project groups related tasks, conversations, documents, and team members into a shared workspace where everyone stays aligned.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Every bot starts with a default project. Users can create additional projects to organize different initiatives, clients, or departments. When chatting with the bot, selecting a project scopes the conversation to that context—the AI understands what you're working on and who else is involved.

Projects connect three core capabilities:

- **Tasks** belong to projects, making it easy to track work across teams
- **Conversations** can be project-scoped, so the AI has relevant context
- **Team members** are assigned to projects, enabling collaboration

## Creating Projects

### Via Chat

```
User: Create a new project called Website Redesign
Bot: Project "Website Redesign" created. Would you like to add team members?
User: Yes, add maria@company.com and john@company.com
Bot: Added Maria and John to the project. You can now assign tasks and chat within this project context.
```

### Via BASIC

```basic
' create-project.bas
PARAM name AS string
PARAM description AS string

DESCRIPTION "Create a new project workspace"

project_id = CREATE PROJECT name WITH DESCRIPTION description
TALK "Project created: " + name

' Add current user as owner
ADD USER TO PROJECT project_id, user.id, "owner"
```

### Via API

```
POST /api/projects
{
  "name": "Website Redesign",
  "description": "Q2 website refresh initiative",
  "members": ["user-id-1", "user-id-2"]
}
```

## Project Structure

Each project contains:

```
Project
├── Settings
│   ├── Name and description
│   ├── Visibility (private/team/public)
│   └── Default assignee
├── Members
│   ├── Owners (full control)
│   ├── Members (can edit)
│   └── Viewers (read only)
├── Tasks
│   ├── Boards and columns
│   ├── Task items
│   └── Dependencies
├── Conversations
│   ├── Project chat history
│   └── AI context from project docs
└── Files
    └── Project-specific documents
```

## Working with Projects

### Switching Project Context

In the chat interface, select a project from the project dropdown. All subsequent conversations use that project's context:

```
[Project: Website Redesign ▼]

User: What tasks are due this week?
Bot: In the Website Redesign project, you have 3 tasks due this week:
     1. Finalize homepage mockups (Friday)
     2. Review navigation structure (Wednesday)
     3. Content inventory spreadsheet (Thursday)
```

### Project-Scoped AI

When a project is selected, the AI automatically:

- References project documents in responses
- Knows about team members and their roles
- Understands project-specific terminology from past conversations
- Suggests relevant tasks and next steps

```
User: Who's working on the design?
Bot: Maria is assigned to the design tasks. She completed the color palette 
     yesterday and is currently working on the homepage mockups. John is 
     handling the technical implementation once designs are approved.
```

### Team Collaboration

Project members can chat together with AI assistance:

```
Maria: @bot summarize what we discussed about the navigation
Bot: Based on your project conversations, the team decided on:
     - Simplified top nav with 5 main sections
     - Mega menu for product categories
     - Mobile-first hamburger menu
     John raised concerns about dropdown accessibility that still need addressing.
```

## BASIC Keywords for Projects

### CREATE PROJECT

```basic
project_id = CREATE PROJECT "Marketing Campaign" WITH DESCRIPTION "Q3 launch"
```

### GET PROJECT

```basic
project = GET PROJECT project_id
TALK "Project: " + project.name
TALK "Members: " + LEN(project.members)
TALK "Open tasks: " + project.task_count
```

### LIST PROJECTS

```basic
' List user's projects
projects = LIST PROJECTS
FOR EACH p IN projects
    TALK p.name + " (" + p.role + ")"
NEXT p

' List projects with filter
active = LIST PROJECTS WHERE "status = 'active'"
```

### ADD USER TO PROJECT

```basic
ADD USER TO PROJECT project_id, user_id, "member"
ADD USER TO PROJECT project_id, email, "owner"
```

### REMOVE USER FROM PROJECT

```basic
REMOVE USER FROM PROJECT project_id, user_id
```

### SET PROJECT

Set the current conversation's project context:

```basic
SET PROJECT project_id
' Subsequent operations use this project context
CREATE TASK "Review designs"  ' Task created in the selected project
```

### DELETE PROJECT

```basic
DELETE PROJECT project_id
' Or via dynamic path
DELETE "/projects/" + project_id
```

## API Reference

### List Projects

```
GET /api/projects
```

Returns projects the authenticated user can access.

### Get Project

```
GET /api/projects/{id}
```

Returns project details including members and task summary.

### Create Project

```
POST /api/projects
{
  "name": "Project Name",
  "description": "Optional description",
  "visibility": "team",
  "members": [
    {"user_id": "...", "role": "owner"},
    {"user_id": "...", "role": "member"}
  ]
}
```

### Update Project

```
PUT /api/projects/{id}
{
  "name": "Updated Name",
  "description": "Updated description"
}
```

### Delete Project

```
DELETE /api/projects/{id}
```

### Project Members

```
GET /api/projects/{id}/members
POST /api/projects/{id}/members
DELETE /api/projects/{id}/members/{user_id}
```

### Project Tasks

```
GET /api/projects/{id}/tasks
POST /api/projects/{id}/tasks
```

### Project Conversations

```
GET /api/projects/{id}/conversations
```

## Database Schema

Projects are stored in the `projects` table:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Project identifier |
| `bot_id` | UUID | Owning bot |
| `name` | TEXT | Project name |
| `description` | TEXT | Optional description |
| `visibility` | TEXT | private, team, or public |
| `settings` | JSONB | Project configuration |
| `created_by` | UUID | Creator user ID |
| `created_at` | TIMESTAMP | Creation time |
| `updated_at` | TIMESTAMP | Last update |

Project membership in `project_members`:

| Column | Type | Description |
|--------|------|-------------|
| `project_id` | UUID | Project reference |
| `user_id` | UUID | User reference |
| `role` | TEXT | owner, member, or viewer |
| `joined_at` | TIMESTAMP | When user joined |

## Default Project

Every bot has a default project that cannot be deleted. Tasks created without specifying a project go here. Users can:

- Rename the default project
- Move tasks from default to specific projects
- Use the default for personal/unorganized work

```basic
' Get the default project
default = GET DEFAULT PROJECT
TALK "Default project: " + default.name
```

## Project Templates

Create projects from templates for common scenarios:

```basic
' Create from template
project_id = CREATE PROJECT FROM TEMPLATE "client-onboarding", "Acme Corp Onboarding"

' Available templates
templates = LIST PROJECT TEMPLATES
```

Built-in templates include:

- **Client Onboarding** - Tasks for new client setup
- **Product Launch** - Launch checklist and milestones
- **Sprint** - Two-week sprint with standard ceremonies
- **Content Calendar** - Monthly content planning

## Best Practices

**Keep projects focused.** A project should represent a distinct initiative with clear boundaries. If a project grows too large, consider splitting it.

**Assign clear ownership.** Every project needs at least one owner responsible for keeping it organized and moving forward.

**Use project context in chat.** When discussing project-specific topics, select the project first so the AI has full context.

**Archive completed projects.** Rather than deleting, archive finished projects to preserve history:

```basic
UPDATE PROJECT project_id SET status = "archived"
```

**Review project membership regularly.** Remove users who are no longer involved to keep conversations relevant.

## Integration with Tasks

Tasks belong to exactly one project. The task view shows the default project by default, with options to filter by project or view all tasks across projects.

```basic
' Create task in specific project
SET PROJECT project_id
CREATE TASK "Design review" DUE DATEADD(NOW(), 7, "day")

' Or specify project directly
CREATE TASK "Design review" IN PROJECT project_id
```

## See Also

- [Tasks API](../chapter-10-api/tasks-api.md) - Task management endpoints
- [Conversations API](../chapter-10-api/conversations-api.md) - Chat history
- [Groups API](../chapter-10-api/groups-api.md) - User group management
- [SET CONTEXT](../chapter-06-gbdialog/keyword-set-context.md) - AI context configuration