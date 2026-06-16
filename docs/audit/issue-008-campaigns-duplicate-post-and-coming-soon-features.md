# Issue #008: CAMPAIGNS — Duplicate POST handler + coming-soon features

**Severity:** LOW
**Components:** `botui/ui/suite/campaigns/`
**Type:** Code quality / Ghost functionality

## Description

### 1. Duplicate POST to `/api/crm/campaigns`
The campaign creation form has **two competing mechanisms** that both POST to `/api/crm/campaigns`:
- **HTMX:** `hx-post="/api/crm/campaigns"` on the `<form>` element (campaigns.html:98)
- **JavaScript:** `CampaignsAPI.create()` via `fetch()` (campaigns.js), which intercepts the form submit with `e.preventDefault()`

Since JS uses `preventDefault()`, the HTMX `hx-post` is **dead code** — it will never fire. Or conversely, if HTMX fires first, the JS handler may be skipped.

### 2. Coming-soon features
The `showListModal()` and `showTemplateModal()` functions show:
```javascript
alert('Coming soon!')
```

These features (contact lists and campaign templates) have UI triggers and modal stubs but no actual implementation.

## Impact

- One of the two POST mechanisms is inactive — dead code.
- The "Coming soon" alerts are a poor user experience.
- Users see buttons that don't work.

## Suggested Fix

1. **Remove** the HTMX `hx-post` from the form (since JS handles it), or remove the JS handler and use HTMX exclusively.
2. **Either implement** lists and templates CRUD, or **remove** the buttons/triggers from the UI.
3. If keeping "coming soon", replace `alert()` with a proper styled modal/banner.
