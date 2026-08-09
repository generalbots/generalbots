# Sheets API 🟡 BETA

> **Spreadsheet creation, editing, formatting, formulas, charts, collaboration, and real-time synchronization**

---

## Base URL

```
/api/sheet
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

> **⚠️ Not enforced yet.** The handlers currently resolve every request to a single hardcoded user identity, so any client can read or overwrite any document by id, and no per-document permission is checked. Real identity, tenancy scoping and a four-role ACL (owner / editor / commenter / viewer) are tracked in [generalbots#789](https://github.com/generalbots/generalbots/issues/789). Do not expose these endpoints to untrusted clients until it lands. See [Excel Parity Plan](../07-user-interface/apps/sheet-excel-parity.md).

> **⚠️ Concurrency.** Every mutating endpoint currently loads the whole document, mutates it, and writes it back with no version check. Two clients editing different cells will lose one edit silently. Optimistic concurrency with a `version` field and `409 Conflict` responses is part of the same issue.

---

## Spreadsheet Operations

### List Spreadsheets

**`GET /api/sheet/list`**

Returns all spreadsheets accessible to the authenticated user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `folder` | string | No | Filter by folder path |
| `search` | string | No | Search by title |
| `sort` | string | No | Sort: `name`, `created_at`, `updated_at` (default: `updated_at`) |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "spreadsheets": [
    {
      "id": "sheet_001",
      "title": "Relatório de Vendas Q2",
      "folder": "/reports",
      "created_by": "user_001",
      "created_at": "2026-06-01T10:00:00Z",
      "updated_at": "2026-06-04T10:00:00Z",
      "sheet_count": 3,
      "collaborators": 5,
      "size_bytes": 245760
    }
  ],
  "total": 28
}
```

---

### Search Spreadsheets

**`GET /api/sheet/search`**

Full-text search across spreadsheet content and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query |
| `scope` | string | No | `title`, `content`, `all` (default: `all`) |
| `limit` | integer | No | Max results (default: 10) |

**Response:**
```json
{
  "results": [
    {
      "id": "sheet_001",
      "title": "Relatório de Vendas Q2",
      "match_type": "title",
      "match_snippet": "...Relatório de **Vendas** Q2...",
      "updated_at": "2026-06-04T10:00:00Z"
    },
    {
      "id": "sheet_005",
      "title": "Forecast 2026",
      "match_type": "content",
      "match_snippet": "...receita de **vendas** no Q2 foi de R$ 2.5M...",
      "updated_at": "2026-06-03T15:00:00Z"
    }
  ],
  "total": 2
}
```

---

### Load Spreadsheet

**`GET /api/sheet/load`**

Loads a spreadsheet with all sheet data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Spreadsheet identifier |
| `sheet` | string | No | Specific sheet name (default: first sheet) |
| `range` | string | No | Cell range (e.g., `A1:Z100`) |
| `format` | string | No | `json`, `csv` (default: `json`) |

**Response:**
```json
{
  "id": "sheet_001",
  "title": "Relatório de Vendas Q2",
  "sheets": [
    {
      "name": "Resumo",
      "id": "sheet_tab_001",
      "rows": 50,
      "columns": 15,
      "data": [
        ["Produto", "Qtd Vendida", "Receita", "Margem"],
        ["Bot Enterprise", "45", "R$ 135.000", "68%"],
        ["Bot Starter", "120", "R$ 36.000", "72%"],
        ["Bot Pro", "78", "R$ 156.000", "65%"]
      ],
      "freeze_position": "A2"
    }
  ],
  "active_sheet": "Resumo",
  "last_editor": "Maria Santos",
  "updated_at": "2026-06-04T10:00:00Z"
}
```

---

### Load from Drive

**`POST /api/sheet/load-from-drive`**

Loads a spreadsheet directly from a Drive (MinIO) bucket.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot_name` | string | Yes | Bot name containing the sheet |
| `file_path` | string | Yes | Path within .gbdrive |
| `format` | string | No | Expected format: `xlsx`, `csv`, `json` |

**Response:**
```json
{
  "id": "sheet_drive_001",
  "title": "dados_vendas.xlsx",
  "source": "drive",
  "bot_name": "salesbot",
  "sheets": [
    {
      "name": "Vendas",
      "rows": 500,
      "columns": 12
    }
  ],
  "loaded_at": "2026-06-04T10:00:00Z"
}
```

---

### Save Spreadsheet

**`POST /api/sheet/save`**

Saves spreadsheet changes back to Drive.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | No | Spreadsheet ID (omit for new) |
| `title` | string | No | Updated title |
| `sheets` | object[] | No | Updated sheet data |
| `folder` | string | No | Target folder in .gbdrive |
| `format` | string | No | Save format: `xlsx`, `csv`, `json` |

**Response:**
```json
{
  "id": "sheet_001",
  "saved": true,
  "path": "reports/vendas_q2.xlsx",
  "size_bytes": 250880,
  "saved_at": "2026-06-04T10:15:00Z"
}
```

---

### Delete Spreadsheet

**`POST /api/sheet/delete`**

Deletes a spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Spreadsheet identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "sheet_001",
  "title": "Relatório de Vendas Q2"
}
```

---

### New Spreadsheet

**`GET /api/sheet/new`**

Creates a new empty spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | No | Spreadsheet title (default: "Untitled") |
| `template` | string | No | Template ID to clone |
| `sheets` | string[] | No | Initial sheet names |

**Response:**
```json
{
  "id": "sheet_002",
  "title": "Nova Planilha",
  "sheets": [
    { "name": "Sheet1", "id": "sheet_tab_002" }
  ],
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

## Cell Operations

### Update Cell

**`POST /api/sheet/cell`**

Updates a single cell value.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name (default: first sheet) |
| `cell` | string | Yes | Cell reference (e.g., `A1`) |
| `value` | any | Yes | Cell value |
| `formula` | string | No | Cell formula |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "cell": "C2",
  "value": "=B2*D2",
  "computed_value": 91800,
  "updated_at": "2026-06-04T10:00:00Z"
}
```

---

### Format Cells

**`POST /api/sheet/format`**

Applies formatting to a range of cells.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Cell range (e.g., `A1:D10`) |
| `bold` | boolean | No | Bold text |
| `italic` | boolean | No | Italic text |
| `font_size` | integer | No | Font size in px |
| `font_color` | string | No | Font color (hex) |
| `bg_color` | string | No | Background color (hex) |
| `align` | string | No | `left`, `center`, `right` |
| `border` | string | No | Border style |
| `number_format` | string | No | Number format: `currency`, `percent`, `date`, `number` |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "A1:D1",
  "format_applied": {
    "bold": true,
    "bg_color": "#1E40AF",
    "font_color": "#FFFFFF",
    "align": "center"
  },
  "cells_formatted": 4
}
```

---

### Set Formula

**`POST /api/sheet/formula`**

Sets a formula in a cell.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `cell` | string | Yes | Cell reference |
| `formula` | string | Yes | Excel/Google Sheets formula |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "cell": "E10",
  "formula": "=SUM(E2:E9)",
  "computed_value": 450000,
  "updated_at": "2026-06-04T10:00:00Z"
}
```

---

### Validate Cell

**`POST /api/sheet/validate-cell`**

Validates a cell value against data validation rules.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `cell` | string | Yes | Cell reference |
| `value` | any | Yes | Value to validate |

**Response:**
```json
{
  "valid": true,
  "cell": "B5",
  "value": "ativo",
  "rule": {
    "type": "list",
    "options": ["ativo", "inativo", "pendente"]
  }
}
```

---

## Merge & Freeze

### Merge Cells

**`POST /api/sheet/merge`**

Merges a range of cells.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Cell range to merge (e.g., `A1:C1`) |
| `type` | string | No | Merge type: `all`, `rows`, `columns` (default: `all`) |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "A1:C1",
  "merged": true,
  "merge_type": "all"
}
```

---

### Unmerge Cells

**`POST /api/sheet/unmerge`**

Unmerges a previously merged cell range.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Range to unmerge |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "A1:C1",
  "unmerged": true
}
```

---

### Freeze Panes

**`POST /api/sheet/freeze`**

Freezes rows and/or columns.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `rows` | integer | No | Number of rows to freeze |
| `columns` | integer | No | Number of columns to freeze |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "frozen_rows": 1,
  "frozen_columns": 0
}
```

---

## Sort & Filter

### Sort Data

**`POST /api/sheet/sort`**

Sorts a range by one or more columns.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Range to sort |
| `sort_by` | object[] | Yes | Sort columns with order |
| `sort_by[].column` | string | Yes | Column letter (e.g., `B`) |
| `sort_by[].order` | string | No | `asc` or `desc` (default: `asc`) |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "A2:D100",
  "sorted": true,
  "sort_by": [
    { "column": "B", "order": "desc" },
    { "column": "A", "order": "asc" }
  ],
  "rows_sorted": 99
}
```

---

### Apply Filter

**`POST /api/sheet/filter`**

Applies a filter to a data range.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Header range (e.g., `A1:D1`) |
| `column` | string | Yes | Column to filter |
| `operator` | string | Yes | `equals`, `contains`, `gt`, `lt`, `between`, `in` |
| `value` | any | Yes | Filter value |
| `value2` | any | No | Second value for `between` |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "column": "C",
  "operator": "gt",
  "value": 50000,
  "visible_rows": 12,
  "hidden_rows": 87,
  "filtered_at": "2026-06-04T10:00:00Z"
}
```

---

### Clear Filter

**`POST /api/sheet/filter/clear`**

Removes all active filters from a sheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "filters_cleared": 3,
  "all_rows_visible": true
}
```

---

## Charts & Visualization

### Create Chart

**`POST /api/sheet/chart`**

Creates a chart from spreadsheet data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `data_range` | string | Yes | Data range for chart |
| `type` | string | Yes | `bar`, `line`, `pie`, `scatter`, `area`, `column` |
| `title` | string | No | Chart title |
| `x_axis` | string | No | X-axis label |
| `y_axis` | string | No | Y-axis label |
| `position` | object | No | Position `{ row, column }` |

**Response:**
```json
{
  "chart_id": "chart_001",
  "type": "bar",
  "title": "Vendas por Produto",
  "data_range": "A1:B5",
  "position": { "row": 7, "column": 1 },
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Delete Chart

**`POST /api/sheet/chart/delete`**

Removes a chart from the spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `chart_id` | string | Yes | Chart identifier |

**Response:**
```json
{
  "chart_id": "chart_001",
  "deleted": true
}
```

---

## Data Validation

### Conditional Formatting

**`POST /api/sheet/conditional-format`**

Applies conditional formatting rules to a range.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Cell range |
| `rules` | object[] | Yes | Formatting rules |
| `rules[].condition` | string | Yes | `greater_than`, `less_than`, `equals`, `contains`, `between` |
| `rules[].value` | any | Yes | Condition value |
| `rules[].format` | object | Yes | Format to apply: `{ bg_color, font_color, bold }` |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "C2:C100",
  "rules_applied": 2,
  "cells_formatted": 45
}
```

---

### Data Validation Rule

**`POST /api/sheet/data-validation`**

Sets a data validation rule on a cell range.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | Yes | Cell range |
| `type` | string | Yes | `list`, `number`, `date`, `text`, `checkbox` |
| `options` | any | Validation options (depends on type) |
| `options.values` | string[] | For `list`: allowed values |
| `options.min` | number | For `number`: minimum |
| `options.max` | number | For `number`: maximum |
| `options.allow_blank` | boolean | Allow empty cells |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "D2:D50",
  "rule_type": "list",
  "rule": {
    "values": ["Ativo", "Inativo", "Pendente"],
    "allow_blank": false
  },
  "applied_to": 49
}
```

---

## Notes & Comments

### Add Note

**`POST /api/sheet/note`**

Adds a note/comment to a cell.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `cell` | string | Yes | Cell reference |
| `content` | string | Yes | Note content |

**Response:**
```json
{
  "note_id": "note_001",
  "cell": "C5",
  "content": "Receita do mês anterior precisa ser verificada",
  "author": "Maria Santos",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Add Comment

**`POST /api/sheet/comment`**

Adds a threaded comment to a cell.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `cell` | string | Yes | Cell reference |
| `content` | string | Yes | Comment content |
| `mentions` | string[] | No | User IDs to mention |

**Response:**
```json
{
  "comment_id": "comment_001",
  "cell": "E10",
  "content": "@Maria Santos este total está correto?",
  "author": "João Silva",
  "mentions": ["user_001"],
  "created_at": "2026-06-04T10:00:00Z",
  "resolved": false
}
```

---

### Reply to Comment

**`POST /api/sheet/comment/reply`**

Replies to an existing comment thread.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `comment_id` | string | Yes | Parent comment identifier |
| `content` | string | Yes | Reply content |

**Response:**
```json
{
  "reply_id": "reply_001",
  "comment_id": "comment_001",
  "content": "Sim, confirmei com o financeiro",
  "author": "Maria Santos",
  "created_at": "2026-06-04T10:05:00Z"
}
```

---

### Resolve Comment

**`POST /api/sheet/comment/resolve`**

Marks a comment thread as resolved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `comment_id` | string | Yes | Comment identifier |

**Response:**
```json
{
  "comment_id": "comment_001",
  "resolved": true,
  "resolved_by": "João Silva",
  "resolved_at": "2026-06-04T10:10:00Z"
}
```

---

### Delete Comment

**`POST /api/sheet/comment/delete`**

Deletes a comment and all its replies.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `comment_id` | string | Yes | Comment identifier |

**Response:**
```json
{
  "comment_id": "comment_001",
  "deleted": true,
  "replies_deleted": 3
}
```

---

### List Comments

**`POST /api/sheet/comments`**

Returns all comments in a spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `resolved` | boolean | No | Filter by resolved status |

**Response:**
```json
{
  "comments": [
    {
      "comment_id": "comment_001",
      "cell": "E10",
      "content": "@Maria Santos este total está correto?",
      "author": { "id": "user_002", "name": "João Silva" },
      "created_at": "2026-06-04T10:00:00Z",
      "resolved": false,
      "replies": [
        {
          "reply_id": "reply_001",
          "content": "Sim, confirmei com o financeiro",
          "author": { "id": "user_001", "name": "Maria Santos" },
          "created_at": "2026-06-04T10:05:00Z"
        }
      ]
    }
  ],
  "total": 5,
  "unresolved": 2
}
```

---

## Cell Protection

### Protect Sheet

**`POST /api/sheet/protect`**

Protects a sheet or range from editing.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | No | Specific range to protect (omit for full sheet) |
| `description` | string | No | Protection reason |
| `allowed_users` | string[] | No | User IDs allowed to edit |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "range": "A1:Z1000",
  "protected": true,
  "description": "Cabeçalho não pode ser alterado",
  "allowed_users": ["user_001"]
}
```

---

### Unprotect Sheet

**`POST /api/sheet/unprotect`**

Removes protection from a sheet or range.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "unprotected": true
}
```

---

### Lock Cells

**`POST /api/sheet/lock-cells`**

Locks specific cells while keeping the rest editable.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `ranges` | string[] | Yes | Cell ranges to lock |
| `allowed_users` | string[] | No | Users allowed to edit locked cells |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "locked_ranges": ["A1:D1", "A10:D10"],
  "locked_cells": 8,
  "allowed_users": ["user_001", "user_002"]
}
```

---

## Import & Export

### Import Data

**`POST /api/sheet/import`**

Imports data from various formats into a spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | No | Target spreadsheet (omit to create new) |
| `source` | string | Yes | `csv`, `json`, `xlsx`, `drive` |
| `data` | any | Yes | Raw data or file content |
| `file_path` | string | For `drive`: path in .gbdrive |
| `sheet_tab` | string | No | Target sheet tab name |
| `start_cell` | string | No | Starting cell (default: `A1`) |
| `has_header` | boolean | Yes | First row is header |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "imported": true,
  "source": "csv",
  "rows_imported": 500,
  "columns_imported": 12,
  "start_cell": "A1",
  "duration_ms": 2340
}
```

---

### Export Spreadsheet

**`POST /api/sheet/export`**

Exports a spreadsheet to various formats.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `format` | string | Yes | `xlsx`, `csv`, `json`, `pdf` |
| `sheet_tab` | string | No | Specific sheet (default: all) |
| `range` | string | No | Specific range |

**Response:**
```json
{
  "export_id": "export_001",
  "format": "xlsx",
  "download_url": "https://storage.example.com/exports/relatorio_vendas.xlsx",
  "size_bytes": 256000,
  "expires_at": "2026-06-05T10:00:00Z"
}
```

---

## Collaboration

### Share Spreadsheet

**`POST /api/sheet/share`**

Shares a spreadsheet with other users.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `user_id` | string | Yes | User to share with |
| `permission` | string | Yes | `view`, `comment`, `edit` |
| `notify` | boolean | No | Send notification (default: true) |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "shared_with": "user_003",
  "permission": "edit",
  "shared_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Collaborators

**`GET /api/sheet/:id/collaborators`**

Returns all collaborators on a spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Spreadsheet identifier |

**Response:**
```json
{
  "collaborators": [
    {
      "user_id": "user_001",
      "name": "Maria Santos",
      "email": "maria@example.com",
      "permission": "edit",
      "last_active_at": "2026-06-04T10:00:00Z",
      "is_online": true
    },
    {
      "user_id": "user_002",
      "name": "João Silva",
      "email": "joao@example.com",
      "permission": "view",
      "last_active_at": "2026-06-03T15:00:00Z",
      "is_online": false
    }
  ],
  "total": 5
}
```

---

## AI Integration

### AI Assist

**`POST /api/sheet/ai`**

Uses AI to analyze or transform spreadsheet data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `sheet_tab` | string | No | Sheet tab name |
| `range` | string | No | Data range to analyze |
| `prompt` | string | Yes | Natural language instruction |
| `operation` | string | No | `analyze`, `transform`, `generate_formula`, `suggest` |

**Response:**
```json
{
  "sheet_id": "sheet_001",
  "prompt": "Analise a tendência de vendas por produto",
  "operation": "analyze",
  "result": {
    "analysis": "Bot Enterprise apresenta crescimento de 15% QoQ, enquanto Bot Starter manteve estável. Bot Pro cresceu 22%, indicando oportunidade de upsell.",
    "suggestions": [
      "Focar esforços de venda em Bot Pro (maior crescimento)",
      "Revisar pricing de Bot Starter para estimular crescimento",
      "Criar campanha de upsell para clientes Bot Starter → Bot Pro"
    ],
    "formulas_generated": [
      {
        "cell": "E2",
        "formula": "=IF(D2>100000, \"Alto\", IF(D2>50000, \"Médio\", \"Baixo\"))",
        "description": "Classificação de performance"
      }
    ]
  },
  "generated_at": "2026-06-04T10:00:00Z"
}
```

---

## WebSocket Real-Time

### Collaborative Editing

**`GET /ws/sheet/:sheet_id`**

Establishes a WebSocket connection for real-time collaborative editing.

> **⚠️ The current wire protocol addresses a cell as a single integer, `row * 26 + col`, with 26 hardcoded on both ends.** Any grid wider than 26 columns therefore delivers every remote edit to the wrong cell, and two clients configured differently corrupt each other's data. Conflict handling is last-write-wins with no sequence numbers, no reconnect recovery and no rendered presence. The replacement protocol — A1 addressing, server-authoritative sequencing with operation transformation, and oplog-based recovery — is tracked in [generalbots#791](https://github.com/generalbots/generalbots/issues/791). The message shapes below are the target protocol, not the current one.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | path | Yes | Spreadsheet identifier |
| `token` | query | Yes | Authentication token |

**WebSocket Messages (Server → Client):**
```json
{
  "event": "cell_updated",
  "data": {
    "user": { "id": "user_001", "name": "Maria Santos" },
    "cell": "A1",
    "value": "Produto",
    "color": "#3B82F6"
  }
}
```

```json
{
  "event": "cursor_moved",
  "data": {
    "user_id": "user_001",
    "cell": "C5",
    "color": "#3B82F6"
  }
}
```

```json
{
  "event": "selection_changed",
  "data": {
    "user_id": "user_002",
    "range": "A1:D10",
    "color": "#10B981"
  }
}
```

```json
{
  "event": "user_joined",
  "data": {
    "user_id": "user_003",
    "name": "Carlos Lima",
    "color": "#F59E0B"
  }
}
```

**WebSocket Messages (Client → Server):**
```json
{
  "event": "edit",
  "data": {
    "cell": "B2",
    "value": "Nova receita",
    "sheet_tab": "Resumo"
  }
}
```

```json
{
  "event": "cursor",
  "data": {
    "cell": "C5"
  }
}
```

---

## External Links

### Create External Link

**`POST /api/sheet/external-link`**

Creates a shareable external link for spreadsheet access.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | string | Yes | Spreadsheet identifier |
| `permission` | string | Yes | `view`, `comment` |
| `expires_in_days` | integer | No | Link expiration (default: 30) |
| `password` | string | No | Password protection |

**Response:**
```json
{
  "link_id": "link_001",
  "url": "https://sheets.example.com/shared/abc123def456",
  "permission": "view",
  "expires_at": "2026-07-04T10:00:00Z",
  "has_password": true,
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get External Links

**`GET /api/sheet/external-link`**

Returns all external links for a spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet_id` | query | Yes | Spreadsheet identifier |

**Response:**
```json
{
  "links": [
    {
      "link_id": "link_001",
      "permission": "view",
      "access_count": 45,
      "last_accessed_at": "2026-06-04T09:30:00Z",
      "expires_at": "2026-07-04T10:00:00Z",
      "active": true
    }
  ]
}
```

---

### Delete External Link

**`DELETE /api/sheet/external-link/:id`**

Revokes an external link.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Link identifier |

**Response:**
```json
{
  "link_id": "link_001",
  "deleted": true,
  "revoked_access_count": 45
}
```

---

## Get Spreadsheet by ID

**`GET /api/sheet/:id`**

Returns metadata for a specific spreadsheet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Spreadsheet identifier |

**Response:**
```json
{
  "id": "sheet_001",
  "title": "Relatório de Vendas Q2",
  "folder": "/reports",
  "created_by": { "id": "user_001", "name": "Maria Santos" },
  "sheets": [
    { "name": "Resumo", "rows": 50, "columns": 15 },
    { "name": "Detalhes", "rows": 500, "columns": 12 },
    { "name": "Gráficos", "rows": 30, "columns": 10 }
  ],
  "permissions": {
    "owner": "user_001",
    "shared_with": 5,
    "external_links": 2
  },
  "created_at": "2026-06-01T10:00:00Z",
  "updated_at": "2026-06-04T10:00:00Z"
}
```

---

## Endpoint status

Endpoints that accept and persist a request but whose effect is not yet visible or enforced. Each is reachable and returns success; the gap is downstream.

| Endpoint | Status | Note |
|----------|--------|------|
| `POST /api/sheet/format` | Partial | Style is stored and applied to the active cell only; number format codes are stored but not rendered |
| `POST /api/sheet/formula` | Partial | A formula must be a single function call; `=SUM(A1:A3)+1`, `=A1&B1`, `=A1^2` and nested calls return `#ERROR!`. String literals are upper-cased |
| `POST /api/sheet/cell` | Partial | Recalculation of dependents silently stops after 1000 cells |
| `POST /api/sheet/merge` | Stored, not rendered | The grid does not draw merges |
| `POST /api/sheet/freeze` | Stored, not rendered | — |
| `POST /api/sheet/filter` | Stored, not applied | No rows are hidden |
| `POST /api/sheet/sort` | Partial | Values move; formulas referencing them are not rewritten |
| `POST /api/sheet/chart` | Stored, not rendered | Series data is snapshotted rather than bound to a range |
| `POST /api/sheet/conditional-format` | Stored, not applied | — |
| `POST /api/sheet/data-validation` | Stored, not enforced | `POST /api/sheet/validate-cell` works but the editor never calls it |
| `POST /api/sheet/comment*` | Stored, not rendered | Threads are complete server-side; the grid draws no marker |
| `POST /api/sheet/protect` | Stored, not enforced | — |
| `POST /api/sheet/export` | Partial | CSV/TSV/JSON/HTML/Markdown work but emit unformatted values; ODS emits content XML rather than a complete package; PDF returns HTML bytes |
| `POST /api/sheet/import` | Partial | `.xlsx` import drops defined names, validation, conditional formatting, charts, images, pivot tables, tables, autofilter, hidden rows, hyperlinks, rich text runs, protection and print setup |
| `POST /api/sheet/share` | Partial | No real ACL is written; see [#789](https://github.com/generalbots/generalbots/issues/789) |
| `GET /api/sheet/list` | Partial | Fully deserialises every document to build the list |

Full detail and the plan to close each gap: [Excel Parity Plan](../07-user-interface/apps/sheet-excel-parity.md).

---

## See Also

- [Sheet - Spreadsheets](../07-user-interface/apps/sheet.md) — user-facing feature reference
- [Excel Parity Plan](../07-user-interface/apps/sheet-excel-parity.md) — current gaps and roadmap
- [Files API](files-api.md) — file operations and Drive management
- [CRM API](crm-api.md) — contact and account data integration
- [Tasks API](tasks-api.md) — task tracking from spreadsheet data
- [Storage API](storage-api.md) — object storage operations
