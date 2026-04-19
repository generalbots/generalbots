# Sources Sync Strategy

Connect external data sources (Gmail, Outlook, Google Drive, OneDrive) to General Bots for LLM search and file operations.

## Overview

Configure accounts in **Suite → Sources → Accounts**. Once connected:

1. **LLM Search** - Emails, calendar, files indexed and searchable via RAG
2. **File Operations** - Access files using `account://` path notation
3. **Email Sending** - Send through connected accounts with `SEND MAIL ... USING`

## Supported Sources

| Source | LLM Search | File Operations | Email Send |
|--------|------------|-----------------|------------|
| Gmail | Emails, Calendar | Google Drive | Yes |
| Outlook | Emails, Calendar | OneDrive | Yes |
| Google Workspace | Full | Google Drive | Yes |
| Microsoft 365 | Full | OneDrive, SharePoint | Yes |
| Custom IMAP | Emails | No | Yes |

## Configuration

All account setup is done through Suite → Sources → Accounts tab:

1. Click **Add Account**
2. Select provider (Google, Microsoft, IMAP)
3. Complete OAuth authentication
4. Configure sync settings

## USE ACCOUNT Keyword

Enables an account for LLM search and file operations:

```basic
USE ACCOUNT "support@company.com"
```

| Capability | Description |
|------------|-------------|
| LLM Search | Account content included in RAG queries |
| File Access | `account://` paths work for COPY, MOVE, etc. |

## File Operations with account:// Paths

```basic
USE ACCOUNT "user@gmail.com"

COPY "account://user@gmail.com/Documents/report.pdf" TO "local/report.pdf"
COPY "data.xlsx" TO "account://user@gmail.com/Shared/data.xlsx"
```

| Keyword | Works with account:// |
|---------|----------------------|
| COPY | Yes |
| MOVE | Yes |
| DELETE | Yes |
| DIR | Yes |
| EXISTS | Yes |
| LOAD | Yes |
| SAVE | Yes |

## Sending Email Through Accounts

```basic
SEND MAIL "customer@example.com", "Subject", body USING "support@company.com"
```

## Default Accounts

In `config.csv`:

```csv
name,value
default-accounts,support@company.com;sales@company.com
```

Bot starts with these accounts enabled.

## Security

- OAuth tokens stored in Vault
- File permissions respected
- All access logged for audit
- Tokens auto-refresh

## Related

- [USE ACCOUNT Keyword](../06-gbdialog/keyword-use-account.md)
- [SEND MAIL Keyword](../06-gbdialog/keyword-send-mail.md)
- [System Limits](system-limits.md)