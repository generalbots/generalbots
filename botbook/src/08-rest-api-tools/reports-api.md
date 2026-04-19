# Reports API

The Reports API provides endpoints for generating, managing, and exporting various types of reports from bot data and analytics.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/reports
```

## Authentication

Uses the standard botserver authentication mechanism with appropriate role-based permissions.

## Endpoints

### Report Generation

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/reports/generate` | Generate a new report |
| GET | `/api/v1/reports/{report_id}/status` | Get report generation status |
| GET | `/api/v1/reports/{report_id}` | Get report metadata |

### Report Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/reports` | List all reports |
| GET | `/api/v1/reports/{report_id}/download` | Download report file |
| DELETE | `/api/v1/reports/{report_id}` | Delete a report |

### Report Scheduling

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/reports/schedule` | Create scheduled report |
| GET | `/api/v1/reports/schedule` | List scheduled reports |
| PUT | `/api/v1/reports/schedule/{schedule_id}` | Update schedule |
| DELETE | `/api/v1/reports/schedule/{schedule_id}` | Delete schedule |

### Report Templates

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/reports/templates` | List available templates |
| POST | `/api/v1/reports/templates` | Create custom template |
| GET | `/api/v1/reports/templates/{template_id}` | Get template details |

## Report Types

| Type | Description |
|------|-------------|
| `usage` | Bot usage and activity metrics |
| `conversations` | Conversation analysis and statistics |
| `performance` | System performance metrics |
| `compliance` | Compliance and audit reports |
| `custom` | Custom report with user-defined metrics |

## Export Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| PDF | `.pdf` | Formatted document with charts |
| CSV | `.csv` | Raw data export |
| Excel | `.xlsx` | Spreadsheet with multiple sheets |
| JSON | `.json` | Machine-readable format |
| HTML | `.html` | Web-viewable format |

## Request Examples

### Generate Usage Report

```bas
report_config = NEW OBJECT
report_config.type = "usage"
report_config.start_date = "2025-01-01"
report_config.end_date = "2025-01-31"
report_config.format = "pdf"
report_config.include_charts = true

result = POST "/api/v1/reports/generate", report_config
TALK "Report ID: " + result.report_id
```

### Check Report Status

```bas
status = GET "/api/v1/reports/rpt-123/status"

IF status.state = "completed" THEN
    TALK "Report ready for download"
    url = GET "/api/v1/reports/rpt-123/download"
ELSE
    TALK "Report generation: " + status.progress + "%"
END IF
```

### Schedule Weekly Report

```bas
schedule = NEW OBJECT
schedule.type = "conversations"
schedule.format = "pdf"
schedule.cron = "0 8 * * 1"
schedule.recipients = ["manager@company.com"]
schedule.timezone = "America/New_York"

result = POST "/api/v1/reports/schedule", schedule
TALK "Scheduled report: " + result.schedule_id
```

### List Available Templates

```bas
templates = GET "/api/v1/reports/templates"
FOR EACH template IN templates
    TALK template.name + " - " + template.description
NEXT
```

### Generate Custom Report

```bas
report = NEW OBJECT
report.type = "custom"
report.title = "Monthly Executive Summary"
report.sections = NEW ARRAY
report.sections.ADD({"type": "usage_summary"})
report.sections.ADD({"type": "top_conversations"})
report.sections.ADD({"type": "satisfaction_scores"})
report.format = "pdf"

result = POST "/api/v1/reports/generate", report
```

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Report created |
| 202 | Report generation started |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Report not found |
| 500 | Internal Server Error |

## Query Parameters

### Filtering

| Parameter | Type | Description |
|-----------|------|-------------|
| `type` | String | Filter by report type |
| `status` | String | Filter by status (pending, completed, failed) |
| `created_after` | DateTime | Reports created after date |
| `created_before` | DateTime | Reports created before date |

### Pagination

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | Integer | Number of results (default: 20, max: 100) |
| `offset` | Integer | Pagination offset |
| `sort` | String | Sort field (created_at, name, type) |
| `order` | String | Sort order (asc, desc) |

## Required Permissions

| Operation | Required Role |
|-----------|---------------|
| Generate report | `report_creator` or `admin` |
| View reports | `report_viewer` or higher |
| Download reports | `report_viewer` or higher |
| Delete reports | `report_admin` or `admin` |
| Manage schedules | `report_admin` or `admin` |
| Manage templates | `report_admin` or `admin` |