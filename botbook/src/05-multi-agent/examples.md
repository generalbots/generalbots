# Examples

Real-world applications built through natural conversation.

---

## Example 1: Cellphone Store CRM

**Request:** "Create a CRM for my cellphone store with customer tracking, inventory, sales, and repair status"

**Generated URL:** `/apps/cellphone-crm`

### Tables Created

| Table | Fields |
|-------|--------|
| `customers` | id, name, phone, email, address, notes |
| `products` | id, name, brand, model, price, cost, stock |
| `sales` | id, customer_id, product_id, quantity, total, date |
| `repairs` | id, customer_id, device, problem, status, price |

### Features

| Feature | Description |
|---------|-------------|
| Customer search | Find by name or phone |
| Inventory alerts | Low stock notifications |
| Sales tracking | Linked to customer and product |
| Repair board | Status workflow tracking |
| Daily summary | Sales report automation |

### Execution Steps

| Step | Task | Time |
|------|------|------|
| 1 | Create database tables | 15s |
| 2 | Generate customer management UI | 45s |
| 3 | Generate product inventory UI | 30s |
| 4 | Generate sales tracking UI | 40s |
| 5 | Generate repair status board | 50s |
| 6 | Add search and filters | 25s |
| **Total** | | **3m 25s** |

---

## Example 2: Restaurant Reservations

**Request:** "Build a reservation system for my restaurant with table management and waitlist"

**Generated URL:** `/apps/restaurant-reservations`

### Tables Created

| Table | Fields |
|-------|--------|
| `tables` | id, number, capacity, location, status |
| `reservations` | id, guest_name, phone, party_size, date, time, table_id, status |
| `waitlist` | id, guest_name, phone, party_size, added_at, estimated_wait |

### Features

| Feature | Description |
|---------|-------------|
| Table layout | Visual availability display |
| Calendar view | Reservation scheduling |
| Waitlist | Real-time queue management |
| SMS notifications | Guest alerts (if configured) |
| Daily bookings | Summary report |

---

## Example 3: Property Management

**Request:** "Create a system to manage rental properties with tenants, leases, and maintenance requests"

**Generated URL:** `/apps/property-manager`

### Tables Created

| Table | Fields |
|-------|--------|
| `properties` | id, address, type, bedrooms, bathrooms, rent, status |
| `tenants` | id, name, phone, email, emergency_contact |
| `leases` | id, property_id, tenant_id, start_date, end_date, rent, deposit |
| `maintenance` | id, property_id, tenant_id, issue, priority, status, assigned_to |
| `payments` | id, lease_id, amount, date, method, status |

### Features

| Feature | Description |
|---------|-------------|
| Property listing | Filters by type, status, rent |
| Tenant directory | With lease history |
| Maintenance tracking | Priority and status workflow |
| Payment tracking | Due date alerts |
| Lease reminders | Expiration notifications |

---

## Example 4: Gym Membership

**Request:** "Build a gym membership system with class scheduling and attendance tracking"

**Generated URL:** `/apps/gym-manager`

### Tables Created

| Table | Fields |
|-------|--------|
| `members` | id, name, phone, email, plan, start_date, expiry_date |
| `classes` | id, name, instructor, day, time, capacity, room |
| `enrollments` | id, member_id, class_id, enrolled_at |
| `attendance` | id, member_id, check_in, check_out |
| `payments` | id, member_id, amount, date, plan |

### Features

| Feature | Description |
|---------|-------------|
| Check-in/out | Member attendance tracking |
| Class schedule | With enrollment management |
| Attendance reports | Usage analytics |
| Expiry alerts | Membership renewal reminders |
| Revenue tracking | Payment summaries |

---

## Example 5: Event Planning

**Request:** "Create an event planning tool with guest lists, vendors, and budget tracking"

**Generated URL:** `/apps/event-planner`

### Tables Created

| Table | Fields |
|-------|--------|
| `events` | id, name, date, venue, budget, status |
| `guests` | id, event_id, name, email, rsvp_status, dietary_needs, table |
| `vendors` | id, event_id, name, service, contact, cost, status |
| `tasks` | id, event_id, task, assignee, due_date, status |
| `expenses` | id, event_id, category, description, amount, paid |

### Features

| Feature | Description |
|---------|-------------|
| Event dashboard | Countdown and overview |
| Guest list | RSVP tracking |
| Vendor management | Contracts and payments |
| Task checklist | Assignment and due dates |
| Budget tracking | Budget vs actual spending |

---

## Example 6: Medical Clinic

**Request:** "Build a patient management system for a small clinic with appointments and medical records"

**Generated URL:** `/apps/clinic-manager`

### Tables Created

| Table | Fields |
|-------|--------|
| `patients` | id, name, dob, phone, email, address, insurance |
| `appointments` | id, patient_id, doctor, date, time, reason, status |
| `records` | id, patient_id, date, diagnosis, treatment, notes, doctor |
| `prescriptions` | id, record_id, medication, dosage, duration |

### Features

| Feature | Description |
|---------|-------------|
| Patient search | With full history |
| Appointment calendar | Per doctor view |
| Medical records | Timeline per patient |
| Prescriptions | Medication tracking |
| Daily list | Appointments per doctor |

---

## Example 7: Inventory System

**Request:** "Simple inventory tracking with suppliers, purchase orders, and stock alerts"

**Generated URL:** `/apps/inventory`

### Tables Created

| Table | Fields |
|-------|--------|
| `products` | id, sku, name, category, quantity, min_stock, location |
| `suppliers` | id, name, contact, email, phone, address |
| `purchase_orders` | id, supplier_id, status, total, created_at |
| `order_items` | id, order_id, product_id, quantity, unit_price |
| `stock_movements` | id, product_id, type, quantity, reason, date |

### Features

| Feature | Description |
|---------|-------------|
| Product list | With stock levels |
| Low stock alerts | Dashboard notifications |
| Supplier directory | Contact management |
| Purchase orders | Creation and tracking |
| Stock history | Movement audit trail |

---

## Complexity Guide

| Complexity | Tables | Time | Example |
|------------|--------|------|---------|
| Simple | 1-2 | 1-2 min | Contact list, tracker |
| Medium | 3-5 | 3-5 min | CRM, basic inventory |
| Complex | 6-10 | 5-10 min | Full business management |
| Large | 10+ | 10+ min | ERP-style systems |

---

## Tips for Better Results

### Be Specific

| Less Effective | More Effective |
|----------------|----------------|
| "Business app" | "CRM for cellphone store with customers, products, sales, and repair tracking" |
| "Inventory system" | "Inventory system with low stock alerts when below 10 units" |
| "Track repairs" | "Repair tracking with statuses: received, diagnosing, repairing, ready, delivered" |

### Include Key Details

| Detail | Example |
|--------|---------|
| Entities | "customers, products, orders" |
| Relationships | "orders linked to customers" |
| Workflows | "status: pending, approved, shipped" |
| Automations | "daily report at 9am" |
| Alerts | "notify when stock below 10" |

---

## Next Steps

- [Task Workflow](./workflow.md) — How tasks execute
- [App Generation](./app-generation.md) — Technical details
- [Data Model](./data-model.md) — Table structure