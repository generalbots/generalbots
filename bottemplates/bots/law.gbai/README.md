# Legal Document Processing Template (law.gbai)

A General Bots template for legal case management, document analysis, and AI-powered legal Q&A.

## Overview

The Law template enables legal professionals to load case documents, query case information using natural language, and get AI-assisted analysis of legal materials. It's designed for law firms, legal departments, and compliance teams who need efficient document-based legal research.

## Features

- **Case Document Loading** - Load and index PDF case files by case number
- **Document Q&A** - Ask questions about loaded legal documents
- **Context-Aware Responses** - AI responses grounded in actual case content
- **Case Summarization** - Generate summaries of case documents
- **Multi-Document Support** - Work with multiple cases simultaneously
- **Audit Trail** - Track all document access and queries

## Package Structure

```
law.gbai/
├── README.md
├── law.gbdialog/
│   └── case.bas           # Case loading and query handler
├── law.gbdrive/           # Case document storage
│   └── (case-*.pdf)       # Case PDF files
├── law.gbkb/              # Knowledge base for legal reference
└── law.gbot/
    └── config.csv         # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `case.bas` | Loads case documents by number and enables document-based Q&A |

## How It Works

### Case Loading Flow

1. User provides a case number (e.g., "12345")
2. Bot searches for `case-12345.pdf` in the drive
3. Document content is extracted and set as context
4. Answer mode switches to "document" for grounded responses
5. User can ask questions about the loaded case

### Code Example

```basic
PARAM cod AS STRING LIKE "12345" DESCRIPTION "Case number to load and query"

DESCRIPTION "Load a legal case document by case number for Q&A and analysis"

text = GET "case-" + cod + ".pdf"

IF text THEN
    SET CONTEXT "Based on this document, answer the person's questions:\n\n" + text
    SET ANSWER MODE "document"
    TALK "Case ${cod} loaded. Ask me anything about the case or request a summary."
ELSE
    TALK "Case not found. Please check the case number."
END IF
```

## Configuration

Configure in `law.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Answer Mode` | Response grounding mode | `document` |
| `Theme Color` | UI accent color | `navy` |
| `LLM Provider` | AI provider | `openai` |
| `Document Retention` | Days to keep loaded docs | `30` |
| `Require Authentication` | Require user login | `true` |

## Usage

### Loading a Case

```
User: Load case 12345
Bot: Case 12345 loaded. Ask me anything about the case or request a summary.
```

### Querying Case Content

```
User: What are the main claims in this case?
Bot: Based on the document, the main claims are:
     1. Breach of contract regarding the delivery terms in Section 3.2
     2. Failure to meet quality standards as specified in Exhibit A
     3. Request for damages in the amount of $250,000

User: Summarize the defendant's response
Bot: The defendant argues that:
     - Delivery delays were caused by force majeure events
     - Quality standards were met according to third-party inspection
     - Damages claimed are excessive and not substantiated
```

### Multiple Cases

```
User: Load case 67890
Bot: Case 67890 loaded. Previous case context cleared.

User: Compare this with case 12345
Bot: I can only analyze the currently loaded case (67890). 
     To compare cases, please use the case comparison feature.
```

## Document Naming Convention

Case documents should follow this naming pattern:

```
case-{case_number}.pdf
```

Examples:
- `case-12345.pdf`
- `case-2024-CV-0001.pdf`
- `case-APPEAL-789.pdf`

## Customization

### Adding Case Search

```basic
' search-cases.bas
PARAM query AS STRING DESCRIPTION "Search term for finding cases"

DESCRIPTION "Search for cases by party name, date, or keywords"

cases = FIND "case_index.csv", "content LIKE '%" + query + "%'"

IF UBOUND(cases) > 0 THEN
    TALK "Found " + UBOUND(cases) + " matching cases:"
    
    FOR EACH c IN cases
        TALK "- Case " + c.case_number + ": " + c.title
    NEXT
ELSE
    TALK "No cases found matching: " + query
END IF
```

### Case Summarization

```basic
' summarize-case.bas
PARAM cod AS STRING DESCRIPTION "Case number to summarize"

DESCRIPTION "Generate an executive summary of a legal case"

text = GET "case-" + cod + ".pdf"

IF text THEN
    summary = LLM "As a legal professional, provide an executive summary of this case including: 
                   1. Parties involved
                   2. Key facts
                   3. Legal issues
                   4. Current status
                   5. Next steps
                   
                   Document: " + text
    
    TALK "## Case " + cod + " Summary\n\n" + summary
    
    ' Save summary for future reference
    SAVE "case_summaries.csv", cod, summary, NOW()
ELSE
    TALK "Case not found."
END IF
```

### Adding Document Types

Support additional legal document types:

```basic
' load-document.bas
PARAM doc_type AS STRING LIKE "contract" DESCRIPTION "Type: case, contract, brief, motion"
PARAM doc_id AS STRING DESCRIPTION "Document identifier"

DESCRIPTION "Load various legal document types"

filename = doc_type + "-" + doc_id + ".pdf"
text = GET filename

IF text THEN
    SET CONTEXT "This is a legal " + doc_type + ". Answer questions based on its content:\n\n" + text
    SET ANSWER MODE "document"
    TALK "Loaded " + doc_type + " " + doc_id + ". Ready for questions."
ELSE
    TALK "Document not found: " + filename
END IF
```

### Compliance Logging

```basic
' Add audit logging to case.bas
IF text THEN
    ' Log access for compliance
    WITH audit_entry
        timestamp = NOW()
        user = GET SESSION "user_email"
        case_number = cod
        action = "document_access"
        ip_address = GET SESSION "client_ip"
    END WITH
    
    SAVE "legal_audit_log.csv", audit_entry
    
    SET CONTEXT "Based on this document..." + text
END IF
```

## Integration Examples

### With Calendar

```basic
' Schedule case deadlines
deadline = LLM "Extract the next deadline date from this case: " + text

IF deadline THEN
    CREATE CALENDAR EVENT "Case " + cod + " Deadline", deadline
    TALK "Deadline added to calendar: " + deadline
END IF
```

### With Email

```basic
' Email case summary to team
summary = LLM "Summarize the key points of this case in 3 paragraphs: " + text

SEND EMAIL "legal-team@firm.com", "Case " + cod + " Summary", summary
TALK "Summary sent to legal team."
```

### With Document Generation

```basic
' Generate response document
response = LLM "Draft a formal response letter addressing the claims in this case: " + text

CREATE DRAFT response, "Response to Case " + cod
TALK "Draft response created. Review in your documents."
```

## Security Considerations

1. **Access Control** - Implement role-based access for sensitive cases
2. **Audit Logging** - Log all document access for compliance
3. **Data Encryption** - Enable encryption for stored documents
4. **Session Timeout** - Configure appropriate session timeouts
5. **Authentication** - Require strong authentication for legal systems
6. **Data Retention** - Follow legal data retention requirements

## Best Practices

1. **Organize documents** - Use consistent naming conventions
2. **Index cases** - Maintain a searchable case index
3. **Regular backups** - Back up case documents frequently
4. **Version control** - Track document versions
5. **Clear context** - Clear previous case context before loading new cases
6. **Verify AI responses** - Always verify AI-generated legal content

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Case not found | Wrong filename | Check naming convention |
| Empty responses | Document not parsed | Verify PDF is text-based |
| Slow loading | Large document | Consider document chunking |
| Context errors | Multiple cases loaded | Clear context between cases |
| Access denied | Missing permissions | Check user authentication |

## Limitations

- PDF documents must be text-based (not scanned images)
- Very large documents may require chunking
- Complex legal analysis should be verified by professionals
- AI responses are assistive, not legal advice

## Related Templates

- `compliance/privacy.gbai` - Privacy and data protection
- `compliance/hipaa-medical.gbai` - Healthcare compliance
- `talk-to-data.gbai` - Natural language document queries
- `ai-search.gbai` - AI-powered document search

## Use Cases

- **Case Research** - Quickly find relevant information in case files
- **Document Review** - AI-assisted document analysis
- **Client Communication** - Generate case status summaries
- **Deadline Tracking** - Extract and track important dates
- **Knowledge Management** - Build searchable legal knowledge bases

## Disclaimer

This template provides AI-assisted document analysis tools. It does not constitute legal advice. All AI-generated content should be reviewed by qualified legal professionals. Users are responsible for ensuring compliance with applicable legal and ethical standards.

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots