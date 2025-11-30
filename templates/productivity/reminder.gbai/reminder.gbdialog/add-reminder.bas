PARAM when AS STRING LIKE "tomorrow at 9am" DESCRIPTION "When to send the reminder (date/time)"
PARAM subject AS STRING LIKE "Call John about project" DESCRIPTION "What to be reminded about"
PARAM notify AS STRING LIKE "email" DESCRIPTION "Notification method: email, sms, or chat" OPTIONAL

DESCRIPTION "Create a reminder for a specific date and time with notification"

IF NOT notify THEN
    notify = "chat"
END IF

reminderid = "REM-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
useremail = GET "session.user_email"
userphone = GET "session.user_phone"

WITH reminder
    id = reminderid
    remindAt = when
    message = subject
    notifyBy = notify
    email = useremail
    phone = userphone
    created = NOW()
    status = "pending"
END WITH

SAVE "reminders.csv", reminder

SET BOT MEMORY "last_reminder", reminderid

TALK "Reminder set: " + subject
TALK "When: " + when
TALK "Notification: " + notify

RETURN reminderid
