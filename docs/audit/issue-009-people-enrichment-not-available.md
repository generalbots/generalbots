# Issue #009: PEOPLE — Enrichment feature explicitly unavailable

**Severity:** LOW
**Components:** `botui/ui/suite/people/people.js` (line 364)
**Type:** Ghost functionality

## Description

The People app has an "Enrichment" feature in the UI that explicitly states it's not implemented:

```javascript
// people.js:364
alert('Enrichment is not available in this build.');
```

This feature appears to be a button/action in the contact detail view that would auto-enrich contact data from external sources (LinkedIn, company databases, etc.).

## Impact

- Users see a feature button that always shows a "not available" alert.
- Poor user experience — the feature should either be implemented or hidden.

## Suggested Fix

1. **Implement** the enrichment API (calls to external data providers like Clearbit, Hunter, etc.) OR
2. **Remove** the enrichment button from the UI if not planned.
3. If the feature is planned but delayed, hide the button behind a feature flag rather than showing an alert.
