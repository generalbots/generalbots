# SET_KB Keyword

**Syntax**

```
SET_KB "kb-name"
```

**Parameters**

- `"kb-name"` – Identifier for a knowledge‑base collection to be associated with the current user.

**Description**

`SET_KB` registers a knowledge‑base (KB) with the user’s session. The keyword validates that the name contains only alphanumeric characters, underscores, or hyphens. It then creates (or ensures the existence of) a vector‑DB collection for the KB and links it to the user in the `user_kb_associations` table. After execution, the KB becomes part of the user’s active knowledge sources and can be queried by `FIND` or used by LLM prompts.

If the KB already exists for the user, the keyword simply confirms the association.

**Example**

```basic
SET_KB "company-policies"
TALK "Knowledge base 'company-policies' is now active."
```

After the command, the `company-policies` collection is available for searches within the current conversation.

**Implementation Notes**

- The operation runs asynchronously in a background thread.
- Errors are logged and returned as runtime errors.
- The keyword always returns `UNIT`.
