# Compliance - ISO 27001

> **Information security compliance**

<img src="../../assets/suite/compliance-screen.svg" alt="Compliance Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Compliance is an ISO 27001 information security compliance management system within General Bots Suite. Run compliance checks, track issues, manage audit logs, assess risks, assign training, and monitor compliance posture. Compliance provides comprehensive tools for maintaining information security standards.

---

## Features

### Checks

| Action | Description |
|--------|-------------|
| **Run** | Execute compliance checks |
| **Schedule** | Automate periodic checks |
| **Custom** | Create custom check rules |
| **Results** | View detailed check results |
| **History** | Track check execution history |

### Issues

| Action | Description |
|--------|-------------|
| **Track** | Monitor open compliance issues |
| **Assign** | Assign issues to team members |
| **Resolve** | Mark issues as resolved |
| **Escalate** | Escalate critical issues |
| **Report** | Generate issue reports |

### Audit Log

| Feature | Description |
|---------|-------------|
| **Search** | Full-text search across events |
| **Filter** | By date, user, action, or category |
| **Export** | Download as CSV or JSON |
| **Retention** | Configurable retention policies |
| **Compliance** | LGPD/GDPR audit support |

### Risk Matrix

| Feature | Description |
|---------|-------------|
| **Assess** | Evaluate risk likelihood and impact |
| **Mitigate** | Define mitigation strategies |
| **Track** | Monitor risk status |
| **Visualize** | Risk heat map |
| **Review** | Periodic risk reassessment |

### Training

| Feature | Description |
|---------|-------------|
| **Assign** | Create training assignments |
| **Track** | Monitor completion status |
| **Reminders** | Automated reminders |
| **Certificates** | Issue completion certificates |
| **Reports** | Training compliance reports |

### Dashboard

| Metric | Description |
|--------|-------------|
| **Score** | Overall compliance score |
| **Trends** | Score changes over time |
| **Issues** | Open/closed issues |
| **Training** | Completion percentages |
| **Deadlines** | Upcoming compliance deadlines |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `G` then `C` | Go to Checks |
| `G` then `I` | Go to Issues |
| `G` then `A` | Go to Audit Log |
| `G` then `R` | Go to Risk Matrix |
| `G` then `T` | Go to Training |
| `N` | Create new issue |
| `/` | Focus search |
| `Ctrl+R` | Run compliance check |

---

## Compliance via Chat

### Running Compliance Check

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Run compliance check</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔍 Compliance Check Started</p>
      <p>📋 Running ISO 27001 checks...</p>
      <p>✅ Access Control: PASS</p>
      <p>✅ Data Encryption: PASS</p>
      <p>⚠️ Backup Verification: WARNING</p>
      <p>❌ Incident Response Plan: FAIL</p>
      <p>📊 Overall: 85% compliant</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
</div>

### Viewing Compliance Score

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show compliance score</p>
      <div class="wa-time">16:05</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 Compliance Dashboard</p>
      <p>🎯 Score: 85/100</p>
      <p>📈 Trend: +3 points (last 30 days)</p>
      <p>⚠️ Open Issues: 4</p>
      <p>✅ Resolved This Month: 12</p>
      <p>📅 Next Audit: 2024-02-15</p>
      <div class="wa-time">16:05</div>
    </div>
  </div>
</div>

---

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/compliance/checks` | GET | List compliance checks |
| `/api/compliance/checks/run` | POST | Run compliance check |
| `/api/compliance/issues` | GET | List compliance issues |
| `/api/compliance/issues` | POST | Create issue |
| `/api/compliance/issues/:id` | PUT | Update issue |
| `/api/compliance/audit` | GET | Query audit log |
| `/api/compliance/audit/export` | GET | Export audit log |
| `/api/compliance/risks` | GET | List risks |
| `/api/compliance/risks` | POST | Create risk assessment |
| `/api/compliance/training` | GET | List training assignments |
| `/api/compliance/training` | POST | Create training assignment |
| `/api/compliance/dashboard` | GET | Get compliance dashboard |

---

## Related Pages

- [Tools](tools.md) — Security dashboard
- [Admin](admin.md) — System administration
- [Compliance API](compliance-api.md) — Compliance REST API
- [RBAC Overview](../../09-security/rbac-overview.md) — Role-based access control