PARAM name AS STRING LIKE "John Smith" DESCRIPTION "Employee's full name"
PARAM email AS STRING LIKE "john.smith@company.com" DESCRIPTION "Employee's work email address"
PARAM jobtitle AS STRING LIKE "Software Engineer" DESCRIPTION "Job title/position"
PARAM department AS STRING LIKE "Engineering" DESCRIPTION "Department name"
PARAM hiredate AS DATE LIKE "2024-01-15" DESCRIPTION "Employment start date (YYYY-MM-DD)"
PARAM phone AS STRING LIKE "+1-555-123-4567" DESCRIPTION "Optional: Phone number"
PARAM manageremail AS STRING LIKE "manager@company.com" DESCRIPTION "Optional: Manager's email"

DESCRIPTION "Adds a new employee to the HR system. Collects required information and creates the employee record with a unique employee number."

' Validate required fields
IF name = "" THEN
    TALK "I need the employee's full name to continue."
    name = HEAR
END IF

IF email = "" THEN
    TALK "What is the employee's work email address?"
    email = HEAR
END IF

IF jobtitle = "" THEN
    TALK "What is the job title/position?"
    jobtitle = HEAR
END IF

IF department = "" THEN
    TALK "Which department will they be joining?"
    department = HEAR
END IF

IF hiredate = "" THEN
    TALK "What is their start date? (YYYY-MM-DD format)"
    hiredate = HEAR
END IF

' Generate employee number
let currentyear = FORMAT NOW() AS "YYYY"
let employeenumber = "EMP" + currentyear + "-" + FORMAT RANDOM(1000, 9999)

' Save employee record
SAVE "employees.csv", employeenumber, name, email, jobtitle, department, hiredate, phone, manageremail

' Store in bot memory for session
SET BOT MEMORY "last_employee", employeenumber

' Send notifications
let hrnotification = "New employee added: " + name + " (" + employeenumber + ") - " + jobtitle + " in " + department
SEND MAIL "hr@company.com", "New Employee Added", hrnotification

' Notify manager if provided
IF manageremail != "" THEN
    let managernotification = "A new team member has been added:\n\nName: " + name + "\nTitle: " + jobtitle + "\nStart Date: " + hiredate
    SEND MAIL manageremail, "New Team Member: " + name, managernotification
END IF

' Confirm to user
TALK "✅ **Employee Added Successfully!**"
TALK ""
TALK "**Employee Details:**"
TALK "• **Employee Number:** " + employeenumber
TALK "• **Name:** " + name
TALK "• **Email:** " + email
TALK "• **Job Title:** " + jobtitle
TALK "• **Department:** " + department
TALK "• **Start Date:** " + hiredate

IF manageremail != "" THEN
    TALK "• **Manager:** " + manageremail
END IF

TALK ""
TALK "📧 Notifications sent to HR and the assigned manager."
