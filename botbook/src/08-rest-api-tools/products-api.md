# Products API

> **Inventory and product catalog management with stock tracking, price lists, and movement history.**

---

## Base URL

```
/api/products
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Items (Products)

### List Items

**`GET /api/products/items`**

Returns all product items with optional filtering and pagination.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `categoryId` | string | No | Filter by category ID |
| `search` | string | No | Search by name or SKU |
| `lowStock` | boolean | No | Only return items below minimum stock |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 50) |

**Request:**

```
GET /api/products/items?categoryId=cat-01&limit=10
```

**Response:**

```json
{
  "items": [
    {
      "id": "item-001",
      "sku": "WIDGET-A1",
      "name": "Widget Alpha",
      "description": "High-performance widget",
      "categoryId": "cat-01",
      "unitPrice": 29.99,
      "currency": "BRL",
      "currentStock": 150,
      "minimumStock": 20,
      "unit": "un",
      "status": "active",
      "createdAt": "2026-01-15T10:00:00Z",
      "updatedAt": "2026-06-01T08:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 42,
    "totalPages": 5
  }
}
```

---

### Create Item

**`POST /api/products/items`**

Creates a new product item.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sku` | string | Yes | Stock keeping unit (unique) |
| `name` | string | Yes | Product name |
| `description` | string | No | Product description |
| `categoryId` | string | Yes | Category identifier |
| `unitPrice` | number | Yes | Unit price |
| `currency` | string | No | Currency code (default: `BRL`) |
| `currentStock` | integer | No | Initial stock (default: 0) |
| `minimumStock` | integer | No | Minimum stock alert threshold |
| `unit` | string | No | Unit of measure (default: `un`) |

**Request:**

```json
{
  "sku": "WIDGET-B2",
  "name": "Widget Beta",
  "description": "Premium widget with extended features",
  "categoryId": "cat-01",
  "unitPrice": 49.99,
  "currency": "BRL",
  "currentStock": 0,
  "minimumStock": 10,
  "unit": "un"
}
```

**Response:**

```json
{
  "id": "item-002",
  "sku": "WIDGET-B2",
  "name": "Widget Beta",
  "description": "Premium widget with extended features",
  "categoryId": "cat-01",
  "unitPrice": 49.99,
  "currency": "BRL",
  "currentStock": 0,
  "minimumStock": 10,
  "unit": "un",
  "status": "active",
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

### Get Item

**`GET /api/products/items/:id`**

Returns full details of a specific item.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Item identifier (path param) |

**Request:**

```
GET /api/products/items/item-001
```

**Response:**

```json
{
  "id": "item-001",
  "sku": "WIDGET-A1",
  "name": "Widget Alpha",
  "description": "High-performance widget",
  "categoryId": "cat-01",
  "unitPrice": 29.99,
  "currency": "BRL",
  "currentStock": 150,
  "minimumStock": 20,
  "unit": "un",
  "status": "active",
  "createdAt": "2026-01-15T10:00:00Z",
  "updatedAt": "2026-06-01T08:30:00Z"
}
```

---

### Update Item

**`PUT /api/products/items/:id`**

Updates an existing product item.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Item identifier (path param) |
| `name` | string | No | Product name |
| `description` | string | No | Description |
| `unitPrice` | number | No | Unit price |
| `minimumStock` | integer | No | Minimum stock threshold |
| `status` | string | No | `active` or `inactive` |

**Request:**

```json
{
  "unitPrice": 34.99,
  "description": "Updated: High-performance widget v2"
}
```

**Response:**

```json
{
  "id": "item-001",
  "sku": "WIDGET-A1",
  "name": "Widget Alpha",
  "unitPrice": 34.99,
  "description": "Updated: High-performance widget v2",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Delete Item

**`DELETE /api/products/items/:id`**

Soft-deletes a product item (sets status to `inactive`).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Item identifier (path param) |

**Request:**

```
DELETE /api/products/items/item-002
```

**Response:**

```json
{
  "deleted": true,
  "id": "item-002"
}
```

---

### Update Stock

**`PUT /api/products/items/:id/stock`**

Adjusts stock level for an item. Supports absolute and relative adjustments.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Item identifier (path param) |
| `quantity` | integer | Yes | Stock change (positive = add, negative = remove) |
| `reason` | string | Yes | Reason for adjustment |
| `type` | string | No | `inbound`, `outbound`, `adjustment` (default: `adjustment`) |

**Request:**

```json
{
  "quantity": 50,
  "reason": "Purchase order PO-2026-042 received",
  "type": "inbound"
}
```

**Response:**

```json
{
  "id": "item-001",
  "previousStock": 150,
  "adjustment": 50,
  "newStock": 200,
  "reason": "Purchase order PO-2026-042 received",
  "type": "inbound",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Get Item Movements

**`GET /api/products/items/:id/movements`**

Returns stock movement history for an item.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Item identifier (path param) |
| `limit` | integer | No | Max results (default: 20) |
| `offset` | integer | No | Pagination offset (default: 0) |

**Request:**

```
GET /api/products/items/item-001/movements?limit=5
```

**Response:**

```json
{
  "itemId": "item-001",
  "movements": [
    {
      "id": "mov-101",
      "type": "inbound",
      "quantity": 50,
      "reason": "Purchase order PO-2026-042",
      "balanceAfter": 200,
      "createdAt": "2026-06-04T12:00:00Z"
    },
    {
      "id": "mov-100",
      "type": "outbound",
      "quantity": -10,
      "reason": "Order ORD-2026-089 fulfilled",
      "balanceAfter": 150,
      "createdAt": "2026-06-03T16:45:00Z"
    }
  ],
  "total": 2
}
```

---

## Services

### List Services

**`GET /api/products/services`**

Returns all service products.

**Response:**

```json
[
  {
    "id": "svc-001",
    "name": "Consulting - Hourly",
    "description": "Technical consulting per hour",
    "unitPrice": 150.00,
    "currency": "BRL",
    "unit": "hora",
    "status": "active"
  }
]
```

---

### Create Service

**`POST /api/products/services`**

Creates a new service product.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Service name |
| `description` | string | No | Service description |
| `unitPrice` | number | Yes | Price per unit |
| `currency` | string | No | Currency code (default: `BRL`) |
| `unit` | string | Yes | Unit of measure (e.g., `hora`, `sessao`, `projeto`) |

**Request:**

```json
{
  "name": "Training - Full Day",
  "description": "Full-day on-site training session",
  "unitPrice": 2500.00,
  "currency": "BRL",
  "unit": "dia"
}
```

**Response:**

```json
{
  "id": "svc-002",
  "name": "Training - Full Day",
  "description": "Full-day on-site training session",
  "unitPrice": 2500.00,
  "currency": "BRL",
  "unit": "dia",
  "status": "active",
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

### Get Service

**`GET /api/products/services/:id`**

Returns details of a specific service.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Service identifier (path param) |

**Request:**

```
GET /api/products/services/svc-001
```

**Response:**

```json
{
  "id": "svc-001",
  "name": "Consulting - Hourly",
  "description": "Technical consulting per hour",
  "unitPrice": 150.00,
  "currency": "BRL",
  "unit": "hora",
  "status": "active",
  "createdAt": "2026-02-01T10:00:00Z"
}
```

---

### Update Service

**`PUT /api/products/services/:id`**

Updates an existing service.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Service identifier (path param) |
| `name` | string | No | Service name |
| `description` | string | No | Description |
| `unitPrice` | number | No | Price per unit |
| `status` | string | No | `active` or `inactive` |

**Request:**

```json
{
  "unitPrice": 175.00
}
```

**Response:**

```json
{
  "id": "svc-001",
  "name": "Consulting - Hourly",
  "unitPrice": 175.00,
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Delete Service

**`DELETE /api/products/services/:id`**

Soft-deletes a service.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Service identifier (path param) |

**Request:**

```
DELETE /api/products/services/svc-002
```

**Response:**

```json
{
  "deleted": true,
  "id": "svc-002"
}
```

---

## Categories

### List Categories

**`GET /api/products/categories`**

Returns all product categories.

**Response:**

```json
[
  {
    "id": "cat-01",
    "name": "Widgets",
    "description": "Widget product line",
    "itemCount": 12
  },
  {
    "id": "cat-02",
    "name": "Accessories",
    "description": "Complementary accessories",
    "itemCount": 8
  }
]
```

---

### Create Category

**`POST /api/products/categories`**

Creates a new product category.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Category name |
| `description` | string | No | Category description |
| `parentId` | string | No | Parent category for nesting |

**Request:**

```json
{
  "name": "Services",
  "description": "Service products"
}
```

**Response:**

```json
{
  "id": "cat-03",
  "name": "Services",
  "description": "Service products",
  "itemCount": 0,
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

## Price Lists

### List Price Lists

**`GET /api/products/price-lists`**

Returns all price lists (customer tiers, promotions, etc.).

**Response:**

```json
[
  {
    "id": "pl-001",
    "name": "Standard Pricing",
    "description": "Default price list",
    "isDefault": true,
    "itemCount": 20
  },
  {
    "id": "pl-002",
    "name": "VIP Partner",
    "description": "Discounted prices for VIP partners",
    "isDefault": false,
    "itemCount": 20
  }
]
```

---

### Create Price List

**`POST /api/products/price-lists`**

Creates a new price list.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Price list name |
| `description` | string | No | Description |
| `isDefault` | boolean | No | Set as default list (default: false) |

**Request:**

```json
{
  "name": "Black Friday 2026",
  "description": "Special promotional pricing",
  "isDefault": false
}
```

**Response:**

```json
{
  "id": "pl-003",
  "name": "Black Friday 2026",
  "description": "Special promotional pricing",
  "isDefault": false,
  "itemCount": 0,
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

## Statistics & Reports

### Get Product Stats

**`GET /api/products/stats`**

Returns aggregate product statistics.

**Response:**

```json
{
  "totalItems": 42,
  "totalServices": 5,
  "totalCategories": 3,
  "totalStockValue": 45750.00,
  "currency": "BRL",
  "lowStockItems": 3,
  "outOfStockItems": 1,
  "lastUpdated": "2026-06-04T12:00:00Z"
}
```

---

### Get Low Stock Items

**`GET /api/products/low-stock`**

Returns items whose current stock is at or below the minimum threshold.

**Response:**

```json
[
  {
    "id": "item-005",
    "sku": "CABLE-X1",
    "name": "USB-C Cable",
    "currentStock": 3,
    "minimumStock": 20,
    "deficit": 17,
    "categoryName": "Accessories"
  },
  {
    "id": "item-012",
    "sku": "ADAPTER-Q2",
    "name": "Power Adapter",
    "currentStock": 0,
    "minimumStock": 15,
    "deficit": 15,
    "categoryName": "Accessories"
  }
]
```

---

## See Also

- [Tasks API](../08-rest-api-tools/tasks-api.md) — Task management tied to product operations
- [Reports API](../08-rest-api-tools/reports-api.md) — Custom report generation
- [Files API](../08-rest-api-tools/files-api.md) — Product image and document storage
