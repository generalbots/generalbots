# GRAPHQL

The `GRAPHQL` keyword executes GraphQL queries and mutations against external APIs, enabling bots to interact with modern GraphQL-based services.

---

## Syntax

```basic
result = GRAPHQL url, query
result = GRAPHQL url, query WITH variables
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | String | The GraphQL endpoint URL |
| `query` | String | The GraphQL query or mutation |
| `WITH` | Clause | Optional variables for the query |

---

## Description

`GRAPHQL` sends queries and mutations to GraphQL APIs. GraphQL allows you to request exactly the data you need in a single request, making it efficient for complex data fetching. The keyword handles query formatting, variable substitution, and response parsing.

Use cases include:
- Fetching specific fields from APIs
- Creating, updating, or deleting data via mutations
- Querying nested relationships in one request
- Interacting with modern API platforms

---

## Examples

### Basic Query

```basic
' Simple query without variables
query = '
    query {
        users {
            id
            name
            email
        }
    }
'

result = GRAPHQL "https://api.example.com/graphql", query

FOR EACH user IN result.data.users
    TALK user.name + ": " + user.email
NEXT
```

### Query with Variables

```basic
' Query with variables
query = '
    query GetUser($id: ID!) {
        user(id: $id) {
            id
            name
            email
            orders {
                id
                total
                status
            }
        }
    }
'

result = GRAPHQL "https://api.example.com/graphql", query WITH id = user_id

TALK "User: " + result.data.user.name
TALK "Orders: " + LEN(result.data.user.orders)
```

### Mutation

```basic
' Create a new record
mutation = '
    mutation CreateUser($name: String!, $email: String!) {
        createUser(input: {name: $name, email: $email}) {
            id
            name
            email
            createdAt
        }
    }
'

result = GRAPHQL "https://api.example.com/graphql", mutation WITH
    name = user_name,
    email = user_email

TALK "User created with ID: " + result.data.createUser.id
```

### With Authentication

```basic
' Set authorization header for GraphQL
SET HEADER "Authorization", "Bearer " + api_token

query = '
    query {
        me {
            id
            name
            role
        }
    }
'

result = GRAPHQL "https://api.example.com/graphql", query

SET HEADER "Authorization", ""

TALK "Logged in as: " + result.data.me.name
```

---

## Common Use Cases

### Fetch User Profile

```basic
' Get detailed user profile
query = '
    query GetProfile($userId: ID!) {
        user(id: $userId) {
            id
            name
            email
            avatar
            settings {
                theme
                language
                notifications
            }
            recentActivity {
                action
                timestamp
            }
        }
    }
'

result = GRAPHQL api_url, query WITH userId = user.id

profile = result.data.user
TALK "Welcome back, " + profile.name + "!"
TALK "Theme: " + profile.settings.theme
```

### Search Products

```basic
' Search with filters
query = '
    query SearchProducts($term: String!, $category: String, $limit: Int) {
        products(search: $term, category: $category, first: $limit) {
            edges {
                node {
                    id
                    name
                    price
                    inStock
                }
            }
            totalCount
        }
    }
'

result = GRAPHQL "https://api.store.com/graphql", query WITH
    term = search_term,
    category = selected_category,
    limit = 10

products = result.data.products.edges
TALK "Found " + result.data.products.totalCount + " products:"

FOR EACH edge IN products
    product = edge.node
    TALK "- " + product.name + ": $" + product.price
NEXT
```

### Create Order

```basic
' Create order mutation
mutation = '
    mutation CreateOrder($input: OrderInput!) {
        createOrder(input: $input) {
            id
            orderNumber
            total
            status
            estimatedDelivery
        }
    }
'

result = GRAPHQL "https://api.store.com/graphql", mutation WITH
    input = '{"customerId": "' + customer_id + '", "items": ' + cart_items + '}'

order = result.data.createOrder
TALK "Order #" + order.orderNumber + " placed!"
TALK "Total: $" + order.total
TALK "Estimated delivery: " + order.estimatedDelivery
```

### Update Record

```basic
' Update mutation
mutation = '
    mutation UpdateUser($id: ID!, $input: UserUpdateInput!) {
        updateUser(id: $id, input: $input) {
            id
            name
            email
            updatedAt
        }
    }
'

result = GRAPHQL api_url, mutation WITH
    id = user.id,
    input = '{"name": "' + new_name + '", "email": "' + new_email + '"}'

TALK "Profile updated!"
```

### Delete Record

```basic
' Delete mutation
mutation = '
    mutation DeleteItem($id: ID!) {
        deleteItem(id: $id) {
            success
            message
        }
    }
'

result = GRAPHQL api_url, mutation WITH id = item_id

IF result.data.deleteItem.success THEN
    TALK "Item deleted successfully"
ELSE
    TALK "Delete failed: " + result.data.deleteItem.message
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = GRAPHQL api_url, query WITH id = resource_id

IF ERROR THEN
    PRINT "GraphQL request failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't fetch that data. Please try again."
ELSE IF result.errors THEN
    ' GraphQL returned errors
    FOR EACH err IN result.errors
        PRINT "GraphQL error: " + err.message
    NEXT
    TALK "The request encountered an error: " + result.errors[0].message
ELSE
    ' Success
    TALK "Data retrieved successfully!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `VALIDATION_ERROR` | Invalid query syntax | Check query format |
| `NOT_FOUND` | Resource doesn't exist | Verify ID/parameters |
| `UNAUTHORIZED` | Missing/invalid auth | Check authentication |
| `FORBIDDEN` | Insufficient permissions | Verify access rights |
| `VARIABLE_REQUIRED` | Missing required variable | Provide all variables |

---

## GraphQL vs REST

| Aspect | GraphQL | REST |
|--------|---------|------|
| **Data fetching** | Request exact fields | Fixed response structure |
| **Multiple resources** | Single request | Multiple requests |
| **Versioning** | Evolving schema | API versions (v1, v2) |
| **Use case** | Complex nested data | Simple CRUD operations |

```basic
' GraphQL - One request for nested data
query = '
    query {
        user(id: "123") {
            name
            orders {
                items {
                    product { name }
                }
            }
        }
    }
'
result = GRAPHQL url, query

' REST equivalent would need multiple calls:
' GET /users/123
' GET /users/123/orders
' GET /orders/{id}/items for each order
' GET /products/{id} for each item
```

---

## Query Building Tips

### Request Only What You Need

```basic
' Good - request specific fields
query = '
    query {
        user(id: "123") {
            name
            email
        }
    }
'

' Avoid - requesting everything
' query {
'     user(id: "123") {
'         id name email phone address avatar settings ...
'     }
' }
```

### Use Fragments for Reusable Fields

```basic
query = '
    fragment UserFields on User {
        id
        name
        email
    }
    
    query {
        user(id: "123") {
            ...UserFields
        }
        users {
            ...UserFields
        }
    }
'
```

---

## Configuration

Configure HTTP settings in `config.csv`:

```csv
name,value
http-timeout,30
http-retry-count,3
```

API keys are stored in Vault:

```bash
vault kv put gbo/graphql/example api_key="your-api-key"
```

---

## Implementation Notes

- Implemented in Rust under `src/web_automation/graphql.rs`
- Sends POST requests with `application/json` content type
- Automatically formats query and variables
- Parses JSON response into accessible objects
- Supports custom headers via SET HEADER
- Handles both queries and mutations

---

## Related Keywords

- [POST](keyword-post.md) — REST POST requests
- [GET](keyword-get.md) — REST GET requests
- [SET HEADER](keyword-set-header.md) — Set authentication headers
- [SOAP](keyword-soap.md) — SOAP/XML web services

---

## Summary

`GRAPHQL` executes queries and mutations against GraphQL APIs. Use it when you need precise control over the data you fetch, especially for nested relationships. GraphQL is more efficient than REST for complex data needs, requiring fewer round trips. Always handle both network errors and GraphQL-specific errors in the response.