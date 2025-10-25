# ON Keyword

**Syntax**

```
ON trigger-type OF "table-name"
```

**Parameters**

- `trigger-type` – The type of database trigger to listen for. Valid values are:
  - `INSERT`
  - `UPDATE`
  - `DELETE`
- `"table-name"` – The name of the database table to monitor.

**Description**

`ON` registers a database trigger for the current session. When the specified event occurs on the given table, the engine records the trigger in the `system_automations` table, linking it to the session. This enables scripts to react to data changes by executing associated actions (e.g., sending a notification, updating a variable).

The keyword performs the following steps:

1. Validates the `trigger-type` and converts it to the internal `TriggerKind` enum.
2. Constructs a parameter name in the form `<table>_<trigger>.rhai` (e.g., `orders_insert.rhai`).
3. Inserts a row into `system_automations` with the trigger kind, target table, and parameter name.
4. Returns the number of rows affected (normally `1` on success).

If the trigger type is invalid, the keyword raises a runtime error.

**Example**

```basic
ON INSERT OF "orders"
TALK "A new order was added. Processing..."
```

After execution, any new row inserted into the `orders` table will cause the session to be notified, allowing the script to handle the event.

**Implementation Notes**

- The keyword runs synchronously but performs the database insertion on a separate thread to avoid blocking.
- Errors during insertion are logged and returned as runtime errors.
