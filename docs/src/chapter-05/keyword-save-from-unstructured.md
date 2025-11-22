# SAVE_FROM_UNSTRUCTURED

Extract and save structured data from unstructured text content.

## Syntax

```basic
SAVE_FROM_UNSTRUCTURED text, schema, destination
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | String | Unstructured text to process |
| `schema` | String | Schema name or definition for extraction |
| `destination` | String | Where to save extracted data (table, file, variable) |

## Description

The `SAVE_FROM_UNSTRUCTURED` keyword uses AI to extract structured information from free-form text like emails, documents, conversations, or web content. It:

- Identifies relevant data points
- Validates against schema
- Transforms to structured format
- Saves to specified destination
- Handles various text formats

## Examples

### Extract Contact Information
```basic
email_text = "Hi, I'm John Smith from Acme Corp. You can reach me at john@acme.com or 555-1234."
SAVE_FROM_UNSTRUCTURED email_text, "contact_schema", "contacts_table"
' Extracts: name, company, email, phone
```

### Process Invoice Data
```basic
invoice_text = GET_FILE_CONTENT("invoice.pdf")
SAVE_FROM_UNSTRUCTURED invoice_text, "invoice_schema", "invoices_db"
' Extracts: invoice_number, date, amount, items, tax
```

### Extract Meeting Notes
```basic
transcript = "Meeting on Jan 15. Present: Alice, Bob. Discussed Q1 targets of $1M. Action: Bob to prepare report by Friday."
SAVE_FROM_UNSTRUCTURED transcript, "meeting_schema", meeting_data
' Extracts: date, attendees, topics, actions, deadlines
```

### Parse Customer Feedback
```basic
review = HEAR "Please tell us about your experience"
SAVE_FROM_UNSTRUCTURED review, "feedback_schema", "reviews_table"
' Extracts: sentiment, rating, issues, suggestions
```

## Schema Definition

### Predefined Schemas

Common schemas available out-of-the-box:
- `contact_schema` - Name, email, phone, company
- `address_schema` - Street, city, state, zip, country
- `invoice_schema` - Number, date, items, amounts
- `meeting_schema` - Date, attendees, agenda, actions
- `feedback_schema` - Sentiment, topics, ratings
- `resume_schema` - Skills, experience, education
- `product_schema` - Name, price, features, specs

### Custom Schema
```basic
' Define custom extraction schema
schema = {
    "fields": [
        {"name": "customer_id", "type": "string", "required": true},
        {"name": "issue_type", "type": "enum", "values": ["billing", "technical", "other"]},
        {"name": "priority", "type": "enum", "values": ["low", "medium", "high"]},
        {"name": "description", "type": "string"}
    ]
}
SAVE_FROM_UNSTRUCTURED support_ticket, schema, "tickets_table"
```

## Destinations

### Database Table
```basic
SAVE_FROM_UNSTRUCTURED text, "schema", "table_name"
' Saves to database table
```

### Variable
```basic
SAVE_FROM_UNSTRUCTURED text, "schema", extracted_data
' Saves to variable for further processing
```

### File
```basic
SAVE_FROM_UNSTRUCTURED text, "schema", "output.json"
' Saves as JSON file
```

### Multiple Destinations
```basic
' Extract once, save multiple places
data = EXTRACT_STRUCTURED(text, "schema")
SAVE_TO_DB data, "table"
SAVE_TO_FILE data, "backup.json"
SEND_TO_API data, "https://api.example.com/data"
```

## Return Value

Returns extraction result object:
- `success`: Boolean indicating success
- `extracted_count`: Number of records extracted
- `data`: Extracted structured data
- `confidence`: Extraction confidence score (0-1)
- `errors`: Any validation errors

## Advanced Features

### Batch Processing
```basic
documents = GET_ALL_FILES("inbox/*.txt")
FOR EACH doc IN documents
    SAVE_FROM_UNSTRUCTURED doc.content, "contract_schema", "contracts_table"
NEXT
```

### Validation Rules
```basic
schema_with_rules = {
    "fields": [...],
    "validation": {
        "email": "must be valid email",
        "phone": "must be 10 digits",
        "amount": "must be positive number"
    }
}
SAVE_FROM_UNSTRUCTURED text, schema_with_rules, destination
```

### Confidence Threshold
```basic
result = SAVE_FROM_UNSTRUCTURED text, schema, destination, min_confidence=0.8
IF result.confidence < 0.8 THEN
    ' Manual review needed
    SEND_FOR_REVIEW text, result
END IF
```

### Multi-Language Support
```basic
' Process text in different languages
spanish_text = "Nombre: Juan, Teléfono: 555-1234"
SAVE_FROM_UNSTRUCTURED spanish_text, "contact_schema", "contacts", language="es"
```

## Use Cases

### CRM Data Entry
```basic
' Auto-populate CRM from emails
email = GET_LATEST_EMAIL()
SAVE_FROM_UNSTRUCTURED email.body, "lead_schema", "crm_leads"
TALK "New lead added to CRM"
```

### Document Processing
```basic
' Process uploaded documents
document = UPLOAD_FILE()
text = EXTRACT_TEXT(document)
SAVE_FROM_UNSTRUCTURED text, detect_schema(document.type), "documents_db"
```

### Form Automation
```basic
' Convert free-text to form submission
user_input = HEAR "Describe your issue"
SAVE_FROM_UNSTRUCTURED user_input, "ticket_schema", "support_tickets"
ticket_id = GET_LAST_INSERT_ID()
TALK "Ticket #" + ticket_id + " created"
```

### Data Migration
```basic
' Migrate unstructured data to structured database
old_notes = GET_LEGACY_NOTES()
FOR EACH note IN old_notes
    SAVE_FROM_UNSTRUCTURED note, "modern_schema", "new_database"
NEXT
```

## Error Handling

```basic
TRY
    SAVE_FROM_UNSTRUCTURED text, schema, destination
CATCH "invalid_schema"
    LOG "Schema validation failed"
CATCH "extraction_failed"
    LOG "Could not extract required fields"
    ' Fall back to manual processing
    CREATE_MANUAL_TASK text
CATCH "save_failed"
    LOG "Database save failed"
    ' Save to backup location
    SAVE_TO_FILE text, "failed_extractions.log"
END TRY
```

## Performance Considerations

- Large texts are processed in chunks
- Extraction is cached for identical inputs
- Schema validation happens before AI processing
- Batch operations are optimized
- Background processing for large datasets

## Best Practices

1. **Define clear schemas**: Be specific about expected fields
2. **Validate important data**: Add validation rules for critical fields
3. **Set confidence thresholds**: Require human review for low confidence
4. **Handle errors gracefully**: Have fallback procedures
5. **Test with samples**: Verify extraction accuracy
6. **Monitor performance**: Track extraction success rates
7. **Document schemas**: Keep schema documentation updated
8. **Version schemas**: Track schema changes over time

## Limitations

- Accuracy depends on text quality
- Complex nested structures may need preprocessing
- Very long texts may need chunking
- Ambiguous data requires human review
- Processing time increases with text length

## Related Keywords

- [GET](./keyword-get.md) - Fetch unstructured content
- [FIND](./keyword-find.md) - Search extracted data
- [FORMAT](./keyword-format.md) - Format extracted data
- [LLM](./keyword-llm.md) - Process with language model

## Implementation

Located in `src/basic/keywords/save_from_unstructured.rs`

Uses LLM for intelligent extraction with schema validation and multiple storage backends.