# Adaptive Automations

Two kinds of schedules coexist:

1. **Classic cron scripts** (`.gbdialog/schedulers/*.bas`) run unchanged.
2. **Agent schedules** store a natural-language goal. At fire time the engine
   plans against current data, executes tool steps, verifies results against
   the original objective (bounded repair loop), merges parallel forks and
   reports. Runs are visible live in **Automations** with per-step checklists.

## Delivery loop

Completed runs notify their owner through email and/or SMS by default
(configurable per schedule), including failure notifications, with retries and
delivery status persisted on the run row.

## API

`GET|POST /api/automations/schedules` · `PUT|DELETE .../{id}` ·
`POST .../{id}/run` · `GET /api/automations/runs` ·
`POST /api/automations/runs/{id}/cancel` ·
`GET /api/automations/runs/{id}/events` (SSE; Bearer header or `?token=`).
