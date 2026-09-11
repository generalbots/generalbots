' classify_media.bas - classify an inbound image or document and file it in Drive.
'
' Tool arguments injected by the runtime before this script runs: path, caption.
' Result layout: media/{year}/{month}/{category}/{file}
'
' The taxonomy is closed: the model only chooses one of the names below and any
' other answer degrades to "unsorted", so a misclassification can never create
' an arbitrary folder.

TAXONOMY = "invoice,receipt,contract,identity,report,unsorted"
PROMPT = "Classifique o conteudo a seguir com uma unica palavra, apenas uma destas: " + TAXONOMY + ". Responda somente a palavra.\n\n"
MAX_ANALYSIS_CHARS = 4000

' 1. Perception: images are described by the vision model, documents are read
'    through text extraction (PDF and office formats are both supported).
lower_path = LCASE(path)
is_image = 0
IF INSTR(lower_path, ".jpg") > 0 OR INSTR(lower_path, ".jpeg") > 0 OR INSTR(lower_path, ".png") > 0 OR INSTR(lower_path, ".webp") > 0 OR INSTR(lower_path, ".gif") > 0 THEN
    is_image = 1
END IF

content = ""
IF is_image = 1 THEN
    content = DESCRIBE IMAGE path
ELSE
    content = GET path
END IF

' 2. Classification: one closed-set decision over the perceived content.
prompt = PROMPT + LEFT(content, MAX_ANALYSIS_CHARS)
raw_answer = LLM prompt
answer = LCASE(TRIM(raw_answer))
answer = REPLACE(answer, ".", "")
answer = REPLACE(answer, ",", "")
answer = REPLACE(answer, ":", "")
IF INSTR(answer, " ") > 0 THEN
    answer = FIRST(SPLIT(answer, " "))
END IF

allowed = SPLIT(TAXONOMY, ",")
category = "unsorted"
FOR EACH candidate IN allowed
    IF candidate = answer THEN
        category = candidate
    END IF
NEXT

' 3. Filing: keep the original leaf name and let the object store create the
'    folder prefix implicitly.
parts = SPLIT(path, "/")
leaf = LAST(parts)

' The date is bound to a variable first: TODAY is a map, and property access on
' a variable is the form used everywhere else in a bot script.
today = TODAY
month = STR(today.month)
IF LEN(month) = 1 THEN
    month = "0" + month
END IF

destination = "media/" + STR(today.year) + "/" + month + "/" + category + "/" + leaf
MOVE path, destination

' 4. Audit trail next to the filed item: keeps the decision reproducible
'    without a read-modify-write over a shared index file.
CREATE FILE destination + ".meta.txt" WITH "category=" + category + "\n" + "path=" + destination + "\n" + "caption=" + caption

TALK "Arquivo classificado como " + category + " e arquivado em " + destination
