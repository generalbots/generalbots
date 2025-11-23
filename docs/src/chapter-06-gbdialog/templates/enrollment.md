# enrollment.bas (Template)

A comprehensive enrollment dialog that gathers user information, confirms it, and saves it to a CSV file.

```basic
REM Enrollment Tool Example

## Complete Enrollment Script

PARAM name AS string          LIKE "Abreu Silva"
DESCRIPTION "Required full name of the individual."

PARAM birthday AS date        LIKE "23/09/2001"
DESCRIPTION "Required birth date of the individual in DD/MM/YYYY format."

PARAM email AS string         LIKE "abreu.silva@example.com"
DESCRIPTION "Required email address for contact purposes."

PARAM personalid AS integer   LIKE "12345678900"
DESCRIPTION "Required Personal ID number of the individual (only numbers)."

PARAM address AS string       LIKE "Rua das Flores, 123 - SP"
DESCRIPTION "Required full address of the individual."

DESCRIPTION "This is the enrollment process, called when the user wants to enrol. Once all information is collected, confirm the details and inform them that their enrollment request has been successfully submitted. Provide a polite and professional tone throughout the interaction."

REM Enrollment Process
TALK "Welcome to the enrollment process! Let's get you registered."

TALK "First, what is your full name?"
HEAR name

TALK "Thank you. What is your birth date? (DD/MM/YYYY)"
HEAR birthday

TALK "What is your email address?"
HEAR email

TALK "Please provide your Personal ID number (numbers only):"
HEAR personalid

TALK "Finally, what is your full address?"
HEAR address

REM Validate and confirm
TALK "Please confirm your details:"
TALK "Name: " + name
TALK "Birth Date: " + birthday
TALK "Email: " + email
TALK "Personal ID: " + personalid
TALK "Address: " + address

TALK "Are these details correct? (yes/no)"
HEAR confirmation

IF confirmation = "yes" THEN
    REM Save to CSV file
    SAVE "enrollments.csv", name, birthday, email, personalid, address
    TALK "Thank you! Your enrollment has been successfully submitted. You will receive a confirmation email shortly."
ELSE
    TALK "Let's start over with the correct information."
    REM In a real implementation, you might loop back or use a different approach
END IF
```

**Purpose**

- Shows how to define parameters with `PARAM` and `DESCRIPTION`.
- Demonstrates a multi‑step data collection flow using `HEAR` and `TALK`.
- Confirms data before persisting it via `SAVE`.

**Keywords used:** `PARAM`, `DESCRIPTION`, `HEAR`, `TALK`, `IF`, `ELSE`, `SAVE`.

---
