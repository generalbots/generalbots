PARAM name AS STRING LIKE "Abreu Silva" DESCRIPTION "Full name of the student"
PARAM birthday AS DATE LIKE "23/09/2001" DESCRIPTION "Birth date in DD/MM/YYYY format"
PARAM email AS EMAIL LIKE "abreu.silva@example.com" DESCRIPTION "Email address for contact"
PARAM personalid AS STRING LIKE "12345678900" DESCRIPTION "Personal ID number (only numbers)"
PARAM address AS STRING LIKE "Rua das Flores, 123 - SP" DESCRIPTION "Full address"

DESCRIPTION "Process student enrollment with validation and confirmation"

enrollmentid = "ENR-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
createdat = FORMAT(NOW(), "YYYY-MM-DD HH:mm:ss")

WITH enrollment
    id = enrollmentid
    studentName = name
    birthDate = birthday
    emailAddress = email
    personalId = personalid
    fullAddress = address
    createdAt = createdat
    status = "pending"
END WITH

SAVE "enrollments.csv", enrollment

SET BOT MEMORY "last_enrollment", enrollmentid

TALK "Enrollment submitted successfully!"
TALK "Enrollment ID: " + enrollmentid
TALK "Name: " + name
TALK "Email: " + email
TALK "Status: Pending review"

SEND EMAIL email, "Enrollment Confirmation", "Dear " + name + ",\n\nYour enrollment request has been submitted.\n\nEnrollment ID: " + enrollmentid + "\n\nWe will review your application and contact you soon.\n\nBest regards,\nAdmissions Team"

RETURN enrollmentid
