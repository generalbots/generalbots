# Authentication Template

The authentication template demonstrates secure user verification flows including login, registration, password validation, and session management.

## Topic: User Authentication & Security

This template is perfect for:
- User login systems
- Account verification
- Password recovery flows
- Session management
- Two-factor authentication

## The Code

```basic
REM Authentication Flow with Retry Logic

PARAM username AS string LIKE "john.doe"
DESCRIPTION "Username or email for authentication"

PARAM password AS string LIKE "********"
DESCRIPTION "User's password (masked input)"

SET max_attempts = 3
SET attempts = 0

TALK "Welcome! Please enter your username:"
HEAR username

LABEL auth_loop

TALK "Enter your password:"
HEAR password AS PASSWORD   ' Masked input

' Verify credentials
user = FIND "users", "username='" + username + "'"

IF user = NULL THEN
    TALK "Username not found. Would you like to register? (yes/no)"
    HEAR register_choice
    IF register_choice = "yes" THEN
        GOTO registration
    ELSE
        TALK "Goodbye!"
        EXIT
    END IF
END IF

IF user.password = HASH(password) THEN
    SET BOT MEMORY "authenticated_user", username
    SET BOT MEMORY "session_start", NOW()
    TALK "Welcome back, " + user.name + "! You are now logged in."
    EXIT
ELSE
    SET attempts = attempts + 1
    IF attempts >= max_attempts THEN
        TALK "Too many failed attempts. Your account is temporarily locked."
        SEND MAIL user.email, "Security Alert", "Multiple failed login attempts detected."
        EXIT
    END IF
    TALK "Incorrect password. " + (max_attempts - attempts) + " attempts remaining."
    GOTO auth_loop
END IF

LABEL registration
TALK "Let's create your account. Enter your email:"
HEAR email
TALK "Create a password (min 8 characters):"
HEAR new_password AS PASSWORD

IF LEN(new_password) < 8 THEN
    TALK "Password too short. Please try again."
    GOTO registration
END IF

INSERT "users", username, email, HASH(new_password), NOW()
TALK "Account created! You can now log in."
```

## Sample Dialogs

These conversations show how the authentication template works in real-world scenarios.

### Dialog 1: Successful Login

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔐</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Auth Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Auth Bot</div>
      <p>Welcome! Please enter your username:</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>maria.santos</p>
      <div class="wa-time">09:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Enter your password:</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>••••••••</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Welcome back, Maria! You are now logged in. ✅</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>
</div>

### Dialog 2: Failed Login with Retry

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔐</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Auth Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Enter your password:</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>••••••••</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Incorrect password. 2 attempts remaining. ⚠️</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Enter your password:</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>••••••••</p>
      <div class="wa-time">14:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Welcome back, João! You are now logged in. ✅</p>
      <div class="wa-time">14:31</div>
    </div>
  </div>
</div>

### Dialog 3: Account Locked

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔐</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Auth Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Incorrect password. 1 attempt remaining. ⚠️</p>
      <div class="wa-time">16:45</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Enter your password:</p>
      <div class="wa-time">16:45</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>••••••••</p>
      <div class="wa-time">16:45 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Too many failed attempts. Your account is temporarily locked. 🔒</p>
      <p>A security alert has been sent to your email.</p>
      <div class="wa-time">16:45</div>
    </div>
  </div>

  <div class="wa-system"><span>Session ended</span></div>
</div>

### Dialog 4: New User Registration

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔐</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Auth Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Username not found. Would you like to register? (yes/no)</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>yes</p>
      <div class="wa-time">10:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Let's create your account. Enter your email:</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>carlos@email.com</p>
      <div class="wa-time">10:16 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Create a password (min 8 characters):</p>
      <div class="wa-time">10:16</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>••••••••••</p>
      <div class="wa-time">10:16 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Account created! 🎉 You can now log in.</p>
      <div class="wa-time">10:16</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `PARAM` | Define expected input parameters |
| `SET` | Assign values to variables |
| `TALK` | Send messages to the user |
| `HEAR` | Capture user input |
| `HEAR AS PASSWORD` | Masked password input |
| `FIND` | Query database for user |
| `IF/ELSE` | Conditional logic |
| `GOTO/LABEL` | Flow control for retry loop |
| `HASH` | Secure password hashing |
| `SET BOT MEMORY` | Store session data |
| `SEND MAIL` | Send security alerts |
| `INSERT` | Create new user record |
| `EXIT` | End the dialog |

## How It Works

1. **Username Input**: Collects the username first
2. **User Lookup**: Checks if user exists in database
3. **Password Verification**: Compares hashed password
4. **Retry Logic**: Allows 3 attempts before lockout
5. **Session Creation**: Stores auth state in bot memory
6. **Registration**: Offers new account creation if user not found

## Security Features

### Password Hashing

```basic
' Never store plain text passwords!
hashed = HASH(password)
INSERT "users", username, email, hashed
```

### Rate Limiting

```basic
IF attempts >= max_attempts THEN
    SET BOT MEMORY "locked_" + username, NOW()
    TALK "Account locked for 15 minutes."
END IF
```

### Two-Factor Authentication

```basic
' Send OTP after password verification
otp = RANDOM(100000, 999999)
SET BOT MEMORY "otp_" + username, otp
SEND MAIL email, "Your verification code", "Code: " + otp

TALK "Enter the 6-digit code sent to your email:"
HEAR user_otp

IF user_otp = GET BOT MEMORY "otp_" + username THEN
    TALK "Two-factor authentication successful!"
ELSE
    TALK "Invalid code."
END IF
```

## Customization Ideas

### Add "Forgot Password"

```basic
TALK "Forgot your password? (yes/no)"
HEAR forgot
IF forgot = "yes" THEN
    reset_token = RANDOM_STRING(32)
    SET BOT MEMORY "reset_" + username, reset_token
    SEND MAIL user.email, "Password Reset", "Click here: /reset/" + reset_token
    TALK "Password reset link sent to your email."
END IF
```

### Session Timeout

```basic
session_start = GET BOT MEMORY "session_start"
IF DATEDIFF("minute", session_start, NOW()) > 30 THEN
    TALK "Session expired. Please log in again."
    SET BOT MEMORY "authenticated_user", ""
END IF
```

### Social Login

```basic
TALK "Login with: 1) Password 2) Google 3) GitHub"
HEAR login_method

SWITCH login_method
    CASE "2"
        ' Redirect to OAuth
        url = GET "auth/google/redirect"
        TALK "Click to login: " + url
    CASE "3"
        url = GET "auth/github/redirect"
        TALK "Click to login: " + url
    DEFAULT
        ' Standard password flow
END SWITCH
```

## Related Templates

- [start.bas](./start.md) - Basic greeting flow
- [enrollment.bas](./enrollment.md) - Data collection patterns

---

<style>
/* Inline WhatsApp Chat Styles for this page */
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-system{text-align:center;margin:15px 0;clear:both}
.wa-system span{background-color:#e1f2fb;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>