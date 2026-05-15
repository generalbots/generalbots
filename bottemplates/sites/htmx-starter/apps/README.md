# HTMX App Templates

Templates for generating web applications with `CREATE SITE`. All apps use HTMX for server communication and are automatically bound to the botserver API.

---

## Overview

These templates serve as reference patterns for the LLM when generating new applications. Each template demonstrates:

- HTMX data binding patterns
- CSS theming with light/dark mode support
- Responsive layouts
- Loading states and error handling
- Modal dialogs and forms

---

## Available Templates

### `/crud`

Generic CRUD (Create, Read, Update, Delete) interface.

**Files:**
- `layout.html` - Full page layout with table, search, filters
- `data-row.html` - Table row template for each record
- `form-modal.html` - Add/edit modal dialog

**Usage:**
```basic
CREATE SITE "contacts", "bottemplates/apps/crud", "
Contact management with:
- Name, email, phone, company fields
- Status filter (active, inactive)
- Search by name or email
- Inline edit and delete
"
```

### `/dashboard`

Analytics dashboard with KPI cards and charts.

**Files:**
- `layout.html` - Dashboard layout with header, KPIs, charts
- `kpi-card.html` - Metric card component
- `chart-container.html` - Chart.js wrapper
- `activity-row.html` - Recent activity table row

**Usage:**
```basic
CREATE SITE "metrics", "bottemplates/apps/dashboard", "
Sales dashboard showing:
- Revenue, customers, orders, growth KPIs
- Monthly revenue chart
- Sales by category pie chart
- Recent orders table
"
```

### `/kanban`

Board-style project management interface.

**Files:**
- `layout.html` - Kanban board layout
- `column.html` - Status column template
- `card.html` - Task card template
- `card-modal.html` - Card detail modal

**Usage:**
```basic
CREATE SITE "projects", "bottemplates/apps/kanban", "
Project board with columns:
- Backlog, In Progress, Review, Done
- Drag and drop cards
- Assignee avatars
- Due date indicators
"
```

### `/admin`

Admin panel template for back-office operations.

**Files:**
- `layout.html` - Admin layout with sidebar navigation
- `sidebar.html` - Navigation component
- `stats-grid.html` - Overview statistics
- `crud-table.html` - Data management table

**Usage:**
```basic
CREATE SITE "admin", "bottemplates/apps/admin", "
Admin panel with:
- Sidebar navigation (Users, Products, Orders, Settings)
- Overview stats on dashboard
- User management table
- System settings form
"
```

### `/components`

Reusable UI components for composition.

| Component | File | Description |
|-----------|------|-------------|
| Data Table | `data-table.html` | Sortable, filterable table |
| Form Modal | `form-modal.html` | Modal with form fields |
| KPI Card | `kpi-card.html` | Metric display card |
| Search | `search-filter.html` | Search input with filters |
| Toast | `toast.html` | Notification toast |
| Pagination | `pagination.html` | Page navigation |
| Empty State | `empty-state.html` | No data placeholder |

---

## HTMX Patterns

### Loading Data

```html
<div id="data-list"
     hx-get="/api/data/${table}"
     hx-trigger="load"
     hx-swap="innerHTML">
    Loading...
</div>
```

### Auto-Refresh

```html
<div hx-get="/api/data/${table}"
     hx-trigger="load, every 30s"
     hx-swap="innerHTML">
</div>
```

### Search with Debounce

```html
<input type="search"
       hx-get="/api/data/${table}"
       hx-trigger="keyup changed delay:300ms"
       hx-target="#results">
```

### Form Submit

```html
<form hx-post="/api/data/${table}"
      hx-target="#data-list"
      hx-swap="afterbegin">
    <input name="field" required>
    <button type="submit">Save</button>
</form>
```

### Delete with Confirmation

```html
<button hx-delete="/api/data/${table}/${id}"
        hx-target="closest tr"
        hx-swap="outerHTML"
        hx-confirm="Delete this item?">
    Delete
</button>
```

### Loading Indicator

```html
<button hx-post="/api/action"
        hx-indicator="#spinner">
    Save
    <span id="spinner" class="htmx-indicator">⏳</span>
</button>
```

---

## API Mapping

Generated HTMX calls automatically map to botserver endpoints:

| HTMX | HTTP Method | Endpoint | BASIC Equivalent |
|------|-------------|----------|------------------|
| `hx-get` | GET | `/api/data/{table}` | `FIND "{table}"` |
| `hx-post` | POST | `/api/data/{table}` | `UPSERT "{table}"` |
| `hx-put` | PUT | `/api/data/{table}/{id}` | `SET "{table}"` |
| `hx-delete` | DELETE | `/api/data/{table}/{id}` | `DELETE "{table}"` |

### Query Parameters

```html
<!-- Filter -->
hx-get="/api/data/leads?status=active"

<!-- Sort -->
hx-get="/api/data/leads?sort=created_at&order=desc"

<!-- Paginate -->
hx-get="/api/data/leads?page=2&limit=20"

<!-- Search -->
hx-get="/api/data/leads?q=john"

<!-- Combine -->
hx-get="/api/data/leads?status=active&sort=name&q=smith"
```

---

## Theming

All templates use CSS custom properties for theming:

```css
:root {
    --color-primary: #0ea5e9;
    --color-success: #10b981;
    --color-warning: #f59e0b;
    --color-danger: #ef4444;
    --color-bg: #f8fafc;
    --color-surface: #ffffff;
    --color-text: #1e293b;
    --color-border: #e2e8f0;
    --radius: 8px;
}

@media (prefers-color-scheme: dark) {
    :root {
        --color-bg: #0f172a;
        --color-surface: #1e293b;
        --color-text: #f1f5f9;
        --color-border: #334155;
    }
}
```

---

## Best Practices

### Writing Good Prompts

**✅ Good:**
```basic
CREATE SITE "crm", "bottemplates/apps/crud", "
Customer management with:
- Fields: name, email, phone, company, status
- Status options: lead, prospect, customer, churned
- Search by name or email
- Filter by status
- Sort by name or created date
- Click row to see full details
"
```

**❌ Bad:**
```basic
CREATE SITE "crm", "bottemplates/apps/crud", "make a crm"
```

### Choosing Templates

| Use Case | Template |
|----------|----------|
| Data management | `/crud` |
| Metrics & analytics | `/dashboard` |
| Task/project boards | `/kanban` |
| Back-office admin | `/admin` |

---

## See Also

- [CREATE SITE Keyword](../../botbook/src/06-gbdialog/keyword-create-site.md)
- [Autonomous Task AI](../../botbook/src/07-gbapp/autonomous-tasks.md)
- [HTMX Documentation](https://htmx.org/docs/)