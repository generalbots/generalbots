# Master Keywords Index - General Bots Templates

Complete reference of all available keywords across all templates.

**Last Updated:** 2024  
**Total Templates:** 19  
**Total Keywords:** 90+  
**Total Code Lines:** 4,000+

---

## 📦 Template Overview

### 1. **default.gbai** - Core Keywords (All Templates Can Use)
Universal keywords that ANY template can call.

| Keyword | Description | Parameters | API Used |
|---------|-------------|------------|----------|
| `WEATHER` | Get weather forecast for any city | location | Open-Meteo (free) |
| `TRANSLATE` | Translate text between languages | text, from_lang, to_lang | LibreTranslate (free) |
| `SEND EMAIL` | Send email to any recipient | to_email, subject, body, from_email | SMTP Queue |
| `SEND SMS` | Send SMS to any phone number | phone_number, message, from_number | SMS Queue |
| `CALCULATE` | Perform math calculations | expression | Built-in |

**Files:**
- `default.gbdialog/weather.vbs` (141 lines)
- `default.gbdialog/translate.vbs` (104 lines)
- `default.gbdialog/send-email.vbs` (69 lines)
- `default.gbdialog/send-sms.vbs` (98 lines)
- `default.gbdialog/calculate.vbs` (217 lines)

---

### 2. **public-apis.gbai** - Free Public APIs (76 Keywords)
Comprehensive collection of free, no-auth APIs.

#### Weather & Environment (8 keywords)
- `7Timer! Astro Weather` - Astronomical weather forecast
- `7Timer! Civil Weather` - 7-day weather forecast
- `Open-Meteo Weather` - Real-time weather data
- `MetaWeather Location Search` - Search locations by city
- `Rain Viewer Radar Map` - Rain radar timestamps
- `OpenSenseMap Weather Stations` - Personal weather station data
- `AQICN Air Quality` - Air quality index by city
- `Get Weather Icon` - Weather condition to emoji

#### Animals & Pets (17 keywords)
- `Random Cat Fact` - Cat facts
- `Random Dog Fact` - Dog facts
- `Random Dog Image` - Dog pictures
- `Random Cat Image` - Cat pictures
- `Random Fox Image` - Fox pictures
- `Random Duck Image` - Duck pictures
- `Random Shiba Inu Image` - Shiba Inu pictures
- `HTTP Cat` - HTTP status code cats
- `HTTP Dog` - HTTP status code dogs
- `PlaceBear Placeholder` - Bear placeholder images
- `PlaceDog Placeholder` - Dog placeholder images
- `PlaceKitten Placeholder` - Kitten placeholder images
- `MeowFacts` - Multiple cat facts
- `Random Axolotl` - Axolotl images and facts
- `Zoo Animals Info` - Zoo animal information
- `Dog Breeds List` - All dog breeds
- `Specific Dog Breed Image` - Image by breed name

#### Entertainment & Humor (19 keywords)
- `Chuck Norris Joke` - Random Chuck Norris joke
- `Chuck Norris Categories` - Available joke categories
- `Chuck Norris Joke by Category` - Category-specific jokes
- `Dad Joke` - Random dad joke
- `Search Dad Jokes` - Search dad jokes by term
- `Bored Activity` - Random activity suggestion
- `Bored Activity by Type` - Activity by category
- `Random Useless Fact` - Useless but true facts
- `Random Fun Fact` - Fun facts
- `Kanye West Quote` - Kanye quotes
- `Advice Slip` - Random advice
- `Search Advice` - Search advice by keyword
- `Corporate Buzzword` - Corporate buzzword generator
- `Yo Momma Joke` - Yo Momma jokes
- `Random Quote` - Inspirational quotes
- `Quote by Author` - Author-specific quotes
- `Programming Quote` - Programming-related quotes
- `Zen Quote` - Zen/Stoicism quotes
- `Affirmation` - Positive affirmations

#### Food & Drink (13 keywords)
- `Random Coffee Image` - Coffee images
- `Random Food Dish` - Food dish images
- `Random Food by Category` - Category-specific food
- `Random Meal Recipe` - Full meal recipes
- `Search Meal by Name` - Search meals
- `Random Cocktail Recipe` - Cocktail recipes
- `Search Cocktail by Name` - Search cocktails
- `Search Cocktail by Ingredient` - Cocktails by ingredient
- `Fruit Information` - Nutritional fruit data
- `All Fruits List` - Complete fruits database
- `Fruits by Family` - Fruits by botanical family
- `Random Taco Recipe` - Taco recipes
- `PunkAPI Beer Info` - Beer recipes and data

#### Data Utility & Geocoding (19 keywords)
- `Generate UUID` - Single UUID generation
- `Generate Multiple UUIDs` - Multiple UUIDs
- `Get My IP Address` - Current public IP
- `Get IP Geolocation` - IP location data
- `Check if Number is Even` - Humor API
- `Random Data Generator` - Test user data
- `Generate Lorem Ipsum` - Lorem ipsum text
- `QR Code Generator` - QR code images
- `Barcode Generator` - Barcode images
- `Country Information` - Detailed country data
- `All Countries List` - 250+ countries
- `Countries by Region` - Countries by continent
- `Currency Converter` - Currency exchange
- `Timezone Info` - Current time by timezone
- `All Timezones List` - 400+ timezones
- `Public Holidays` - Holidays by country/year
- `Number Facts` - Interesting number facts
- `Date Facts` - Historical date facts
- `Random User Generator` - Realistic user profiles

**Files:**
- `public-apis.gbdialog/weather-apis.vbs` (244 lines)
- `public-apis.gbdialog/animals-apis.vbs` (366 lines)
- `public-apis.gbdialog/entertainment-apis.vbs` (438 lines)
- `public-apis.gbdialog/food-apis.vbs` (503 lines)
- `public-apis.gbdialog/data-utility-apis.vbs` (568 lines)
- `public-apis.gbdialog/science-space-apis.vbs` (595 lines)

---

### 3. **marketing.gbai** - Marketing & Social Media
Keywords for marketing automation and social media posting.

| Keyword | Description | Parameters | Status |
|---------|-------------|------------|--------|
| `GET IMAGE` | Generate/fetch image for marketing | prompt | ✅ Ready |
| `POST TO INSTAGRAM` | Post to Instagram account | username, password, image, caption | ⚙️ Requires API setup |
| `BROADCAST` | Send message to multiple users | message, recipient_list | 📝 Existing |
| `POSTER` | Create automated social media posts | - | 📝 Existing |

**Files:**
- `marketing.gbdialog/get-image.vbs` (47 lines) - NEW
- `marketing.gbdialog/post-to-instagram.vbs` (46 lines) - NEW
- `marketing.gbdialog/broadcast.bas` - Existing
- `marketing.gbdialog/poster.bas` - Existing

---

### 4. **ai-search.gbai** - Document Search & QR Codes
AI-powered document search and QR code processing.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `QR` | Scan and process QR codes | image |
| `START` | Initialize AI search session | - |

**Files:**
- `ai-search.gbdialog/qr.bas` - Existing
- `ai-search.gbdialog/start.bas` - Existing

---

### 5. **edu.gbai** - Education & Enrollment
Student enrollment and educational processes.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `ENROLLMENT` | Student enrollment process | name, birthday, email, personalid, address |
| `START` | Initialize education bot | - |

**Files:**
- `edu.gbdialog/enrollment.bas` - Existing
- `edu.gbdialog/start.bas` - Existing

---

### 6. **store.gbai** - E-commerce
Online store and shopping cart management.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `CHECKOUT` | Process shopping cart checkout | NomeDoCliente, pedidos |
| `START` | Initialize store bot | - |

**Files:**
- `store.gbdialog/checkout.bas` - Existing
- `store.gbdialog/start.bas` - Existing

---

### 7. **llm-tools.gbai** - LLM Integration Tools
Tools for LLM-powered bots.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `GET PRICE` | Get product price from database | product |
| `START` | Initialize LLM tools | - |

**Files:**
- `llm-tools.gbdialog/get-price.bas` - Existing
- `llm-tools.gbdialog/start.bas` - Existing

---

### 8. **llm-server.gbai** - LLM REST API Server
Turn LLM into REST API endpoints.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `START` | Initialize LLM server | - |

**Files:**
- `llm-server.gbdialog/start.bas` - Existing

---

### 9. **reminder.gbai** - Reminders & Scheduling
Task reminders and scheduling system.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `REMINDER` | Manage reminders | - |
| `ADD REMINDER` | Add new reminder | - |
| `START` | Initialize reminder bot | - |

**Files:**
- `reminder.gbdialog/reminder.bas` - Existing
- `reminder.gbdialog/add-reminder.bas` - Existing
- `reminder.gbdialog/start.bas` - Existing

---

### 10. **talk-to-data.gbai** - Data Analytics
Talk to your data with natural language.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `START` | Initialize data analytics | - |
| `NOTIFY LATEST ORDERS` | Notify about new orders | - |

**Files:**
- `talk-to-data.gbdialog/start.bas` - Existing
- `talk-to-data.gbdialog/notify-latest-orders.bas` - Existing

---

### 11. **announcements.gbai** - Announcements System
Broadcast announcements and notifications.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `AUTH` | Authentication | - |
| `CHANGE SUBJECT` | Change announcement subject | - |
| `START` | Initialize announcements | - |
| `UPDATE SUMMARY` | Update announcement summary | - |

**Files:**
- `announcements.gbdialog/auth.bas` - Existing
- `announcements.gbdialog/change-subject.bas` - Existing
- `announcements.gbdialog/start.bas` - Existing
- `announcements.gbdialog/update-summary.bas` - Existing

---

### 12. **api-client.gbai** - External API Integration
Connect to external APIs like Microsoft Partner Center.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `MSFT PARTNER CENTER` | Microsoft Partner Center API | - |
| `CLIMATE` | Weather/climate data | location, unit |

**Files:**
- `api-client.gbdialog/msft-partner-center.bas` - Existing
- `api-client.gbdialog/climate.vbs` - Existing

---

### 13. **backup.gbai** - Backup System
Automated backup and data protection.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `BACKUP TO SERVER` | Backup data to server | - |

**Files:**
- `backup.gbdialog/backup-to-server.bas` - Existing

---

### 14. **bi.gbai** - Business Intelligence
BI dashboards and analytics.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `BI USER` | User BI interface | - |
| `BI ADMIN` | Admin BI interface | - |

**Files:**
- `bi.gbdialog/bi-user.bas` - Existing
- `bi.gbdialog/bi-admin.bas` - Existing

---

### 15. **broadcast.gbai** - Broadcasting
Mass message broadcasting system.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `BROADCAST` | Send broadcast messages | - |

**Files:**
- `broadcast.gbdialog/broadcast.bas` - Existing

---

### 16. **law.gbai** - Legal Case Management
Legal case tracking and management.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `CASE` | Manage legal cases | - |

**Files:**
- `law.gbdialog/case.bas` - Existing

---

### 17. **whatsapp.gbai** - WhatsApp Integration
WhatsApp bot integration.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `CREATE TASK` | Create WhatsApp task | - |
| `SEND` | Send WhatsApp message | - |

**Files:**
- `whatsapp.gbdialog/create-task.bas` - Existing
- `whatsapp.gbdialog/send.bas` - Existing

---

### 18. **template.gbai** - Generic Template
Base template for creating new bots.

| Keyword | Description | Parameters |
|---------|-------------|------------|
| `SEND` | Generic send function | - |

**Files:**
- `template.gbdialog/send.bas` - Existing

---

### 19. **crawler.gbai** - Web Crawler
Web scraping and crawling functionality.

**Status:** No keywords yet (empty template)

---

## 📊 Statistics Summary

| Category | Templates | Keywords | Lines of Code |
|----------|-----------|----------|---------------|
| Core/Default | 1 | 5 | 629 |
| Public APIs | 1 | 76 | 2,714 |
| Marketing | 1 | 4 | 93 |
| E-commerce | 1 | 2 | - |
| Education | 1 | 2 | - |
| AI/LLM | 3 | 4 | - |
| Business | 4 | 6 | - |
| Communication | 4 | 8 | - |
| Other | 3 | 3 | - |
| **TOTAL** | **19** | **110+** | **4,000+** |

---

## 🎯 Quick Reference - Most Used Keywords

### Essential (Every Bot Needs)
```vbs
WEATHER location              ' Get weather forecast
TRANSLATE text, from, to      ' Translate text
SEND EMAIL to, subject, body  ' Send email
CALCULATE expression          ' Do math
```

### Data Generation
```vbs
Generate UUID                 ' Create unique IDs
Random User Generator         ' Test user data
Generate Lorem Ipsum          ' Placeholder text
QR Code Generator            ' Create QR codes
```

### Fun & Entertainment
```vbs
Random Dog Image             ' Cute dog pics
Chuck Norris Joke            ' Random jokes
Random Quote                 ' Inspirational quotes
Random Meal Recipe           ' Cooking recipes
```

### Business
```vbs
Country Information          ' Global data
Currency Converter           ' Money conversion
Public Holidays             ' Holiday calendar
GET IMAGE                   ' Marketing images
```

---

## 🚀 How to Use

### Call a keyword from any template:
```vbs
' Example 1: Get weather
weather_data = WEATHER "London"
TALK "Temperature: " + weather_data.temperature + "°C"

' Example 2: Translate
result = TRANSLATE "Hello World", "en", "es"
TALK result.translated

' Example 3: Send email
email_result = SEND EMAIL "user@example.com", "Important", "Hello!"

' Example 4: Calculate
calc = CALCULATE "15 + 25"
TALK "Answer: " + calc.answer
```

---

## 📚 Documentation Files

- `README.md` - Overview and introduction
- `MASTER_KEYWORDS_INDEX.md` - This file
- `public-apis.gbai/README.md` - Public APIs documentation
- `public-apis.gbai/QUICKSTART.md` - Quick start guide
- `public-apis.gbai/KEYWORDS_CHECKLIST.md` - Implementation checklist

---

## 🔧 Keywords Needed (Future Development)

### High Priority
- [ ] `SEND WHATSAPP` - WhatsApp message sending
- [ ] `GENERATE PDF` - PDF document generation
- [ ] `CONVERT IMAGE` - Image format conversion
- [ ] `SPEECH TO TEXT` - Audio transcription
- [ ] `TEXT TO SPEECH` - Text-to-audio conversion
- [ ] `COMPRESS FILE` - File compression
- [ ] `ENCRYPT DATA` - Data encryption
- [ ] `DECRYPT DATA` - Data decryption
- [ ] `VALIDATE EMAIL` - Email validation
- [ ] `VALIDATE PHONE` - Phone number validation

### Medium Priority
- [ ] `GET NEWS` - News headlines API
- [ ] `GET STOCKS` - Stock market data
- [ ] `GET CRYPTO` - Cryptocurrency prices
- [ ] `SHORTEN URL` - URL shortening
- [ ] `EXPAND URL` - URL expansion
- [ ] `CHECK DOMAIN` - Domain availability
- [ ] `GEOCODE ADDRESS` - Address to coordinates
- [ ] `REVERSE GEOCODE` - Coordinates to address
- [ ] `DISTANCE BETWEEN` - Calculate distance
- [ ] `ROUTE PLANNER` - Calculate route

### Low Priority
- [ ] `GENERATE MEME` - Meme generation
- [ ] `FACE DETECTION` - Detect faces in images
- [ ] `OCR TEXT` - Extract text from images
- [ ] `SENTIMENT ANALYSIS` - Analyze text sentiment
- [ ] `SPELL CHECK` - Check spelling
- [ ] `GRAMMAR CHECK` - Check grammar
- [ ] `SUMMARIZE TEXT` - Text summarization
- [ ] `EXTRACT KEYWORDS` - Keyword extraction
- [ ] `CLASSIFY TEXT` - Text classification
- [ ] `DETECT LANGUAGE` - Language detection

---

## ✅ Quality Standards

All keywords must have:
- ✅ Clear DESCRIPTION
- ✅ Proper PARAM definitions with examples
- ✅ Error handling (IF/ELSE)
- ✅ User-friendly TALK messages
- ✅ Return value documentation
- ✅ Comments explaining logic
- ✅ Example usage in documentation

---

## 🤝 Contributing

To add new keywords:

1. Choose appropriate template directory
2. Create `.vbs` or `.bas` file in `template.gbdialog/` folder
3. Follow existing keyword patterns
4. Add error handling
5. Document parameters and return values
6. Update this index file
7. Test thoroughly

---

## 📞 Support

- Documentation: `/templates/README.md`
- Issues: Report via GitHub
- Community: General Bots Discord

---

**Last Updated:** 2024  
**Maintained by:** General Bots Community  
**License:** Follows General Bots license

**🎉 Ready to build amazing bots with 110+ keywords!**