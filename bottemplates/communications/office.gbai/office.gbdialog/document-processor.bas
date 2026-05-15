' Document Processor Bot - Demonstrates file operations keywords
' This template shows how to use READ, WRITE, COPY, MOVE, LIST, COMPRESS, EXTRACT, etc.

' ============================================================================
' WEBHOOK: External systems can trigger document processing via HTTP POST
' Endpoint: /api/office/webhook/process-documents
' ============================================================================
WEBHOOK "process-documents"

TALK "Document Processor initialized..."

' ============================================================================
' EXAMPLE 1: Reading and writing files
' ============================================================================

' Read a configuration file
config_content = READ "config/settings.json"
TALK "Loaded configuration file"

' Read a text report
daily_report = READ "reports/daily-summary.txt"

' Write processed data to a new file
processed_data = "Processed at: " + NOW() + "\n"
processed_data = processed_data + "Original content length: " + LEN(daily_report) + " characters\n"
processed_data = processed_data + "Status: Complete\n"

WRITE "reports/processed/" + FORMAT(TODAY(), "yyyy-MM-dd") + "-log.txt", processed_data
TALK "Processing log created"

' Write JSON data
summary_json = #{
    "date": TODAY(),
    "files_processed": 5,
    "total_size_kb": 1250,
    "status": "success"
}
WRITE "reports/summary.json", summary_json

' ============================================================================
' EXAMPLE 2: Listing directory contents
' ============================================================================

' List all files in the inbox folder
inbox_files = LIST "inbox/"
TALK "Found " + UBOUND(inbox_files) + " files in inbox"

' Process each file in the inbox
FOR EACH file IN inbox_files
    TALK "Processing: " + file
NEXT file

' List reports folder
report_files = LIST "reports/"
TALK "Total reports in archive: " + UBOUND(report_files)

' List with subdirectory
template_files = LIST "templates/documents/"

' ============================================================================
' EXAMPLE 3: Copying files
' ============================================================================

' Copy a template for a new customer
customer_name = "acme-corp"
COPY "templates/invoice-template.docx", "customers/" + customer_name + "/invoice-draft.docx"
TALK "Invoice template copied for " + customer_name

' Copy to backup location
COPY "data/important-data.xlsx", "backups/" + FORMAT(TODAY(), "yyyy-MM-dd") + "-data-backup.xlsx"
TALK "Backup created"

' Copy multiple files using a loop
template_types = ["contract", "nda", "proposal"]
FOR EACH template_type IN template_types
    COPY "templates/" + template_type + ".docx", "customers/" + customer_name + "/" + template_type + ".docx"
NEXT template_type
TALK "All templates copied for customer"

' ============================================================================
' EXAMPLE 4: Moving and renaming files
' ============================================================================

' Move processed files from inbox to processed folder
FOR EACH file IN inbox_files
    ' Extract just the filename from the path
    filename = file
    MOVE "inbox/" + filename, "processed/" + FORMAT(TODAY(), "yyyy-MM-dd") + "-" + filename
NEXT file
TALK "Inbox files moved to processed folder"

' Rename a file (move within same directory)
MOVE "drafts/report-v1.docx", "drafts/report-final.docx"
TALK "Report renamed to final version"

' Archive old files
old_reports = LIST "reports/2023/"
FOR EACH old_report IN old_reports
    MOVE "reports/2023/" + old_report, "archive/2023/" + old_report
NEXT old_report

' ============================================================================
' EXAMPLE 5: Deleting files
' ============================================================================

' Delete temporary files
temp_files = LIST "temp/"
FOR EACH temp_file IN temp_files
    DELETE_FILE "temp/" + temp_file
NEXT temp_file
TALK "Temporary files cleaned up"

' Delete a specific file
DELETE_FILE "cache/old-cache.dat"

' Delete processed inbox files older than 30 days
' (In real usage, you'd check file dates)
DELETE_FILE "processed/old-file.txt"

' ============================================================================
' EXAMPLE 6: Creating ZIP archives
' ============================================================================

' Compress monthly reports into a single archive
monthly_reports = [
    "reports/week1.pdf",
    "reports/week2.pdf",
    "reports/week3.pdf",
    "reports/week4.pdf",
    "reports/summary.xlsx"
]

archive_name = "archives/monthly-" + FORMAT(TODAY(), "yyyy-MM") + ".zip"
COMPRESS monthly_reports, archive_name
TALK "Monthly reports compressed to: " + archive_name

' Compress customer documents
customer_docs = LIST "customers/" + customer_name + "/"
customer_archive = "archives/customers/" + customer_name + "-documents.zip"
COMPRESS customer_docs, customer_archive
TALK "Customer documents archived"

' ============================================================================
' EXAMPLE 7: Extracting archives
' ============================================================================

' Extract uploaded archive
uploaded_archive = "uploads/new-documents.zip"
extracted_files = EXTRACT uploaded_archive, "inbox/extracted/"
TALK "Extracted " + UBOUND(extracted_files) + " files from archive"

' Process extracted files
FOR EACH extracted_file IN extracted_files
    TALK "Extracted: " + extracted_file
NEXT extracted_file

' Extract to specific destination
EXTRACT "imports/data-import.zip", "data/imported/"

' ============================================================================
' EXAMPLE 8: Upload and download operations
' ============================================================================

' Download a file from external URL
external_url = "https://example.com/reports/external-report.pdf"
local_path = "downloads/external-report-" + FORMAT(TODAY(), "yyyy-MM-dd") + ".pdf"
downloaded_path = DOWNLOAD external_url, local_path
TALK "Downloaded external report to: " + downloaded_path

' Download multiple files
download_urls = [
    "https://api.example.com/exports/data1.csv",
    "https://api.example.com/exports/data2.csv",
    "https://api.example.com/exports/data3.csv"
]

counter = 1
FOR EACH url IN download_urls
    DOWNLOAD url, "imports/data-" + counter + ".csv"
    counter = counter + 1
NEXT url
TALK "Downloaded " + UBOUND(download_urls) + " data files"

' Upload a file to storage
HEAR attachment AS FILE
IF attachment != "" THEN
    upload_destination = "uploads/" + attachment.filename
    upload_url = UPLOAD attachment, upload_destination
    TALK "File uploaded to: " + upload_url
END IF

' ============================================================================
' EXAMPLE 9: PDF Generation
' ============================================================================

' Generate an invoice PDF from template
invoice_data = #{
    "invoice_number": "INV-2024-001",
    "customer_name": "Acme Corporation",
    "customer_address": "123 Business Ave, Suite 100",
    "date": FORMAT(TODAY(), "MMMM dd, yyyy"),
    "due_date": FORMAT(DATEADD(TODAY(), "day", 30), "MMMM dd, yyyy"),
    "items": [
        #{ "description": "Consulting Services", "quantity": 10, "rate": 150, "amount": 1500 },
        #{ "description": "Software License", "quantity": 1, "rate": 500, "amount": 500 },
        #{ "description": "Support Package", "quantity": 1, "rate": 200, "amount": 200 }
    ],
    "subtotal": 2200,
    "tax": 176,
    "total": 2376
}

invoice_pdf = GENERATE_PDF "templates/invoice.html", invoice_data, "invoices/INV-2024-001.pdf"
TALK "Invoice PDF generated: " + invoice_pdf.url

' Generate a report PDF
report_data = #{
    "title": "Monthly Performance Report",
    "period": FORMAT(TODAY(), "MMMM yyyy"),
    "author": "Office Bot",
    "generated_at": NOW(),
    "metrics": #{
        "total_sales": 125000,
        "new_customers": 45,
        "satisfaction_score": 4.7
    }
}

report_pdf = GENERATE_PDF "templates/report.html", report_data, "reports/monthly-" + FORMAT(TODAY(), "yyyy-MM") + ".pdf"
TALK "Report PDF generated: " + report_pdf.url

' ============================================================================
' EXAMPLE 10: Merging PDF files
' ============================================================================

' Merge multiple PDFs into a single document
pdfs_to_merge = [
    "documents/cover-page.pdf",
    "documents/table-of-contents.pdf",
    "documents/chapter1.pdf",
    "documents/chapter2.pdf",
    "documents/chapter3.pdf",
    "documents/appendix.pdf"
]

merged_pdf = MERGE_PDF pdfs_to_merge, "publications/complete-manual.pdf"
TALK "PDFs merged into: " + merged_pdf.url

' Merge customer documents for a single package
customer_pdfs = [
    "customers/" + customer_name + "/contract.pdf",
    "customers/" + customer_name + "/terms.pdf",
    "customers/" + customer_name + "/invoice.pdf"
]

customer_package = MERGE_PDF customer_pdfs, "customers/" + customer_name + "/welcome-package.pdf"
TALK "Customer welcome package created"

' ============================================================================
' EXAMPLE 11: Document workflow automation
' ============================================================================

TALK "Starting document workflow..."

' Step 1: Check inbox for new documents
new_docs = LIST "inbox/"
doc_count = UBOUND(new_docs)

IF doc_count > 0 THEN
    TALK "Processing " + doc_count + " new documents"

    ' Step 2: Copy originals to backup
    FOR EACH doc IN new_docs
        COPY "inbox/" + doc, "backups/inbox/" + FORMAT(TODAY(), "yyyy-MM-dd") + "-" + doc
    NEXT doc

    ' Step 3: Move to processing folder
    FOR EACH doc IN new_docs
        MOVE "inbox/" + doc, "processing/" + doc
    NEXT doc

    ' Step 4: Process documents (simplified)
    processing_docs = LIST "processing/"
    processed_list = []

    FOR EACH doc IN processing_docs
        ' Read and process
        content = READ "processing/" + doc

        ' Write processed version
        WRITE "completed/" + doc, content

        ' Track processed file
        processed_list = processed_list + [doc]

        ' Clean up processing folder
        DELETE_FILE "processing/" + doc
    NEXT doc

    ' Step 5: Archive completed documents
    IF UBOUND(processed_list) > 0 THEN
        COMPRESS processed_list, "archives/batch-" + FORMAT(NOW(), "yyyy-MM-dd-HHmm") + ".zip"
    END IF

    TALK "Workflow complete: " + UBOUND(processed_list) + " documents processed"
ELSE
    TALK "No new documents to process"
END IF

' ============================================================================
' Return webhook response with summary
' ============================================================================

result = #{
    "status": "success",
    "timestamp": NOW(),
    "documents_processed": doc_count,
    "operations": #{
        "files_read": 5,
        "files_written": 8,
        "files_copied": 6,
        "files_moved": doc_count,
        "archives_created": 2,
        "pdfs_generated": 2,
        "pdfs_merged": 2
    }
}

TALK "Document processing complete!"
