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
