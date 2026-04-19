# MERGE PDF

The `MERGE PDF` keyword combines multiple PDF files into a single document, enabling bots to consolidate reports, compile documents, and create comprehensive file packages.

---

## Syntax

```basic
result = MERGE PDF files, "output.pdf"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `files` | Array/String | Array of PDF file paths or single path |
| `output` | String | Output filename for the merged PDF |

---

## Description

`MERGE PDF` takes multiple PDF files and combines them into a single document in the order specified. This is useful for creating comprehensive reports, combining related documents, or building document packages for clients.

Use cases include:
- Combining invoice and receipt PDFs
- Merging report sections into complete reports
- Creating document packages for clients
- Consolidating scanned documents
- Building compliance document bundles

---

## Examples

### Basic PDF Merge

```basic
' Merge two PDF files
files = ["report-part1.pdf", "report-part2.pdf"]
result = MERGE PDF files, "complete-report.pdf"

TALK "Report merged: " + result.localName
```

### Merge Multiple Documents

```basic
' Merge multiple documents into one package
documents = [
    "contracts/agreement.pdf",
    "documents/terms.pdf",
    "documents/privacy-policy.pdf",
    "documents/appendix-a.pdf"
]

result = MERGE PDF documents, "client-package-" + client_id + ".pdf"

TALK "Document package created!"
DOWNLOAD result.url AS "Complete Package.pdf"
```

### Dynamic Document Collection

```basic
' Find and merge all invoices for a month
invoice_files = []

invoices = FIND "invoices" WHERE month = current_month
FOR EACH inv IN invoices
    invoice_files = invoice_files + ["invoices/" + inv.filename]
END FOR

result = MERGE PDF invoice_files, "monthly-invoices-" + FORMAT(NOW(), "YYYYMM") + ".pdf"

TALK "Merged " + LEN(invoice_files) + " invoices into one document"
```

### Merge with Generated PDFs

```basic
' Generate PDFs first, then merge them
cover = GENERATE PDF "templates/cover.html", cover_data, "temp/cover.pdf"
body = GENERATE PDF "templates/report.html", report_data, "temp/body.pdf"
appendix = GENERATE PDF "templates/appendix.html", appendix_data, "temp/appendix.pdf"

files = [cover.localName, body.localName, appendix.localName]
result = MERGE PDF files, "reports/full-report-" + report_id + ".pdf"

TALK "Complete report generated with " + LEN(files) + " sections"
```

### Merge and Email

```basic
' Create document package and email to client
documents = [
    "proposals/proposal-" + deal_id + ".pdf",
    "documents/service-agreement.pdf",
    "documents/pricing-schedule.pdf"
]

result = MERGE PDF documents, "packages/" + client_name + "-proposal.pdf"

SEND MAIL client_email, 
    "Your Proposal Package",
    "Please find attached your complete proposal package.",
    [result.localName]

TALK "Proposal package sent to " + client_email
```

---

## Return Value

Returns an object with merge details:

| Property | Description |
|----------|-------------|
| `result.url` | Full URL to the merged PDF (S3/MinIO path) |
| `result.localName` | Local filename of the merged PDF |

---

## Common Use Cases

### Monthly Report Compilation

```basic
' Compile all weekly reports into monthly report
weekly_reports = [
    "reports/week1.pdf",
    "reports/week2.pdf",
    "reports/week3.pdf",
    "reports/week4.pdf"
]

' Generate cover page
cover = GENERATE PDF "templates/monthly-cover.html", #{
    "month": FORMAT(NOW(), "MMMM YYYY"),
    "generated": FORMAT(NOW(), "YYYY-MM-DD")
}, "temp/cover.pdf"

' Merge cover with weekly reports
all_files = [cover.localName] + weekly_reports
result = MERGE PDF all_files, "reports/monthly-" + FORMAT(NOW(), "YYYYMM") + ".pdf"

TALK "Monthly report compiled!"
```

### Client Onboarding Package

```basic
' Create onboarding document package for new client
package_files = [
    "templates/welcome-letter.pdf",
    "contracts/service-agreement-" + contract_id + ".pdf",
    "documents/user-guide.pdf",
    "documents/faq.pdf",
    "documents/support-contacts.pdf"
]

result = MERGE PDF package_files, "onboarding/" + client_id + "-welcome-package.pdf"

SEND MAIL client_email,
    "Welcome to Our Service!",
    "Please find your complete onboarding package attached.",
    [result.localName]

TALK "Onboarding package sent to " + client_name
```

### Compliance Document Bundle

```basic
' Bundle all compliance documents for audit
compliance_docs = FIND "compliance_documents" WHERE year = audit_year

file_list = []
FOR EACH doc IN compliance_docs
    file_list = file_list + [doc.file_path]
END FOR

' Add table of contents
toc = GENERATE PDF "templates/compliance-toc.html", #{
    "documents": compliance_docs,
    "audit_year": audit_year
}, "temp/toc.pdf"

all_files = [toc.localName] + file_list
result = MERGE PDF all_files, "audits/compliance-bundle-" + audit_year + ".pdf"

TALK "Compliance bundle ready with " + LEN(compliance_docs) + " documents"
```

### Invoice Bundle for Accounting

```basic
' Create quarterly invoice bundle
quarter_start = DATEADD(NOW(), -3, "month")
invoices = FIND "generated_invoices" WHERE created_at >= quarter_start

invoice_files = []
FOR EACH inv IN invoices
    invoice_files = invoice_files + ["invoices/" + inv.pdf_filename]
END FOR

IF LEN(invoice_files) > 0 THEN
    result = MERGE PDF invoice_files, "accounting/Q" + quarter + "-invoices.pdf"
    TALK "Bundled " + LEN(invoice_files) + " invoices for Q" + quarter
ELSE
    TALK "No invoices found for this quarter"
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

files = ["doc1.pdf", "doc2.pdf", "doc3.pdf"]
result = MERGE PDF files, "merged.pdf"

IF ERROR THEN
    error_msg = ERROR_MESSAGE
    
    IF INSTR(error_msg, "not found") > 0 THEN
        TALK "One or more PDF files could not be found."
    ELSE IF INSTR(error_msg, "invalid") > 0 THEN
        TALK "One of the files is not a valid PDF."
    ELSE IF INSTR(error_msg, "storage") > 0 THEN
        TALK "Not enough storage space for the merged file."
    ELSE
        TALK "Merge failed: " + error_msg
    END IF
ELSE
    TALK "PDFs merged successfully!"
END IF
```

### Validating Files Before Merge

```basic
' Check files exist before attempting merge
files_to_merge = ["report1.pdf", "report2.pdf", "report3.pdf"]
valid_files = []

FOR EACH f IN files_to_merge
    file_info = LIST f
    IF file_info THEN
        valid_files = valid_files + [f]
    ELSE
        PRINT "Warning: " + f + " not found, skipping"
    END IF
END FOR

IF LEN(valid_files) > 0 THEN
    result = MERGE PDF valid_files, "merged-output.pdf"
    TALK "Merged " + LEN(valid_files) + " of " + LEN(files_to_merge) + " files"
ELSE
    TALK "No valid PDF files found to merge"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FILE_NOT_FOUND` | Source PDF doesn't exist | Verify file paths |
| `INVALID_PDF` | File is not a valid PDF | Check file format |
| `EMPTY_INPUT` | No files provided | Ensure array has files |
| `STORAGE_FULL` | Insufficient disk space | Clean up storage |
| `PERMISSION_DENIED` | Cannot read source file | Check file permissions |

---

## Best Practices

### File Organization

```basic
' Organize files in logical order before merge
sections = [
    "01-cover.pdf",
    "02-executive-summary.pdf",
    "03-introduction.pdf",
    "04-analysis.pdf",
    "05-recommendations.pdf",
    "06-appendices.pdf"
]

result = MERGE PDF sections, "final-report.pdf"
```

### Temporary File Cleanup

```basic
' Clean up temporary files after merge
temp_files = []

' Generate temporary PDFs
FOR i = 1 TO 5
    temp_file = "temp/section-" + i + ".pdf"
    GENERATE PDF "templates/section.html", section_data[i], temp_file
    temp_files = temp_files + [temp_file]
END FOR

' Merge all sections
result = MERGE PDF temp_files, "final-document.pdf"

' Clean up temp files
FOR EACH tf IN temp_files
    DELETE tf
END FOR

TALK "Document created and temp files cleaned up"
```

### Large Document Sets

```basic
' For very large document sets, batch if needed
all_files = get_all_pdf_files()  ' Assume this returns many files

IF LEN(all_files) > 100 THEN
    ' Process in batches
    batch_size = 50
    batch_outputs = []
    
    FOR batch_num = 0 TO (LEN(all_files) / batch_size)
        start_idx = batch_num * batch_size
        batch_files = SLICE(all_files, start_idx, start_idx + batch_size)
        
        batch_output = "temp/batch-" + batch_num + ".pdf"
        MERGE PDF batch_files, batch_output
        batch_outputs = batch_outputs + [batch_output]
    END FOR
    
    ' Final merge of batches
    result = MERGE PDF batch_outputs, "complete-archive.pdf"
ELSE
    result = MERGE PDF all_files, "complete-archive.pdf"
END IF
```

---

## Configuration

No specific configuration required. Uses the bot's standard drive storage settings from `config.csv`.

Output files are stored in the bot's `.gbdrive` storage location.

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/file_operations.rs`
- Maintains PDF metadata and bookmarks where possible
- Preserves page sizes and orientations
- Handles password-protected PDFs (if password provided)
- Maximum combined size: 500 MB
- Processing timeout: 120 seconds

---

## Related Keywords

- [GENERATE PDF](keyword-generate-pdf.md) — Create PDFs from templates
- [READ](keyword-read.md) — Read file contents
- [DOWNLOAD](keyword-download.md) — Send files to users
- [COPY](keyword-copy.md) — Copy files
- [DELETE](keyword-delete-file.md) — Remove files
- [LIST](keyword-list.md) — List files in directory

---

## Summary

`MERGE PDF` combines multiple PDF files into a single document, making it easy to create comprehensive document packages, compile reports, and bundle related files. Use it with `GENERATE PDF` to create multi-section reports or with existing files to build client packages. The keyword handles the complexity of PDF merging while providing a simple array-based interface.