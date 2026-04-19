# Enrollment Template

The enrollment template demonstrates how to build a complete data collection workflow that gathers user information step-by-step, validates inputs, confirms details, and saves the data.

## Topic: User Registration & Data Collection

This template is perfect for:
- Customer onboarding flows
- Event registrations
- Lead capture forms
- Survey collection
- Application submissions

## The Code

```basic
REM Enrollment Tool Example

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

DESCRIPTION "This is the enrollment process, called when the user wants to enrol."

REM Start enrollment
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
    SAVE "enrollments.csv", name, birthday, email, personalid, address
    TALK "Thank you! Your enrollment has been successfully submitted."
ELSE
    TALK "Let's start over with the correct information."
END IF
```

## Sample Dialogs

These conversations show how the enrollment template works in real-world scenarios.

### Dialog 1: Successful Enrollment

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Registration Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Hi, I want to register</p>
      <div class="wa-time">10:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Registration Bot</div>
      <p>Welcome to the enrollment process! Let's get you registered. 📋</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>First, what is your full name?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Maria Santos Silva</p>
      <div class="wa-time">10:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Thank you. What is your birth date? (DD/MM/YYYY)</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>15/03/1990</p>
      <div class="wa-time">10:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>What is your email address?</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>santos@email.com</p>
      <div class="wa-time">10:32 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Please provide your Personal ID number (numbers only):</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>12345678901</p>
      <div class="wa-time">10:32 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Finally, what is your full address?</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Rua das Palmeiras, 456 - São Paulo, SP</p>
      <div class="wa-time">10:33 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Please confirm your details:</p>
      <p><strong>Name:</strong> Maria Santos Silva</p>
      <p><strong>Birth Date:</strong> 15/03/1990</p>
      <p><strong>Email:</strong> santos@email.com</p>
      <p><strong>Personal ID:</strong> 12345678901</p>
      <p><strong>Address:</strong> Rua das Palmeiras, 456 - São Paulo, SP</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Are these details correct? (yes/no)</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>yes</p>
      <div class="wa-time">10:33 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Thank you! Your enrollment has been successfully submitted. ✅</p>
      <p>You will receive a confirmation email shortly.</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>
</div>

### Dialog 2: User Corrects Information

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Registration Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Please confirm your details:</p>
      <p><strong>Name:</strong> João Pedro</p>
      <p><strong>Birth Date:</strong> 20/05/1985</p>
      <p><strong>Email:</strong> joao@email.com</p>
      <p><strong>Personal ID:</strong> 98765432100</p>
      <p><strong>Address:</strong> Av. Brasil, 100 - RJ</p>
      <div class="wa-time">14:22</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Are these details correct? (yes/no)</p>
      <div class="wa-time">14:22</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>no</p>
      <div class="wa-time">14:23 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Let's start over with the correct information.</p>
      <div class="wa-time">14:23</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>First, what is your full name?</p>
      <div class="wa-time">14:23</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>João Pedro Oliveira</p>
      <div class="wa-time">14:24 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-system"><span>User continues with corrected information...</span></div>
</div>

### Dialog 3: LLM-Assisted Natural Input

When using the LLM, users can provide information naturally:

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Smart Registration</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I'd like to sign up. My name is Carlos Ferreira, born on March 5th 1992, email carlos.f@gmail.com, ID 11122233344, and I live at Rua do Sol, 789 in Brasília</p>
      <div class="wa-time">16:45 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Smart Registration</div>
      <p>I've captured your information! Let me confirm:</p>
      <p><strong>Name:</strong> Carlos Ferreira</p>
      <p><strong>Birth Date:</strong> 05/03/1992</p>
      <p><strong>Email:</strong> carlos.f@gmail.com</p>
      <p><strong>Personal ID:</strong> 11122233344</p>
      <p><strong>Address:</strong> Rua do Sol, 789 - Brasília</p>
      <div class="wa-time">16:45</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Is everything correct? 👆</p>
      <div class="wa-time">16:45</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Perfect! Yes</p>
      <div class="wa-time">16:46 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Registration complete! Welcome aboard, Carlos!</p>
      <div class="wa-time">16:46</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `PARAM` | Define expected input parameters with types and examples |
| `DESCRIPTION` | Provide context for LLM understanding |
| `TALK` | Send messages to the user |
| `HEAR` | Wait for and capture user input |
| `IF/ELSE` | Conditional logic for confirmation |
| `SAVE` | Persist data to CSV file |

## How It Works

1. **Parameter Definition**: The `PARAM` declarations tell the LLM what information to collect
2. **Step-by-Step Collection**: Each `HEAR` captures one piece of data
3. **Confirmation Loop**: User reviews all data before submission
4. **Data Persistence**: `SAVE` stores the validated data

## Customization Ideas

### Add Validation

```basic
HEAR email
IF NOT INSTR(email, "@") THEN
    TALK "Please enter a valid email address"
    HEAR email
END IF
```

### Add to Database Instead of CSV

```basic
INSERT "users", name, birthday, email, personalid, address
```

### Send Confirmation Email

```basic
SEND MAIL email, "Welcome!", "Your registration is complete, " + name
```

## Related Templates

- [start.bas](./start.md) - Basic greeting flow
- [auth.bas](./auth.md) - Authentication patterns

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
