# SYNCHRONIZE

Synchronizes data from an external API endpoint to a local database table with automatic pagination.

## Status

⚠️ **Planned Feature** - This keyword is documented for the Bling ERP integration template but implementation is pending.

## Syntax

```basic
SYNCHRONIZE endpoint, tableName, keyField, pageParam, limitParam
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `endpoint` | String | API endpoint path (appended to `host` variable) |
| `tableName` | String | Target table name (with optional connection prefix) |
| `keyField` | String | Primary key field for upsert operations |
| `pageParam` | String | Name of the pagination parameter in API |
| `limitParam` | String | Name of the limit parameter in API |

## Description

`SYNCHRONIZE` provides a high-level abstraction for syncing paginated API data to a database table. It:

1. Iterates through all pages of the API endpoint
2. Fetches data using the configured `host`, `limit`, and `pages` variables
3. Performs upsert (merge) operations on the target table
4. Tracks statistics in the `REPORT` variable
5. Handles rate limiting automatically

## Prerequisites

The following variables must be defined (typically via `config.csv` param-* entries):

```csv
name,value
param-host,https://api.example.com/v1
param-limit,100
param-pages,50
```

## Example

```basic
' Sync categories from ERP
pageVariable = "pagina"
limitVariable = "limite"

SEND EMAIL admin, "Syncing categories..."
SYNCHRONIZE /categorias/receitas-despesas, maria.CategoriaReceita, Id, pageVariable, limitVariable
SEND EMAIL admin, REPORT
RESET REPORT

' Sync payment methods
SYNCHRONIZE /formas-pagamentos, maria.FormaDePagamento, Id, pageVariable, limitVariable
SEND EMAIL admin, REPORT
RESET REPORT
```

## Equivalent Manual Implementation

Until `SYNCHRONIZE` is implemented, use this pattern:

```basic
' Manual sync equivalent
pageVariable = "pagina"
limitVariable = "limite"
tableName = "maria.CategoriaReceita"
endpoint = "/categorias/receitas-despesas"

page = 1
totalSynced = 0

DO WHILE page > 0 AND page <= pages
    url = host + endpoint + "?" + pageVariable + "=" + page + "&" + limitVariable + "=" + limit
    res = GET url
    WAIT 0.33  ' Rate limiting
    
    IF res.data AND UBOUND(res.data) > 0 THEN
        MERGE tableName WITH res.data BY "Id"
        totalSynced = totalSynced + UBOUND(res.data)
        page = page + 1
        
        IF UBOUND(res.data) < limit THEN
            page = 0  ' Last page
        END IF
    ELSE
        page = 0  ' No more data
    END IF
LOOP

TALK "Synced " + totalSynced + " records to " + tableName
```

## Used In

- [Bling ERP Template](../../../templates/bling.gbai/) - ERP synchronization scripts

## Related Keywords

- [GET](./keyword-get.md) - HTTP GET requests
- [MERGE](./keyword-merge.md) - Upsert data operations
- [SET SCHEDULE](./keyword-set-schedule.md) - Schedule sync jobs
- [REPORT / RESET REPORT](./keyword-report.md) - Sync statistics

## Implementation Notes

When implemented, `SYNCHRONIZE` should:

1. Use the global `host`, `limit`, `pages` variables from config
2. Support connection prefixes (e.g., `maria.TableName`)
3. Handle API errors gracefully with retry logic
4. Update the `REPORT` variable with sync statistics
5. Support both REST JSON responses and paginated arrays

## See Also

- [Script Execution Flow](./script-execution-flow.md) - How config variables are injected
- [Data Operations](./keywords-data.md) - Data manipulation keywords