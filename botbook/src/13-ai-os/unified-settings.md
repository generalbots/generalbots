# Unified Settings App

## Overview

The suite desktop historically exposed settings through several overlapping
windows: separate `admin` (administration) and `settings` (personal
preferences) entries, plus orphan `.html` pages reachable only by direct URL.
Issue #1295 consolidated them into a **single Settings app** with two views
controlled by a `?view=` query parameter.

## Architecture

| View | Route | Purpose |
|---|---|---|
| Personal settings | `/suite/settings/index.html` | Theme, profile, wallpapers, keyboard, accessibility |
| Administration | `/suite/settings/index.html?view=admin` | RBAC groups, users, orgs, security, integrations |

Both registry entries (`admin` and `settings` in `window-manager.js`
`APPS_REGISTRY`) point at the unified page — `admin` passes `?view=admin`.
The orphan pages removed in the consolidation:

| Removed | Replaced by |
|---|---|
| `admin/index.html` | redirect stub → unified Settings |
| `settings/security.html` | settings view (security section) |
| `settings/profile.html` | settings view (profile section) |
| `settings/wallpapers.html` | settings view (wallpapers section) |

## Wallpaper picker

The unified Settings page embeds the wallpaper gallery with a live preview.
The picker reads wallpapers from `desktop.css` (`.gb-wallpaper-*` classes);
each entry renders a thumbnail (gradient CSS) with an Apply action that sets
the active wallpaper class on the desktop root and persists the choice in
`localStorage` (`gb-wallpaper`). The desktop shell loads `wallpaper.css`
unconditionally so picker styles work in every theme.

## RBAC gating

Administration sections are rendered only when the session carries an
admin role (`data-admin-only` attributes; the renderer checks the resolved
user role from the suite token). Non-admin users see only personal settings,
and the admin view falls back to the personal view when access is denied.

## Operating System panel (shell mode only)

Issue #1852 adds a low-level **Operating System** section for deployments
that are the whole OS: the Tauri desktop app, the desktop-local build and the
General Bots OS kiosk image. It never appears in a normal browser/cloud
deployment.

| Piece | Location |
|---|---|
| Mode detection | `settings/main/02_edition.js` — `isShellMode(config)` |
| Nav + section | `settings/index.html`, `settings/partials/07_os.html` |
| Controller | `settings/settings-os.js` |
| Product flag | `shell_mode=true` in `.product.desktop-local` (`/api/product`) |
| API | `GET /api/system/os`, `GET /api/system/os/logs`, `POST /api/system/os/services/restart` |

The minimal panel shows hostname, OS/kernel/architecture, uptime, timezone,
service status, disk usage and a log tail. Hostname and timezone are
read-only in this phase. **Restart** is the only mutating tool: it requires a
typed service-name confirmation and writes an `os_service_restart` entry to
the audit log. New OS endpoints enforce shell mode, administrator access and
a loopback host; hiding the panel in the browser is not the security gate.

### OS appliance pages (#2466, OS edition)

The OS edition navigation now includes appliance pages for **Network**, **Power & Battery**, **Display**, **Sound**, **Keyboard & Region**, **Storage & Reset**, **Users**, and **Diagnostics**. These nodes are registered in `settings/nav-registry.js` as `editions: ["os"]`, `scope: "admin"`, `control: "shell"` and are hidden in cloud/desktop editions.

Each page renders a read-only status section that explains what is unavailable and why; it attempts the corresponding `/api/os/*` endpoint and degrades gracefully to an explicit unavailable/read-only state when the backend is not present (no localStorage stubs, no destructive actions). **Power & Battery** also queries `GET /api/system/power/status` (and `/api/system/os/power/status`) when available and renders live battery/AC telemetry (presence, charge, state, AC online, estimated remaining time, detected sources) read-only; low-battery tray/ARIA notifications are emitted at 20%, 10%, and 5%, while lid policy and configurable critical actions remain backend/host follow-up work (#2445). Real gated backend routes for the remaining appliance pages (shell mode + admin + loopback, sanitized errors) remain follow-up work. Labels are localized through `settings-os-appliance.ftl` in all 8 supported locales.

## Verification

- Desktop launcher → Settings opens the unified page.
- Launcher → Settings with admin session shows the administration sections.
- Direct navigation to the old orphan URLs redirects to the unified page
  (no 404, no broken shell).
- Wallpaper Apply updates the desktop background immediately and survives
  reload.