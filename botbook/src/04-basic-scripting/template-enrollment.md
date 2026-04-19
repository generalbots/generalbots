# Enrollment Tool Example

This example shows a complete enrollment tool with parameter definitions and data saving.

## Complete Enrollment Script

```basic
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

DESCRIPTION  "This is the enrollment process, called when the user wants to enrol. Once all information is collected, confirm the details and inform them that their enrollment request has been successfully submitted. Provide a polite and professional tone throughout the interaction."

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

## Tool Parameters

This tool defines 5 parameters with specific types and validation:

1. **name** (string): Full name with example format
2. **birthday** (date): Birth date in DD/MM/YYYY format  
3. **email** (string): Email address for contact
4. **personalid** (integer): Numeric personal ID
5. **address** (string): Complete physical address

## Data Storage

The `SAVE` command writes the collected data to a CSV file:
- Creates "enrollments.csv" if it doesn't exist
- Appends new records with all fields
- Maintains data consistency across sessions

## Usage Flow

1. User initiates enrollment process
2. Bot collects each piece of information sequentially
3. User confirms accuracy of entered data
4. Data is saved to persistent storage
5. Confirmation message is sent

## Error Handling

The script includes:
- Input validation through parameter types
- Confirmation step to prevent errors
- Clear user prompts with format examples
- Graceful handling of correction requests

This example demonstrates a complete, production-ready tool implementation using the BASIC scripting language.
