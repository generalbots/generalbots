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

Once a signed plugin manifest is registered, its functions become callable from BASIC scripts. The runtime:

1. Resolves the function name against the installed plugin registry
2. Resolves auth fields from Vault at `auth_vault_path` via `botcoresecrets`
3. Injects authorization headers
4. Executes the HTTPS request via `reqwest` with redirects disabled
5. Returns the response to the BASIC script

## Security

The registry is initialized empty at boot. A manifest must pass every gate before `PLUGIN_CALL` can reach it.

- Manifests require an HMAC-SHA256 signature over their canonical JSON. The key is a server-side secret configured through `GB_PLUGIN_SIGNING_KEY`; an unset key rejects every manifest.
- `base_url` must use HTTPS and its host must appear in `GB_PLUGIN_ALLOWED_HOSTS`.
- Credentials are resolved at call time from Vault at `auth_vault_path` (fields `key`, `token`, `username`, `password`); no manifest field holds a secret value.
- `PLUGIN_CALL` additionally requires the `plugin:call` permission, a `bot_id` matching the calling session, and a manifest path contained in the work directory (`GB_WORK_PATH` / `WORK_PATH`).
- Network errors return `PluginError`, never panic.

## Configuration

Enabled automatically when the `designer` feature flag is active. No additional configuration required.
