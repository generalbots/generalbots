# FILL

Populates a document template with data from variables or objects.

## Syntax

```basic
result = FILL template, data
FILL template, data TO output_path
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `template` | String | Path to template file (Word, Excel, PDF, or text) |
| `data` | Object | Key-value pairs for placeholder replacement |
| `output_path` | String | Optional destination path for filled document |

## Description

`FILL` replaces placeholders in document templates with actual data values. Placeholders use double curly braces like `{{name}}` or `{{company}}`. This is useful for generating personalized documents, contracts, invoices, and reports.

## Examples

### Basic Template Fill

```basic
data = #{
    name: "John Smith",
    company: "Acme Corp",
    date: FORMAT(TODAY(), "MMMM d, yyyy")
}

result = FILL "templates/contract.docx", data
TALK "Document generated: " + result.path
```

### Invoice Generation

```basic
invoice_data = #{
    invoice_number: "INV-2025-001",
    customer_name: customer.name,
    customer_email: customer.email,
    items: order_items,
    subtotal: subtotal,
    tax: tax_amount,
    total: total_amount,
    due_date: FORMAT(DATEADD("day", 30, TODAY()), "yyyy-MM-dd")
}

FILL "templates/invoice.docx", invoice_data TO "invoices/INV-2025-001.docx"
TALK "Invoice generated and saved"
```

### Certificate Generation

```basic
certificate = #{
    recipient: participant.name,
    course: "AI Fundamentals",
    completion_date: FORMAT(TODAY(), "MMMM d, yyyy"),
    instructor: "Dr. Sarah Johnson",
    certificate_id: GUID()
}

FILL "templates/certificate.docx", certificate TO "certificates/" + certificate.certificate_id + ".docx"
```

### Email Template

```basic
email_data = #{
    first_name: user.first_name,
    order_id: order.id,
    tracking_number: shipment.tracking,
    delivery_date: shipment.estimated_delivery
}

body = FILL "templates/shipping-notification.txt", email_data
SEND MAIL user.email, "Your order has shipped!", body
```

## Supported Template Formats

| Format | Extension | Placeholder Style |
|--------|-----------|-------------------|
| Word | `.docx` | `{{placeholder}}` |
| Excel | `.xlsx` | `{{placeholder}}` |
| Text | `.txt` | `{{placeholder}}` |
| HTML | `.html` | `{{placeholder}}` |
| Markdown | `.md` | `{{placeholder}}` |

## Return Value

Returns an object containing:

| Property | Description |
|----------|-------------|
| `path` | Path to the generated document |
| `content` | Document content (for text formats) |
| `size` | File size in bytes |

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Generate a contract for Acme Corp</p>
      <div class="wa-time">11:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I'll create the contract. What's the contact person's name?</p>
      <div class="wa-time">11:15</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Jane Wilson</p>
      <div class="wa-time">11:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Contract generated!</p>
      <p></p>
      <p>📄 <strong>Service Agreement</strong></p>
      <p>• Company: Acme Corp</p>
      <p>• Contact: Jane Wilson</p>
      <p>• Date: May 15, 2025</p>
      <p></p>
      <p>The contract has been saved to your Drive.</p>
      <p>Would you like me to email it to Jane?</p>
      <div class="wa-time">11:15</div>
    </div>
  </div>
</div>

## Template Example

A template file might look like:

```
SERVICE AGREEMENT

This agreement is entered into on {{date}} between:

Company: {{company_name}}
Contact: {{contact_name}}
Email: {{contact_email}}

SERVICES:
{{service_description}}

TERMS:
Duration: {{duration}} months
Payment: ${{monthly_amount}} per month
Start Date: {{start_date}}

Signature: _____________________
```

## Advanced: Lists and Tables

For repeating data, use array placeholders:

```basic
data = #{
    customer: "Acme Corp",
    items: [
        #{name: "Widget", qty: 10, price: 29.99},
        #{name: "Gadget", qty: 5, price: 49.99}
    ],
    total: 549.85
}

FILL "templates/order.docx", data TO "orders/order-123.docx"
```

In the template, use `{{#items}}...{{/items}}` for loops.

## See Also

- [GENERATE PDF](./keyword-generate-pdf.md) - Convert filled documents to PDF
- [MERGE PDF](./keyword-merge-pdf.md) - Combine multiple documents
- [UPLOAD](./keyword-upload.md) - Upload generated documents
- [SEND MAIL](./keyword-send-mail.md) - Email generated documents

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:500px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-message{margin-bottom:10px}
.wa-message.user{text-align:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;display:inline-block;text-align:left}
.wa-message.bot .wa-bubble{background-color:#fff;display:inline-block}
.wa-bubble{padding:8px 12px;border-radius:8px;box-shadow:0 1px .5px rgba(0,0,0,.13);max-width:85%}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
</style>