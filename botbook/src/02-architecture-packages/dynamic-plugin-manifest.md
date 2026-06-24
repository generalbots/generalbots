# Dynamic Plugin Manifest Engine 🟡 BETA

> **Runtime-discoverable integration plugins**

## Overview

The Dynamic Plugin Manifest Engine enables bots to dynamically load and execute manifests of native, custom, and partner integration plugins. Plugins are defined as JSON/YAML manifest files stored in MinIO Drive buckets (`{bot}.gbai/{bot}.gbplugins/`) and are hot-reloaded by DriveMonitor on change.

## Architecture

```
MinIO Drive
  └── {bot}.gbai/
      └── {bot}.gbplugins/
          └── plugin-name/
              ├── manifest.json    # PluginManifest definition
              ├── functions/       # Exposed functions (Rhai scripts)
              └── assets/          # Static assets
```

## Plugin Manifest Schema

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `name` | String | Display name |
| `version` | String | SemVer version |
| `base_url` | String | Base URL for API calls |
| `permissions` | Vec\<String\> | Required permission scopes |
| `authentication` | AuthType | API key, OAuth2, or none |
| `functions` | Vec\<ExposedFunction\> | Callable BASIC functions |

## CRUD REST API

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/plugins` | Create plugin manifest |
| GET | `/api/plugins` | List all plugins |
| GET | `/api/plugins/{id}` | Get plugin details |
| PUT | `/api/plugins/{id}` | Update plugin manifest |
| DELETE | `/api/plugins/{id}` | Delete plugin |
| PATCH | `/api/plugins/{id}/permissions` | Grant/revoke permissions |

## Rhai BASIC Integration

Once a plugin manifest is registered, its functions become callable from BASIC scripts. The runtime:

1. Resolves the function name against registered plugin manifests
2. Fetches the API key from Vault via `botcoresecrets`
3. Injects authorization headers
4. Executes the HTTP request via `reqwest`
5. Returns the response to the BASIC script

## Security

- API keys are fetched from Vault at runtime, never stored in manifests
- Permissions are granular per-plugin per-session
- Rate limiting via `governor` crate per-plugin
- Network errors return `PluginError`, never panic

## Configuration

Enabled automatically when the `designer` feature flag is active. No additional configuration required.
