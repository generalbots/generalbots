PARAM subject AS STRING LIKE "circular" DESCRIPTION "Subject to switch conversation to: circular, comunicado, or geral"

DESCRIPTION "Switch conversation subject when user wants to change topic"

kbname = LLM "Return single word: circular, comunicado or geral based on: " + subject
ADD_KB kbname

TALK "Subject changed to " + subject
