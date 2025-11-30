# Privacy Rights Center Template (privacy.gbai)

A comprehensive LGPD/GDPR compliance template for General Bots that enables organizations to handle data subject rights requests automatically.

## Overview

This template provides a complete privacy portal that helps organizations comply with:

- **LGPD** (Lei Geral de Proteção de Dados - Brazil)
- **GDPR** (General Data Protection Regulation - EU)
- **CCPA** (California Consumer Privacy Act - US)
- **Other privacy regulations** with similar data subject rights

## Features

### Data Subject Rights Implemented

| Right | LGPD Article | GDPR Article | Dialog File |
|-------|--------------|--------------|-------------|
| Access | Art. 18 | Art. 15 | `request-data.bas` |
| Rectification | Art. 18 III | Art. 16 | `rectify-data.bas` |
| Erasure (Deletion) | Art. 18 VI | Art. 17 | `delete-data.bas` |
| Data Portability | Art. 18 V | Art. 20 | `export-data.bas` |
| Consent Management | Art. 8 | Art. 7 | `manage-consents.bas` |
| Object to Processing | Art. 18 IV | Art. 21 | `object-processing.bas` |

### Key Capabilities

- **Identity Verification** - Email-based verification codes before processing requests
- **Audit Trail** - Complete logging of all privacy requests for compliance
- **Multi-format Export** - JSON, CSV, XML export options for data portability
- **Consent Tracking** - Granular consent management with history
- **Email Notifications** - Automated confirmations and reports
- **SLA Tracking** - Response time monitoring (default: 72 hours)

## Installation

1. Copy the template to your bot's packages directory:

```bash
cp -r templates/privacy.gbai /path/to/your/bot/packages/
```

2. Configure the bot settings in `privacy.gbot/config.csv`:

```csv
name,value
Company Name,Your Company Name
Privacy Officer Email,privacy@yourcompany.com
DPO Contact,dpo@yourcompany.com
```

3. Restart General Bots to load the template.

## Configuration Options

### Required Settings

| Setting | Description | Example |
|---------|-------------|---------|
| `Company Name` | Your organization name | Acme Corp |
| `Privacy Officer Email` | Contact for privacy matters | privacy@acme.com |
| `DPO Contact` | Data Protection Officer | dpo@acme.com |

### Optional Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `Session Timeout` | 900 | Session timeout in seconds |
| `Response SLA Hours` | 72 | Max hours to respond to requests |
| `Data Retention Days` | 30 | Days to retain completed request data |
| `Enable HIPAA Mode` | false | Enable PHI handling features |
| `Require 2FA` | false | Require two-factor authentication |

## File Structure

```
privacy.gbai/
├── README.md                    # This file
├── privacy.gbdialog/
│   ├── start.bas               # Main entry point
│   ├── request-data.bas        # Data access requests
│   ├── delete-data.bas         # Data erasure requests
│   ├── export-data.bas         # Data portability
│   └── manage-consents.bas     # Consent management
├── privacy.gbot/
│   └── config.csv              # Bot configuration
└── privacy.gbui/
    └── index.html              # Web portal UI
```

## Usage Examples

### Starting the Privacy Portal

Users can access the privacy portal by saying:

- "I want to access my data"
- "Delete my information"
- "Export my data"
- "Manage my consents"
- Or selecting options 1-6 from the menu

### API Integration

The template exposes REST endpoints for integration:

```
POST /api/privacy/request      - Submit a new request
GET  /api/privacy/requests     - List user's requests
GET  /api/privacy/request/:id  - Get request status
POST /api/privacy/consent      - Update consents
```

### Webhook Events

The template emits webhook events for integration:

- `privacy.request.created` - New request submitted
- `privacy.request.completed` - Request fulfilled
- `privacy.consent.updated` - Consent preferences changed
- `privacy.data.deleted` - User data erased

## Customization

### Adding Custom Consent Categories

Edit `manage-consents.bas` to add new consent categories:

```basic
consent_categories = [
    {
        "id": "custom_category",
        "name": "Custom Category Name",
        "description": "Description for users",
        "required": FALSE,
        "legal_basis": "Consent"
    }
]
```

### Branding the UI

Modify `privacy.gbui/index.html` to match your branding:

- Update CSS variables for colors
- Replace logo and company name
- Add custom legal text

### Email Templates

Customize email notifications by editing the `SEND MAIL` blocks in each dialog file.

## Compliance Notes

### Response Deadlines

| Regulation | Standard Deadline | Extended Deadline |
|------------|-------------------|-------------------|
| LGPD | 15 days | - |
| GDPR | 30 days | 90 days (complex) |
| CCPA | 45 days | 90 days |

### Data Retention

Some data may need to be retained for legal compliance:

- Financial records (tax requirements)
- Legal dispute documentation
- Fraud prevention records
- Regulatory compliance data

The template handles this by anonymizing retained records while deleting identifiable information.

### Audit Requirements

All requests are logged to `privacy_requests` and `consent_history` tables with:

- Timestamp
- User identifier
- Request type
- IP address
- Completion status
- Legal basis

## Support

For questions about this template:

- **Documentation**: https://docs.pragmatismo.com.br/privacy-template
- **Issues**: https://github.com/GeneralBots/BotServer/issues
- **Email**: support@pragmatismo.com.br

## License

This template is part of General Bots and is licensed under AGPL-3.0.

---

**Note**: This template provides technical implementation for privacy compliance. Organizations should consult with legal counsel to ensure full compliance with applicable regulations.