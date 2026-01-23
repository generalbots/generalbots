# App Launcher Integration Guide

## Overview

The `apps-manifest.json` file provides a complete mapping between Cargo.toml features and user-friendly app descriptions for the botui app launcher.

## File Location

```
botserver/apps-manifest.json
```

## Structure

### Categories

Apps are organized into 8 categories:

1. **Communication** (💬) - Chat, Mail, Meet, WhatsApp, Telegram, etc.
2. **Productivity** (⚡) - Tasks, Calendar, Project, Goals, Workspaces, etc.
3. **Documents** (📄) - Drive, Docs, Sheet, Slides, Paper
4. **Media** (🎬) - Video, Player, Canvas
5. **Learning** (📚) - Learn, Research, Sources
6. **Analytics** (📈) - Analytics, Dashboards, Monitoring
7. **Development** (⚙️) - Automation, Designer, Editor
8. **Administration** (🔐) - Attendant, Security, Settings, Directory
9. **Core** (🏗️) - Cache, LLM, Vector DB

### App Schema

Each app includes:

```json
{
  "id": "tasks",
  "name": "Tasks",
  "description": "Task management with scheduling",
  "feature": "tasks",
  "icon": "✅",
  "enabled_by_default": true,
  "dependencies": ["automation", "drive", "monitoring"]
}
```

### Bundles

Pre-configured feature sets:

- **minimal** - Essential infrastructure (chat, automation, drive, cache)
- **lightweight** - Basic productivity (chat, drive, tasks, people)
- **full** - Complete feature set
- **communications** - All communication apps
- **productivity** - Productivity suite
- **documents** - Document suite

## Integration with botui

### Reading the Manifest

```javascript
// In botui/ui/suite/js/app-launcher.js
fetch('/api/apps/manifest')
  .then(res => res.json())
  .then(manifest => {
    renderAppLauncher(manifest);
  });
```

### Rendering Apps

```javascript
function renderAppLauncher(manifest) {
  const categories = manifest.categories;
  
  for (const [categoryId, category] of Object.entries(categories)) {
    const categoryEl = createCategory(category);
    
    category.apps.forEach(app => {
      const appCard = createAppCard(app);
      categoryEl.appendChild(appCard);
    });
  }
}
```

### App Card Template

```html
<div class="app-card" data-feature="${app.feature}">
  <div class="app-icon">${app.icon}</div>
  <div class="app-name">${app.name}</div>
  <div class="app-description">${app.description}</div>
  <div class="app-toggle">
    <input type="checkbox" 
           ${app.enabled_by_default ? 'checked' : ''}
           ${app.core_dependency ? 'disabled' : ''}>
  </div>
  ${app.dependencies.length > 0 ? 
    `<div class="app-deps">Requires: ${app.dependencies.join(', ')}</div>` 
    : ''}
</div>
```

## Backend API Endpoint

Add to `botserver/src/main.rs`:

```rust
async fn get_apps_manifest() -> Json<serde_json::Value> {
    let manifest = include_str!("../apps-manifest.json");
    let value: serde_json::Value = serde_json::from_str(manifest)
        .expect("Invalid apps-manifest.json");
    Json(value)
}

// In router configuration:
api_router = api_router.route("/api/apps/manifest", get(get_apps_manifest));
```

## Compilation Testing

Use the `test_apps.sh` script to verify all apps compile:

```bash
cd /home/rodriguez/src/gb
./test_apps.sh
```

This will:
1. Test each app feature individually
2. Report which apps pass/fail compilation
3. Provide a summary of results

## Core Dependencies

These apps cannot be disabled (marked with `core_dependency: true`):

- **automation** - Required for .gbot script execution
- **drive** - S3 storage used throughout
- **cache** - Redis integrated into sessions

## Feature Bundling

When a user enables an app, all its dependencies are automatically enabled:

- Enable `tasks` → Automatically enables `automation`, `drive`, `monitoring`
- Enable `mail` → Automatically enables `mail_core`, `drive`
- Enable `research` → Automatically enables `llm`, `vectordb`

## Syncing with Cargo.toml

When adding new features to `Cargo.toml`:

1. Add the feature definition in `Cargo.toml`
2. Add the app entry in `apps-manifest.json`
3. Update the app launcher UI in botui
4. Run `./test_apps.sh` to verify compilation
5. Commit both files together

## Example: Adding a New App

### 1. In Cargo.toml

```toml
[features]
myapp = ["dep:myapp-crate", "drive"]
```

### 2. In apps-manifest.json

```json
{
  "id": "myapp",
  "name": "My App",
  "description": "My awesome app",
  "feature": "myapp",
  "icon": "🚀",
  "enabled_by_default": false,
  "dependencies": ["drive"]
}
```

### 3. Test

```bash
cargo check -p botserver --no-default-features --features myapp
```

## Notes

- Icons use emoji for cross-platform compatibility
- Dependencies are automatically resolved by Cargo
- Core dependencies are shown but cannot be toggled off
- The manifest version matches botserver version
