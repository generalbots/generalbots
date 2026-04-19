# SEND MAIL

Send email messages.

## Syntax

### Single Line

```basic
SEND MAIL to, subject, body
SEND MAIL to, subject, body USING "account@example.com"
```

### Multi-Line Block with Variable Substitution

```basic
BEGIN MAIL recipient
Subject: Email subject here

Dear ${customerName},

Your order ${orderId} is ready.

Thank you!
END MAIL
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `to` | String | Recipient email address(es), comma-separated for multiple |
| `subject` | String | Email subject line |
| `body` | String | Email body (plain text or HTML) |
| `account` | String | (Optional) Connected account to send through |
| `${variable}` | Expression | Variable substitution within MAIL blocks |

## Description

The `SEND MAIL` keyword sends emails using either:

1. **Default SMTP** - Configuration from `config.csv`
2. **Connected Account** - Send through Gmail, Outlook, etc. configured in Sources app

## BEGIN MAIL / END MAIL Blocks

The `BEGIN MAIL / END MAIL` block syntax allows you to write elegant multi-line emails with automatic variable substitution using `${variable}` syntax.

### Syntax

```basic
BEGIN MAIL recipient
Subject: Email subject ${variable}

Dear ${customerName},

Your order ${orderId} has been shipped.

Tracking: ${trackingNumber}

Best regards,
The Team
END MAIL
```

### How It Works

1. **First line after `BEGIN MAIL`**: Should contain the email recipient
2. **Line starting with `Subject:`**: Email subject line (supports `${variable}`)
3. **Blank line after subject**: Separates subject from body
4. **Body lines**: Email content with automatic `${variable}` substitution
5. **Each line** is converted to string concatenation with proper newline handling

**Input:**
```basic
nome = "João"
pedido = "12345"

BEGIN MAIL "cliente@example.com"
Subject: Confirmação do Pedido ${pedido}

Olá ${nome},

Seu pedido foi confirmado!

Atenciosamente,
Equipe de Vendas
END MAIL
```

**Converted to:**
```basic
SEND MAIL "cliente@example.com", "Confirmação do Pedido 12345", "Olá " + nome + ",\n\nSeu pedido foi confirmado!\n\nAtenciosamente,\nEquipe de Vendas"
```

### Variable Substitution Rules

- `${variableName}` - Replaced with the variable value
- `${FUNCTION(args)}` - Function calls are evaluated and substituted
- Plain text without `${}` is treated as a string literal
- Special characters like `$` (not followed by `{`) are preserved
- Newlines are preserved as `\n` in the final email body

### Examples

#### Simple Email

```basic
email = "customer@example.com"
nome = "Maria"

BEGIN MAIL email
Subject: Bem-vindo ao nosso serviço!

Olá ${nome},

Obrigado por se cadastrar!

Atenciosamente,
Equipe
END MAIL
```

#### With Function Calls

```basic
BEGIN MAIL "cliente@empresa.com"
Subject: Pedido ${pedidoId} - Confirmação

Prezado ${nomeCliente},

Confirmamos seu pedido #${pedidoId} no valor de ${FORMAT(total, "currency")}.

Entrega prevista para: ${FORMAT(dataEntrega, "dd/MM/yyyy")}

Atenciosamente,
Departamento de Vendas
END MAIL
```

#### HTML Email

```basic
BEGIN MAIL "cliente@exemplo.com"
Subject: Seu pedido foi enviado!

<h1>Confirmação de Pedido</h1>

<p>Olá ${nome},</p>
<p>Seu pedido <strong>${pedidoId}</strong> foi enviado com sucesso!</p>

<p>Valor: <em>${FORMAT(valor, "currency")}</em></p>

<p>Atenciosamente,<br>Loja Virtual</p>
END MAIL
```

### Real-World Example: Wedding Confirmation

```basic
PARAM nomeNoivo AS STRING LIKE "Carlos" DESCRIPTION "Nome do noivo"
PARAM nomeNoiva AS STRING LIKE "Ana" DESCRIPTION "Nome da noiva"
PARAM emailNoivo AS EMAIL LIKE "noivo@example.com" DESCRIPTION "Email do noivo"
PARAM emailNoiva AS EMAIL LIKE "noiva@example.com" DESCRIPTION "Email da noiva"
PARAM protocolo AS STRING LIKE "CAS123456" DESCRIPTION "Protocolo"

casamentoId = "CAS-" + FORMAT(NOW(), "yyyyMMddHHmmss")
tipoTexto = "Religioso Simples"

BEGIN MAIL emailNoivo
Subject: Confirmação de Casamento - Protocolo ${protocolo}

Queridos ${nomeNoivo} e ${nomeNoiva},

Parabéns pelo compromisso de amor que estão assumindo! Recebemos a solicitação de casamento no Santuário Cristo Redentor.

DADOS DA SOLICITAÇÃO:
Protocolo: ${protocolo}
ID: ${casamentoId}
Noivo: ${nomeNoivo}
Noiva: ${nomeNoiva}
Tipo: ${tipoTexto}

Nossa equipe verificará a disponibilidade e enviará todas as instruções necessárias em breve.

Que Deus abençoe a união de vocês!

Atenciosamente,
Secretaria do Santuário Cristo Redentor
Tel: (21) 4101-0770 | WhatsApp: (21) 99566-5883
END MAIL
```

### Multiple Recipients

Send the same email to multiple people:

```basic
BEGIN MAIL "team1@company.com"
Subject: Meeting Reminder

Team meeting tomorrow at 3 PM.
END MAIL

BEGIN MAIL "team2@company.com"
Subject: Meeting Reminder

Team meeting tomorrow at 3 PM.
END MAIL
```

Or use comma-separated recipients:

```basic
recipients = "john@company.com, jane@company.com, bob@company.com"
SEND MAIL recipients, "Meeting Update", "Meeting rescheduled to 4 PM"
```

### Advantages

1. **Cleaner Syntax** - No more repetitive string concatenation for email body
2. **Easier to Read** - Multi-line emails are natural to write and maintain
3. **Template-Like** - Write emails like templates with `${variable}` placeholders
4. **Automatic Newlines** - Blank lines in the block become `\n` in the email
5. **Perfect for TOOL Functions** - Variables are automatically filled by user input

## Examples

## Configuration

Default SMTP in `config.csv`:

```csv
name,value
email-from,noreply@example.com
email-server,smtp.example.com
email-port,587
email-user,smtp-user@example.com
email-pass,smtp-password
```

## Examples

```basic
SEND MAIL "user@example.com", "Welcome!", "Thank you for signing up."
```

```basic
recipients = "john@example.com, jane@example.com"
SEND MAIL recipients, "Team Update", "Meeting tomorrow at 3 PM"
```

```basic
body = "<h1>Welcome!</h1><p>Thank you for joining us.</p>"
SEND MAIL "user@example.com", "Getting Started", body
```

## USING Clause

Send through a connected account configured in Suite → Sources → Accounts:

```basic
SEND MAIL "customer@example.com", "Subject", body USING "support@company.com"
```

The email appears from that account's address with proper authentication.

```basic
SEND MAIL "customer@example.com", "Ticket Update", "Your ticket has been resolved." USING "support@company.com"
```

## Delivery Status

```basic
status = SEND MAIL "user@example.com", "Test", "Message"
IF status = "sent" THEN
    TALK "Email delivered successfully"
END IF
```

## Best Practices

1. Use connected accounts for better deliverability
2. Validate email addresses before sending
3. Implement delays for bulk emails
4. Handle failures gracefully

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Auth failed | Invalid credentials | Check config.csv or re-authenticate account |
| Not sending | Firewall blocking | Verify port 587/465 is open |
| Going to spam | No domain auth | Configure SPF/DKIM |
| Account not found | Not configured | Add account in Suite → Sources |

## See Also

- [USE ACCOUNT](./keyword-use-account.md)
- [WAIT](./keyword-wait.md)

## Implementation

Located in `src/basic/keywords/send_mail.rs`
