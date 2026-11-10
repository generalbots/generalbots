# Complete Keyword Reference 🟡 BETA

The keyword and function reference is **generated** from the runtime
registrations — do not maintain a table here:

- [Generated BASIC reference](./generated/index.md) — counts, status legend,
  categories, and the list of names that are advertised but not registered.
- One page per category under `generated/`: `dialog`, `language`, `control`,
  `data`, `kb`, `http`, `files`, `ai`, `agent`, `industry`, `media`, `messaging`,
  `social`, `system`, `botserver`.
- [Grammar summary](./generated/grammar-summary.md) — block constructs,
  statement forms by first token, first-token collisions, directives, rewrites.

```bash
python3 scripts/gen_basic_reference.py --report   # inventory
python3 scripts/gen_basic_reference.py            # regenerate the pages
python3 scripts/gen_basic_reference.py --check    # fail on drift
```

A row may claim `Implemented` only when a registration exists:
`register_custom_syntax`, `register_fn`, a compiler directive, or a rewrite in
`convert_multiword_keywords.rs`. `get_all_keywords()` is a list of names, not a
list of features — `ADD TOOL`, `USE MCP`, `BOT REFLECTION`, `SECURITY …`,
`PLAN START` and others appear there with nothing behind them, and the generated
index lists every one of those.

---

## Input Types for HEAR

| Type | Example | Description |
|------|---------|-------------|
| `STRING` | `HEAR name AS STRING` | Any text input |
| `INTEGER` | `HEAR age AS INTEGER` | Whole number |
| `NUMBER` | `HEAR price AS NUMBER` | Decimal number |
| `BOOLEAN` | `HEAR confirm AS BOOLEAN` | Yes/No, True/False |
| `DATE` | `HEAR birthday AS DATE` | Date value |
| `EMAIL` | `HEAR contact AS EMAIL` | Valid email address |
| `MOBILE` | `HEAR phone AS MOBILE` | Phone number |
| `MONEY` | `HEAR amount AS MONEY` | Currency amount |
| `HOUR` | `HEAR time AS HOUR` | Time value |
| `ZIPCODE` | `HEAR zip AS ZIPCODE` | Postal code |
| `LANGUAGE` | `HEAR lang AS LANGUAGE` | Language code |
| `FILE` | `HEAR doc AS FILE` | File upload |
| `AUDIO` | `HEAR recording AS AUDIO` | Audio upload |
| `IMAGE` | `HEAR photo AS IMAGE` | Image upload |
| `QRCODE` | `HEAR code AS QRCODE` | QR code from image |
| `LOGIN` | `HEAR user AS LOGIN` | Authenticated user |
| Menu | `HEAR choice AS "A", "B", "C"` | Selection from options |

---

## FORMAT Patterns

| Pattern | Example | Result |
|---------|---------|--------|
| `YYYY` | `FORMAT date AS "YYYY"` | 2025 |
| `MM` | `FORMAT date AS "MM"` | 01 |
| `DD` | `FORMAT date AS "DD"` | 15 |
| `YYYY-MM-DD` | `FORMAT date AS "YYYY-MM-DD"` | 2025-01-15 |
| `HH:mm:ss` | `FORMAT time AS "HH:mm:ss"` | 14:30:00 |
| `#,##0` | `FORMAT number AS "#,##0"` | 1,234 |
| `#,##0.00` | `FORMAT number AS "#,##0.00"` | 1,234.56 |
| `0%` | `FORMAT decimal AS "0%"` | 75% |

---

## AGGREGATE Operations

| Operation | Example | Description |
|-----------|---------|-------------|
| `SUM` | `AGGREGATE "SUM", data, "amount"` | Sum of values |
| `AVG` | `AGGREGATE "AVG", data, "amount"` | Average |
| `COUNT` | `AGGREGATE "COUNT", data, "id"` | Count records |
| `MIN` | `AGGREGATE "MIN", data, "price"` | Minimum value |
| `MAX` | `AGGREGATE "MAX", data, "price"` | Maximum value |

---

## Schedule Patterns (Cron)

| Pattern | Description |
|---------|-------------|
| `0 8 * * *` | Every day at 8:00 AM |
| `0 9 * * 1` | Every Monday at 9:00 AM |
| `0 0 1 * *` | First day of every month |
| `*/15 * * * *` | Every 15 minutes |
| `0 8,17 * * *` | At 8:00 AM and 5:00 PM |
| `0 8 * * 1-5` | Weekdays at 8:00 AM |

---

## Best Practices

### DO ✅

```basic
' Use descriptive variable names
let customername = "John Smith"
let orderamount = 150.00

' Use SAVE with variable names as field names
SAVE "orders.csv", customername, orderamount, orderdate

' Use spaces in keywords
SET BOT MEMORY "last_order", orderid
SET CONTEXT "assistant" AS "You are helpful."
ADD SUGGESTION "help" AS "Get Help"
CLEAR SUGGESTIONS
USE KB "products"
USE TOOL "search"

' Use HEAR AS for type validation
HEAR email AS EMAIL
HEAR amount AS MONEY
HEAR confirm AS BOOLEAN
```

### DON'T ❌

```basic
' Don't use underscores in keywords (old syntax)
SET_BOT_MEMORY "key", value  ' WRONG - use spaces

' Don't use complex object operations
SET object.field = value  ' WRONG
SAVE "table", object.id, object  ' WRONG

' Don't skip input validation
HEAR value  ' OK but less safe
' Better: HEAR value AS STRING
```

---

## Migration from Old Syntax

| Old Syntax | New Syntax |
|------------|------------|
| `SET_BOT_MEMORY` | `SET BOT MEMORY` |
| `GET_BOT_MEMORY()` | `GET BOT MEMORY()` |
| `SET_CONTEXT` | `SET CONTEXT` |
| `ADD_SUGGESTION` | `ADD SUGGESTION` |
| `CLEAR_SUGGESTIONS` | `CLEAR SUGGESTIONS` |
| `USE_KB` | `USE KB` |
| `USE_TOOL` | `USE TOOL` |
| `CLEAR_KB` | `CLEAR KB` |
| `CLEAR_TOOLS` | `CLEAR TOOLS` |
| `SET_SCHEDULE` | `SET SCHEDULE` |
| `SET_HEADER` | `SET HEADER` |
| `CLEAR_HEADERS` | `CLEAR HEADERS` |
| `DELETE_HTTP` | `DELETE HTTP` |
| `DELETE_FILE` | `DELETE FILE` |
| `GENERATE_PDF` | `GENERATE PDF` |
| `MERGE_PDF` | `MERGE PDF` |
| `GROUP_BY` | `GROUP BY` |
| `SAVE_FROM_UNSTRUCTURED` | `SAVE FROM UNSTRUCTURED` |

---

*Last updated: January 2025*