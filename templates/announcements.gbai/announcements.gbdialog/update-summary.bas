SET SCHEDULE "59 * * * *"

let text = GET "announcements.gbkb/news/news.pdf"
let resume = LLM "In a few words, resume this: " + text

SET BOT MEMORY "resume", resume

let text1 = GET "announcements.gbkb/auxiliom/auxiliom.pdf"
SET BOT MEMORY "auxiliom", text1

let text2 = GET "announcements.gbkb/toolbix/toolbix.pdf"
SET BOT MEMORY "toolbix", text2
