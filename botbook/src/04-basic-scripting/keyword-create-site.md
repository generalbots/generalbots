# CREATE SITE Keyword

> **Generate Complete HTMX Web Applications Autonomously**

---

## Overview

`CREATE SITE` is the cornerstone keyword for autonomous app generation in General Bots. It transforms natural language descriptions into fully functional HTMX-based web applications that are automatically bound to the botserver API.

**The key insight:** Generated apps don't need their own backend - they use botserver's infrastructure through HTMX calls.

---

## Syntax

```basic
CREATE SITE "alias", "template-dir", "prompt"
```

**Parameters:**

| Parameter | Description |
|-----------|-------------|
| `alias` | Name of the new site (becomes URL endpoint and folder name) |
| `template-dir` | Path to HTML templates that serve as reference |
| `prompt` | Natural language description of what to build |

---

## How It Works

### 1. Template Loading

The system reads all `.html` files from the template directory:

```
templates/app/
├── layout.html          # Page structure
├── components.html      # Reusable UI components
├── forms.html           # Form patterns
└── data-table.html      # List/table patterns
```

All templates are combined with separators to give the LLM context about available patterns.

### 2. LLM Generation

The combined templates + prompt are sent to the LLM with instructions to:

- Clone the template structure and styling
- Use only local `_assets` (no external CDNs)
- Bind all data operations to botserver API via HTMX
- Generate semantic, accessible HTML

### 3. Deployment

The generated `index.html` is written to `<site_path>/<alias>/`:

```
sites/
└── crm/
    ├── index.html       # Generated application
    └── _assets/         # Copied from templates
        ├── htmx.min.js
        ├── app.js
        └── styles.css
```

The site is immediately available at `/<alias>` endpoint.

---

## Generated HTMX Patterns

### Data Lists with Auto-Refresh

```html
<!-- Lead list that loads on page and refreshes every 30s -->
<div id="leads"
     hx-get="/api/data/leads"
     hx-trigger="load, every 30s"
     hx-swap="innerHTML">
    <div class="loading">Loading leads...</div>
</div>
```

### Create Forms

```html
<form hx-post="/api/data/leads"
      hx-target="#leads"
      hx-swap="afterbegin"
      hx-indicator="#saving">
    <input name="name" placeholder="Lead name" required>
    <input name="email" type="email" placeholder="Email">
    <input name="phone" placeholder="Phone">
    <select name="status">
        <option value="new">New</option>
        <option value="contacted">Contacted</option>
        <option value="qualified">Qualified</option>
    </select>
    <button type="submit">
        <span class="btn-text">Add Lead</span>
        <span id="saving" class="htmx-indicator">Saving...</span>
    </button>
</form>
```

### Inline Editing

```html
<div class="lead-card"
     hx-get="/api/data/leads/${id}"
     hx-trigger="click"
     hx-target="#detail-panel"
     hx-swap="innerHTML">
    <h3>${name}</h3>
    <p>${email}</p>
    <span class="status status-${status}">${status}</span>
</div>
```

### Delete with Confirmation

```html
<button hx-delete="/api/data/leads/${id}"
        hx-target="closest .lead-card"
        hx-swap="outerHTML swap:1s"
        hx-confirm="Delete this lead?">
    🗑️ Delete
</button>
```

### Search and Filter

```html
<input type="search"
       name="q"
       placeholder="Search leads..."
       hx-get="/api/data/leads"
       hx-trigger="keyup changed delay:300ms"
       hx-target="#leads"
       hx-include="[name='status-filter']">

<select name="status-filter"
        hx-get="/api/data/leads"
        hx-trigger="change"
        hx-target="#leads"
        hx-include="[name='q']">
    <option value="">All Statuses</option>
    <option value="new">New</option>
    <option value="contacted">Contacted</option>
</select>
```

---

## API Binding

Generated HTMX calls map directly to botserver endpoints:

| HTMX Attribute | Endpoint | BASIC Equivalent |
|----------------|----------|------------------|
| `hx-get="/api/data/{table}"` | List/Read | `FIND "{table}"` |
| `hx-post="/api/data/{table}"` | Create | `UPSERT "{table}"` |
| `hx-put="/api/data/{table}/{id}"` | Update | `SET "{table}"` |
| `hx-delete="/api/data/{table}/{id}"` | Delete | `DELETE "{table}"` |

### Query Parameters

The API supports filtering via query params:

```html
<!-- Filter by status -->
hx-get="/api/data/leads?status=new"

<!-- Sort by date -->
hx-get="/api/data/leads?sort=created_at&order=desc"

<!-- Pagination -->
hx-get="/api/data/leads?page=2&limit=20"

<!-- Search -->
hx-get="/api/data/leads?q=john"
```

---

## JavaScript Integration

Generated apps include `app.js` for interactions that need JavaScript:

```javascript
// Toast notifications
function showToast(message, type = 'success') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

// HTMX event hooks
document.body.addEventListener('htmx:afterSwap', (e) => {
    if (e.detail.target.id === 'leads') {
        showToast('Data updated');
    }
});

document.body.addEventListener('htmx:responseError', (e) => {
    showToast('Error: ' + e.detail.error, 'error');
});

// Modal handling
function openModal(id) {
    document.getElementById(id).classList.add('active');
}

function closeModal(id) {
    document.getElementById(id).classList.remove('active');
}
```

---

## Examples

### Simple CRM

```basic
CREATE SITE "crm", "bottemplates/apps/crud", "
Build a CRM with:
- Contact list with search and filters
- Add/edit contact form (name, email, phone, company)
- Status tracking (lead, prospect, customer)
- Activity timeline per contact
- Quick action buttons (call, email)
"
```

### Dashboard

```basic
CREATE SITE "dashboard", "bottemplates/apps/dashboard", "
Create an executive dashboard showing:
- KPI cards (revenue, customers, orders, growth)
- Revenue chart (last 12 months)
- Top products table
- Recent orders list
- Auto-refresh every 60 seconds
"
```

### Project Tracker

```basic
CREATE SITE "projects", "bottemplates/apps/kanban", "
Build a project management board with:
- Kanban columns (Backlog, In Progress, Review, Done)
- Drag and drop cards between columns
- Card details (title, description, assignee, due date)
- Filter by assignee or status
- Quick add card form
"
```

### E-commerce Admin

```basic
CREATE SITE "shop-admin", "bottemplates/apps/admin", "
Create an admin panel for e-commerce:
- Products table with inline editing
- Order management with status updates
- Customer list with order history
- Inventory alerts (low stock)
- Sales summary charts
"
```

---

## Template Structure

Organize templates for reuse:

```
bottemplates/apps/
├── crud/                   # CRUD application templates
│   ├── layout.html         # Base page structure
│   ├── data-row.html       # Table row template
│   └── form-modal.html     # Modal form pattern
├── dashboard/              # Dashboard templates
│   ├── layout.html         # Dashboard layout
│   ├── kpi-card.html       # Metric card
│   └── chart.html          # Chart container
├── kanban/                 # Board templates
│   ├── layout.html         # Kanban layout
│   ├── column.html         # Single column
│   └── card.html           # Task card
├── admin/                  # Admin panel templates
│   ├── layout.html         # Admin structure
│   └── sidebar.html        # Navigation
└── components/             # Reusable components
    ├── data-table.html     # Sortable table
    ├── search-filter.html  # Search with filters
    └── toast.html          # Notifications
```

---

## Styling

Generated apps use CSS variables for theming:

```css
:root {
    --color-primary: #0ea5e9;
    --color-success: #10b981;
    --color-warning: #f59e0b;
    --color-danger: #ef4444;
    --color-bg: #ffffff;
    --color-text: #1e293b;
    --color-border: #e2e8f0;
    --radius: 8px;
    --shadow: 0 1px 3px rgba(0,0,0,0.1);
}

@media (prefers-color-scheme: dark) {
    :root {
        --color-bg: #0f172a;
        --color-text: #f1f5f9;
        --color-border: #334155;
    }
}
```

---

## Storage

Generated sites and their data live in `.gbdrive`:

```
project.gbdrive/
├── apps/
│   └── crm/
│       ├── index.html      # Generated app
│       └── _assets/        # Static files
├── data/
│   └── contacts.json       # App data
└── uploads/
    └── ...                 # User uploads
```

---

## Error Handling

Generated apps handle errors gracefully:

```html
<!-- Loading state -->
<div class="htmx-indicator">
    <span class="spinner"></span> Loading...
</div>

<!-- Error display -->
<div id="error-container"
     hx-swap-oob="true"
     class="error-message">
</div>
```

```javascript
// Global error handler
document.body.addEventListener('htmx:responseError', function(e) {
    document.getElementById('error-container').innerHTML = 
        `<div class="alert alert-error">
            ${e.detail.xhr.responseText || 'An error occurred'}
        </div>`;
});
```

---

## Best Practices

### Be Specific in Prompts

✅ **Good:**
```basic
CREATE SITE "crm", "bottemplates/apps/crud", "
Build a real estate CRM with:
- Lead list showing name, phone, property interest, status
- Filter by status (new, contacted, showing, offer, closed)
- Add lead form with validation
- Click lead to see full details and notes
- Add note button on detail view
"
```

❌ **Bad:**
```basic
CREATE SITE "crm", "templates/app", "Make a CRM"
```

### Use Appropriate Templates

Match your template to the use case:
- `bottemplates/apps/crud` - General CRUD applications
- `bottemplates/apps/dashboard` - Dashboards and reports
- `bottemplates/apps/kanban` - Board-style interfaces
- `bottemplates/apps/admin` - Admin panels

### Test with Dry Run

Use autonomous task dry run mode to preview before deploying:

```basic
' In supervised/dry-run mode
CREATE SITE "test-crm", "bottemplates/apps/crud", "..."
' Review generated HTML before committing
```

---

## See Also

- [Autonomous Task AI](../07-gbapp/autonomous-tasks.md) - How the machine does the work
- [HTMX Architecture](../07-user-interface/htmx-architecture.md) - UI patterns
- [.gbdrive Storage](../02-templates/gbdrive.md) - File management
- [API Reference](../08-rest-api-tools/README.md) - botserver endpoints