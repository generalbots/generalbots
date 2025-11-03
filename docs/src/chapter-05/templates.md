# Templates System

The **Templates** directory is the foundation for the `templates.html` interface in GeneralBots.  
It contains all official `.gbai` template packages used to generate bot dialogs, announcements, and default automation flows.

---

## Overview

Templates define reusable bot configurations, dialog flows, and knowledge bases.  
Each template package is stored under the `templates/` directory and follows a consistent structure:

```
templates/
├── default.gbai/
│   ├── default.gbot/
│   ├── default.gbdialog/
│   └── default.gbkb/
└── announcements.gbai/
    ├── announcements.gbot/
    ├── announcements.gbdialog/
    └── announcements.gbkb/
```

Each `.gbai` folder represents a **template group**, containing:
- `.gbdialog/` — BASIC dialog scripts defining conversational flows.
- `.gbkb/` — Knowledge base files used for contextual responses.
- `.gbot/` — Bot configuration files defining behavior and metadata.

---

## Template Groups

### `default.gbai`

The **Default Template** provides the base configuration for new bots and general automation.  
It includes standard dialogs, system prompts, and basic workflows used across all bots.

**Contents:**
- `default.gbot/` — Core bot configuration.
- `default.gbdialog/` — Default dialog scripts (e.g., greetings, help, onboarding).
- `default.gbkb/` — Default knowledge base entries.

**Purpose:**
Used as the starting point for new bot instances and as the fallback template when no specific configuration is provided.

---

### `announcements.gbai`

The **Announcements Template** defines dialogs and content for broadcasting messages or system updates.  
It is used by bots that handle notifications, alerts, or scheduled announcements.

**Contents:**
- `announcements.gbot/` — Announcement bot configuration.
- `announcements.gbdialog/` — Dialog scripts for announcement delivery.
- `announcements.gbkb/` — Knowledge base entries related to announcements.

**Purpose:**
Used for bots that send periodic updates, news, or system-wide messages.


---

## Implementation Notes

- Templates are modular and can be extended by adding new `.gbai` folders.
- Each template group must include at least one `.gbdialog` and `.gbot` directory.
- The bot engine automatically detects and loads templates at startup.
- Custom templates can be created by duplicating an existing `.gbai` folder and modifying its contents.

---

## Summary

The `templates/` directory is the backbone of the GeneralBots template system.  
Each `.gbai` package encapsulates dialogs, knowledge bases, and configurations for specific bot behaviors.
