PARAM searchterm AS STRING LIKE "John" DESCRIPTION "Name, email, or employee number to search for"

DESCRIPTION "Searches for employees in the HR system by name, email, or employee number."

IF searchterm = "" THEN
    TALK "What would you like to search for? You can enter a name, email, or employee number."
    searchterm = HEAR
END IF

IF searchterm = "" THEN
    TALK "I need a search term to find employees."
    RETURN
END IF

TALK "🔍 Searching for employees matching: " + searchterm
TALK ""
TALK "To search the employee database, I'll look through the records for you."
TALK "You can search by:"
TALK "• Full name or partial name"
TALK "• Email address"
TALK "• Employee number (e.g., EMP2024-1234)"
TALK ""
TALK "💡 Tip: For best results, use the exact employee number if you have it."

SET BOT MEMORY "last_search", searchterm
