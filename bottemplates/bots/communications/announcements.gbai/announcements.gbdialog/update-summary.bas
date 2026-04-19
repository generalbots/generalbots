SET SCHEDULE "59 * * * *"

text = GET "announcements.gbkb/news/news.pdf"
resume = LLM "In a few words, resume this: " + text
SET BOT MEMORY "resume", resume

text1 = GET "announcements.gbkb/auxiliom/auxiliom.pdf"
SET BOT MEMORY "auxiliom", text1

text2 = GET "announcements.gbkb/toolbix/toolbix.pdf"
SET BOT MEMORY "toolbix", text2
