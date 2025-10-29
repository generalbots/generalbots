LET resume1 = GET_BOT_MEMORY("general")
LET resume2 = GET_BOT_MEMORY("auxiliom")
LET resume3 = GET_BOT_MEMORY("toolbix")

SET_CONTEXT "general", resume1
SET_CONTEXT "auxiliom", resume2
SET_CONTEXT "toolbix", resume3


ADD_SUGGESTION "general", "Show me the weekly announcements"
ADD_SUGGESTION "auxiliom", "Will Auxiliom help me with what?"
ADD_SUGGESTION "auxiliom", "What does Auxiliom do?" , "fixed"
ADD_SUGGESTION "toolbix", "Show me Toolbix features"
ADD_SUGGESTION "toolbix", "How can Toolbix help my business?", "fixed"


TALK "You can ask me about any of the announcements or circulars."
