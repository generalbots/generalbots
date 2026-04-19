# Data Model

Tables store your application data. When you describe what you need, the system creates table definitions automatically in `tables.bas`.

![Data Model Flow](../assets/05-multi-agent/data-model-flow.svg)

---

## What Is a Table?

A table organizes related data into rows and columns, similar to a spreadsheet.

| id | name | phone | email |
|----|------|-------|-------|
| 1 | John Smith | 555-1234 | john@email.com |
| 2 | Maria Garcia | 555-5678 | maria@email.com |

---

## Creating Tables

Describe what data you need to store.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need a customers table with name, phone, and email</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>customers table created with fields:</p>
      <p>name (STRING)</p>
      <p>phone (STRING)</p>
      <p>email (STRING)</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

---

## Adding Fields

Add new fields to existing tables through conversation.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add address to customers</p>
      <div class="wa-time">14:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Address field added to customers table.</p>
      <div class="wa-time">14:15</div>
    </div>
  </div>
</div>

---

## Field Types

| Type | Use For | Example |
|------|---------|---------|
| `STRING` | Short text | Names, phones |
| `TEXT` | Long text | Notes, descriptions |
| `INTEGER` | Whole numbers | Age, quantity |
| `DECIMAL` | Numbers with decimals | Price, weight |
| `BOOLEAN` | Yes/No values | Active, completed |
| `DATE` | Dates only | Birth date |
| `DATETIME` | Date and time | Appointment time |

---

## Searching Data

Use natural language to find records.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Find customers named Smith</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Found 2 customers:</strong></p>
      <p>John Smith - 555-1234</p>
      <p>Jane Smith - 555-9876</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
</div>

---

## Linking Tables

Create relationships between tables.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create orders table linked to customers</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>orders table created with:</p>
      <p>customer_id (links to customers)</p>
      <p>date (DATETIME)</p>
      <p>total (DECIMAL)</p>
      <p>status (STRING)</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>
</div>

---

## Shared Data

All applications within a bot share the same tables. Change data in one app, and it updates everywhere.

| Concept | Description |
|---------|-------------|
| One bot = one database | All apps share tables |
| Schema in tables.bas | Single source of truth |
| Auto-sync | Changes deploy automatically |

---

## TABLE Keyword

Tables are defined in `.gbdialog/tables.bas` using the TABLE keyword:

| Syntax | Description |
|--------|-------------|
| `TABLE name` | Start table definition |
| `field AS TYPE` | Define a field |
| `END TABLE` | End table definition |

---

## FIND Keyword

Query data using the FIND keyword:

| Syntax | Description |
|--------|-------------|
| `FIND * IN table` | Get all records |
| `FIND * IN table WHERE condition` | Filter records |
| `FIND field1, field2 IN table` | Select specific fields |

---

## Next Steps

- [Designer Guide](./designer.md) — Modify tables through conversation
- [Examples](./examples.md) — Real-world data models