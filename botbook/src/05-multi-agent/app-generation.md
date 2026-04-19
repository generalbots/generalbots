# App Generation

From natural language to complete application, the generation process handles database design, UI creation, and automation setup.

![App Generation Flow](../assets/05-multi-agent/app-generation-flow.svg)

---

## How It Works

You describe what you need. The system analyzes your request, identifies the domain, extracts entities, and generates everything automatically.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create an inventory app for my store</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Done:</strong></p>
      <p>products table created</p>
      <p>categories table created</p>
      <p>suppliers table created</p>
      <p>App available at /apps/inventory</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

---

## What Gets Created

| Component | Location | Description |
|-----------|----------|-------------|
| Database tables | `tables.bas` | Data schema with fields and types |
| Web pages | `/apps/{name}/` | HTMX-based user interface |
| Voice commands | `.gbdialog/tools/` | Activated by trigger phrases |
| Automations | `.gbdialog/schedulers/` | Timed tasks and reports |

---

## Modifying Your App

Use conversation to change any aspect of your application.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add a supplier field to products</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Supplier field added to products table.</p>
      <p>Form updated in products.html.</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Change the header color to red</p>
      <div class="wa-time">14:21</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Header color updated to red in styles.css.</p>
      <div class="wa-time">14:21</div>
    </div>
  </div>
</div>

---

## Multiple Apps, Shared Data

Create different applications that share the same database tables.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a reports app using the same inventory data</p>
      <div class="wa-time">15:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Done:</strong></p>
      <p>App created at /apps/inventory-reports</p>
      <p>Uses existing tables: products, categories, suppliers</p>
      <div class="wa-time">15:01</div>
    </div>
  </div>
</div>

---

## Generation Process

| Phase | Description |
|-------|-------------|
| **Intent Analysis** | LLM identifies domain and required entities |
| **Schema Design** | Tables and relationships defined |
| **UI Generation** | HTMX pages created for each entity |
| **Tool Creation** | Voice commands for common actions |
| **Scheduler Setup** | Automations for reports and alerts |

---

## Next Steps

- [Designer Guide](./designer.md) — All modification commands
- [Data Model](./data-model.md) — Understanding table definitions