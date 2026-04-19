PARAM doc AS QRCODE LIKE "photo of QR code" DESCRIPTION "QR Code image to scan and load document"

DESCRIPTION "Scan a QR Code to load and query a document"

text = GET doc

IF text THEN
    SET CONTEXT "Based on this document, answer the person's questions:\n\n" + text
    TALK "Document ${doc} loaded. You can ask me anything about it."
    SEND FILE doc
ELSE
    TALK "Document not found, please try again."
END IF
