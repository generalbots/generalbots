# Role-Based Tool Access 🟡 BETA

The BASIC runtime exposes a `role` variable so bot scripts can restrict features and tools to administrators. The bot itself can remain public (anonymous users may chat and use public tools), while management/administrative tools require an admin role.

## Runtime `role` Variable

Every BASIC script (including `start.bas`) has access to `role`, which resolves to:

| Value | Condition |
|-------|-----------|
| `"admin"` | The session user belongs to an RBAC group whose name contains `admin` (e.g. group `admin`, `administrators`) |
| `"user"` | Otherwise (anonymous visitors, regular users) |

The role is resolved from `rbac_user_groups` joined to `rbac_groups` using the session's `user_id`.

### Web

When the visitor is logged in, the suite session token (`gb-access-token`) is resolved to the real user and their role. Anonymous visitors get `role = "user"`.

### WhatsApp

WhatsApp derives a deterministic `user_id` per phone number (`UUIDv5(NAMESPACE_DNS, "wa:{phone}")`). To grant admin to a WhatsApp number, add that deterministic UUID to the `admin` group:

```sql
-- Compute the user_id for a phone number (e.g. 5521972102162):
-- SELECT gen_random_uuid() FROM ... (or use a UUID v5 generator)

INSERT INTO rbac_groups (id, name, display_name, description, is_active)
VALUES (gen_random_uuid(), 'admin', 'Administrators', 'Bot administration group', true);

INSERT INTO rbac_user_groups (id, user_id, group_id, added_at)
VALUES (gen_random_uuid(), '<uuid_v5_of_wa_phone>', '<admin_group_id>', NOW());
```

## Gating Tools by Role

### In `start.bas` (recommended)

Only associate and expose admin tools to administrators:

```basic
IF role = "admin" THEN
    USE TOOL "chart-batizados"
    USE TOOL "chart-mode"
    USE TOOL "pendencias"
    USE TOOL "revisar-pendencias"
END IF

IF role = "admin" THEN
    ADD SUGGESTION TOOL "chart-batizados" as "Grafico Batizados"
    ADD SUGGESTION TEXT "O que tenho pendente para aprovar" as "Revisar Pendencias"
END IF
```

This hides private suggestions from non-admins and prevents the tools from being associated with non-admin sessions.

### Marking Tools Admin-Only (defense in depth)

Add `"admin_only": true` to the tool's `.mcp.json`:

```json
{
  "name": "chart-batizados",
  "description": "CONSULTA DE DADOS e ESTATISTICAS de batizados",
  "input_schema": {
    "type": "object",
    "properties": {},
    "required": []
  },
  "admin_only": true
}
```

Even if a non-admin session somehow obtains the tool name, the execution gate blocks it:
- `run_llm_tool_call` — blocks admin-only tools for non-admin users
- `run_tool_exec` (TOOL_EXEC / message_type 6) — same check

Tools whose name ends in `_update` are admin-only by convention (no `admin_only` flag required).

## Behavior Summary

| Scenario | Role | Admin tools associated? | Private suggestions visible? | Admin tool execution |
|----------|------|-------------------------|------------------------------|----------------------|
| Anonymous web visitor | `user` | No | No | Blocked |
| Logged-in non-admin | `user` | No | No | Blocked |
| Admin (web login or WhatsApp admin number) | `admin` | Yes | Yes | Allowed |

## Related

- [USE TOOL](./keyword-use-tool.md)
- [RBAC Overview](./rbac-overview.md)
- [RBAC Configuration](./rbac-configuration.md)
