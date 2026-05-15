TABLE students
    Id uuid key
    Name string(255)
    Email string(255)
    RegisteredAt timestamp
END TABLE

PARAM name AS STRING LIKE "John Doe" DESCRIPTION "Full name of the student"
PARAM email AS STRING LIKE "john@example.com" DESCRIPTION "Email address of the student"

DESCRIPTION "Register a new student by saving their name and email to the students table"

studentId = UUID()
registeredAt = NOW()

WITH student
    id = studentId
    name = name
    email = email
    registeredAt = registeredAt
END WITH

SAVE "students", studentId, student

TALK "Student registered successfully!"
TALK "Student ID: " + studentId
TALK "Name: " + name
TALK "Email: " + email

RETURN studentId
