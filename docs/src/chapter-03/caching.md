# Caching (Optional)

Caching can improve response times for frequently accessed knowledge‑base queries.

## In‑Memory Cache

The bot maintains an LRU (least‑recently‑used) cache of the last 100 `FIND` results. This cache is stored in the bot’s process memory and cleared on restart.

## Persistent Cache

For longer‑term caching, the `gbkb` package can write query results to a local SQLite file (`cache.db`). The cache key is a hash of the query string and collection name.

## Configuration

Add the following to `.gbot/config.csv`:

```csv
key,value
cache_enabled,true
cache_max_entries,500
```

## Usage Example

```basic
SET_KB "company-policies"
FIND "vacation policy" INTO RESULT   ' first call hits Qdrant
FIND "vacation policy" INTO RESULT   ' second call hits cache
TALK RESULT
```

The second call returns instantly from the cache.

## Cache Invalidation

- When a document is added or updated, the cache for that collection is cleared.
- Manual invalidation: `CLEAR_CACHE "company-policies"` (custom keyword provided by the system).

## Benefits

- Reduces latency for hot queries.
- Lowers load on Qdrant.
- Transparent to the script author; caching is automatic.
