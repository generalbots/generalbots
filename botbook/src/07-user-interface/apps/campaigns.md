# Campaigns - Marketing Automation


## Sending a Campaign

Click **Send** on any campaign card to dispatch it immediately via `POST /api/crm/campaigns/:id/send`.

Campaign metrics (opens, clicks, unsubscribes) load via `GET /api/crm/metrics/campaign/:id`.

## Enabling Campaigns

Add `campaigns` to `apps=` in `botserver/.product`:

```
apps=...,campaigns
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/crm/campaigns` | GET | List campaigns |
| `/api/crm/campaigns` | POST | Create campaign |
| `/api/crm/campaigns/:id/send` | POST | Send campaign |
| `/api/crm/metrics/campaign/:id` | GET | Campaign metrics |
| `/api/crm/lists` | GET | Marketing lists |
| `/api/crm/templates` | GET | Email templates |
