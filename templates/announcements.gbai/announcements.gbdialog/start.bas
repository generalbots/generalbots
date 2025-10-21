REM start.bas - Runs automatically when user connects via web
REM This is the entry point for each session

LET resume = GET_BOT_MEMORY("resume")

IF resume <> "" THEN
    TALK resume
ELSE
    TALK "Welcome! I'm loading the latest information..."
END IF

REM Add knowledge base for weekly announcements
ADD_KB "weekly"

TALK "You can ask me about any of the announcements or circulars."
TALK "If you'd like to login or signup, just type 'auth'."
