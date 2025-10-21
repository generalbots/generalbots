REM Simple KISS authentication - signup/login only, no recovery
REM This script is called when user needs authentication

TALK "Welcome! Please choose an option:"
TALK "Type 'signup' to create a new account"
TALK "Type 'login' to access your existing account"

HEAR choice

IF choice = "signup" THEN
    TALK "Great! Let's create your account."
    TALK "Enter your email:"
    HEAR email

    TALK "Enter your password:"
    HEAR password

    TALK "Confirm your password:"
    HEAR confirm_password

    IF password <> confirm_password THEN
        TALK "Passwords don't match. Please try again."
        RETURN false
    END IF

    REM Create user in database
    LET user_id = GENERATE_UUID()
    LET result = EXEC "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, NOW())", user_id, email, SHA256(password)

    IF result > 0 THEN
        SET_USER user_id
        TALK "Account created successfully! You are now logged in."
        RETURN true
    ELSE
        TALK "Error creating account. Email may already exist."
        RETURN false
    END IF

ELSE IF choice = "login" THEN
    TALK "Please enter your email:"
    HEAR email

    TALK "Enter your password:"
    HEAR password

    REM Query user from database
    LET user = FIND "users", "email=" + email

    IF user = NULL THEN
        TALK "Invalid email or password."
        RETURN false
    END IF

    LET password_hash = SHA256(password)
    IF user.password_hash = password_hash THEN
        SET_USER user.id
        TALK "Welcome back! You are now logged in."
        RETURN true
    ELSE
        TALK "Invalid email or password."
        RETURN false
    END IF

ELSE
    TALK "Invalid option. Please type 'signup' or 'login'."
    RETURN false
END IF
