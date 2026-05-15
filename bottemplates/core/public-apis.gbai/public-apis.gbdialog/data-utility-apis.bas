REM General Bots: Data Utility & Geocoding APIs - Free Data Services
REM Based on public-apis list - No authentication required

REM ============================================
REM DATA KEYWORD - Generate UUID
REM ============================================
DESCRIPTION "Generate a random UUID v4"

uuid_data = GET "https://www.uuidgenerator.net/api/version4"

TALK "🔑 Generated UUID:"
TALK uuid_data

RETURN uuid_data

REM ============================================
REM DATA KEYWORD - Generate Multiple UUIDs
REM ============================================
PARAM count AS integer LIKE 5
DESCRIPTION "Generate multiple UUIDs"

TALK "🔑 Generated " + count + " UUIDs:"

uuids = NEW ARRAY

FOR i = 1 TO count
    uuid = GET "https://www.uuidgenerator.net/api/version4"
    uuids.PUSH(uuid)
    TALK i + ". " + uuid
NEXT i

RETURN uuids

REM ============================================
REM DATA KEYWORD - Get My IP Address
REM ============================================
DESCRIPTION "Get your current public IP address"

ip_data = GET "https://api.ipify.org?format=json"

TALK "🌐 Your Public IP Address:"
TALK ip_data.ip

RETURN ip_data.ip

REM ============================================
REM DATA KEYWORD - Get IP Geolocation
REM ============================================
PARAM ip_address AS string LIKE "8.8.8.8"
DESCRIPTION "Get geolocation information for an IP address"

geo_url = "http://ip-api.com/json/" + ip_address

geo_data = GET geo_url

IF geo_data.status = "success" THEN
    TALK "🌍 IP Geolocation for " + ip_address + ":"
    TALK "📍 Country: " + geo_data.country + " (" + geo_data.countryCode + ")"
    TALK "🏙️ City: " + geo_data.city
    TALK "📮 ZIP Code: " + geo_data.zip
    TALK "🗺️ Coordinates: " + geo_data.lat + ", " + geo_data.lon
    TALK "⏰ Timezone: " + geo_data.timezone
    TALK "🏢 ISP: " + geo_data.isp
    TALK "🏛️ Organization: " + geo_data.org

    RETURN geo_data
ELSE
    TALK "❌ Could not get geolocation for IP: " + ip_address
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - Check if Number is Even
REM ============================================
PARAM number AS integer LIKE 42
DESCRIPTION "Check if a number is even (humor API)"

even_data = GET "https://api.isevenapi.xyz/api/iseven/" + number

TALK "🔢 Is " + number + " even?"
TALK even_data.iseven

IF even_data.iseven = TRUE THEN
    TALK "✅ Yes, " + number + " is even!"
ELSE
    TALK "❌ No, " + number + " is odd!"
END IF

RETURN even_data.iseven

REM ============================================
REM DATA KEYWORD - Random Data Generator
REM ============================================
DESCRIPTION "Generate random test data (name, address, etc)"

random_data = GET "https://random-data-api.com/api/v2/users"

TALK "👤 Random User Data:"
TALK "Name: " + random_data.first_name + " " + random_data.last_name
TALK "Username: " + random_data.username
TALK "Email: " + random_data.email
TALK "Phone: " + random_data.phone_number
TALK "Date of Birth: " + random_data.date_of_birth
TALK "Address: " + random_data.address.street_address
TALK "City: " + random_data.address.city
TALK "State: " + random_data.address.state
TALK "Country: " + random_data.address.country

RETURN random_data

REM ============================================
REM DATA KEYWORD - Generate Lorem Ipsum
REM ============================================
PARAM paragraphs AS integer LIKE 3
DESCRIPTION "Generate Lorem Ipsum placeholder text"

lorem_url = "https://loripsum.net/api/" + paragraphs + "/medium/plaintext"

lorem_text = GET lorem_url

TALK "📝 Lorem Ipsum Text (" + paragraphs + " paragraphs):"
TALK lorem_text

RETURN lorem_text

REM ============================================
REM DATA KEYWORD - QR Code Generator
REM ============================================
PARAM text AS string LIKE "https://pragmatismo.com.br"
PARAM size AS integer LIKE 200
DESCRIPTION "Generate a QR code for any text or URL"

qr_url = "https://api.qrserver.com/v1/create-qr-code/?size=" + size + "x" + size + "&data=" + text

TALK "📱 QR Code generated for:"
TALK text
TALK ""
TALK "🔗 QR Code URL:"
TALK qr_url

file = DOWNLOAD qr_url
SEND FILE file

RETURN qr_url

REM ============================================
REM DATA KEYWORD - Barcode Generator
REM ============================================
PARAM barcode_data AS string LIKE "1234567890"
PARAM format AS "code128", "ean13", "upca", "code39"
DESCRIPTION "Generate a barcode image"

barcode_url = "https://barcodeapi.org/api/" + format + "/" + barcode_data

TALK "📊 Barcode generated:"
TALK "Format: " + format
TALK "Data: " + barcode_data
TALK ""
TALK "🔗 Barcode URL:"
TALK barcode_url

file = DOWNLOAD barcode_url
SEND FILE file

RETURN barcode_url

REM ============================================
REM DATA KEYWORD - Country Information
REM ============================================
PARAM country AS string LIKE "brazil"
DESCRIPTION "Get detailed information about a country"

country_url = "https://restcountries.com/v3.1/name/" + country

country_data = GET country_url

IF country_data AND UBOUND(country_data) > 0 THEN
    info = country_data[0]

    TALK "🌍 Country Information: " + info.name.common
    TALK "🏛️ Official Name: " + info.name.official
    TALK "🏳️ Capital: " + info.capital[0]
    TALK "🗺️ Region: " + info.region + " (" + info.subregion + ")"
    TALK "👥 Population: " + info.population
    TALK "📏 Area: " + info.area + " km²"
    TALK "🌐 Languages: " + JOIN(info.languages)
    TALK "💰 Currencies: " + JOIN(info.currencies)
    TALK "⏰ Timezones: " + JOIN(info.timezones)
    TALK "🚗 Drives on: " + info.car.side
    TALK "🌐 Top Level Domain: " + info.tld[0]

    IF info.flags.png THEN
        TALK ""
        TALK "🏴 Flag:"
        file = DOWNLOAD info.flags.png
        SEND FILE file
    END IF

    RETURN info
ELSE
    TALK "❌ Country not found: " + country
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - All Countries List
REM ============================================
DESCRIPTION "Get a list of all countries"

countries = GET "https://restcountries.com/v3.1/all"

TALK "🌍 Total Countries: " + UBOUND(countries)
TALK ""
TALK "First 20 countries:"

counter = 0
FOR EACH country IN countries
    IF counter < 20 THEN
        TALK "• " + country.name.common + " (" + country.cca2 + ")"
    END IF
    counter = counter + 1
END FOR

IF counter > 20 THEN
    TALK "... and " + (counter - 20) + " more countries"
END IF

RETURN countries

REM ============================================
REM DATA KEYWORD - Countries by Region
REM ============================================
PARAM region AS "africa", "americas", "asia", "europe", "oceania"
DESCRIPTION "Get countries from a specific region"

region_url = "https://restcountries.com/v3.1/region/" + region

countries = GET region_url

TALK "🌍 Countries in " + region + ":"
TALK "Total: " + UBOUND(countries)
TALK ""

FOR EACH country IN countries
    TALK "• " + country.name.common + " - Capital: " + country.capital[0]
END FOR

RETURN countries

REM ============================================
REM DATA KEYWORD - Currency Converter
REM ============================================
PARAM amount AS number LIKE 100
PARAM from_currency AS string LIKE "USD"
PARAM to_currency AS string LIKE "EUR"
DESCRIPTION "Convert currency amounts (Note: Free tier available)"

exchange_url = "https://api.exchangerate-api.com/v4/latest/" + from_currency

exchange_data = GET exchange_url

IF exchange_data.rates THEN
    rate = exchange_data.rates[to_currency]
    converted = amount * rate

    TALK "💱 Currency Conversion:"
    TALK amount + " " + from_currency + " = " + converted + " " + to_currency
    TALK "Exchange Rate: 1 " + from_currency + " = " + rate + " " + to_currency
    TALK "Updated: " + exchange_data.date

    result = NEW OBJECT
    result.amount = amount
    result.from = from_currency
    result.to = to_currency
    result.rate = rate
    result.converted = converted

    RETURN result
ELSE
    TALK "❌ Could not fetch exchange rates"
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - Timezone Info
REM ============================================
PARAM timezone AS string LIKE "America/New_York"
DESCRIPTION "Get current time in a specific timezone"

time_url = "http://worldtimeapi.org/api/timezone/" + timezone

time_data = GET time_url

IF time_data.datetime THEN
    TALK "⏰ Current Time in " + timezone + ":"
    TALK "🕐 DateTime: " + time_data.datetime
    TALK "📅 Date: " + time_data.date
    TALK "⏲️ Time: " + time_data.time
    TALK "🌍 UTC Offset: " + time_data.utc_offset
    TALK "📆 Day of Week: " + time_data.day_of_week
    TALK "📆 Day of Year: " + time_data.day_of_year
    TALK "📆 Week Number: " + time_data.week_number

    RETURN time_data
ELSE
    TALK "❌ Could not fetch timezone data for: " + timezone
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - All Timezones List
REM ============================================
DESCRIPTION "Get a list of all available timezones"

timezones = GET "http://worldtimeapi.org/api/timezone"

TALK "🌍 Available Timezones (" + UBOUND(timezones) + " total):"
TALK ""

counter = 0
FOR EACH tz IN timezones
    IF counter < 30 THEN
        TALK "• " + tz
    END IF
    counter = counter + 1
END FOR

IF counter > 30 THEN
    TALK "... and " + (counter - 30) + " more timezones"
END IF

RETURN timezones

REM ============================================
REM DATA KEYWORD - Public Holidays
REM ============================================
PARAM country_code AS string LIKE "US"
PARAM year AS integer LIKE 2024
DESCRIPTION "Get public holidays for a country and year"

holidays_url = "https://date.nager.at/api/v3/PublicHolidays/" + year + "/" + country_code

holidays = GET holidays_url

IF holidays THEN
    TALK "🎉 Public Holidays in " + country_code + " for " + year + ":"
    TALK "Total: " + UBOUND(holidays)
    TALK ""

    FOR EACH holiday IN holidays
        TALK "📅 " + holiday.date + " - " + holiday.name
        IF holiday.localName <> holiday.name THEN
            TALK "   (" + holiday.localName + ")"
        END IF
    END FOR

    RETURN holidays
ELSE
    TALK "❌ Could not fetch holidays for: " + country_code
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - Number Facts
REM ============================================
PARAM number AS integer LIKE 42
DESCRIPTION "Get an interesting fact about a number"

fact_url = "http://numbersapi.com/" + number

number_fact = GET fact_url

TALK "🔢 Fact about " + number + ":"
TALK number_fact

RETURN number_fact

REM ============================================
REM DATA KEYWORD - Random Number Fact
REM ============================================
DESCRIPTION "Get a random number fact"

random_fact = GET "http://numbersapi.com/random"

TALK "🔢 Random Number Fact:"
TALK random_fact

RETURN random_fact

REM ============================================
REM DATA KEYWORD - Date Facts
REM ============================================
PARAM month AS integer LIKE 3
PARAM day AS integer LIKE 14
DESCRIPTION "Get interesting facts about a specific date"

date_url = "http://numbersapi.com/" + month + "/" + day + "/date"

date_fact = GET date_url

TALK "📅 Fact about " + month + "/" + day + ":"
TALK date_fact

RETURN date_fact

REM ============================================
REM DATA KEYWORD - Math Fact
REM ============================================
PARAM number AS integer LIKE 1729
DESCRIPTION "Get a mathematical fact about a number"

math_url = "http://numbersapi.com/" + number + "/math"

math_fact = GET math_url

TALK "🧮 Math Fact about " + number + ":"
TALK math_fact

RETURN math_fact

REM ============================================
REM DATA KEYWORD - Yes or No Decision
REM ============================================
DESCRIPTION "Get a random Yes or No answer"

decision = GET "https://yesno.wtf/api"

answer = decision.answer
image = decision.image

TALK "🎲 Random Decision:"
TALK UCASE(answer) + "!"

file = DOWNLOAD image
SEND FILE file

RETURN decision

REM ============================================
REM DATA KEYWORD - Postcode Lookup UK
REM ============================================
PARAM postcode AS string LIKE "SW1A1AA"
DESCRIPTION "Look up UK postcode information"

postcode_clean = REPLACE(postcode, " ", "")
postcode_url = "https://api.postcodes.io/postcodes/" + postcode_clean

postcode_data = GET postcode_url

IF postcode_data.status = 200 THEN
    result = postcode_data.result

    TALK "📮 UK Postcode Information:"
    TALK "Postcode: " + result.postcode
    TALK "🏙️ Region: " + result.region
    TALK "🗺️ District: " + result.admin_district
    TALK "📍 Coordinates: " + result.latitude + ", " + result.longitude
    TALK "🏛️ Parliamentary Constituency: " + result.parliamentary_constituency
    TALK "🌍 Country: " + result.country

    RETURN result
ELSE
    TALK "❌ Invalid postcode: " + postcode
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - Brazilian CEP Lookup
REM ============================================
PARAM cep AS string LIKE "01310-100"
DESCRIPTION "Look up Brazilian postal code (CEP) information"

cep_clean = REPLACE(cep, "-", "")
cep_url = "https://viacep.com.br/ws/" + cep_clean + "/json/"

cep_data = GET cep_url

IF NOT cep_data.erro THEN
    TALK "📮 CEP Information:"
    TALK "CEP: " + cep_data.cep
    TALK "🏙️ City: " + cep_data.localidade + " - " + cep_data.uf
    TALK "🏘️ Neighborhood: " + cep_data.bairro
    TALK "🛣️ Street: " + cep_data.logradouro
    TALK "🗺️ Region: " + cep_data.regiao
    TALK "☎️ DDD: " + cep_data.ddd

    RETURN cep_data
ELSE
    TALK "❌ Invalid CEP: " + cep
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - JSON Placeholder Post
REM ============================================
DESCRIPTION "Get sample post data for testing"

post = GET "https://jsonplaceholder.typicode.com/posts/1"

TALK "📝 Sample Post Data:"
TALK "Title: " + post.title
TALK "User ID: " + post.userId
TALK "Post ID: " + post.id
TALK ""
TALK "Body:"
TALK post.body

RETURN post

REM ============================================
REM DATA KEYWORD - Random User Generator
REM ============================================
DESCRIPTION "Generate a random user with realistic data"

user_data = GET "https://randomuser.me/api/"

IF user_data.results AND UBOUND(user_data.results) > 0 THEN
    user = user_data.results[0]

    TALK "👤 Random User Generated:"
    TALK "Name: " + user.name.first + " " + user.name.last
    TALK "Gender: " + user.gender
    TALK "📧 Email: " + user.email
    TALK "📱 Phone: " + user.phone
    TALK "🎂 Date of Birth: " + user.dob.date
    TALK "📍 Location: " + user.location.city + ", " + user.location.state + ", " + user.location.country
    TALK "📮 Postcode: " + user.location.postcode
    TALK "🌐 Nationality: " + user.nat
    TALK ""
    TALK "📷 Picture: " + user.picture.large

    file = DOWNLOAD user.picture.large
    SEND FILE file

    RETURN user
ELSE
    TALK "❌ Could not generate random user"
    RETURN NULL
END IF

REM ============================================
REM DATA KEYWORD - Multiple Random Users
REM ============================================
PARAM count AS integer LIKE 5
DESCRIPTION "Generate multiple random users"

users_url = "https://randomuser.me/api/?results=" + count

users_data = GET users_url

IF users_data.results THEN
    TALK "👥 Generated " + count + " Random Users:"
    TALK ""

    counter = 1
    FOR EACH user IN users_data.results
        TALK counter + ". " + user.name.first + " " + user.name.last
        TALK "   Email: " + user.email
        TALK "   Location: " + user.location.city + ", " + user.location.country
        TALK ""
        counter = counter + 1
    END FOR

    RETURN users_data.results
ELSE
    TALK "❌ Could not generate random users"
    RETURN NULL
END IF
