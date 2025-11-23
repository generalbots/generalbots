# CHANGE THEME

## Syntax

```basic
CHANGE THEME theme-name
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| theme-name | String | Name of the CSS theme file (without .css extension) |

## Description

Changes the visual theme of the bot interface by loading a different CSS file from the `.gbtheme` folder. The change applies immediately to all connected users.

## Examples

### Basic Theme Switch

```basic
' Switch to dark mode
CHANGE THEME "dark"

' Back to default
CHANGE THEME "default"

' Retro Windows 95 style
CHANGE THEME "3dbevel"
```

### Conditional Theme

```basic
hour = HOUR(NOW())

IF hour >= 18 OR hour < 6 THEN
    CHANGE THEME "dark"
ELSE
    CHANGE THEME "light"
END IF
```

### User Preference

```basic
TALK "Which theme would you prefer?"
ADD SUGGESTION "default" AS "Default"
ADD SUGGESTION "dark" AS "Dark Mode"
ADD SUGGESTION "3dbevel" AS "Retro Style"

HEAR choice
CHANGE THEME choice
SET BOT MEMORY "user_theme" AS choice
TALK "Theme changed!"
```

### Seasonal Themes

```basic
month = MONTH(NOW())

IF month = 12 THEN
    CHANGE THEME "holiday"
ELSE IF month >= 6 AND month <= 8 THEN
    CHANGE THEME "summer"
ELSE
    CHANGE THEME "default"
END IF
```

## Notes

- Theme files must be in the `.gbtheme` folder
- Don't include the `.css` extension in the theme name
- Changes apply to all connected users immediately
- If theme file doesn't exist, falls back to default

## Related

- [Chapter 04: gbtheme Reference](../chapter-04/README.md)
- [CSS Customization](../chapter-04/css.md)