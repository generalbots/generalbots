# Collaboration API (comments, reactions, presence)

> **Cross-app collaboration layer** — threaded comments with `@`-mentions and
> emoji reactions, plus live presence (who is viewing / typing), attachable to
> **any** resource. Resources are addressed generically:

| `resource_type` | `resource_id` |
|-----------------|---------------|
| `drive:file` | `{bucket}:{path}` (e.g. `default.gbai:docs/plan.md`) |
| `sheet` | Sheet document id |
| `doc` | Document id |
| `task` | Task id |
| `calendar` | Event id |

All endpoints require a JWT bearer token, except where noted.

## Comments

### List (threaded)

**`GET /api/collab/comments?resource_type=drive:file&resource_id={id}`**

Top-level comments come back with inline `replies`. Soft-deleted rows are excluded.

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "resource_type": "drive:file",
    "resource_id": "default.gbai:docs/plan.md",
    "author_id": "maria@example.com",
    "author_name": "maria",
    "parent_id": null,
    "body": "Please review section 2 @joao",
    "mentions": ["joao"],
    "created_at": "2026-08-15T12:00:00Z",
    "updated_at": "2026-08-15T12:00:00Z",
    "reactions": [{ "emoji": "👍", "user_id": "joao@example.com" }],
    "replies": []
  }
]
```

### Create

**`POST /api/collab/comments`**

```json
{
  "resource_type": "drive:file",
  "resource_id": "default.gbai:docs/plan.md",
  "body": "Please review section 2 @joao",
  "parent_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

- `parent_id` is optional — set it to reply to another comment.
- `@mention` tokens are extracted and stored in `mentions` (for future
  notification rendering).

### Delete

**`DELETE /api/collab/comments/{id}`** — soft-delete. Author or admin only;
returns 404 for other users.

## Reactions

**`POST /api/collab/comments/{id}/reactions`** — toggle an emoji.

```json
{ "emoji": "👍" }
```

Returns `{ "success": true, "added": true|false }` (`added=false` means the
reaction was removed).

## Presence

### Heartbeat

**`POST /api/collab/presence`** — call on open and every ~30s; set
`typing: true` while the user is typing in the resource's input.

```json
{
  "resource_type": "drive:file",
  "resource_id": "default.gbai:docs/plan.md",
  "typing": true
}
```

### Who is active

**`GET /api/collab/presence?resource_type=drive:file&resource_id={id}`**

Returns users with a heartbeat within the last 60 seconds, typing users first:

```json
[
  { "user_id": "maria@example.com", "user_name": "maria", "typing": true, "last_seen": "2026-08-15T12:01:00Z" }
]
```

## Security notes

- All mutating endpoints are authenticated; the anonymous allowlist is not
  touched — nothing under `/api/collab` is public.
- `resource_type`/`resource_id` are length- and charset-validated; comment
  bodies are capped at 8000 chars.
- Deletion is author-scoped (or admin), reactions are per-user toggles.
