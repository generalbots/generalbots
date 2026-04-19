PARAM message AS STRING LIKE "Olá {name}, confira nossas novidades!" DESCRIPTION "Message to broadcast, supports {name} and {telefone} variables"
PARAM template_file AS FILE LIKE "header.jpg" DESCRIPTION "Header image file for the template"
PARAM list_file AS FILE LIKE "contacts.xlsx" DESCRIPTION "File with contacts (must have telefone column)"
PARAM filter AS STRING LIKE "Perfil=VIP" DESCRIPTION "Filter condition for contact list" OPTIONAL

DESCRIPTION "Send marketing broadcast message to a filtered contact list via WhatsApp template"

report = LLM "Esta mensagem será aprovada pelo WhatsApp META como Template? Responda OK se sim, ou explique o problema: " + message

IF report <> "OK" THEN
    TALK "Atenção: " + report
END IF

IF filter THEN
    list = FIND list_file, filter
ELSE
    list = FIND list_file
END IF

IF UBOUND(list) = 0 THEN
    TALK "Nenhum contato encontrado."
    RETURN 0
END IF

PUBLISH

SET MAX LINES 2020

index = 1
sent = 0

DO WHILE index < UBOUND(list)
    row = list[index]

    SEND TEMPLATE TO row.telefone, template_file

    WAIT 0.1

    WITH logEntry
        timestamp = NOW()
        phone = row.telefone
        name = row.name
        status = "sent"
    END WITH

    SAVE "broadcast_log.csv", logEntry

    sent = sent + 1
    index = index + 1
LOOP

TALK "Broadcast enviado para " + sent + " contatos."

RETURN sent
