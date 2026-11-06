REM General Bots: Animals & Pets APIs - Free Animal Data Integration
REM Based on public-apis list - No authentication required

REM ============================================
REM ANIMAL KEYWORD - Random Cat Fact
REM ============================================
DESCRIPTION "Get a random cat fact"

cat_fact = GET "https://catfact.ninja/fact"

TALK "🐱 Random Cat Fact:"
TALK cat_fact.fact

RETURN cat_fact.fact

REM ============================================
REM ANIMAL KEYWORD - Random Dog Fact
REM ============================================
DESCRIPTION "Get a random dog fact"

dog_fact = GET "https://dogapi.dog/api/v2/facts"

IF dog_fact.data AND UBOUND(dog_fact.data) > 0 THEN
    fact_text = dog_fact.data[0].attributes.body

    TALK "🐶 Random Dog Fact:"
    TALK fact_text

    RETURN fact_text
ELSE
    TALK "❌ Could not fetch dog fact"
    RETURN NULL
END IF

REM ============================================
REM ANIMAL KEYWORD - Random Dog Image
REM ============================================
DESCRIPTION "Get a random dog image URL"

dog_image = GET "https://random.dog/woof.json"

image_url = dog_image.url

TALK "🐕 Random Dog Image:"
TALK image_url

file = DOWNLOAD image_url
SEND FILE file

RETURN image_url

REM ============================================
REM ANIMAL KEYWORD - Random Cat Image
REM ============================================
DESCRIPTION "Get a random cat image from Cataas"

cat_url = "https://cataas.com/cat"

TALK "🐈 Random Cat Image:"
TALK cat_url

file = DOWNLOAD cat_url
SEND FILE file

RETURN cat_url

REM ============================================
REM ANIMAL KEYWORD - Random Fox Image
REM ============================================
DESCRIPTION "Get a random fox image"

fox_data = GET "https://randomfox.ca/floof/"

image_url = fox_data.image

TALK "🦊 Random Fox Image:"
TALK image_url

file = DOWNLOAD image_url
SEND FILE file

RETURN image_url

REM ============================================
REM ANIMAL KEYWORD - Random Duck Image
REM ============================================
DESCRIPTION "Get a random duck image"

duck_url = "https://random-d.uk/api/random"
duck_data = GET duck_url

image_url = duck_data.url
message = duck_data.message

TALK "🦆 Random Duck Image:"
TALK message
TALK image_url

file = DOWNLOAD image_url
SEND FILE file

RETURN image_url

REM ============================================
REM ANIMAL KEYWORD - Random Shiba Inu Image
REM ============================================
DESCRIPTION "Get a random Shiba Inu dog image"

shiba_data = GET "https://shibe.online/api/shibes?count=1"

IF UBOUND(shiba_data) > 0 THEN
    image_url = shiba_data[0]

    TALK "🐕 Random Shiba Inu Image:"
    TALK image_url

    file = DOWNLOAD image_url
    SEND FILE file

    RETURN image_url
ELSE
    TALK "❌ Could not fetch Shiba image"
    RETURN NULL
END IF

REM ============================================
REM ANIMAL KEYWORD - HTTP Cat (HTTP Status Cats)
REM ============================================
PARAM status_code AS integer LIKE 404
DESCRIPTION "Get a cat image representing an HTTP status code"

cat_url = "https://http.cat/" + status_code

TALK "🐱 HTTP Cat for status " + status_code + ":"
TALK cat_url

file = DOWNLOAD cat_url
SEND FILE file

RETURN cat_url

REM ============================================
REM ANIMAL KEYWORD - HTTP Dog (HTTP Status Dogs)
REM ============================================
PARAM status_code AS integer LIKE 404
DESCRIPTION "Get a dog image representing an HTTP status code"

dog_url = "https://httpstatusdogs.com/img/" + status_code + ".jpg"

TALK "🐶 HTTP Dog for status " + status_code + ":"
TALK dog_url

file = DOWNLOAD dog_url
SEND FILE file

RETURN dog_url

REM ============================================
REM ANIMAL KEYWORD - PlaceBear Placeholder
REM ============================================
PARAM width AS integer LIKE 400
PARAM height AS integer LIKE 300
DESCRIPTION "Get a placeholder bear image of specified dimensions"

bear_url = "https://placebear.com/" + width + "/" + height

TALK "🐻 Bear Placeholder Image (" + width + "x" + height + "):"
TALK bear_url

file = DOWNLOAD bear_url
SEND FILE file

RETURN bear_url

REM ============================================
REM ANIMAL KEYWORD - PlaceDog Placeholder
REM ============================================
PARAM width AS integer LIKE 400
PARAM height AS integer LIKE 300
DESCRIPTION "Get a placeholder dog image of specified dimensions"

dog_url = "https://placedog.net/" + width + "/" + height

TALK "🐕 Dog Placeholder Image (" + width + "x" + height + "):"
TALK dog_url

file = DOWNLOAD dog_url
SEND FILE file

RETURN dog_url

REM ============================================
REM ANIMAL KEYWORD - PlaceKitten Placeholder
REM ============================================
PARAM width AS integer LIKE 400
PARAM height AS integer LIKE 300
DESCRIPTION "Get a placeholder kitten image of specified dimensions"

kitten_url = "https://placekitten.com/" + width + "/" + height

TALK "🐱 Kitten Placeholder Image (" + width + "x" + height + "):"
TALK kitten_url

file = DOWNLOAD kitten_url
SEND FILE file

RETURN kitten_url

REM ============================================
REM ANIMAL KEYWORD - MeowFacts
REM ============================================
PARAM count AS integer LIKE 1
DESCRIPTION "Get random cat facts (up to 100)"

facts_url = "https://meowfacts.herokuapp.com/?count=" + count

meow_data = GET facts_url

TALK "🐱 Random Cat Facts:"

FOR EACH fact IN meow_data.data
    TALK "• " + fact
END FOR

RETURN meow_data.data

REM ============================================
REM ANIMAL KEYWORD - Random Axolotl
REM ============================================
DESCRIPTION "Get random axolotl picture and facts"

axolotl_data = GET "https://theaxolotlapi.netlify.app/.netlify/functions/axolotl"

image_url = axolotl_data.url
facts = axolotl_data.facts

TALK "🦎 Random Axolotl:"
TALK image_url

IF facts THEN
    TALK ""
    TALK "📚 Axolotl Facts:"
    FOR EACH fact IN facts
        TALK "• " + fact
    END FOR
END IF

file = DOWNLOAD image_url
SEND FILE file

RETURN axolotl_data

REM ============================================
REM ANIMAL KEYWORD - Zoo Animals Info
REM ============================================
DESCRIPTION "Get information about various zoo animals"

zoo_data = GET "https://zoo-animal-api.herokuapp.com/animals/rand"

name = zoo_data.name
latin_name = zoo_data.latin_name
animal_type = zoo_data.animal_type
habitat = zoo_data.habitat
lifespan = zoo_data.lifespan
diet = zoo_data.diet
image_url = zoo_data.image_link

TALK "🦁 Random Zoo Animal: " + name
TALK "🔬 Latin Name: " + latin_name
TALK "📦 Type: " + animal_type
TALK "🏡 Habitat: " + habitat
TALK "⏳ Lifespan: " + lifespan
TALK "🍖 Diet: " + diet
TALK "📷 Image: " + image_url

IF image_url THEN
    file = DOWNLOAD image_url
    SEND FILE file
END IF

RETURN zoo_data

REM ============================================
REM ANIMAL KEYWORD - Multiple Random Dogs
REM ============================================
PARAM count AS integer LIKE 3
DESCRIPTION "Get multiple random dog images"

dog_url = "https://dog.ceo/api/breeds/image/random/" + count

dog_data = GET dog_url

IF dog_data.status = "success" THEN
    TALK "🐕 " + count + " Random Dog Images:"

    FOR EACH image IN dog_data.message
        TALK image
        file = DOWNLOAD image
        SEND FILE file
        WAIT 1
    END FOR

    RETURN dog_data.message
ELSE
    TALK "❌ Could not fetch dog images"
    RETURN NULL
END IF

REM ============================================
REM ANIMAL KEYWORD - Dog Breeds List
REM ============================================
DESCRIPTION "Get a list of all dog breeds"

breeds_url = "https://dog.ceo/api/breeds/list/all"

breeds_data = GET breeds_url

IF breeds_data.status = "success" THEN
    breed_count = 0
    breed_list = NEW ARRAY

    TALK "🐕 Available Dog Breeds:"

    FOR EACH breed IN breeds_data.message
        breed_count = breed_count + 1
        breed_list.PUSH(breed)
        IF breed_count <= 20 THEN
            TALK "• " + breed
        END IF
    END FOR

    IF breed_count > 20 THEN
        TALK "... and " + (breed_count - 20) + " more breeds"
    END IF

    RETURN breed_list
ELSE
    TALK "❌ Could not fetch breed list"
    RETURN NULL
END IF

REM ============================================
REM ANIMAL KEYWORD - Specific Dog Breed Image
REM ============================================
PARAM breed AS string LIKE "husky"
DESCRIPTION "Get a random image of a specific dog breed"

breed_url = "https://dog.ceo/api/breed/" + breed + "/images/random"

breed_data = GET breed_url

IF breed_data.status = "success" THEN
    image_url = breed_data.message

    TALK "🐕 Random " + breed + " image:"
    TALK image_url

    file = DOWNLOAD image_url
    SEND FILE file

    RETURN image_url
ELSE
    TALK "❌ Breed not found: " + breed
    TALK "Use 'Dog Breeds List' keyword to see available breeds"
    RETURN NULL
END IF
