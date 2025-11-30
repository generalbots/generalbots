# HEAR Keyword

The `HEAR` keyword pauses script execution and waits for user input. With optional type validation, it automatically verifies and normalizes input, retrying with helpful error messages when validation fails.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Basic Syntax

```basic
HEAR variable_name
HEAR variable_name AS TYPE
HEAR variable_name AS "Option1", "Option2", "Option3"
```

The simplest form accepts any input. Adding `AS TYPE` enables automatic validation with user-friendly retry prompts.

## Simple HEAR

```basic
TALK "What would you like to know?"
HEAR question
TALK "You asked: " + question
```

The script waits for any user message and stores it in the variable.

## Validated Input Types

When using `HEAR AS <TYPE>`, the system validates input automatically, retries up to 3 times with helpful messages, and returns normalized values.

### Text Types

**EMAIL** validates email format and normalizes to lowercase:

```basic
TALK "What's your email address?"
HEAR email AS EMAIL
TALK "We'll send confirmation to: " + email
```

Accepts: `User@Example.COM` → Returns: `user@example.com`

**NAME** validates name format with proper capitalization:

```basic
TALK "What's your full name?"
HEAR name AS NAME
```

Accepts letters, spaces, hyphens, apostrophes. Auto-capitalizes: `john doe` → `John Doe`

**URL** validates and normalizes URLs:

```basic
TALK "Enter your website:"
HEAR website AS URL
```

Auto-adds `https://` if protocol missing.

**PASSWORD** validates minimum strength:

```basic
TALK "Create a password (minimum 8 characters):"
HEAR password AS PASSWORD
```

Requires 8+ characters. Never echoes the actual password back.

**COLOR** accepts color names or hex values:

```basic
HEAR color AS COLOR
```

Accepts: `red`, `#FF0000`, `rgb(255, 0, 0)` → Returns: `#FF0000`

### Numeric Types

**INTEGER** validates whole numbers:

```basic
TALK "How many items?"
HEAR quantity AS INTEGER
```

Removes formatting (commas, spaces). Returns numeric value.

**FLOAT** / **DECIMAL** validates decimal numbers:

```basic
TALK "Enter the temperature:"
HEAR temperature AS FLOAT
```

Handles both `.` and `,` as decimal separators.

**MONEY** / **CURRENCY** / **AMOUNT** validates monetary values:

```basic
TALK "How much to transfer?"
HEAR amount AS MONEY
```

Accepts: `100`, `1,234.56`, `R$ 100,00`, `$100.00` → Returns: `1234.56`

**CREDITCARD** / **CARD** validates card numbers with Luhn algorithm:

```basic
TALK "Enter your card number:"
HEAR card AS CREDITCARD
```

Returns masked format: `4111 **** **** 1111`

### Date and Time Types

**DATE** validates and parses dates:

```basic
TALK "When is your birthday?"
HEAR birthday AS DATE
```

Accepts: `25/12/2024`, `12/25/2024`, `2024-12-25`, `December 25, 2024`, `today`, `tomorrow`, `hoje`, `amanhã`

Returns: ISO format `YYYY-MM-DD`

**HOUR** / **TIME** validates time input:

```basic
TALK "What time for the meeting?"
HEAR meeting_time AS HOUR
```

Accepts: `14:30`, `2:30 PM` → Returns: `14:30`

### Brazilian Document Types

**CPF** validates Brazilian individual taxpayer ID:

```basic
TALK "Enter your CPF:"
HEAR cpf AS CPF
```

Validates 11 digits with mod 11 check. Returns: `123.456.789-09`

**CNPJ** validates Brazilian company taxpayer ID:

```basic
TALK "Enter your company's CNPJ:"
HEAR cnpj AS CNPJ
```

Validates 14 digits. Returns: `12.345.678/0001-95`

### Contact Types

**MOBILE** / **PHONE** validates phone numbers:

```basic
TALK "What's your phone number?"
HEAR phone AS MOBILE
```

Accepts 10-15 digits, auto-formats based on detected country.

**ZIPCODE** / **CEP** / **POSTALCODE** validates postal codes:

```basic
HEAR cep AS ZIPCODE
```

Supports Brazilian CEP, US ZIP, UK postcode formats.

### Menu Selection

Provide options directly in the HEAR statement:

```basic
TALK "Choose your fruit:"
HEAR fruit AS "Apple", "Banana", "Orange", "Mango"
```

Accepts exact match, case-insensitive match, numeric selection (`1`, `2`, `3`), or partial match if unique.

**BOOLEAN** validates yes/no responses:

```basic
TALK "Do you agree to the terms?"
HEAR agreed AS BOOLEAN
IF agreed THEN
    TALK "Thank you!"
END IF
```

True: `yes`, `y`, `sim`, `ok`, `sure`, `confirm`
False: `no`, `n`, `não`, `cancel`, `deny`

**LANGUAGE** validates language codes:

```basic
HEAR language AS LANGUAGE
```

Accepts: `en`, `pt`, `English`, `Português` → Returns: ISO 639-1 code

### Media Types

**IMAGE** / **PHOTO** waits for image upload:

```basic
TALK "Send a photo of your document:"
HEAR document_photo AS IMAGE
```

Returns URL to uploaded image.

**QRCODE** waits for image and decodes QR:

```basic
TALK "Send me the QR code:"
HEAR qr_data AS QRCODE
```

Uses vision API to decode. Returns decoded data.

**AUDIO** / **VOICE** transcribes audio input:

```basic
TALK "Send a voice message:"
HEAR transcription AS AUDIO
```

Uses Whisper for transcription. Returns text.

**VIDEO** analyzes video content:

```basic
TALK "Send a video of the issue:"
HEAR video_description AS VIDEO
```

Uses vision API to describe. Returns description.

**FILE** / **DOCUMENT** waits for file upload:

```basic
TALK "Upload your contract:"
HEAR contract AS DOCUMENT
```

Accepts PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX, TXT, CSV. Returns URL.

### Authentication

**LOGIN** waits for OAuth completion:

```basic
TALK "Click the link to authenticate:"
HEAR user AS LOGIN
```

Returns user object with tokens after OAuth callback.

## Complete Examples

### Registration Flow

```basic
TALK "Let's create your account!"

TALK "What's your full name?"
HEAR name AS NAME

TALK "Enter your email address:"
HEAR email AS EMAIL

TALK "Enter your CPF:"
HEAR cpf AS CPF

TALK "What's your phone number?"
HEAR phone AS MOBILE

TALK "Choose a password:"
HEAR password AS PASSWORD

TALK "What's your birth date?"
HEAR birthdate AS DATE

TALK "Select your gender:"
HEAR gender AS "Male", "Female", "Other", "Prefer not to say"

SAVE "users.csv", name, email, cpf, phone, birthdate, gender, NOW()
TALK "Account created for " + name + "!"
```

### Payment Flow

```basic
TALK "Enter the amount:"
HEAR amount AS MONEY

IF amount < 1 THEN
    TALK "Minimum payment is R$ 1.00"
    RETURN
END IF

TALK "How would you like to pay?"
HEAR method AS "Credit Card", "Debit Card", "PIX", "Boleto"

TALK "Confirm payment of R$ " + FORMAT(amount, "#,##0.00") + "?"
HEAR confirm AS BOOLEAN

IF confirm THEN
    TALK "Processing payment..."
ELSE
    TALK "Payment cancelled."
END IF
```

## Validation Behavior

When validation fails, the system automatically prompts for correction:

```
User: my email
Bot: Please enter a valid email address (e.g., user@example.com)
User: test@example.com
Bot: Email confirmed!
```

After 3 failed attempts, execution continues with an empty value. Check for this:

```basic
HEAR email AS EMAIL
IF email = "" THEN
    TALK "Unable to validate email. Please contact support."
    RETURN
END IF
```

## Best Practices

**Always use appropriate types** — automatic validation is safer than manual checking:

```basic
' Good
HEAR email AS EMAIL

' Avoid
HEAR email
IF NOT email CONTAINS "@" THEN ...
```

**Provide context before HEAR** — users should know what to enter:

```basic
TALK "Enter the transfer amount (minimum R$ 1.00):"
HEAR amount AS MONEY
```

**Use menus for limited options**:

```basic
HEAR method AS "Credit Card", "Debit Card", "PIX"
```

**Combine with SET CONTEXT** for AI-enhanced input handling:

```basic
SET CONTEXT "You are a banking assistant. Confirm amounts before processing."
HEAR amount AS MONEY
```

## Validation Summary

| Type | Example Input | Normalized Output |
|------|---------------|-------------------|
| EMAIL | `User@Example.COM` | `user@example.com` |
| NAME | `john DOE` | `John Doe` |
| INTEGER | `1,234` | `1234` |
| MONEY | `R$ 1.234,56` | `1234.56` |
| DATE | `25/12/2024` | `2024-12-25` |
| HOUR | `2:30 PM` | `14:30` |
| BOOLEAN | `yes` / `sim` | `true` |
| CPF | `12345678909` | `123.456.789-09` |
| MOBILE | `11999998888` | `(11) 99999-8888` |
| CREDITCARD | `4111111111111111` | `4111 **** **** 1111` |
| QRCODE | [image] | decoded data |
| AUDIO | [audio file] | transcribed text |

## See Also

- [TALK Keyword](./keyword-talk.md) - Output messages
- [Dialog Basics](./basics.md) - Conversation patterns
- [Template Variables](./template-variables.md) - Variable substitution