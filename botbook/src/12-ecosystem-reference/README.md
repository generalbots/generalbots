# Chapter 12: Ecosystem & Reference

General Bots supports full white-label customization, allowing you to rebrand the entire platform.

## Overview

White-labeling allows you to:

- Replace "General Bots" branding with your own product name
- Enable/disable specific apps in the suite
- Set a default theme for all users
- Customize logos, colors, and other visual elements
- Control which APIs are available based on enabled apps

## Configuration File

The white-label settings are defined in the `.product` file located in the root of the botserver directory.

### File Location

```
botserver/
├── .product          # White-label configuration
├── src/
├── Cargo.toml
└── ...
```

### Configuration Format

The `.product` file uses a simple `key=value` format:

```ini
# Product Configuration File
# Lines starting with # are comments

# Product name (replaces "General Bots" throughout the application)
name=My Custom Platform

# Active apps (comma-separated list)
apps=chat,drive,tasks,calendar

# Default theme
theme=sentient

# Optional customizations
logo=/static/my-logo.svg
favicon=/static/favicon.ico
primary_color=#3b82f6
support_email=support@mycompany.com
docs_url=https://docs.mycompany.com
copyright=© {year} {name}. All rights reserved.
```

## Configuration Options

### name

**Type:** String  
**Default:** `General Bots`

The product name that replaces "General Bots" throughout the application, including:

- Page titles
- Welcome messages
- Footer text
- Email templates
- API responses

```ini
name=Acme Bot Platform
```

### apps

**Type:** Comma-separated list  
**Default:** All apps enabled

Specifies which apps are active in the suite. Only listed apps will:

- Appear in the navigation menu
- Have their APIs enabled
- Be accessible to users

**Available apps:**

| App | Description |
|-----|-------------|
| `chat` | Main chat interface |
| `mail` | Email client |
| `calendar` | Calendar and scheduling |
| `drive` | File storage |
| `tasks` | Task management |
| `docs` | Document editor |
| `paper` | Notes and quick documents |
| `sheet` | Spreadsheet editor |
| `slides` | Presentation editor |
| `meet` | Video conferencing |
| `research` | Research assistant |
| `sources` | Data sources management |
| `analytics` | Analytics dashboard |
| `admin` | Administration panel |
| `monitoring` | System monitoring |
| `settings` | User settings |

**Example - Minimal setup:**

```ini
apps=chat,drive,tasks
```

**Example - Full productivity suite:**

```ini
apps=chat,mail,calendar,drive,tasks,docs,sheet,slides,meet
```

### theme

**Type:** String  
**Default:** `sentient`

Sets the default theme for new users and the login page.

**Available themes:**

| Theme | Description |
|-------|-------------|
| `sentient` | Default neon green theme |
| `dark` | Dark mode with blue accents |
| `light` | Light mode with blue accents |
| `blue` | Blue theme |
| `purple` | Purple theme |
| `green` | Green theme |
| `orange` | Orange theme |
| `cyberpunk` | Cyberpunk aesthetic |
| `retrowave` | 80s retrowave style |
| `vapordream` | Vaporwave aesthetic |
| `y2kglow` | Y2K neon style |
| `arcadeflash` | Arcade game style |
| `discofever` | Disco theme |
| `grungeera` | 90s grunge style |
| `jazzage` | Jazz age gold |
| `mellowgold` | Mellow gold tones |
| `midcenturymod` | Mid-century modern |
| `polaroidmemories` | Vintage polaroid |
| `saturdaycartoons` | Cartoon style |
| `seasidepostcard` | Beach/ocean theme |
| `typewriter` | Typewriter/monospace |
| `3dbevel` | 3D beveled style |
| `xeroxui` | Classic Xerox UI |
| `xtreegold` | XTree Gold DOS style |

```ini
theme=dark
```

### logo

**Type:** URL/Path (optional)  
**Default:** General Bots logo

URL or path to your custom logo. Supports SVG, PNG, or other image formats.

```ini
logo=/static/branding/my-logo.svg
logo=https://cdn.mycompany.com/logo.png
```

### favicon

**Type:** URL/Path (optional)  
**Default:** General Bots favicon

URL or path to your custom favicon.

```ini
favicon=/static/branding/favicon.ico
```

### primary_color

**Type:** Hex color code (optional)  
**Default:** Theme-dependent

Override the primary accent color across the UI.

```ini
primary_color=#3b82f6
primary_color=#e11d48
```

### support_email

**Type:** Email address (optional)  
**Default:** None

Support email displayed in help sections and error messages.

```ini
support_email=support@mycompany.com
```

### docs_url

**Type:** URL (optional)  
**Default:** `https://docs.pragmatismo.com.br`

URL to your documentation site.

```ini
docs_url=https://docs.mycompany.com
```

### copyright

**Type:** String (optional)  
**Default:** `© {year} {name}. All rights reserved.`

Copyright text for the footer. Supports placeholders:

- `{year}` - Current year
- `{name}` - Product name

```ini
copyright=© {year} {name} - A product of My Company Inc.
```

## API Integration

### Product Configuration Endpoint

The current product configuration is available via API:

```
GET /api/product
```

**Response:**

```json
{
  "name": "My Custom Platform",
  "apps": ["chat", "drive", "tasks", "calendar"],
  "theme": "dark",
  "logo": "/static/my-logo.svg",
  "favicon": null,
  "primary_color": "#3b82f6",
  "docs_url": "https://docs.mycompany.com",
  "copyright": "© 2025 My Custom Platform. All rights reserved."
}
```

### App-Gated APIs

When an app is disabled in the `.product` file, its corresponding APIs return `403 Forbidden`:

```json
{
  "error": "app_disabled",
  "message": "The 'calendar' app is not enabled for this installation"
}
```

## UI Integration

### JavaScript Access

The product configuration is available in the frontend:

```javascript
// Fetch product config
const response = await fetch('/api/product');
const product = await response.json();

console.log(product.name);  // "My Custom Platform"
console.log(product.apps);  // ["chat", "drive", "tasks"]
```

### Conditional App Rendering

The navigation menu automatically hides disabled apps. If you need to check manually:

```javascript
function isAppEnabled(appName) {
  return window.productConfig?.apps?.includes(appName) ?? false;
}

if (isAppEnabled('calendar')) {
  // Show calendar features
}
```

## Examples

### Example 1: Simple Chat Bot

A minimal setup for a chat-only bot:

```ini
name=Support Bot
apps=chat
theme=dark
support_email=help@example.com
```

### Example 2: Internal Tools Platform

An internal company platform with productivity tools:

```ini
name=Acme Internal Tools
apps=chat,drive,tasks,docs,calendar,meet
theme=light
logo=/static/acme-logo.svg
primary_color=#1e40af
docs_url=https://wiki.acme.internal
copyright=© {year} Acme Corporation - Internal Use Only
```

### Example 3: Customer Service Platform

A customer service focused deployment:

```ini
name=ServiceDesk Pro
apps=chat,tasks,analytics,admin,monitoring
theme=blue
support_email=admin@servicedesk.com
docs_url=https://help.servicedesk.com
```

### Example 4: Full Suite

Enable all features (default behavior):

```ini
name=Enterprise Suite
apps=chat,mail,calendar,drive,tasks,docs,paper,sheet,slides,meet,research,sources,analytics,admin,monitoring,settings
theme=sentient
```

## Reloading Configuration

The product configuration is loaded at server startup. To apply changes:

1. Edit the `.product` file
2. Restart the server

```bash
# Restart the server to apply changes
systemctl restart botserver
# or
docker-compose restart botserver
```

## Environment Variable Override

You can override the `.product` file location using an environment variable:

```bash
export PRODUCT_CONFIG_PATH=/etc/myapp/.product
```

## Best Practices

1. **Version Control**: Include the `.product` file in your deployment configuration (but not in the main repo if it contains sensitive branding)

2. **Minimal Apps**: Only enable the apps your users need to reduce complexity and improve performance

3. **Consistent Branding**: Ensure your logo, colors, and theme work well together

4. **Documentation**: Update your `docs_url` to point to customized documentation for your users

5. **Testing**: Test the UI with your specific app combination to ensure navigation works correctly

## Troubleshooting

### Apps Not Hiding

If disabled apps still appear:

1. Clear browser cache
2. Verify the `.product` file syntax
3. Check server logs for configuration errors
4. Restart the server

### API Returns 403

If APIs return "app_disabled" errors:

1. Check the `apps` list in `.product`
2. Ensure the app name is spelled correctly (lowercase)
3. Restart the server after changes

### Branding Not Updating

If the product name doesn't change:

1. Hard refresh the browser (Ctrl+Shift+R)
2. Clear application cache
3. Verify the `name` field in `.product`
4. Check for syntax errors (missing `=` sign)

## Related Documentation

- [Theme Customization](../07-user-interface/README.md) - Detailed theme configuration
- [UI Components](../07-user-interface/README.md) - UI customization options
- [Configuration](../10-configuration-deployment/README.md) - General server configuration
- [Authentication](../09-security/README.md) - Auth customization for white-label