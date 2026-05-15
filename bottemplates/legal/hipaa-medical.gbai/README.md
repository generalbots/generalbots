# HIPAA Medical Privacy Template

A HIPAA-compliant healthcare privacy portal template for General Bots.

## Overview

This template provides healthcare organizations with a ready-to-deploy patient privacy rights management system that complies with:

- **HIPAA** (Health Insurance Portability and Accountability Act)
- **HITECH Act** (Health Information Technology for Economic and Clinical Health)
- State-specific healthcare privacy regulations

## Features

### Patient Rights Management

1. **Access Medical Records** - Patients can request copies of their Protected Health Information (PHI)
2. **Request Amendments** - Patients can request corrections to their medical records
3. **Accounting of Disclosures** - Track and report who has accessed patient PHI
4. **Request Restrictions** - Allow patients to limit how their PHI is used or shared
5. **Confidential Communications** - Patients can specify preferred contact methods
6. **File Privacy Complaints** - Streamlined complaint submission process
7. **Revoke Authorizations** - Withdraw previous consent for PHI disclosure

### HIPAA Compliance Features

- **Audit Trail** - Complete logging of all PHI access and requests
- **Encryption** - AES-256 at rest, TLS 1.3 in transit
- **Access Controls** - Role-based access control (RBAC)
- **Break Glass** - Emergency access procedures with audit
- **Minimum Necessary** - Automatic enforcement of minimum necessary standard
- **PHI Detection** - Automatic detection and redaction of PHI in communications
- **Breach Notification** - Built-in breach response workflow

## Installation

1. Copy this template to your General Bots instance:

```bash
cp -r templates/hipaa-medical.gbai /path/to/your/bot/
```

2. Configure the bot settings in `hipaa.gbot/config.csv`:

```csv
Covered Entity Name,Your Healthcare Organization
Privacy Officer Email,privacy@yourhealthcare.org
HIPAA Security Officer,security@yourhealthcare.org
```

3. Deploy the template:

```bash
botserver --deploy hipaa-medical.gbai
```

## Configuration

### Required Settings

| Setting | Description | Example |
|---------|-------------|---------|
| `Covered Entity Name` | Your organization's legal name | Memorial Hospital |
| `Privacy Officer Email` | HIPAA Privacy Officer contact | privacy@hospital.org |
| `HIPAA Security Officer` | Security Officer contact | security@hospital.org |
| `Covered Entity NPI` | National Provider Identifier | 1234567890 |

### Security Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `Require 2FA` | true | Two-factor authentication required |
| `Session Timeout` | 300 | Session timeout in seconds (5 minutes) |
| `Encryption At Rest` | AES-256 | Data encryption standard |
| `PHI Auto Redaction` | true | Automatically redact PHI in logs |

### Compliance Timelines

| Requirement | Deadline | Setting |
|-------------|----------|---------|
| Access Requests | 30 days | `Response SLA Hours` |
| Urgent Requests | 48 hours | `Urgent Response Hours` |
| Breach Notification | 60 hours | `Breach Notification Hours` |

## Dialogs

### Main Entry Point

- `start.bas` - Main menu for patient privacy rights

### Patient Rights Dialogs

- `access-phi.bas` - Request medical records
- `request-amendment.bas` - Request record corrections
- `accounting-disclosures.bas` - View access history
- `request-restrictions.bas` - Limit PHI use/sharing
- `confidential-communications.bas` - Set contact preferences
- `file-complaint.bas` - Submit privacy complaints
- `revoke-authorization.bas` - Withdraw consent

## Integration

### Patient Portal Integration

Connect to your existing patient portal:

```basic
' In your custom dialog
patient = GET PATIENT FROM "portal" WHERE mrn = patient_mrn
IF patient.verified THEN
    CALL "access-phi.bas"
END IF
```

### EHR Integration

The template can integrate with common EHR systems:

- Epic
- Cerner
- Meditech
- Allscripts

Configure your EHR connection in the bot settings or use the FHIR API for standard integration.

## Audit Requirements

All interactions are logged to the `hipaa_audit_log` table with:

- Timestamp
- Session ID
- Action performed
- User/patient identifier
- IP address
- User agent
- Outcome

Retain audit logs for a minimum of 6 years (2,190 days) per HIPAA requirements.

## Customization

### Adding Custom Dialogs

Create new `.bas` files in the `hipaa.gbdialog` folder:

```basic
' custom-workflow.bas
TALK "Starting custom HIPAA workflow..."
' Your custom logic here
```

### Branding

Customize the welcome message and organization details in `config.csv`.

## Support

For questions about this template:

- **Documentation**: See General Bots docs
- **Issues**: GitHub Issues
- **HIPAA Guidance**: consult your compliance officer

## Disclaimer

This template is provided as a compliance aid and does not constitute legal advice. Healthcare organizations are responsible for ensuring their HIPAA compliance program meets all regulatory requirements. Consult with healthcare compliance professionals and legal counsel.

## License

AGPL-3.0 - See LICENSE file in the main repository.

---

Built with ❤️ by Pragmatismo