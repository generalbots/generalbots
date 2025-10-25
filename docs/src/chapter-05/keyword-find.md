# FIND Keyword

**Syntax**

```
FIND "table-name", "filter-expression"
```

**Parameters**

- `"table-name"` – The name of the database table to query.
- `"filter-expression"` – A simple `column=value` expression used to filter rows.

**Description**

`FIND` executes a read‑only query against the configured PostgreSQL database. It builds a SQL statement of the form:

```sql
SELECT * FROM table-name WHERE filter-expression LIMIT 10
```

The keyword returns an array of dynamic objects representing the matching rows. The result can be used directly in BASIC scripts or passed to other keywords (e.g., `TALK`, `FORMAT`). Errors during query execution are logged and returned as runtime errors.

**Example**

```basic
SET results = FIND "customers", "country=US"
TALK "Found " + LENGTH(results) + " US customers."
```

The script retrieves up to ten rows from the `customers` table where the `country` column equals `US` and stores them in `results`. The `LENGTH` function (provided by the BASIC runtime) can then be used to count the rows.

**Implementation Notes**

- The filter expression is parsed by `utils::parse_filter` and bound safely to prevent SQL injection.
- Only a limited subset of SQL is supported (simple equality filters). Complex queries should be performed via custom tools or the `GET` keyword.
- The keyword runs synchronously within the script but performs the database call on a separate thread to avoid blocking the engine.
