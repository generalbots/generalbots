resume1 = GET BOT MEMORY("resume")
resume2 = GET BOT MEMORY("auxiliom")
resume3 = GET BOT MEMORY("toolbix")

SET CONTEXT "general"  AS resume1
SET CONTEXT "auxiliom" AS resume2
SET CONTEXT "toolbix"  AS resume3

CLEAR SUGGESTIONS

ADD SUGGESTION "general" AS "bot-suggest-weekly-announcements"
ADD SUGGESTION "general" AS "bot-suggest-latest-circulars"
ADD SUGGESTION "auxiliom" AS "bot-suggest-what-is-auxiliom"
ADD SUGGESTION "auxiliom" AS "bot-suggest-auxiliom-services"
ADD SUGGESTION "toolbix" AS "bot-suggest-toolbix-features"
ADD SUGGESTION "toolbix" AS "bot-suggest-toolbix-for-business"

ADD TOOL "change-subject"

TALK resume1
TALK "Ask me about any announcement or circular."
