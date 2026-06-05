# Templates - Bot Templates

> **Pre-built bot templates for rapid deployment across industries**

<img src="../../assets/suite/templates-app-screen.svg" alt="Templates Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Templates provides a curated library of pre-built bot configurations that can be deployed instantly. Browse by category, preview functionality, customize settings, and deploy — reducing bot setup time from hours to minutes.

---

## Features

### Templates

| Capability | Description |
|------------|-------------|
| Browse | Explore available bot templates |
| Preview | Test template before deployment |
| Deploy | One-click deployment to your account |
| Duplicate | Clone templates for customization |

### Categories

| Category | Description |
|----------|-------------|
| Business | CRM, Sales, Support, HR bots |
| Service | Helpdesk, Booking, FAQ bots |
| Lifestyle | Personal assistants, Reminders bots |
| Industry | Healthcare, Education, Finance bots |

### Custom

| Capability | Description |
|------------|-------------|
| Create | Build custom templates from existing bots |
| Share | Publish templates to community |
| Import | Import templates from marketplace |
| Export | Export templates as portable files |

### Stats

| Capability | Description |
|------------|-------------|
| Usage | Track template deployment count |
| Rating | Community ratings and reviews |
| Downloads | Download statistics per template |
| Trending | Most popular templates this week |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + T` | Open template library |
| `Ctrl + F` | Search templates |
| `Ctrl + D` | Deploy selected template |
| `Ctrl + P` | Preview template |
| `Ctrl + C` | Create custom template |
| `Ctrl + S` | Share template |
| `Esc` | Close template browser |

---

## Templates via Chat

<div class="wa-chat">

**User:** Show bot templates

**Bot:** Here are our top bot templates:

| Template | Category | Rating | Deployments |
|----------|----------|--------|-------------|
| Helpdesk Pro | Business | 4.8 | 1,234 |
| Appointment Scheduler | Service | 4.7 | 987 |
| Personal Assistant | Lifestyle | 4.9 | 2,156 |
| HR Onboarding | Business | 4.6 | 543 |

Categories available: **Business**, **Service**, **Lifestyle**, **Industry**

Would you like to explore a category or search for something specific?

---

**User:** Deploy a helpdesk template

**Bot:** I found the **Helpdesk Pro** template:

- **Rating:** 4.8/5 (234 reviews)
- **Features:** Ticket management, FAQ, escalation, SLA tracking
- **Setup Time:** ~5 minutes

To deploy, please provide:
1. **Bot name** — What should the bot be called?
2. **Department** — Which team is this for?
3. **Custom greeting** — (optional) Custom welcome message

Or deploy with default settings by typing "Deploy with defaults."

</div>

---

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/templates` | GET | List all templates |
| `/api/templates/{id}` | GET | Get template by ID |
| `/api/templates/{id}/preview` | GET | Preview template functionality |
| `/api/templates/{id}/deploy` | POST | Deploy template |
| `/api/templates/{id}/stats` | GET | Get template usage stats |
| `/api/templates/{id}/reviews` | GET | Get template reviews |
| `/api/templates/categories` | GET | List template categories |
| `/api/templates/search` | GET | Search templates |
| `/api/templates/custom` | POST | Create custom template |
| `/api/templates/custom/{id}` | PUT | Update custom template |
| `/api/templates/custom/{id}` | DELETE | Delete custom template |
| `/api/templates/custom/{id}/share` | POST | Publish to marketplace |
| `/api/templates/import` | POST | Import template from file |
| `/api/templates/{id}/export` | GET | Export template as file |

---

## Related Pages

- [Bot Builder](../bot-builder.md) — Visual bot creation tool
- [Marketplace](../marketplace.md) — Community templates marketplace
- [Deployment](../deployment.md) — Bot deployment guides
- [Custom Bots](../custom-bots.md) — Building bots from scratch
