# HEAR Keyword - Input Validation Reference

> Complete reference for HEAR keyword with automatic input validation in General Bots BASIC

## Overview

The `HEAR` keyword waits for user input with optional automatic validation. When using `HEAR AS <TYPE>`, the system will:

1. Wait for user input
2. Validate against the specified type
3. **Automatically retry** with a helpful error message if invalid
4. Return the normalized/parsed value once valid

This eliminates the need for manual validation loops and provides a consistent, user-friendly experience.

---

## Table of Contents

1. [Basic HEAR](#basic-hear)
2. [Text Validation Types](#text-validation-types)
3. [Numeric Types](#numeric-types)
4. [Date/Time Types](#datetime-types)
5. [Brazilian Document Types](#brazilian-document-types)
6. [Contact Types](#contact-types)
7. [Menu Selection](#menu-selection)
8. [Media Types](#media-types)
9. [Authentication Types](#authentication-types)
10. [Examples](#examples)
11. [Best Practices](#best-practices)

---

## Basic HEAR

```basic
' Simple HEAR without validation - accepts any input
HEAR response
TALK "You said: " + response
```

---

## Text Validation Types

### HEAR AS EMAIL

Validates email address format and normalizes to lowercase.

```basic
TALK "What's your email address?"
HEAR email AS EMAIL
TALK "We'll send confirmation to: " + email
```

**Validation:**
- Must contain `@` symbol
- Must have valid domain format
- Normalized to lowercase

**Error message:** "Please enter a valid email address (e.g., user@example.com)"

---

### HEAR AS NAME

Validates name format (letters, spaces, hyphens, apostrophes).

```basic
TALK "What's your full name?"
HEAR name AS NAME
TALK "Nice to meet you, " + name + "!"
```

**Validation:**
- Minimum 2 characters
- Maximum 100 characters
- Only letters, spaces, hyphens, apostrophes
- Auto-capitalizes first letter of each word

**Error message:** "Please enter a valid name (letters and spaces only)"

---

### HEAR AS URL

Validates and normalizes URL format.

```basic
TALK "Enter your website URL:"
HEAR website AS URL
TALK "I'll check " + website
```

**Validation:**
- Valid URL format
- Auto-adds `https://` if protocol missing

**Error message:** "Please enter a valid URL"

---

### HEAR AS PASSWORD

Validates password strength (minimum requirements).

```basic
TALK "Create a password (minimum 8 characters):"
HEAR password AS PASSWORD
' Returns "[PASSWORD SET]" - actual password stored securely
```

**Validation:**
- Minimum 8 characters
- Returns strength indicator (weak/medium/strong)
- Never echoes the actual password

**Error message:** "Password must be at least 8 characters"

---

### HEAR AS COLOR

Validates and normalizes color values.

```basic
TALK "Pick a color:"
HEAR color AS COLOR
TALK "You selected: " + color  ' Returns hex format like #FF0000
```

**Accepts:**
- Named colors: "red", "blue", "green", etc.
- Hex format: "#FF0000" or "FF0000"
- RGB format: "rgb(255, 0, 0)"

**Returns:** Normalized hex format (#RRGGBB)

---

### HEAR AS UUID

Validates UUID/GUID format.

```basic
TALK "Enter the transaction ID:"
HEAR transaction_id AS UUID
```

---

## Numeric Types

### HEAR AS INTEGER

Validates and parses integer numbers.

```basic
TALK "How old are you?"
HEAR age AS INTEGER
TALK "In 10 years you'll be " + STR(age + 10)
```

**Validation:**
- Accepts whole numbers only
- Removes formatting (commas, spaces)
- Returns numeric value

**Error message:** "Please enter a valid whole number"

---

### HEAR AS FLOAT / DECIMAL

Validates and parses decimal numbers.

```basic
TALK "Enter the temperature:"
HEAR temperature AS FLOAT
TALK "Temperature is " + STR(temperature) + "°C"
```

**Validation:**
- Accepts decimal numbers
- Handles both `.` and `,` as decimal separator
- Returns numeric value rounded to 2 decimal places

---

### HEAR AS MONEY / CURRENCY / AMOUNT

Validates and normalizes monetary amounts.

```basic
TALK "How much would you like to transfer?"
HEAR amount AS MONEY
TALK "Transferring R$ " + FORMAT(amount, "#,##0.00")
```

**Accepts:**
- "100"
- "100.00"
- "1,234.56" (US format)
- "1.234,56" (Brazilian/European format)
- "R$ 100,00"
- "$100.00"

**Returns:** Normalized decimal value (e.g., "1234.56")

**Error message:** "Please enter a valid amount (e.g., 100.00 or R$ 100,00)"

---

### HEAR AS CREDITCARD / CARD

Validates credit card number using Luhn algorithm.

```basic
TALK "Enter your card number:"
HEAR card AS CREDITCARD
' Returns masked format: "4111 **** **** 1111"
```

**Validation:**
- 13-19 digits
- Passes Luhn checksum
- Detects card type (Visa, Mastercard, Amex, etc.)

**Returns:** Masked card number with metadata about card type

---

## Date/Time Types

### HEAR AS DATE

Validates and parses date input.

```basic
TALK "When is your birthday?"
HEAR birthday AS DATE
TALK "Your birthday is " + FORMAT(birthday, "MMMM d")
```

**Accepts multiple formats:**
- "25/12/2024" (DD/MM/YYYY)
- "12/25/2024" (MM/DD/YYYY)
- "2024-12-25" (ISO format)
- "25 Dec 2024"
- "December 25, 2024"
- "today", "tomorrow", "yesterday"
- "hoje", "amanhã", "ontem" (Portuguese)

**Returns:** Normalized ISO date (YYYY-MM-DD)

**Error message:** "Please enter a valid date (e.g., 25/12/2024 or 2024-12-25)"

---

### HEAR AS HOUR / TIME

Validates and parses time input.

```basic
TALK "What time should we schedule the meeting?"
HEAR meeting_time AS HOUR
TALK "Meeting scheduled for " + meeting_time
```

**Accepts:**
- "14:30" (24-hour format)
- "2:30 PM" (12-hour format)
- "14:30:00" (with seconds)

**Returns:** Normalized 24-hour format (HH:MM)

**Error message:** "Please enter a valid time (e.g., 14:30 or 2:30 PM)"

---

## Brazilian Document Types

### HEAR AS CPF

Validates Brazilian CPF (individual taxpayer ID).

```basic
TALK "Enter your CPF:"
HEAR cpf AS CPF
TALK "CPF validated: " + cpf  ' Returns formatted: 123.456.789-09
```

**Validation:**
- 11 digits
- Valid check digits (mod 11 algorithm)
- Rejects known invalid patterns (all same digit)

**Returns:** Formatted CPF (XXX.XXX.XXX-XX)

**Error message:** "Please enter a valid CPF (11 digits)"

---

### HEAR AS CNPJ

Validates Brazilian CNPJ (company taxpayer ID).

```basic
TALK "Enter your company's CNPJ:"
HEAR cnpj AS CNPJ
TALK "CNPJ validated: " + cnpj  ' Returns formatted: 12.345.678/0001-95
```

**Validation:**
- 14 digits
- Valid check digits

**Returns:** Formatted CNPJ (XX.XXX.XXX/XXXX-XX)

**Error message:** "Please enter a valid CNPJ (14 digits)"

---

## Contact Types

### HEAR AS MOBILE / PHONE / TELEPHONE

Validates phone number format.

```basic
TALK "What's your phone number?"
HEAR phone AS MOBILE
TALK "We'll send SMS to: " + phone
```

**Validation:**
- 10-15 digits
- Auto-formats based on detected country

**Returns:** Formatted phone number

**Error message:** "Please enter a valid mobile number"

---

### HEAR AS ZIPCODE / CEP / POSTALCODE

Validates postal code format.

```basic
TALK "What's your ZIP code?"
HEAR cep AS ZIPCODE
TALK "Your ZIP code is: " + cep
```

**Supports:**
- Brazilian CEP: 8 digits → "12345-678"
- US ZIP: 5 or 9 digits → "12345" or "12345-6789"
- UK postcode: alphanumeric → "SW1A 1AA"

**Returns:** Formatted postal code with country detection

---

## Menu Selection

### HEAR AS "Option1", "Option2", "Option3"

Presents a menu and validates selection.

```basic
TALK "Choose your preferred fruit:"
HEAR fruit AS "Apple", "Banana", "Orange", "Mango"
TALK "You selected: " + fruit
```

**Accepts:**
- Exact match: "Apple"
- Case-insensitive: "apple"
- Numeric selection: "1", "2", "3"
- Partial match: "app" → "Apple" (if unique)

**Automatically adds suggestions** for the menu options.

**Error message:** "Please select one of: Apple, Banana, Orange, Mango"

---

### HEAR AS BOOLEAN

Validates yes/no response.

```basic
TALK "Do you agree to the terms?"
HEAR agreed AS BOOLEAN
IF agreed THEN
    TALK "Thank you for agreeing!"
ELSE
    TALK "You must agree to continue."
END IF
```

**Accepts (true):** "yes", "y", "true", "1", "sim", "ok", "sure", "confirm"

**Accepts (false):** "no", "n", "false", "0", "não", "cancel", "deny"

**Returns:** "true" or "false" (with boolean metadata)

**Error message:** "Please answer yes or no"

---

### HEAR AS LANGUAGE

Validates language code or name.

```basic
TALK "What language do you prefer?"
HEAR language AS LANGUAGE
SET CONTEXT LANGUAGE language
TALK "Language set to: " + language
```

**Accepts:**
- ISO codes: "en", "pt", "es", "fr", "de"
- Full names: "English", "Portuguese", "Spanish"
- Native names: "Português", "Español", "Français"

**Returns:** ISO 639-1 language code

---

## Media Types

### HEAR AS IMAGE / PHOTO / PICTURE

Waits for image upload.

```basic
TALK "Please send a photo of your document:"
HEAR document_photo AS IMAGE
TALK "Image received: " + document_photo
' Returns URL to the uploaded image
```

**Validation:**
- Must receive image attachment
- Accepts: JPG, PNG, GIF, WebP

**Error message:** "Please send an image"

---

### HEAR AS QRCODE

Waits for image with QR code and reads it.

```basic
TALK "Send me a photo of the QR code:"
HEAR qr_data AS QRCODE
TALK "QR code contains: " + qr_data
```

**Process:**
1. Waits for image upload
2. Calls BotModels vision API to decode QR
3. Returns the decoded data

**Error message:** "Please send an image containing a QR code"

---

### HEAR AS AUDIO / VOICE / SOUND

Waits for audio input and transcribes to text.

```basic
TALK "Send me a voice message:"
HEAR transcription AS AUDIO
TALK "You said: " + transcription
```

**Process:**
1. Waits for audio attachment
2. Calls BotModels speech-to-text API
3. Returns transcribed text

**Error message:** "Please send an audio file or voice message"

---

### HEAR AS VIDEO

Waits for video upload and describes content.

```basic
TALK "Send a video of the problem:"
HEAR video_description AS VIDEO
TALK "I can see: " + video_description
```

**Process:**
1. Waits for video attachment
2. Calls BotModels vision API to describe
3. Returns AI-generated description

**Error message:** "Please send a video"

---

### HEAR AS FILE / DOCUMENT / DOC / PDF

Waits for document upload.

```basic
TALK "Please upload your contract:"
HEAR contract AS DOCUMENT
TALK "Document received: " + contract
```

**Accepts:** PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX, TXT, CSV

**Returns:** URL to the uploaded file

---

## Authentication Types

### HEAR AS LOGIN

Waits for Active Directory/OAuth login completion.

```basic
TALK "Please click the link to authenticate:"
HEAR user AS LOGIN
TALK "Welcome, " + user.name + "!"
```

**Process:**
1. Generates authentication URL
2. Waits for OAuth callback
3. Returns user object with tokens

---

## Examples

### Complete Registration Flow

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

' All inputs are now validated and normalized
TALK "Creating account for " + name + "..."

TABLE new_user
    ROW name, email, cpf, phone, birthdate, gender, NOW()
END TABLE
SAVE "users.csv", new_user

TALK "✅ Account created successfully!"
```

### Payment Flow

```basic
TALK "💳 Let's process your payment"

TALK "Enter the amount:"
HEAR amount AS MONEY

IF amount < 1 THEN
    TALK "Minimum payment is R$ 1.00"
    RETURN
END IF

TALK "How would you like to pay?"
HEAR method AS "Credit Card", "Debit Card", "PIX", "Boleto"

IF method = "PIX" THEN
    TALK "Enter the PIX key (phone, email, or CPF):"
    ' Note: We could create HEAR AS PIX_KEY if needed
    HEAR pix_key
ELSEIF method = "Boleto" THEN
    TALK "Enter the barcode (47-48 digits):"
    HEAR barcode AS INTEGER
END IF

TALK "Confirm payment of R$ " + FORMAT(amount, "#,##0.00") + "?"
HEAR confirm AS BOOLEAN

IF confirm THEN
    TALK "✅ Processing payment..."
ELSE
    TALK "Payment cancelled."
END IF
```

### Customer Support with Media

```basic
TALK "How can I help you today?"
HEAR issue AS "Report a bug", "Request feature", "Billing question", "Other"

IF issue = "Report a bug" THEN
    TALK "Please describe the problem:"
    HEAR description
    
    TALK "Can you send a screenshot of the issue?"
    HEAR screenshot AS IMAGE
    
    TALK "Thank you! We've logged your bug report."
    TALK "Reference: BUG-" + FORMAT(NOW(), "yyyyMMddHHmmss")
    
ELSEIF issue = "Billing question" THEN
    TALK "Please upload your invoice or send the transaction ID:"
    HEAR reference
END IF
```

---

## Best Practices

### 1. Always Use Appropriate Types

```basic
' ❌ Bad - no validation
HEAR email
IF NOT email CONTAINS "@" THEN
    TALK "Invalid email"
    ' Need to implement retry logic...
END IF

' ✅ Good - automatic validation and retry
HEAR email AS EMAIL
' Guaranteed to be valid when we get here
```

### 2. Combine with Context

```basic
SET CONTEXT "You are a helpful banking assistant. 
When asking for monetary values, always confirm before processing."

TALK "How much would you like to withdraw?"
HEAR amount AS MONEY
' LLM and validation work together
```

### 3. Use Menu for Limited Options

```basic
' ❌ Bad - open-ended when options are known
HEAR payment_method
IF payment_method <> "credit" AND payment_method <> "debit" THEN
    ' Handle unknown input...
END IF

' ✅ Good - constrained to valid options
HEAR payment_method AS "Credit Card", "Debit Card", "PIX"
```

### 4. Provide Context Before HEAR

```basic
' ❌ Bad - no context
HEAR value AS MONEY

' ✅ Good - user knows what to enter
TALK "Enter the transfer amount (minimum R$ 1.00):"
HEAR amount AS MONEY
```

### 5. Use HEAR AS for Security-Sensitive Data

```basic
' CPF is automatically validated
HEAR cpf AS CPF

' Credit card passes Luhn check and is masked
HEAR card AS CREDITCARD

' Password never echoed back
HEAR password AS PASSWORD
```

---

## Error Handling

Validation errors are handled automatically, but you can customize:

```basic
' The system automatically retries up to 3 times
' After 3 failures, execution continues with empty value

' You can check if validation succeeded:
HEAR email AS EMAIL
IF email = "" THEN
    TALK "Unable to validate email after multiple attempts."
    TALK "Please contact support for assistance."
    RETURN
END IF
```

---

## Metadata Access

Some validation types provide additional metadata:

```basic
HEAR card AS CREDITCARD
' card = "**** **** **** 1234"
' Metadata available: card_type, last_four

HEAR date AS DATE
' date = "2024-12-25"
' Metadata available: original input, parsed format

HEAR audio AS AUDIO
' audio = "transcribed text here"
' Metadata available: language, confidence
```

---

## Integration with BotModels

Media types (QRCODE, AUDIO, VIDEO) automatically call BotModels services:

| Type | BotModels Endpoint | Service |
|------|-------------------|---------|
| QRCODE | `/api/v1/vision/qrcode` | QR Code detection |
| AUDIO | `/api/v1/speech/to-text` | Whisper transcription |
| VIDEO | `/api/v1/vision/describe-video` | BLIP2 video description |
| IMAGE (with question) | `/api/v1/vision/vqa` | Visual Q&A |

Configure BotModels URL in `config.csv`:
```
botmodels-url,http://localhost:8001
botmodels-enabled,true
```

---

## Summary Table

| Type | Example Input | Normalized Output |
|------|---------------|-------------------|
| EMAIL | "User@Example.COM" | "user@example.com" |
| NAME | "john DOE" | "John Doe" |
| INTEGER | "1,234" | 1234 |
| MONEY | "R$ 1.234,56" | "1234.56" |
| DATE | "25/12/2024" | "2024-12-25" |
| HOUR | "2:30 PM" | "14:30" |
| BOOLEAN | "yes" / "sim" | "true" |
| CPF | "12345678909" | "123.456.789-09" |
| MOBILE | "11999998888" | "(11) 99999-8888" |
| CREDITCARD | "4111111111111111" | "4111 **** **** 1111" |
| QRCODE | [image] | "decoded QR data" |
| AUDIO | [audio file] | "transcribed text" |

---

*HEAR AS validation - Making input handling simple, secure, and user-friendly.*