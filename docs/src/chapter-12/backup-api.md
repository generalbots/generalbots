# Backup API

## Overview

The Backup API provides comprehensive backup and restore functionality for BotServer systems, including databases, file storage, configurations, and user data. It supports automated backups, manual backups, point-in-time recovery, and disaster recovery operations.

## Endpoints

### List Backups

Lists all available backups with filtering options.

**Endpoint**: `GET /api/backups/list`

**Authentication**: Required (Admin)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | No | Filter by backup type: `full`, `incremental`, `differential`, `transaction` |
| `start_date` | string | No | Filter backups after this date (ISO 8601) |
| `end_date` | string | No | Filter backups before this date (ISO 8601) |
| `status` | string | No | Filter by status: `completed`, `in_progress`, `failed` |
| `page` | integer | No | Page number for pagination (default: 1) |
| `per_page` | integer | No | Results per page (default: 20, max: 100) |

**Response**:
```json
{
  "success": true,
  "data": {
    "backups": [
      {
        "id": "backup_123456",
        "type": "full",
        "status": "completed",
        "size_bytes": 5368709120,
        "size_formatted": "5.0 GB",
        "created_at": "2024-01-15T02:00:00Z",
        "completed_at": "2024-01-15T02:45:32Z",
        "duration_seconds": 2732,
        "location": "s3://backups/2024-01-15/full-backup-123456.tar.gz.enc",
        "encrypted": true,
        "verified": true,
        "retention_until": "2024-04-15T02:00:00Z",
        "components": [
          "database",
          "files",
          "configurations",
          "user_data"
        ],
        "checksum": "sha256:a1b2c3d4e5f6...",
        "metadata": {
          "bot_count": 15,
          "user_count": 234,
          "file_count": 1523,
          "database_size": 2147483648
        }
      }
    ],
    "total": 156,
    "page": 1,
    "per_page": 20,
    "total_pages": 8
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/backups/list?type=full&status=completed" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Create Backup

Initiates a new backup operation.

**Endpoint**: `POST /api/backups/create`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "type": "full",
  "components": ["database", "files", "configurations", "user_data"],
  "description": "Weekly full backup",
  "retention_days": 90,
  "encrypt": true,
  "verify": true,
  "compress": true,
  "compression_level": 6,
  "location": "s3://backups/manual",
  "notification": {
    "email": ["admin@example.com"],
    "webhook": "https://monitoring.example.com/webhook"
  }
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | Yes | Backup type: `full`, `incremental`, `differential`, `transaction` |
| `components` | array | No | Components to backup (default: all) |
| `description` | string | No | Description of the backup |
| `retention_days` | integer | No | Days to retain backup (default: 90) |
| `encrypt` | boolean | No | Encrypt backup (default: true) |
| `verify` | boolean | No | Verify after creation (default: true) |
| `compress` | boolean | No | Compress backup (default: true) |
| `compression_level` | integer | No | Compression level 1-9 (default: 6) |
| `location` | string | No | Storage location (default: configured backup location) |
| `notification` | object | No | Notification settings |

**Response**:
```json
{
  "success": true,
  "data": {
    "backup_id": "backup_123456",
    "status": "in_progress",
    "started_at": "2024-01-15T14:30:00Z",
    "estimated_duration_seconds": 2700,
    "estimated_size_bytes": 5000000000,
    "progress_url": "/api/backups/status/backup_123456"
  },
  "message": "Backup initiated successfully"
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/api/backups/create" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "full",
    "description": "Pre-upgrade backup",
    "verify": true
  }'
```

---

### Get Backup Status

Retrieves the current status of a backup operation.

**Endpoint**: `GET /api/backups/status/{backup_id}`

**Authentication**: Required (Admin)

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `backup_id` | string | Yes | Backup identifier |

**Response**:
```json
{
  "success": true,
  "data": {
    "backup_id": "backup_123456",
    "status": "in_progress",
    "progress_percent": 65,
    "current_phase": "backing_up_files",
    "started_at": "2024-01-15T14:30:00Z",
    "elapsed_seconds": 1755,
    "estimated_remaining_seconds": 945,
    "bytes_processed": 3489660928,
    "total_bytes": 5368709120,
    "files_processed": 1234,
    "total_files": 1523,
    "current_operation": "Backing up: /data/uploads/bot-123/documents/large-file.pdf",
    "phases_completed": [
      "database_backup",
      "configuration_backup"
    ],
    "phases_remaining": [
      "file_backup",
      "verification"
    ]
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/backups/status/backup_123456" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Get Backup Details

Retrieves detailed information about a specific backup.

**Endpoint**: `GET /api/backups/{backup_id}`

**Authentication**: Required (Admin)

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `backup_id` | string | Yes | Backup identifier |

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "backup_123456",
    "type": "full",
    "status": "completed",
    "description": "Weekly automated backup",
    "size_bytes": 5368709120,
    "size_formatted": "5.0 GB",
    "created_at": "2024-01-15T02:00:00Z",
    "completed_at": "2024-01-15T02:45:32Z",
    "duration_seconds": 2732,
    "location": "s3://backups/2024-01-15/full-backup-123456.tar.gz.enc",
    "encrypted": true,
    "encryption_algorithm": "AES-256-GCM",
    "compressed": true,
    "compression_ratio": 3.2,
    "verified": true,
    "verification_date": "2024-01-15T02:50:00Z",
    "retention_until": "2024-04-15T02:00:00Z",
    "components": {
      "database": {
        "size_bytes": 2147483648,
        "tables_count": 45,
        "records_count": 1234567,
        "backup_method": "pg_dump"
      },
      "files": {
        "size_bytes": 3000000000,
        "files_count": 1523,
        "directories_count": 156
      },
      "configurations": {
        "size_bytes": 1048576,
        "files_count": 23
      },
      "user_data": {
        "size_bytes": 220160896,
        "users_count": 234
      }
    },
    "checksum": "sha256:a1b2c3d4e5f6...",
    "metadata": {
      "server_version": "1.2.3",
      "server_hostname": "botserver-prod-01",
      "backup_software_version": "2.1.0",
      "created_by": "system_scheduler",
      "tags": ["weekly", "production", "automated"]
    },
    "restore_count": 0,
    "last_restore_date": null
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/backups/backup_123456" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Restore from Backup

Initiates a restore operation from a backup.

**Endpoint**: `POST /api/backups/restore`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "backup_id": "backup_123456",
  "components": ["database", "files"],
  "target_environment": "staging",
  "restore_options": {
    "database": {
      "drop_existing": false,
      "point_in_time": "2024-01-15T14:30:00Z"
    },
    "files": {
      "overwrite_existing": false,
      "restore_path": "/restore/files"
    }
  },
  "dry_run": false,
  "notification": {
    "email": ["admin@example.com"]
  }
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `backup_id` | string | Yes | ID of backup to restore from |
| `components` | array | No | Components to restore (default: all) |
| `target_environment` | string | No | Target environment: `production`, `staging`, `development` |
| `restore_options` | object | No | Component-specific restore options |
| `dry_run` | boolean | No | Simulate restore without executing (default: false) |
| `notification` | object | No | Notification settings |

**Response**:
```json
{
  "success": true,
  "data": {
    "restore_id": "restore_789012",
    "backup_id": "backup_123456",
    "status": "in_progress",
    "started_at": "2024-01-15T15:00:00Z",
    "estimated_duration_seconds": 3600,
    "progress_url": "/api/backups/restore/status/restore_789012",
    "warnings": [
      "Database restore will require application downtime",
      "Files will be restored to alternate location to prevent overwrite"
    ]
  },
  "message": "Restore initiated successfully"
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/api/backups/restore" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "backup_id": "backup_123456",
    "components": ["database"],
    "target_environment": "staging"
  }'
```

---

### Get Restore Status

Retrieves the current status of a restore operation.

**Endpoint**: `GET /api/backups/restore/status/{restore_id}`

**Authentication**: Required (Admin)

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `restore_id` | string | Yes | Restore operation identifier |

**Response**:
```json
{
  "success": true,
  "data": {
    "restore_id": "restore_789012",
    "backup_id": "backup_123456",
    "status": "in_progress",
    "progress_percent": 42,
    "current_phase": "restoring_database",
    "started_at": "2024-01-15T15:00:00Z",
    "elapsed_seconds": 1512,
    "estimated_remaining_seconds": 2088,
    "bytes_restored": 2255651840,
    "total_bytes": 5368709120,
    "current_operation": "Restoring table: conversation_messages",
    "phases_completed": [
      "verification",
      "preparation"
    ],
    "phases_remaining": [
      "database_restore",
      "file_restore",
      "post_restore_validation"
    ],
    "warnings": [],
    "errors": []
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/backups/restore/status/restore_789012" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Verify Backup

Verifies the integrity of a backup without restoring it.

**Endpoint**: `POST /api/backups/verify/{backup_id}`

**Authentication**: Required (Admin)

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `backup_id` | string | Yes | Backup identifier |

**Request Body**:
```json
{
  "deep_verification": true,
  "test_restore": false,
  "components": ["database", "files"]
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "backup_id": "backup_123456",
    "verification_status": "passed",
    "verified_at": "2024-01-15T16:00:00Z",
    "verification_duration_seconds": 245,
    "checks_performed": {
      "checksum_validation": "passed",
      "file_integrity": "passed",
      "encryption_validation": "passed",
      "compression_validation": "passed",
      "metadata_validation": "passed",
      "component_completeness": "passed"
    },
    "components_verified": {
      "database": {
        "status": "passed",
        "size_verified": 2147483648,
        "checksum_match": true
      },
      "files": {
        "status": "passed",
        "files_verified": 1523,
        "missing_files": 0,
        "corrupted_files": 0
      }
    },
    "issues": [],
    "recommendations": [
      "Backup is healthy and can be used for restore operations",
      "Next verification scheduled for 2024-02-15"
    ]
  },
  "message": "Backup verification completed successfully"
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/api/backups/verify/backup_123456" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"deep_verification": true}'
```

---

### Delete Backup

Deletes a backup (respects retention policies).

**Endpoint**: `DELETE /api/backups/{backup_id}`

**Authentication**: Required (Admin)

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `backup_id` | string | Yes | Backup identifier |

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `force` | boolean | No | Force deletion even if retention policy not met (default: false) |
| `reason` | string | No | Reason for deletion (required if force=true) |

**Response**:
```json
{
  "success": true,
  "data": {
    "backup_id": "backup_123456",
    "deleted_at": "2024-01-15T16:30:00Z",
    "space_freed_bytes": 5368709120,
    "space_freed_formatted": "5.0 GB"
  },
  "message": "Backup deleted successfully"
}
```

**Example**:
```bash
curl -X DELETE "http://localhost:3000/api/backups/backup_123456" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Configure Backup Schedule

Configures automated backup schedules.

**Endpoint**: `POST /api/backups/schedule/configure`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "schedules": [
    {
      "name": "daily_incremental",
      "type": "incremental",
      "enabled": true,
      "cron": "0 2 * * *",
      "components": ["database", "files"],
      "retention_days": 30,
      "notification": {
        "on_success": false,
        "on_failure": true,
        "email": ["admin@example.com"]
      }
    },
    {
      "name": "weekly_full",
      "type": "full",
      "enabled": true,
      "cron": "0 2 * * 0",
      "components": ["database", "files", "configurations", "user_data"],
      "retention_days": 90,
      "notification": {
        "on_success": true,
        "on_failure": true,
        "email": ["admin@example.com", "backup-team@example.com"]
      }
    }
  ]
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "schedules_configured": 2,
    "next_backup": {
      "schedule_name": "daily_incremental",
      "scheduled_time": "2024-01-16T02:00:00Z"
    }
  },
  "message": "Backup schedules configured successfully"
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/api/backups/schedule/configure" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d @schedule-config.json
```

---

### Get Backup Schedule

Retrieves current backup schedule configuration.

**Endpoint**: `GET /api/backups/schedule`

**Authentication**: Required (Admin)

**Response**:
```json
{
  "success": true,
  "data": {
    "schedules": [
      {
        "id": "schedule_001",
        "name": "daily_incremental",
        "type": "incremental",
        "enabled": true,
        "cron": "0 2 * * *",
        "next_run": "2024-01-16T02:00:00Z",
        "last_run": "2024-01-15T02:00:00Z",
        "last_status": "completed",
        "total_runs": 365,
        "successful_runs": 363,
        "failed_runs": 2
      }
    ]
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/backups/schedule" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Get Backup Statistics

Retrieves backup system statistics and metrics.

**Endpoint**: `GET /api/backups/statistics`

**Authentication**: Required (Admin)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | No | Time period: `day`, `week`, `month`, `year` (default: month) |

**Response**:
```json
{
  "success": true,
  "data": {
    "period": "month",
    "start_date": "2024-01-01T00:00:00Z",
    "end_date": "2024-01-31T23:59:59Z",
    "summary": {
      "total_backups": 31,
      "successful_backups": 30,
      "failed_backups": 1,
      "success_rate": 96.77,
      "total_size_bytes": 166440345600,
      "total_size_formatted": "155 GB",
      "average_backup_size_bytes": 5368709120,
      "average_duration_seconds": 2650,
      "total_storage_used_bytes": 523986010112,
      "total_storage_used_formatted": "488 GB"
    },
    "by_type": {
      "full": {
        "count": 4,
        "success_rate": 100,
        "average_size_bytes": 5368709120,
        "average_duration_seconds": 2732
      },
      "incremental": {
        "count": 27,
        "success_rate": 96.3,
        "average_size_bytes": 536870912,
        "average_duration_seconds": 320
      }
    },
    "retention_compliance": {
      "backups_expired": 8,
      "backups_deleted": 8,
      "compliance_rate": 100
    },
    "storage_locations": {
      "s3_primary": {
        "backups_count": 156,
        "size_bytes": 314572800000,
        "utilization_percent": 62.4
      },
      "s3_archive": {
        "backups_count": 48,
        "size_bytes": 209413210112,
        "utilization_percent": 41.5
      }
    }
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/backups/statistics?period=month" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

## Backup Types

### Full Backup
- Complete backup of all data
- Baseline for incremental/differential backups
- Longest duration, largest size
- Fastest restore time

### Incremental Backup
- Only backs up data changed since last backup (any type)
- Smallest size, fastest backup
- Requires all incremental backups for restore

### Differential Backup
- Backs up data changed since last full backup
- Medium size, medium backup time
- Only requires last full backup for restore

### Transaction Log Backup
- Continuous backup of database transactions
- Enables point-in-time recovery
- Very frequent (every 15 minutes)
- Small size per backup

## Best Practices

### Backup Strategy
1. **3-2-1 Rule**: 3 copies of data, 2 different media types, 1 off-site
2. **Regular Testing**: Test restores monthly
3. **Encryption**: Always encrypt backups containing sensitive data
4. **Verification**: Verify backup integrity after creation
5. **Monitoring**: Set up alerts for backup failures

### Retention Policy
- **Daily Incremental**: 30 days
- **Weekly Full**: 90 days (12 weeks)
- **Monthly Full**: 1 year
- **Yearly Full**: 7 years (compliance requirement)

### Performance Optimization
- Schedule backups during low-usage periods
- Use incremental backups for daily operations
- Compress backups to save storage space
- Use parallel backup processes for large datasets

### Security
- Encrypt all backups with AES-256-GCM
- Store encryption keys in secure key management system
- Implement access controls on backup storage
- Audit backup access regularly
- Test disaster recovery procedures

## Error Codes

| Code | Description |
|------|-------------|
| `BACKUP_NOT_FOUND` | Specified backup does not exist |
| `BACKUP_IN_PROGRESS` | Backup operation already in progress |
| `INSUFFICIENT_STORAGE` | Not enough storage space for backup |
| `BACKUP_VERIFICATION_FAILED` | Backup integrity check failed |
| `RESTORE_IN_PROGRESS` | Restore operation already in progress |
| `INVALID_BACKUP_TYPE` | Invalid backup type specified |
| `RETENTION_POLICY_VIOLATION` | Cannot delete backup due to retention policy |
| `ENCRYPTION_KEY_NOT_FOUND` | Encryption key not available |
| `BACKUP_CORRUPTED` | Backup file is corrupted |
| `COMPONENT_NOT_FOUND` | Specified backup component does not exist |

## Webhooks

Subscribe to backup events via webhooks:

### Webhook Events
- `backup.started` - Backup operation started
- `backup.completed` - Backup completed successfully
- `backup.failed` - Backup operation failed
- `backup.verified` - Backup verification completed
- `restore.started` - Restore operation started
- `restore.completed` - Restore completed successfully
- `restore.failed` - Restore operation failed

### Webhook Payload Example
```json
{
  "event": "backup.completed",
  "timestamp": "2024-01-15T02:45:32Z",
  "data": {
    "backup_id": "backup_123456",
    "type": "full",
    "size_bytes": 5368709120,
    "duration_seconds": 2732,
    "status": "completed"
  }
}
```

## See Also

- [Storage API](./storage-api.md) - Storage management
- [Admin API](./admin-api.md) - System administration
- [Monitoring API](./monitoring-api.md) - System monitoring
- [Chapter 11: Security](../chapter-11/README.md) - Security policies