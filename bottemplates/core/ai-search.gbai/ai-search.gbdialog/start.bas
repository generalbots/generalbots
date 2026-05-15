ADD TOOL "qr"
USE KB "sheetlib"

CLEAR SUGGESTIONS
ADD SUGGESTION "scan" AS "Scan a QR Code"
ADD SUGGESTION "find" AS "Find a procedure"
ADD SUGGESTION "help" AS "How to search documents"
ADD SUGGESTION "test kb" AS "Test KB injection"

BEGIN TALK
General Bots AI Search

Comprehensive Document Search with AI summaries and EDM integration.

**Options:**
• Scan a QR Code - Send a photo to scan
• Find a Procedure - Ask about any process
• Test KB - Verify sheetlib knowledge base injection

**Examples:**
- How to send a fax?
- How to clean the machine?
- How to find a contact?
- What is in the sheetlib KB?
END TALK

BEGIN SYSTEM PROMPT
You are a document search assistant. Help users find procedures and information from documents.
When users want to scan QR codes, use the qr tool.
Provide clear, concise answers based on document content.
END SYSTEM PROMPT
