PARAM name AS NAME LIKE "John Smith" DESCRIPTION "Employee's full name"
PARAM email AS EMAIL LIKE "john.smith@company.com" DESCRIPTION "Work email address"
PARAM jobtitle AS STRING LIKE "Software Engineer" DESCRIPTION "Job title or position"
PARAM department AS STRING LIKE "Engineering" DESCRIPTION "Department name"
PARAM hiredate AS DATE LIKE "2024-01-15" DESCRIPTION "Employment start date (YYYY-MM-DD)"
PARAM phone AS PHONE LIKE "+1-555-123-4567" DESCRIPTION "Phone number" OPTIONAL
PARAM manageremail AS EMAIL LIKE "manager@company.com" DESCRIPTION "Manager's email address" OPTIONAL

DESCRIPTION "Add a new employee to the HR system with a unique employee number"

currentyear = FORMAT(NOW(), "YYYY")
employeenumber = "EMP" + currentyear + "-" + FORMAT(RANDOM(1000, 9999))

WITH employee
    number = employeenumber
    fullName = name
    emailAddress = email
    title = jobtitle
    dept = department
    startDate = hiredate
    phoneNumber = phone
    manager = manageremail
END WITH

SAVE "employees.csv", employee

SET BOT MEMORY "last_employee", employeenumber

hrnotification = "New employee added: " + name + " (" + employeenumber + ") - " + jobtitle + " in " + department
SEND EMAIL "hr@company.com", "New Employee Added", hrnotification

IF manageremail THEN
    managernotification = "New team member:\n\nName: " + name + "\nTitle: " + jobtitle + "\nStart Date: " + hiredate
    SEND EMAIL manageremail, "New Team Member: " + name, managernotification
END IF

TALK "Employee added: " + name
TALK "Employee Number: " + employeenumber
TALK "Email: " + email
TALK "Title: " + jobtitle
TALK "Department: " + department
TALK "Start Date: " + hiredate

RETURN employeenumber
