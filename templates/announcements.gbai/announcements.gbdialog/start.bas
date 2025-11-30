let resume1 = GET BOT MEMORY("resume")
let resume2 = GET BOT MEMORY("auxiliom")
let resume3 = GET BOT MEMORY("toolbix")

SET CONTEXT "general"  AS  resume1
SET CONTEXT "auxiliom" AS  resume2
SET CONTEXT "toolbix"  AS  resume3

CLEAR SUGGESTIONS

ADD SUGGESTION "general" AS "Show me the weekly announcements"
ADD SUGGESTION "auxiliom" AS "Explain Auxiliom to me"
ADD SUGGESTION "auxiliom" AS "What does Auxiliom provide?"
ADD SUGGESTION "toolbix" AS "Show me Toolbix features"
ADD SUGGESTION "toolbix" AS "How can Toolbix help my business?"

TALK resume1
TALK "You can ask me about any of the announcements or circulars."
