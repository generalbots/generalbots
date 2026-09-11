' start.bas - media filing assistant session entry point.
'
' USE TOOL is mandatory: the tool runner only executes tools the script
' associated with the session, so without this line the model may request
' classify_media and the call is skipped.

USE TOOL "classify_media"

ADD_SUGGESTION "O que você faz?"
ADD_SUGGESTION "Como os arquivos são organizados?"

TALK "Envie uma foto ou um documento que eu classifico e arquivo automaticamente no Drive."
