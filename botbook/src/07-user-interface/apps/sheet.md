# Sheet 🟡 BETA - Spreadsheets

> **Excel-like spreadsheet with AI**

<img src="../../assets/suite/sheet-screen.svg" alt="Sheet Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Sheet is the spreadsheet application in General Bots Suite. Create, analyze, and visualize data with Excel-like functionality enhanced by AI. Build formulas, generate charts, import and export CSV files, and collaborate on datasets with powerful filtering and sorting.

---

## Features

### Cells

| Action | Description |
|--------|-------------|
| Edit | Double-click or type to edit cell |
| Format | Numbers, currency, dates, percentages |
| Merge | Merge cells for headers |
| Resize | Drag column and row borders |
| Reference | Use A1-style cell references |
| Multi-Select | Ctrl+Click for non-contiguous ranges |

### Formulas

| Function | Syntax | Description |
|----------|--------|-------------|
| SUM | `=SUM(A1:A10)` | Sum of range |
| AVERAGE | `=AVERAGE(B1:B5)` | Average of range |
| COUNT | `=COUNT(C1:C20)` | Count non-empty cells |
| IF | `=IF(A1>10,"Yes","No")` | Conditional logic |
| VLOOKUP | `=VLOOKUP(A1,D:E,2,0)` | Vertical lookup |
| CONCAT | `=CONCAT(A1," ",B1)` | Join text |
| MAX | `=MAX(A1:A10)` | Maximum value |
| MIN | `=MIN(A1:A10)` | Minimum value |
| DATE | `=DATE(2025,5,15)` | Create date |
| BOT_AI_PROMPT | `=BOT_AI_PROMPT("Analyze ", A1)` | AI-powered cell evaluation with intelligent caching — see [AI Sheet Cache](../../03-knowledge-ai/ai-sheet-cache.md) |

### Charts

| Type | Best For |
|------|----------|
| Bar | Comparing categories |
| Line | Trends over time |
| Pie | Proportion of parts |
| Scatter | Correlation analysis |
| Area | Cumulative totals |

### Filters

| Filter | Description |
|--------|-------------|
| Text Contains | Partial text match |
| Text Equals | Exact text match |
| Number Range | Min/max values |
| Date Range | Date between dates |
| Empty/Non-Empty | Presence check |
| Custom Formula | Advanced filtering |

### Sort

| Option | Description |
|--------|-------------|
| Ascending | A-Z, 0-9, oldest-newest |
| Descending | Z-A, 9-0, newest-oldest |
| Multi-Column | Sort by multiple columns |
| Custom Order | Sort by custom list |

### Import/Export

| Format | Import | Export |
|--------|--------|--------|
| CSV | Yes | Yes |
| JSON | Yes | Yes |
| Excel (.xlsx) | Yes | Yes |
| TSV | Yes | Yes |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy cell |
| `Ctrl+V` | Paste cell |
| `Ctrl+X` | Cut cell |
| `Tab` | Next cell (right) |
| `Enter` | Confirm and move down |
| `Delete` | Clear cell content |
| `Ctrl+A` | Select all |
| `Ctrl+Shift+F` | Open filter panel |
| `F2` | Edit active cell |
| `Ctrl+Arrow` | Navigate to edge of data |

---

## Sheet via Chat

### Creating a Sales Report

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a sales report</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 I've created a new spreadsheet: <strong>Sales Report</strong></p>
      <p>Here's the structure:</p>
      <p>| Month | Revenue | Orders | Avg Order |</p>
      <p>| Jan | $12,400 | 156 | $79.49 |</p>
      <p>| Feb | $15,200 | 189 | $80.42 |</p>
      <p>| Mar | $18,900 | 234 | $80.77 |</p>
      <p>[Open in Sheet] [Edit via Chat]</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
</div>

### Calculating Totals

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Calculate the total revenue</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔢 I've added a <strong>SUM</strong> formula to cell B16:</p>
      <p>=SUM(B2:B15)</p>
      <p>Result: <strong>$246,500</strong></p>
      <p>I've also added averages for Orders and Avg Order columns.</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

### Generating a Chart

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a chart showing revenue by month</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📈 Chart created: <strong>Monthly Revenue</strong></p>
      <p>Type: Bar chart</p>
      <p>Data range: A1:B15</p>
      <p>[View Chart] [Edit Chart] [Export as PNG]</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
</div>

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sheet/list` | GET | List all spreadsheets |
| `/api/sheet/search` | GET | Search spreadsheets |
| `/api/sheet/load` | GET | Load spreadsheet by ID |
| `/api/sheet/save` | POST | Save spreadsheet |
| `/api/sheet/delete` | POST | Delete spreadsheet |
| `/api/sheet/cell` | POST | Update cell value |
| `/api/sheet/format` | POST | Format cells |
| `/api/sheet/formula` | POST | Evaluate spreadsheet formula |
| `/api/sheet/range` | POST | Get range values |
| `/api/sheet/worksheet-meta` | POST | Update worksheet metadata |
| `/api/sheet/pivot` | POST | Create pivot table |
| `/api/sheet/export` | POST | Export spreadsheet |
| `/api/sheet/share` | POST | Share spreadsheet |
| `/api/sheet/new` | GET | Create new blank spreadsheet |
| `/api/sheet/merge` | POST | Merge cells |
| `/api/sheet/unmerge` | POST | Unmerge cells |
| `/api/sheet/freeze` | POST | Freeze panes |
| `/api/sheet/sort` | POST | Sort range |
| `/api/sheet/filter` | POST | Filter data |
| `/api/sheet/chart` | POST | Create chart |
| `/api/sheet/conditional-format` | POST | Apply conditional formatting |
| `/api/sheet/data-validation` | POST | Apply data validation rules |
| `/api/sheet/ai` | POST | Query spreadsheet AI assistant |
| `/api/sheet/:id` | GET | Get spreadsheet by ID |
| `/ws/sheet/:sheet_id` | GET | WebSocket for collaborative editing |

### Create Spreadsheet Request

```json
{
    "title": "Sales Report",
    "sheets": [
        {
            "name": "Revenue",
            "rows": 20,
            "columns": 10,
            "data": {
                "A1": "Month",
                "B1": "Revenue",
                "A2": "Jan",
                "B2": 12400
            }
        }
    ]
}
```

### Cell Update Request

```json
{
    "cells": {
        "B16": "=SUM(B2:B15)",
        "C16": "=AVERAGE(C2:C15)"
    }
}
```

### Spreadsheet Response

```json
{
    "id": "sheet-789",
    "title": "Sales Report",
    "sheets": [
        {
            "name": "Revenue",
            "data": [
                ["Month", "Revenue", "Orders"],
                ["Jan", 12400, 156],
                ["Feb", 15200, 189]
            ],
            "charts": [
                {
                    "id": "chart-001",
                    "type": "bar",
                    "title": "Monthly Revenue",
                    "data_range": "A1:B15"
                }
            ]
        }
    ],
    "created_at": "2025-05-15T09:00:00Z",
    "updated_at": "2025-05-15T11:00:00Z"
}
```

---

## Configuration

Sheet settings can be configured in `config.csv`:

```csv
key,value
max-rows,1200000
max-columns,256
auto-calculate,true
default-sheet-count,1
```

---

## Troubleshooting

### Formula Not Calculating

1. Check formula syntax (must start with `=`)
2. Verify cell references are valid
3. Check for circular references
4. Ensure referenced cells contain valid data

### Import Failing

1. Verify file format (CSV, JSON, Excel)
2. Check file encoding (UTF-8 recommended)
3. Ensure file size is under limit
4. Check for malformed data rows

### Chart Not Displaying

1. Verify data range is valid
2. Check that data contains numeric values
3. Ensure chart type matches data structure
4. Refresh the page

---

## See Also

- [Suite Manual](../suite-manual.md) - Complete user guide
- [Drive](./drive.md) - File storage for imports
- [Chat App](./chat.md) - Create sheets via chat
- [BASIC Database Keywords](../../04-basic-scripting/keyword-database.md) - Script integration
