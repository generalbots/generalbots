# SET_CONTEXT Keyword

**Syntax**

```
SET_CONTEXT "context-string"
```

**Parameters**

- `"context-string"` – Arbitrary text that will be stored as the session’s context.

**Description**

`SET_CONTEXT` saves a string in the session’s Redis cache (if configured). It can be retrieved later with `GET_CONTEXT` (not a separate keyword; the engine reads the stored value automatically). This is useful for persisting short pieces of information across multiple dialog turns.

**Example**

```basic
SET_CONTEXT "order_id=12345"
TALK "Your order ID has been saved."
```

Later in the script you could retrieve it via a custom function or by accessing the Redis key directly.
