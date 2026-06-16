# Issue #012: DRIVE SENTIENT — Complete mock with no real API

**Severity:** LOW (prototype)
**Components:** `botui/ui/suite/drive/drive-sentient.html`, `drive-sentient.js`
**Type:** Ghost functionality / Prototype

## Description

`drive-sentient.html` and `drive-sentient.js` are a **visual prototype** with zero real API calls. All data is mocked in-memory:

- `uploadFile()` — creates a fake upload progress bar locally
- `simulateUpload()` — timer-based fake progress
- `processAIQuery()` — returns hardcoded AI responses
- `initAppLauncher()` — navigates to other apps via `window.location.href`

The file only makes one external request: a Google Fonts CDN stylesheet (violating the "NO CDN" rule from AGENTS.md).

## Impact

- This file appears to be a production app in the suite but has no backend integration.
- New developers may waste time trying to understand or fix it.
- The CDN reference violates AGENTS.md rule: "NEVER use CDN links — all assets must be local."

## Suggested Fix

1. **Move** to a `prototypes/` directory outside the main suite.
2. **Remove** the CDN stylesheet link (replace with local font).
3. OR **Implement** actual API integration to make it functional.
4. **Add** a clear comment at the top: `⚠ PROTOTYPE — No backend integration`.
