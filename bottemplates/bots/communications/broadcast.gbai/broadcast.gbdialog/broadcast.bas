PARAM message AS STRING LIKE "Hello {name}, how are you?" DESCRIPTION "Message to broadcast, supports {name} and {mobile} variables"
PARAM listfile AS STRING LIKE "broadcast.csv" DESCRIPTION "CSV file with contacts (name, mobile columns)"
PARAM filter AS STRING LIKE "status=active" DESCRIPTION "Filter condition for contact list" OPTIONAL

DESCRIPTION "Send broadcast message to a list of contacts from CSV file"

IF NOT listfile THEN
    listfile = "broadcast.csv"
END IF

IF filter THEN
    list = FIND listfile, filter
ELSE
    list = FIND listfile
END IF

IF UBOUND(list) = 0 THEN
    TALK "No contacts found in " + listfile
    RETURN 0
END IF

index = 1
sent = 0

DO WHILE index < UBOUND(list)
    row = list[index]

    msg = REPLACE(message, "{name}", row.name)
    msg = REPLACE(msg, "{mobile}", row.mobile)

    TALK TO row.mobile, msg
    WAIT 5

    WITH logEntry
        timestamp = NOW()
        user = USERNAME
        from = FROM
        mobile = row.mobile
        name = row.name
        status = "sent"
    END WITH

    SAVE "Log.xlsx", logEntry

    sent = sent + 1
    index = index + 1
LOOP

TALK "Broadcast sent to " + sent + " contacts."

RETURN sent
