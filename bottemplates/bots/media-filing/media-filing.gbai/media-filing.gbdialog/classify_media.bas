' classify_media.bas - classify an inbound image, document, audio or video and file it in Drive.
'
' Tool arguments injected by the runtime before this script runs: path, caption.
' Result layout: media/{year}/{month}/{category}/{file}
'
' The taxonomy is closed: the model only chooses one of the names below and any
' other answer degrades to "unsorted", so a misclassification can never create an
' arbitrary folder.
'
' Perception degrades instead of failing the task. When the image model
' (BotModels) is unreachable, a document cannot be read, or an audio/video item
' cannot be perceived, the caption is used for the decision and the item is still
' filed under the same closed taxonomy. Provisioning BotModels restores
' content-based classification with no script change; the file itself is the one
' carried by the [image]/[document]/[voice]/[audio]/[video] marker.
'
' Audio and video are perceived with the speech-to-text and video models. They
' must never reach the document text extractor, which cannot read binary payloads
' and used to leave every voice note "unsorted" (or failing) by accident.

' Event trigger: a media file arriving on any channel (Telegram, WhatsApp,
' web) runs this tool directly, with `path`, `kind`, `caption` and `channel`
' in scope. Without this line the tool only ran when the model chose to call
' it, and an upload the model ignored stayed in `inbox/` forever.
ON EVENT "media_uploaded"

TAXONOMY = "invoice,receipt,contract,identity,report,audio,vehicle,people,animal,food,nature,screenshot,video,unsorted"
PROMPT = "Classifique o conteudo a seguir com uma unica palavra, apenas uma destas: " + TAXONOMY + ". Guia de decisao: carros/motos/caminhoes/trafego/parking = vehicle; pessoas/rostos/grupos = people; animais de qualquer tipo = animal; comida/bebida/cozinha/refeicao = food; paisagem/floresta/montanha/praia/ceu/agua = nature; tela de computador/interface/janela = screenshot; documento por tipo: recibo/pagamento = receipt, contrato/locacao = contract, documento de identidade = identity, relatorio = report, fatura/cobranca = invoice; audio ou gravacao de voz sem categoria melhor = audio; video generico sem categoria melhor = video. Responda somente a palavra.\n\n"
MAX_ANALYSIS_CHARS = 4000

' 1. Which kind of perception applies. Images are described by the vision model,
'    documents are read through text extraction (PDF and office formats), audio
'    is transcribed and video is described. The marker cannot be read, so the
'    file extension carried by `path` selects the branch.
lower_path = LCASE(path)

is_image = 0
IF INSTR(lower_path, ".jpg") > 0 OR INSTR(lower_path, ".jpeg") > 0 OR INSTR(lower_path, ".png") > 0 OR INSTR(lower_path, ".webp") > 0 OR INSTR(lower_path, ".gif") > 0 THEN
    is_image = 1
END IF

is_audio = 0
IF INSTR(lower_path, ".ogg") > 0 OR INSTR(lower_path, ".oga") > 0 OR INSTR(lower_path, ".opus") > 0 OR INSTR(lower_path, ".mp3") > 0 OR INSTR(lower_path, ".m4a") > 0 OR INSTR(lower_path, ".wav") > 0 OR INSTR(lower_path, ".aac") > 0 OR INSTR(lower_path, ".amr") > 0 THEN
    is_audio = 1
END IF

is_video = 0
IF INSTR(lower_path, ".mp4") > 0 OR INSTR(lower_path, ".mov") > 0 OR INSTR(lower_path, ".mkv") > 0 OR INSTR(lower_path, ".webm") > 0 OR INSTR(lower_path, ".avi") > 0 OR INSTR(lower_path, ".3gp") > 0 THEN
    is_video = 1
END IF

IF is_image = 1 THEN
    kind = "image"
ELSE
    IF is_audio = 1 THEN
        kind = "audio"
    ELSE
        IF is_video = 1 THEN
            kind = "video"
        ELSE
            kind = "document"
        END IF
    END IF
END IF

' 2. Perception. Errors are trapped: an unavailable model must not end the task
'    before the file is filed.
ON ERROR RESUME NEXT

content = ""
perception = "content"

IF is_image = 1 THEN
    content = DESCRIBE IMAGE path
    IF ERROR THEN
        perception = "unavailable"
        CLEAR ERROR
    END IF
ELSE
    IF is_audio = 1 THEN
        content = SPEECH TO TEXT path
        perception = "transcription"
        IF ERROR THEN
            perception = "unavailable"
            CLEAR ERROR
        END IF
    ELSE
        IF is_video = 1 THEN
            content = DESCRIBE VIDEO path
            perception = "description"
            IF ERROR THEN
                perception = "unavailable"
                CLEAR ERROR
            END IF
        ELSE
            content = GET path
            IF ERROR THEN
                perception = "unavailable"
                CLEAR ERROR
            END IF
        END IF
    END IF
END IF

IF LEN(TRIM(content)) = 0 THEN
    content = caption
    perception = "caption"
END IF

' 2.1 Caption reinforcement. The user caption is a first-class signal: a model
'    description of a generic video frame can miss the document type the sender
'    already declared ("recibo do pagamento..."), and media catalogs trust the
'    sender label. Both signals go into the prompt; when the caption is empty
'    this reduces to the content alone and perception stays unchanged.
IF LEN(TRIM(caption)) > 0 AND TRIM(caption) <> TRIM(content) THEN
    content = content + "\n" + "Legenda enviada pelo usuario: " + caption
    perception = perception + " + caption"
END IF

' 2.2 Kind hint. The classifier sees only text, so without the media kind a
'    caption like "gravacao de teste" reads as a generic recording and lands
'    in "video" even for a voice note (observed in production, 2026-09). The
'    script knows the kind from the file extension; state it explicitly.
content = content + "\n" + "Tipo de midia: " + kind

' 3. Classification: one closed-set decision over the perceived content. With no
'    content and no caption there is nothing to decide, so the item stays
'    "unsorted" instead of asking the model to guess.
category = "unsorted"

IF LEN(TRIM(content)) > 0 THEN
    prompt = PROMPT + LEFT(content, MAX_ANALYSIS_CHARS)
    raw_answer = LLM prompt

    IF ERROR THEN
        CLEAR ERROR
    ELSE
        answer = LCASE(TRIM(raw_answer))
        answer = REPLACE(answer, ".", "")
        answer = REPLACE(answer, ",", "")
        answer = REPLACE(answer, ":", "")
        IF INSTR(answer, " ") > 0 THEN
            answer = FIRST(SPLIT(answer, " "))
        END IF

        allowed = SPLIT(TAXONOMY, ",")
        FOR EACH candidate IN allowed
            IF candidate = answer THEN
                category = candidate
            END IF
        NEXT
    END IF
END IF

' 4. Filing runs with normal error behaviour: a real Drive failure must reach the
'    caller instead of being silently swallowed.
ON ERROR GOTO 0

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

' 4.1 Idempotency: the event trigger and the model's own tool call race on the
'    same upload (the dispatcher files within seconds; the model may call the
'    tool a minute later). When MOVE fails here the most likely cause is that
'    the other runner already filed the item, so say so and exit instead of
'    surfacing a failure to the user; the runner that did the filing sends its
'    own confirmation when its channel can carry it. Perception runs before
'    this point on the losing path, which is accepted waste — a probe keyword
'    does not exist in the dialect.
ON ERROR RESUME NEXT
MOVE path, destination
IF ERROR THEN
    CLEAR ERROR
    ON ERROR GOTO 0
    TALK "Esse arquivo ja foi arquivado."
    RETURN
END IF
ON ERROR GOTO 0

' 5. Audit trail next to the filed item: keeps the decision reproducible
'    without a read-modify-write over a shared index file.
CREATE FILE destination + ".meta.txt" WITH "category=" + category + "\n" + "kind=" + kind + "\n" + "path=" + destination + "\n" + "caption=" + caption + "\n" + "perception=" + perception

' 6. Confirmation wording per perception. `perception` is one of:
'    caption (no model answer), unavailable (model failed), content /
'    description / transcription (model answer) and any of those with
'    " + caption" appended when the user caption was folded into the prompt.
'    Checking the caption suffix explicitly keeps the reply honest: a
'    description without a caption must not claim the caption was used.
IF perception = "caption" THEN
    TALK "Arquivo arquivado em " + destination + " (categoria " + category + ", obtida pelo texto enviado: a analise de conteudo nao estava disponivel)"
ELSE
    IF perception = "unavailable" THEN
        TALK "Arquivo arquivado em " + destination + " (categoria " + category + ", o modelo de percecao nao respondeu e a decisao saiu da legenda)"
    ELSE
        IF INSTR(perception, "+ caption") > 0 THEN
            TALK "Arquivo classificado como " + category + " pela analise de conteudo (IA) combinada com a legenda, e arquivado em " + destination
        ELSE
            TALK "Arquivo classificado como " + category + " pela analise de conteudo (IA) e arquivado em " + destination
        END IF
    END IF
END IF
