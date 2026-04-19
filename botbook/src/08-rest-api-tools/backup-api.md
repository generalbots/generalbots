# Backup API

The Backup API provides endpoints for creating, managing, and restoring backups of bot data and configurations.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/backup
```

## Authentication

Uses the standard botserver authentication mechanism with administrator-level permissions required.

## Endpoints

### Backup Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/backup/create` | Create a new backup |
| GET | `/api/v1/backup/list` | List all backups |
| GET | `/api/v1/backup/{backup_id}` | Get backup details |
| DELETE | `/api/v1/backup/{backup_id}` | Delete a backup |

### Restore Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/backup/restore/{backup_id}` | Restore from backup |
| GET | `/api/v1/backup/restore/{job_id}/status` | Check restore status |

### Scheduling

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/backup/schedule` | Create backup schedule |
| GET | `/api/v1/backup/schedule` | List backup schedules |
| PUT | `/api/v1/backup/schedule/{schedule_id}` | Update schedule |
| DELETE | `/api/v1/backup/schedule/{schedule_id}` | Delete schedule |

## Request Examples

### Create Backup

```bas
backup_options = NEW OBJECT
backup_options.type = "full"
backup_options.include_files = true
backup_options.include_database = true

result = POST "/api/v1/backup/create", backup_options
TALK "Backup created: " + result.backup_id
```

### List Backups

```bas
backups = GET "/api/v1/backup/list"
FOR EACH backup IN backups
    TALK backup.id + " - " + backup.created_at + " (" + backup.size + ")"
NEXT
```

### Restore from Backup

```bas
restore_options = NEW OBJECT
restore_options.target = "staging"

result = POST "/api/v1/backup/restore/backup-123", restore_options
TALK "Restore job started: " + result.job_id
```

### Schedule Automated Backup

```bas
schedule = NEW OBJECT
schedule.cron = "0 2 * * *"
schedule.type = "incremental"
schedule.retention_days = 30

POST "/api/v1/backup/schedule", schedule
TALK "Backup schedule created"
```

## Backup Types

| Type | Description |
|------|-------------|
| `full` | Complete backup of all data |
| `incremental` | Only changes since last backup |
| `differential` | Changes since last full backup |

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Backup created |
| 202 | Restore job accepted |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Backup not found |
| 409 | Restore already in progress |
| 500 | Internal Server Error |

## Required Permissions

| Operation | Required Role |
|-----------|---------------|
| Create backup | `admin` or `backup_operator` |
| List backups | `admin` or `backup_operator` |
| Restore backup | `admin` |
| Manage schedules | `admin` |
| Delete backup | `admin` |