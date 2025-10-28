LET resume = GET_BOT_MEMORY("resume")

IF resume <> "" THEN
    TALK resume
END IF

ADD_KB "weekly"

TALK "You can ask me about any of the announcements or circulars."
