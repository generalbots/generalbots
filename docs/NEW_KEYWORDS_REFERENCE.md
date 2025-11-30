# General Bots - New Keywords Reference

> Complete reference for all new BASIC keywords implemented for General Bots Office Suite

## Table of Contents

1. [Social Media Keywords](#social-media-keywords)
2. [Form Handling & Webhooks](#form-handling--webhooks)
3. [Template Messaging](#template-messaging)
4. [Lead Scoring & CRM](#lead-scoring--crm)
5. [Math Functions](#math-functions)
6. [Date/Time Functions](#datetime-functions)
7. [String Functions](#string-functions)
8. [Validation Functions](#validation-functions)
9. [Array Functions](#array-functions)
10. [Error Handling](#error-handling)

---

## Social Media Keywords

### POST TO

Post content to social media platforms.

```basic
' Post to single platform
POST TO INSTAGRAM image, "Check out our new feature! #AI #Automation"

' Post to specific platforms
POST TO FACEBOOK image, caption
POST TO LINKEDIN image, caption  
POST TO TWITTER image, caption

' Post to multiple platforms at once
POST TO "instagram,facebook,linkedin" image, caption
```

### POST TO ... AT (Scheduled Posting)

Schedule posts for future publishing.

```basic
' Schedule a post
POST TO INSTAGRAM AT "2025-02-01 10:00" image, caption
POST TO FACEBOOK AT "2025-02-15 09:00" image, "Coming soon!"
```

### GET METRICS

Retrieve engagement metrics for posts.

```basic
' Get Instagram metrics
metrics = GET INSTAGRAM METRICS "post-id"
TALK "Likes: " + metrics.likes + ", Comments: " + metrics.comments

' Get Facebook metrics
fb_metrics = GET FACEBOOK METRICS "post-id"
TALK "Shares: " + fb_metrics.shares

' Get LinkedIn metrics
li_metrics = GET LINKEDIN METRICS "post-id"

' Get Twitter metrics
tw_metrics = GET TWITTER METRICS "post-id"
```

### GET POSTS

List posts from a platform.

```basic
' Get all Instagram posts
posts = GET INSTAGRAM POSTS

' Get Facebook posts
fb_posts = GET FACEBOOK POSTS
```

### DELETE POST

Remove a scheduled or published post.

```basic
DELETE POST "post-id"
```

---

## Form Handling & Webhooks

### ON FORM SUBMIT

Register webhook handlers for form submissions from landing pages.

```basic
' Basic form handler
ON FORM SUBMIT "landing-page-form"
    ' Access form fields
    name = fields.name
    email = fields.email
    phone = fields.phone
    
    ' Access metadata
    source = metadata.utm_source
    referrer = metadata.referrer
    
    ' Process the submission
    SAVE "leads.csv", name, email, phone, source
    SEND MAIL email, "Welcome!", "Thank you for your interest..."
END ON

' Form handler with validation
ON FORM SUBMIT "contact-form" WITH VALIDATION
    ' Validation is automatically applied
    ' Required fields, email format, phone format checked
END ON
```

### WEBHOOK

Create custom webhook endpoints.

```basic
' Register a webhook endpoint
WEBHOOK "order-received"

' The webhook will be available at:
' https://your-server/bot-name/webhook/order-received
```

### SET SCHEDULE

Schedule scripts to run at specific times.

```basic
' Run daily at 9 AM
SET SCHEDULE "0 9 * * *", "daily-report.bas"

' Run every Monday at 10 AM
SET SCHEDULE "0 10 * * 1", "weekly-summary.bas"

' Run on specific date
SET SCHEDULE DATEADD(TODAY(), 3, "day"), "followup.bas"
```

---

## Template Messaging

### SEND TEMPLATE

Send templated messages across multiple channels.

```basic
' Send email template
SEND TEMPLATE "welcome", "email", "user@example.com", #{name: "John", product: "Pro Plan"}

' Send WhatsApp template
SEND TEMPLATE "order-confirmation", "whatsapp", "+1234567890", #{order_id: "12345"}

' Send SMS template
SEND TEMPLATE "verification", "sms", "+1234567890", #{code: "123456"}

' Send to multiple channels at once
SEND TEMPLATE "announcement", "email,whatsapp", recipient, variables

' Simplified syntax without variables
SEND TEMPLATE "reminder", "email", "user@example.com"
```

### SEND TEMPLATE TO (Bulk)

Send templates to multiple recipients.

```basic
' Bulk email send
recipients = ["a@example.com", "b@example.com", "c@example.com"]
count = SEND TEMPLATE "newsletter" TO "email" recipients, #{month: "January"}
TALK "Sent to " + count + " recipients"
```

### CREATE TEMPLATE

Create new message templates.

```basic
' Create email template
CREATE TEMPLATE "welcome", "email", "Welcome to {{company}}!", "Hello {{name}}, thank you for joining us!"

' Create WhatsApp template
CREATE TEMPLATE "order-update", "whatsapp", "", "Your order {{order_id}} is {{status}}."
```

### GET TEMPLATE

Retrieve a template.

```basic
template = GET TEMPLATE "welcome"
TALK "Template body: " + template.body
```

---

## Lead Scoring & CRM

### SCORE LEAD

Calculate lead score based on profile and behavior data.

```basic
' Score a lead
lead_data = NEW OBJECT
lead_data.email = "john@company.com"
lead_data.name = "John Smith"
lead_data.company = "Acme Corp"
lead_data.job_title = "VP of Engineering"
lead_data.industry = "Technology"
lead_data.company_size = "Enterprise"

score = SCORE LEAD lead_data

TALK "Score: " + score.score
TALK "Grade: " + score.grade
TALK "Status: " + score.status
TALK "Recommendations: " + score.recommendations[0]
```

### AI SCORE LEAD

Use AI/LLM-enhanced scoring for better accuracy.

```basic
' AI-enhanced scoring
score = AI SCORE LEAD lead_data

TALK "AI Score: " + score.score
TALK "AI Confidence: " + score.breakdown.ai_confidence
```

### GET LEAD SCORE

Retrieve existing lead score.

```basic
' Get stored score
score = GET LEAD SCORE "lead-id"
TALK "Current score: " + score.score
```

### QUALIFY LEAD

Check if lead meets qualification threshold.

```basic
' Default threshold (70)
result = QUALIFY LEAD "lead-id"
IF result.qualified THEN
    TALK "Lead is qualified: " + result.status
END IF

' Custom threshold
result = QUALIFY LEAD "lead-id", 80
IF result.qualified THEN
    TALK "Lead meets 80+ threshold"
END IF
```

### UPDATE LEAD SCORE

Manually adjust a lead's score.

```basic
' Add points for engagement
new_score = UPDATE LEAD SCORE "lead-id", 10, "Attended webinar"

' Deduct points for inactivity
new_score = UPDATE LEAD SCORE "lead-id", -5, "No response to email"
```

---

## Math Functions

### Basic Math

```basic
' Absolute value
result = ABS(-42)         ' Returns 42

' Rounding
result = ROUND(3.7)       ' Returns 4
result = ROUND(3.14159, 2) ' Returns 3.14

' Integer conversion
result = INT(3.9)         ' Returns 3
result = FIX(-3.9)        ' Returns -3
result = FLOOR(3.7)       ' Returns 3
result = CEIL(3.2)        ' Returns 4
```

### Min/Max

```basic
' Two values
result = MAX(10, 20)      ' Returns 20
result = MIN(10, 20)      ' Returns 10

' Array values
arr = [5, 2, 8, 1, 9]
result = MAX(arr)         ' Returns 9
result = MIN(arr)         ' Returns 1
```

### Other Math Functions

```basic
' Modulo
result = MOD(17, 5)       ' Returns 2

' Random numbers
result = RANDOM()         ' Random 0-1
result = RANDOM(100)      ' Random 0-99
result = RANDOM(1, 10)    ' Random 1-10

' Sign
result = SGN(-5)          ' Returns -1
result = SGN(5)           ' Returns 1
result = SGN(0)           ' Returns 0

' Square root
result = SQR(16)          ' Returns 4
result = SQRT(25)         ' Returns 5

' Logarithms
result = LOG(10)          ' Natural log
result = LOG10(100)       ' Base 10 log

' Exponential
result = EXP(2)           ' e^2

' Power
result = POW(2, 8)        ' Returns 256
result = POWER(3, 4)      ' Returns 81

' Trigonometry
result = SIN(0)           ' Returns 0
result = COS(0)           ' Returns 1
result = TAN(0)           ' Returns 0
pi = PI()                 ' Returns 3.14159...

' Aggregation
arr = [10, 20, 30, 40, 50]
result = SUM(arr)         ' Returns 150
result = AVG(arr)         ' Returns 30
result = AVERAGE(arr)     ' Returns 30
```

---

## Date/Time Functions

### Current Date/Time

```basic
' Current date and time
now = NOW()               ' "2025-01-22 14:30:45"
now_utc = NOW_UTC()       ' UTC time

' Current date only
today = TODAY()           ' "2025-01-22"

' Current time only
time = TIME()             ' "14:30:45"

' Unix timestamp
ts = TIMESTAMP()          ' 1737556245
```

### Date Components

```basic
date = "2025-01-22"

year = YEAR(date)         ' Returns 2025
month = MONTH(date)       ' Returns 1
day = DAY(date)           ' Returns 22
weekday = WEEKDAY(date)   ' Returns 4 (Wednesday)
week = WEEKNUM(date)      ' Returns 4

datetime = "2025-01-22 14:30:45"
hour = HOUR(datetime)     ' Returns 14
minute = MINUTE(datetime) ' Returns 30
second = SECOND(datetime) ' Returns 45
```

### Date Arithmetic

```basic
' Add to date
future = DATEADD("2025-01-22", 7, "day")      ' "2025-01-29"
future = DATEADD("2025-01-22", 1, "month")    ' "2025-02-22"
future = DATEADD("2025-01-22", 1, "year")     ' "2026-01-22"
future = DATEADD("2025-01-22 10:00", 2, "hour") ' "2025-01-22 12:00:00"

' Subtract from date (negative values)
past = DATEADD("2025-01-22", -7, "day")       ' "2025-01-15"

' Date difference
days = DATEDIFF("2025-01-01", "2025-01-22", "day")    ' 21
months = DATEDIFF("2025-01-01", "2025-06-01", "month") ' 5
years = DATEDIFF("2020-01-01", "2025-01-01", "year")   ' 5
```

### Date Formatting

```basic
' Format date
formatted = FORMAT_DATE("2025-01-22", "DD/MM/YYYY")   ' "22/01/2025"
formatted = FORMAT_DATE("2025-01-22", "MMMM DD, YYYY") ' "January 22, 2025"
```

### Date Validation

```basic
' Check if valid date
valid = ISDATE("2025-01-22")    ' true
valid = ISDATE("invalid")       ' false
```

### End of Month

```basic
' Get last day of month
eom = EOMONTH("2025-01-15", 0)   ' "2025-01-31"
eom = EOMONTH("2025-01-15", 1)   ' "2025-02-28"
eom = EOMONTH("2025-01-15", -1)  ' "2024-12-31"
```

---

## String Functions

### Length and Substrings

```basic
' String length
length = LEN("Hello World")      ' Returns 11

' Left portion
result = LEFT("Hello World", 5)   ' Returns "Hello"

' Right portion
result = RIGHT("Hello World", 5)  ' Returns "World"

' Middle portion
result = MID("Hello World", 7)    ' Returns "World"
result = MID("Hello World", 7, 3) ' Returns "Wor"
```

### Case Conversion

```basic
' Uppercase
result = UCASE("hello")          ' Returns "HELLO"
result = UPPER("hello")          ' Returns "HELLO"

' Lowercase
result = LCASE("HELLO")          ' Returns "hello"
result = LOWER("HELLO")          ' Returns "hello"
```

### Trimming

```basic
' Trim whitespace
result = TRIM("  hello  ")       ' Returns "hello"
result = LTRIM("  hello")        ' Returns "hello"
result = RTRIM("hello  ")        ' Returns "hello"
```

### Search and Replace

```basic
' Find position (1-based)
pos = INSTR("Hello World", "World")      ' Returns 7
pos = INSTR(5, "one two one", "one")     ' Returns 9 (starting from position 5)

' Replace
result = REPLACE("Hello World", "World", "Universe")  ' Returns "Hello Universe"
```

### Split and Join

```basic
' Split string to array
parts = SPLIT("a,b,c,d", ",")    ' Returns ["a", "b", "c", "d"]

' Join array to string
arr = ["a", "b", "c"]
result = JOIN(arr, "-")          ' Returns "a-b-c"
```

---

## Validation Functions

### Type Conversion

```basic
' String to number
num = VAL("42")                  ' Returns 42.0
num = VAL("3.14")                ' Returns 3.14
num = CINT("42.7")               ' Returns 43 (rounded integer)
num = CDBL("3.14")               ' Returns 3.14

' Number to string
str = STR(42)                    ' Returns "42"
str = CSTR(3.14)                 ' Returns "3.14"
```

### Null/Empty Checks

```basic
' Check for null
result = ISNULL(value)           ' true if null/unit

' Check for empty
result = ISEMPTY("")             ' true
result = ISEMPTY([])             ' true (empty array)
result = ISEMPTY(#{})            ' true (empty map)
```

### Type Checks

```basic
' Check types
result = ISSTRING("hello")       ' true
result = ISNUMBER(42)            ' true
result = ISNUMBER(3.14)          ' true
result = ISBOOL(true)            ' true
result = ISARRAY([1, 2, 3])      ' true
result = ISOBJECT(#{a: 1})       ' true
result = IS_NUMERIC("42")        ' true
result = IS_DATE("2025-01-22")   ' true

' Get type name
type = TYPEOF(value)             ' Returns type name as string
```

### Coalesce/Default Values

```basic
' Return first non-null value
result = NVL(null_value, "default")      ' Returns "default"
result = COALESCE(null_value, "default") ' Same as NVL
```

### Conditional

```basic
' Inline if
result = IIF(score >= 70, "Pass", "Fail")

' Choose from list (1-based index)
result = CHOOSE(2, ["A", "B", "C"])  ' Returns "B"
```

---

## Array Functions

### Creating Arrays

```basic
' Create array
arr = ARRAY(1, 2, 3, 4, 5)

' Create range
arr = RANGE(1, 10)              ' [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
arr = RANGE(0, 10, 2)           ' [0, 2, 4, 6, 8, 10]
```

### Array Info

```basic
' Array bounds
arr = [10, 20, 30, 40, 50]
upper = UBOUND(arr)              ' Returns 4 (last index)
lower = LBOUND(arr)              ' Returns 0 (first index)
count = COUNT(arr)               ' Returns 5 (length)
```

### Searching

```basic
' Check if contains
result = CONTAINS(arr, 30)       ' true
result = IN_ARRAY(30, arr)       ' true (alternate syntax)

' Find index
index = INDEX_OF(arr, 30)        ' Returns 2 (-1 if not found)
```

### Sorting

```basic
' Sort ascending
sorted = SORT(arr)

' Sort descending
sorted = SORT_DESC(arr)
```

### Uniqueness

```basic
' Remove duplicates
arr = [1, 2, 2, 3, 3, 3]
unique = UNIQUE(arr)             ' [1, 2, 3]
unique = DISTINCT(arr)           ' Same as UNIQUE
```

### Manipulation

```basic
' Add to end
arr = PUSH(arr, 60)

' Remove from end
last = POP(arr)

' Add to beginning
arr = UNSHIFT(arr, 0)

' Remove from beginning
first = SHIFT(arr)

' Reverse
reversed = REVERSE(arr)

' Get slice
part = SLICE(arr, 1)             ' From index 1 to end
part = SLICE(arr, 1, 3)          ' From index 1 to 3 (exclusive)
```

### Combining

```basic
' Concatenate arrays
combined = CONCAT(arr1, arr2)

' Flatten nested arrays
nested = [[1, 2], [3, 4], [5]]
flat = FLATTEN(nested)           ' [1, 2, 3, 4, 5]
```

### Element Access

```basic
' First and last elements
first = FIRST_ELEM(arr)
last = LAST_ELEM(arr)
```

---

## Error Handling

### Throwing Errors

```basic
' Throw an error
THROW "Something went wrong"

' Raise (alias for THROW)
RAISE "Invalid input"
```

### Error Objects

```basic
' Create error object (for inspection)
err = ERROR("Validation failed")
' err.error = true
' err.message = "Validation failed"
' err.timestamp = "2025-01-22 14:30:45"

' Check if value is error
IF IS_ERROR(result) THEN
    msg = GET_ERROR_MESSAGE(result)
    TALK "Error occurred: " + msg
END IF
```

### Assertions

```basic
' Assert condition
ASSERT score >= 0, "Score cannot be negative"
ASSERT NOT ISEMPTY(name), "Name is required"

' If assertion fails, throws error with message
```

### Logging

```basic
' Log messages at different levels
LOG_ERROR "Critical failure in processing"
LOG_WARN "Unusual condition detected"
LOG_INFO "Processing completed"
LOG_DEBUG "Variable value: " + STR(x)
```

### TRY/CATCH Pattern

While Rhai doesn't support traditional TRY...CATCH, use this pattern:

```basic
' Using error checking pattern
result = potentially_failing_operation()

IF IS_ERROR(result) THEN
    LOG_ERROR GET_ERROR_MESSAGE(result)
    result = TRY_RESULT(false, (), "Operation failed")
ELSE
    result = TRY_RESULT(true, result, "")
END IF

' result.success = true/false
' result.value = the actual value if successful
' result.error = error message if failed
```

---

## Campaign Examples

### Welcome Campaign

```basic
' Start welcome campaign for new lead
lead_email = "newuser@example.com"
lead_name = "John"

' Send immediate welcome
vars = #{name: lead_name, company: "Acme"}
SEND TEMPLATE "welcome-email-1", "email", lead_email, vars

' Schedule follow-ups
SET SCHEDULE DATEADD(TODAY(), 2, "day"), "send-welcome-2.bas"
SET SCHEDULE DATEADD(TODAY(), 5, "day"), "send-welcome-3.bas"
SET SCHEDULE DATEADD(TODAY(), 14, "day"), "send-welcome-final.bas"

' Score the lead
score = SCORE LEAD #{email: lead_email, name: lead_name}
TALK "Welcome campaign started. Initial score: " + score.score
```

### Social Media Campaign

```basic
' Product launch social media campaign
product_name = "AI Assistant Pro"
launch_date = "2025-02-15"

' Generate image
product_image = IMAGE "Product launch graphic for " + product_name

' Schedule posts across platforms
hashtags = "#AI #Automation #NewProduct"

' Day -7: Teaser
POST TO "instagram,facebook,linkedin" AT DATEADD(launch_date, -7, "day") + " 10:00" product_image, "Something big is coming... 🔥 " + hashtags

' Day -3: Feature preview  
POST TO "instagram,twitter" AT DATEADD(launch_date, -3, "day") + " 10:00" product_image, "Sneak peek! " + hashtags

' Launch day
POST TO "instagram,facebook,twitter,linkedin" AT launch_date + " 09:00" product_image, "🚀 IT'S HERE! " + product_name + " is now LIVE! " + hashtags

TALK "Social media campaign scheduled with " + 6 + " posts"
```

### Lead Nurturing with AI Scoring

```basic
' Process form submission
ON FORM SUBMIT "landing-page"
    ' Extract data
    lead = #{
        email: fields.email,
        name: fields.name,
        company: fields.company,
        job_title: fields.title,
        industry: fields.industry
    }
    
    ' AI-powered scoring
    score = AI SCORE LEAD lead
    
    ' Route based on score
    IF score.score >= 85 THEN
        ' Hot lead - immediate action
        CREATE TASK "Contact hot lead: " + lead.email, "sales", "high"
        SEND TEMPLATE "hot-lead-welcome", "email,sms", lead.email, lead
    ELSE IF score.score >= 70 THEN
        ' Warm lead - accelerated nurture
        SEND TEMPLATE "warm-welcome", "email", lead.email, lead
        SET SCHEDULE DATEADD(NOW(), 3, "day"), "warm-nurture-2.bas"
    ELSE
        ' Cold lead - standard drip
        SEND TEMPLATE "cold-welcome", "email", lead.email, lead
        SET SCHEDULE DATEADD(NOW(), 7, "day"), "cold-nurture-2.bas"
    END IF
    
    ' Save to CRM
    SAVE "leads", lead.email, lead.name, score.score, score.grade, NOW()
END ON
```

---

## Configuration

### Social Media Credentials

Store in `bot_settings` table:
- `instagram_credentials`: `{access_token, user_id}`
- `facebook_credentials`: `{access_token, page_id}`
- `linkedin_credentials`: `{access_token, person_urn}`
- `twitter_credentials`: `{bearer_token}`

### Lead Scoring Weights

Default weights (customizable per bot):

| Factor | Default Weight |
|--------|---------------|
| Company Size | 10 |
| Industry Match | 15 |
| Job Title | 15 |
| Location | 5 |
| Email Opens | 5 |
| Email Clicks | 10 |
| Page Visits | 5 |
| Form Submissions | 15 |
| Content Downloads | 10 |
| Pricing Page Visits | 20 |
| Demo Requests | 25 |
| Trial Signups | 30 |
| Inactivity Penalty | -15 |

---

*Last updated: January 2025*
*General Bots v5.0*