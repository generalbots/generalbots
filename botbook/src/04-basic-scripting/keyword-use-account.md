# USE ACCOUNT

Activate a connected account for LLM search and file operations.

## Syntax

```basic
USE ACCOUNT "email@example.com"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `email` | String | Email address of connected account |

## Description

Enables a connected account in **mixed mode**:

1. **LLM Search** - Account content (emails, calendar, files) becomes searchable for RAG
2. **File Operations** - Enables `account://` path notation for COPY, MOVE, etc.

Accounts must be configured in **Suite → Sources → Accounts** before use.

## Examples

```basic
USE ACCOUNT "support@company.com"
```

```basic
USE ACCOUNT "support@company.com"
USE ACCOUNT "sales@company.com"
```

```basic
USE KB "product-docs"
USE ACCOUNT "support@company.com"
```

## Mixed Mode Behavior

### LLM Search

Once enabled, the LLM automatically searches the account when answering questions.

### File Operations (account:// paths)

```basic
USE ACCOUNT "user@gmail.com"

COPY "account://user@gmail.com/Documents/report.pdf" TO "reports/report.pdf"
```

### Path Notation

| Path | Description |
|------|-------------|
| `account://email/path` | File in connected account's drive |
| `local/path` | Local bot storage (.gbdrive) |

## Supported Providers

| Provider | LLM Search | File Operations |
|----------|------------|-----------------|
| Gmail | Emails, Calendar | Google Drive |
| Outlook | Emails, Calendar | OneDrive |
| Google Workspace | Full | Google Drive |
| Microsoft 365 | Full | OneDrive, SharePoint |

## Default Accounts

In `config.csv`:

```csv
name,value
default-accounts,support@company.com;sales@company.com
```

## Prerequisites

Configure accounts in Suite before use:

1. Open Suite → **Sources**
2. Click **Accounts** tab
3. Click **Add Account**
4. Select provider (Gmail, Outlook, etc.)
5. Complete OAuth authentication

## Security

- Only accounts configured in Sources are accessible
- OAuth tokens stored securely in Vault
- Each session can only access authorized accounts
- All file access is logged for audit

## See Also

- [USE KB](./keyword-use-kb.md)
- [SEND MAIL](./keyword-send-mail.md)
- [Sources Sync Strategy](../10-configuration-deployment/sources-sync-strategy.md)