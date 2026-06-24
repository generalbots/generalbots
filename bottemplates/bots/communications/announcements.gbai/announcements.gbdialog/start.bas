resume1 = GET BOT MEMORY("resume")
resume2 = GET BOT MEMORY("auxiliom")
resume3 = GET BOT MEMORY("toolbix")

SET CONTEXT "general"  AS resume1
SET CONTEXT "auxiliom" AS resume2
SET CONTEXT "toolbix"  AS resume3

CLEAR SUGGESTIONS

ADD SUGGESTION "general" AS "Weekly announcements"
ADD SUGGESTION "general" AS "Latest circulars"
ADD SUGGESTION "auxiliom" AS "What is Auxiliom?"
ADD SUGGESTION "auxiliom" AS "Auxiliom services"
ADD SUGGESTION "toolbix" AS "Toolbix features"
ADD SUGGESTION "toolbix" AS "Toolbix for business"

ADD TOOL "change-subject"

TALK resume1
TALK "Ask me about any announcement or circular."
