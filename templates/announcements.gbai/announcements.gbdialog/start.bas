let resume1 = GET_BOT_MEMORY("general");
let resume2 = GET_BOT_MEMORY("auxiliom");
let resume3 = GET_BOT_MEMORY("toolbix");

SET_CONTEXT "general"  AS  resume1;
SET_CONTEXT "auxiliom" AS  resume2;
SET_CONTEXT "toolbix"  AS  resume3;


ADD_SUGGESTION "general" AS "Show me the weekly announcements"
ADD_SUGGESTION "auxiliom" AS "Will Auxiliom help me with what?"
ADD_SUGGESTION "auxiliom" AS "What does Auxiliom do?"
ADD_SUGGESTION "toolbix" AS "Show me Toolbix features"
ADD_SUGGESTION "toolbix" AS "How can Toolbix help my business?"


TALK "You can ask me about any of the announcements or circulars."
