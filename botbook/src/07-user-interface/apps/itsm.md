# ITSM → Tickets (unified)

> **IT service management folded into the Tickets app**

The former standalone **ITSM** app has been **unified into Tickets**. There is no separate
ITSM app in the launcher or catalog anymore — all IT service management capabilities are
available inside the **Tickets** app under the CMDB / KB / Problems / Changes tabs.

See [Tickets](./tickets.md) for the full documentation.

## What was unified

| ITSM concept | Now in Tickets |
|--------------|----------------|
| Incidents & service requests | `support_tickets` with `record_type` (`ticket` \| `problem` \| `change`) |
| CMDB (configuration items) | `ticket_cis` — `/api/tickets/cis` CRUD |
| Knowledge base articles | `ticket_kb_articles` — `/api/tickets/kb` CRUD |
| Problems | Tickets with `record_type = 'problem'` |
| Changes | Tickets with `record_type = 'change'` |

## Migration

Migration `6.5.33-tickets-itsm-unification` added `record_type` to `support_tickets`,
plus the `ticket_cis` and `ticket_kb_articles` tables. The dead in-memory ITSM duplicate
(`botattendant/src/routes_itsm.rs`, `incident.rs`, `cmdb.rs`, `knowledge.rs`) was removed.

## Related

- [Tickets](./tickets.md)
- [ITSM API](../08-rest-api-tools/tickets-api.md)
