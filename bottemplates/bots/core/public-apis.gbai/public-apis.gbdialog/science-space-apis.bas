REM General Bots: Science, Space & Books APIs - Free Knowledge Integration
REM Based on public-apis list - No authentication required

REM ============================================
REM SPACE KEYWORD - Random Space Image (NASA APOD)
REM ============================================
DESCRIPTION "Get NASA's Astronomy Picture of the Day (demo key)"

apod_data = GET "https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY"

IF apod_data.title THEN
    TALK "🌌 NASA Astronomy Picture of the Day:"
    TALK "📸 " + apod_data.title
    TALK ""
    TALK "📅 Date: " + apod_data.date
    TALK "📝 Explanation:"
    TALK apod_data.explanation
    TALK ""

    IF apod_data.media_type = "image" THEN
        TALK "🖼️ Image URL: " + apod_data.url
        file = DOWNLOAD apod_data.url
        SEND FILE file
    ELSE IF apod_data.media_type = "video" THEN
        TALK "🎥 Video URL: " + apod_data.url
    END IF

    IF apod_data.copyright THEN
        TALK "©️ Copyright: " + apod_data.copyright
    END IF

    RETURN apod_data
ELSE
    TALK "❌ Could not fetch NASA APOD"
    RETURN NULL
END IF

REM ============================================
REM SPACE KEYWORD - ISS Current Location
REM ============================================
DESCRIPTION "Get current location of International Space Station"

iss_location = GET "http://api.open-notify.org/iss-now.json"

IF iss_location.message = "success" THEN
    lat = iss_location.iss_position.latitude
    lon = iss_location.iss_position.longitude
    timestamp = iss_location.timestamp

    TALK "🛰️ International Space Station Location:"
    TALK "📍 Latitude: " + lat
    TALK "📍 Longitude: " + lon
    TALK "⏰ Timestamp: " + timestamp
    TALK ""
    TALK "🗺️ View on map: https://www.google.com/maps?q=" + lat + "," + lon

    RETURN iss_location
ELSE
    TALK "❌ Could not fetch ISS location"
    RETURN NULL
END IF

REM ============================================
REM SPACE KEYWORD - People in Space Right Now
REM ============================================
DESCRIPTION "Get list of astronauts currently in space"

space_people = GET "http://api.open-notify.org/astros.json"

IF space_people.message = "success" THEN
    TALK "👨‍🚀 People Currently in Space: " + space_people.number
    TALK ""

    FOR EACH person IN space_people.people
        TALK "• " + person.name + " (" + person.craft + ")"
    END FOR

    RETURN space_people
ELSE
    TALK "❌ Could not fetch space crew data"
    RETURN NULL
END IF

REM ============================================
REM SPACE KEYWORD - SpaceX Launch Info
REM ============================================
DESCRIPTION "Get latest SpaceX launch information"

launch_data = GET "https://api.spacexdata.com/v4/launches/latest"

IF launch_data.name THEN
    TALK "🚀 Latest SpaceX Launch:"
    TALK "Mission: " + launch_data.name
    TALK ""
    TALK "📅 Date: " + launch_data.date_utc
    TALK "🎯 Success: " + launch_data.success
    TALK "🔢 Flight Number: " + launch_data.flight_number

    IF launch_data.details THEN
        TALK ""
        TALK "📝 Details:"
        TALK launch_data.details
    END IF

    IF launch_data.links.webcast THEN
        TALK ""
        TALK "🎥 Webcast: " + launch_data.links.webcast
    END IF

    IF launch_data.links.patch.small THEN
        TALK ""
        TALK "🏴 Mission Patch:"
        file = DOWNLOAD launch_data.links.patch.small
        SEND FILE file
    END IF

    RETURN launch_data
ELSE
    TALK "❌ Could not fetch SpaceX launch data"
    RETURN NULL
END IF

REM ============================================
REM SPACE KEYWORD - Next SpaceX Launch
REM ============================================
DESCRIPTION "Get next upcoming SpaceX launch"

next_launch = GET "https://api.spacexdata.com/v4/launches/next"

IF next_launch.name THEN
    TALK "🚀 Next SpaceX Launch:"
    TALK "Mission: " + next_launch.name
    TALK ""
    TALK "📅 Scheduled: " + next_launch.date_utc
    TALK "🔢 Flight Number: " + next_launch.flight_number

    IF next_launch.details THEN
        TALK ""
        TALK "📝 Details:"
        TALK next_launch.details
    END IF

    RETURN next_launch
ELSE
    TALK "❌ Could not fetch next SpaceX launch"
    RETURN NULL
END IF

REM ============================================
REM SCIENCE KEYWORD - Random Math Problem
REM ============================================
PARAM difficulty AS "easy", "medium", "hard"
DESCRIPTION "Get a random math problem to solve"

REM Generate based on difficulty
result = NEW OBJECT

IF difficulty = "easy" THEN
    num1 = INT(RND() * 10) + 1
    num2 = INT(RND() * 10) + 1
    operation = "+"
    answer = num1 + num2

ELSE IF difficulty = "medium" THEN
    num1 = INT(RND() * 50) + 10
    num2 = INT(RND() * 50) + 10
    operations = NEW ARRAY
    operations.PUSH("+")
    operations.PUSH("-")
    operations.PUSH("*")
    operation = operations[INT(RND() * 3)]

    IF operation = "+" THEN
        answer = num1 + num2
    ELSE IF operation = "-" THEN
        answer = num1 - num2
    ELSE
        answer = num1 * num2
    END IF

ELSE IF difficulty = "hard" THEN
    num1 = INT(RND() * 100) + 50
    num2 = INT(RND() * 20) + 5
    operations = NEW ARRAY
    operations.PUSH("+")
    operations.PUSH("-")
    operations.PUSH("*")
    operations.PUSH("/")
    operation = operations[INT(RND() * 4)]

    IF operation = "+" THEN
        answer = num1 + num2
    ELSE IF operation = "-" THEN
        answer = num1 - num2
    ELSE IF operation = "*" THEN
        answer = num1 * num2
    ELSE
        answer = num1 / num2
    END IF
END IF

result.problem = num1 + " " + operation + " " + num2
result.answer = answer
result.difficulty = difficulty

TALK "🧮 Math Problem (" + difficulty + "):"
TALK result.problem + " = ?"
TALK ""
TALK "💡 Think about it..."
WAIT 3
TALK ""
TALK "✅ Answer: " + answer

RETURN result

REM ============================================
REM SCIENCE KEYWORD - Periodic Table Element
REM ============================================
PARAM element AS string LIKE "hydrogen"
DESCRIPTION "Get information about a chemical element"

element_url = "https://neelpatel05.pythonanywhere.com/element/atomicname?atomicname=" + element

element_data = GET element_url

IF element_data.atomicName THEN
    TALK "🧪 Chemical Element: " + element_data.atomicName
    TALK ""
    TALK "⚛️ Symbol: " + element_data.symbol
    TALK "🔢 Atomic Number: " + element_data.atomicNumber
    TALK "⚖️ Atomic Mass: " + element_data.atomicMass
    TALK "📊 Group: " + element_data.groupBlock
    TALK "🌡️ Boiling Point: " + element_data.boilingPoint
    TALK "🌡️ Melting Point: " + element_data.meltingPoint
    TALK "📏 Density: " + element_data.density
    TALK "⚡ Electronegativity: " + element_data.electronegativity
    TALK "📅 Year Discovered: " + element_data.yearDiscovered

    RETURN element_data
ELSE
    TALK "❌ Element not found: " + element
    RETURN NULL
END IF

REM ============================================
REM SCIENCE KEYWORD - Random Science Fact
REM ============================================
DESCRIPTION "Get a random science fact"

science_facts = NEW ARRAY
science_facts.PUSH("The human body contains about 37.2 trillion cells.")
science_facts.PUSH("Light travels at approximately 299,792 kilometers per second.")
science_facts.PUSH("Water covers about 71% of Earth's surface.")
science_facts.PUSH("The human brain contains about 86 billion neurons.")
science_facts.PUSH("DNA stands for Deoxyribonucleic Acid.")
science_facts.PUSH("Sound travels faster through water than air.")
science_facts.PUSH("Octopuses have three hearts and blue blood.")
science_facts.PUSH("The sun is about 109 times wider than Earth.")
science_facts.PUSH("A single bolt of lightning contains 1 billion volts.")
science_facts.PUSH("Humans share about 60% of their DNA with bananas.")

random_index = INT(RND() * UBOUND(science_facts))
fact = science_facts[random_index]

TALK "🔬 Random Science Fact:"
TALK fact

RETURN fact

REM ============================================
REM SCIENCE KEYWORD - Earthquake Data
REM ============================================
DESCRIPTION "Get recent earthquake data worldwide"

quake_url = "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson"

quake_data = GET quake_url

IF quake_data.features THEN
    TALK "🌍 Significant Earthquakes (Last Month):"
    TALK "Total Events: " + quake_data.metadata.count
    TALK ""

    counter = 0
    FOR EACH quake IN quake_data.features
        IF counter < 5 THEN
            props = quake.properties
            magnitude = props.mag
            place = props.place
            time_ms = props.time

            TALK "📍 " + place
            TALK "   Magnitude: " + magnitude
            TALK "   Time: " + time_ms
            TALK ""
        END IF
        counter = counter + 1
    END FOR

    IF counter > 5 THEN
        TALK "... and " + (counter - 5) + " more earthquakes"
    END IF

    RETURN quake_data.features
ELSE
    TALK "❌ Could not fetch earthquake data"
    RETURN NULL
END IF

REM ============================================
REM BOOKS KEYWORD - Random Quote from Book
REM ============================================
DESCRIPTION "Get a random book quote"

quote = GET "https://api.quotable.io/random?tags=literature"

IF quote.content THEN
    TALK "📚 Book Quote:"
    TALK '"' + quote.content + '"'
    TALK ""
    TALK "— " + quote.author

    IF quote.tags THEN
        TALK ""
        TALK "Tags: " + JOIN(quote.tags, ", ")
    END IF

    RETURN quote
ELSE
    TALK "❌ Could not fetch book quote"
    RETURN NULL
END IF

REM ============================================
REM BOOKS KEYWORD - Bible Verse of the Day
REM ============================================
DESCRIPTION "Get Bible verse of the day"

verse = GET "https://beta.ourmanna.com/api/v1/get?format=json"

IF verse.verse.details.text THEN
    TALK "📖 Bible Verse of the Day:"
    TALK verse.verse.details.reference
    TALK ""
    TALK verse.verse.details.text
    TALK ""
    TALK "Version: " + verse.verse.details.version

    RETURN verse
ELSE
    TALK "❌ Could not fetch Bible verse"
    RETURN NULL
END IF

REM ============================================
REM BOOKS KEYWORD - Random Quran Verse
REM ============================================
DESCRIPTION "Get a random verse from the Quran"

REM Random surah (1-114) and ayah
surah = INT(RND() * 114) + 1
ayah = INT(RND() * 20) + 1

quran_url = "https://api.alquran.cloud/v1/ayah/" + surah + ":" + ayah + "/en.asad"

quran_data = GET quran_url

IF quran_data.data.text THEN
    TALK "📖 Quran Verse:"
    TALK "Surah " + quran_data.data.surah.number + ": " + quran_data.data.surah.englishName
    TALK "Ayah " + quran_data.data.numberInSurah
    TALK ""
    TALK quran_data.data.text

    RETURN quran_data.data
ELSE
    TALK "❌ Could not fetch Quran verse"
    RETURN NULL
END IF

REM ============================================
REM BOOKS KEYWORD - Poetry Search
REM ============================================
PARAM search_term AS string LIKE "love"
DESCRIPTION "Search for poems containing a specific word"

poetry_url = "https://poetrydb.org/lines/" + search_term

poems = GET poetry_url

IF poems AND UBOUND(poems) > 0 THEN
    TALK "📜 Found " + UBOUND(poems) + " poems with '" + search_term + "':"
    TALK ""

    counter = 0
    FOR EACH poem IN poems
        IF counter < 3 THEN
            TALK "📖 " + poem.title
            TALK "✍️ By " + poem.author
            TALK ""

            REM Show first few lines
            line_count = 0
            FOR EACH line IN poem.lines
                IF line_count < 4 THEN
                    TALK "   " + line
                END IF
                line_count = line_count + 1
            END FOR
            TALK ""
        END IF
        counter = counter + 1
    END FOR

    IF counter > 3 THEN
        TALK "... and " + (counter - 3) + " more poems"
    END IF

    RETURN poems
ELSE
    TALK "❌ No poems found for: " + search_term
    RETURN NULL
END IF

REM ============================================
REM BOOKS KEYWORD - Random Poem
REM ============================================
DESCRIPTION "Get a random poem"

REM Get random poem
poem_data = GET "https://poetrydb.org/random/1"

IF poem_data AND UBOUND(poem_data) > 0 THEN
    poem = poem_data[0]

    TALK "📜 Random Poem:"
    TALK ""
    TALK "📖 " + poem.title
    TALK "✍️ By " + poem.author
    TALK ""

    FOR EACH line IN poem.lines
        TALK line
    END FOR

    RETURN poem
ELSE
    TALK "❌ Could not fetch random poem"
    RETURN NULL
END IF

REM ============================================
REM BOOKS KEYWORD - Shakespeare Quote
REM ============================================
DESCRIPTION "Get a random Shakespeare quote"

shakespeare = GET "https://api.quotable.io/random?tags=famous-quotes&author=william-shakespeare"

IF shakespeare.content THEN
    TALK "🎭 Shakespeare Quote:"
    TALK '"' + shakespeare.content + '"'
    TALK ""
    TALK "— William Shakespeare"

    RETURN shakespeare
ELSE
    REM Fallback to any Shakespeare source
    TALK "🎭 Shakespeare Quote:"
    TALK '"To be, or not to be, that is the question."'
    TALK ""
    TALK "— William Shakespeare (Hamlet)"

    result = NEW OBJECT
    result.content = "To be, or not to be, that is the question."
    result.author = "William Shakespeare"
    RETURN result
END IF

REM ============================================
REM SCIENCE KEYWORD - Random Wikipedia Summary
REM ============================================
DESCRIPTION "Get a random Wikipedia article summary"

wiki_data = GET "https://en.wikipedia.org/api/rest_v1/page/random/summary"

IF wiki_data.title THEN
    TALK "📚 Random Wikipedia Article:"
    TALK "Title: " + wiki_data.title
    TALK ""
    TALK "📝 Summary:"
    TALK wiki_data.extract
    TALK ""
    TALK "🔗 Read more: " + wiki_data.content_urls.desktop.page

    IF wiki_data.thumbnail THEN
        TALK ""
        TALK "🖼️ Thumbnail:"
        file = DOWNLOAD wiki_data.thumbnail.source
        SEND FILE file
    END IF

    RETURN wiki_data
ELSE
    TALK "❌ Could not fetch Wikipedia article"
    RETURN NULL
END IF

REM ============================================
REM SCIENCE KEYWORD - Today in History
REM ============================================
DESCRIPTION "Get historical events that happened today"

today = NOW()
month = MONTH(today)
day = DAY(today)

history_url = "https://history.muffinlabs.com/date/" + month + "/" + day

history_data = GET history_url

IF history_data.data.Events THEN
    TALK "📅 Today in History (" + month + "/" + day + "):"
    TALK ""

    counter = 0
    FOR EACH event IN history_data.data.Events
        IF counter < 5 THEN
            TALK "📜 " + event.year + ": " + event.text
            TALK ""
        END IF
        counter = counter + 1
    END FOR

    IF counter > 5 THEN
        TALK "... and " + (counter - 5) + " more events"
    END IF

    IF history_data.data.Births THEN
        TALK ""
        TALK "🎂 Notable Births:"
        birth_count = 0
        FOR EACH birth IN history_data.data.Births
            IF birth_count < 3 THEN
                TALK "• " + birth.year + ": " + birth.text
            END IF
            birth_count = birth_count + 1
        END FOR
    END IF

    RETURN history_data.data
ELSE
    TALK "❌ Could not fetch historical data"
    RETURN NULL
END IF

REM ============================================
REM SCIENCE KEYWORD - Age Calculator
REM ============================================
PARAM birth_date AS date LIKE "1990-01-15"
DESCRIPTION "Calculate age and interesting facts"

birth = DATEVALUE(birth_date)
today = NOW()

years = YEAR(today) - YEAR(birth)
days = DATEDIFF(today, birth, "d")
hours = days * 24
minutes = hours * 60

TALK "🎂 Age Calculator:"
TALK "Birth Date: " + birth_date
TALK ""
TALK "📊 You are:"
TALK "• " + years + " years old"
TALK "• " + days + " days old"
TALK "• " + hours + " hours old"
TALK "• " + minutes + " minutes old"
TALK ""

REM Calculate next birthday
next_birthday = DATEVALUE(YEAR(today) + "-" + MONTH(birth) + "-" + DAY(birth))
IF next_birthday < today THEN
    next_birthday = DATEADD(next_birthday, 1, "yyyy")
END IF
days_until = DATEDIFF(next_birthday, today, "d")

TALK "🎉 Next birthday in " + days_until + " days!"

result = NEW OBJECT
result.years = years
result.days = days
result.hours = hours
result.next_birthday = next_birthday

RETURN result
