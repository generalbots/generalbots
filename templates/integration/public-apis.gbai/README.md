# General Bots Public APIs Integration

This package provides 50+ free API keywords for General Bots, allowing you to integrate various public services without requiring API keys or authentication.

## 📦 Package Contents

This `.gbai` template includes the following BASIC keyword files:

- `weather-apis.bas` - Weather data and forecasts
- `animals-apis.bas` - Animal facts and images
- `entertainment-apis.bas` - Jokes, quotes, and fun content
- `food-apis.bas` - Food recipes and drink information
- `data-utility-apis.bas` - Data utilities and geocoding

## 🌤️ Weather APIs

### 7Timer! Astro Weather
Get 7-day astronomical weather forecast for stargazing.
```vbs
PARAM location AS string LIKE "116.39,39.90"
' Returns: Weather data for astronomy observation
```

### 7Timer! Civil Weather
Get 7-day civil weather forecast with temperature.
```vbs
PARAM location AS string LIKE "116.39,39.90"
' Returns: Temperature, precipitation, wind data
```

### Open-Meteo Weather
Get real-time weather data (70+ years of historical data available).
```vbs
PARAM latitude AS number LIKE 52.52
PARAM longitude AS number LIKE 13.41
' Returns: Current weather conditions
```

### Rain Viewer Radar Map
Get available rain radar map timestamps.
```vbs
DESCRIPTION "Get available rain radar map timestamps"
' Returns: Radar data for visualization
```

### OpenSenseMap Weather Stations
Get data from personal weather stations in a bounding box.
```vbs
PARAM bbox AS string LIKE "7.6,51.2,7.8,51.3"
' Returns: Temperature data from senseBoxes
```

### Air Quality Index
Get Air Quality Index data for major cities.
```vbs
PARAM city AS string LIKE "beijing"
' Returns: AQI level and health recommendations
```

## 🐾 Animals APIs

### Random Cat Fact
```vbs
DESCRIPTION "Get a random cat fact"
' Returns: Interesting cat fact
```

### Random Dog Fact
```vbs
DESCRIPTION "Get a random dog fact"
' Returns: Interesting dog fact
```

### Random Dog Image
```vbs
DESCRIPTION "Get a random dog image URL"
' Returns: URL and downloads image
```

### Random Cat Image
```vbs
DESCRIPTION "Get a random cat image from Cataas"
' Returns: Cat image URL
```

### Random Fox Image
```vbs
DESCRIPTION "Get a random fox image"
' Returns: Fox image URL
```

### Random Duck Image
```vbs
DESCRIPTION "Get a random duck image"
' Returns: Duck image URL
```

### Random Shiba Inu Image
```vbs
DESCRIPTION "Get a random Shiba Inu dog image"
' Returns: Shiba Inu image URL
```

### HTTP Cat (Status Code Cats)
```vbs
PARAM status_code AS integer LIKE 404
' Returns: Cat image representing HTTP status
```

### HTTP Dog (Status Code Dogs)
```vbs
PARAM status_code AS integer LIKE 404
' Returns: Dog image representing HTTP status
```

### PlaceBear Placeholder
```vbs
PARAM width AS integer LIKE 400
PARAM height AS integer LIKE 300
' Returns: Bear placeholder image
```

### PlaceDog Placeholder
```vbs
PARAM width AS integer LIKE 400
PARAM height AS integer LIKE 300
' Returns: Dog placeholder image
```

### PlaceKitten Placeholder
```vbs
PARAM width AS integer LIKE 400
PARAM height AS integer LIKE 300
' Returns: Kitten placeholder image
```

### MeowFacts
```vbs
PARAM count AS integer LIKE 1
' Returns: Random cat facts (up to 100)
```

### Random Axolotl
```vbs
DESCRIPTION "Get random axolotl picture and facts"
' Returns: Axolotl image and facts
```

### Zoo Animals Info
```vbs
DESCRIPTION "Get information about various zoo animals"
' Returns: Animal data with images
```

### Dog Breeds List
```vbs
DESCRIPTION "Get a list of all dog breeds"
' Returns: Array of dog breeds
```

### Specific Dog Breed Image
```vbs
PARAM breed AS string LIKE "husky"
' Returns: Image of specified breed
```

## 😄 Entertainment APIs

### Chuck Norris Joke
```vbs
DESCRIPTION "Get a random Chuck Norris joke"
' Returns: Chuck Norris joke
```

### Chuck Norris Categories
```vbs
DESCRIPTION "Get available Chuck Norris joke categories"
' Returns: Array of categories
```

### Chuck Norris Joke by Category
```vbs
PARAM category AS string LIKE "dev"
' Returns: Joke from specific category
```

### Dad Joke
```vbs
DESCRIPTION "Get a random dad joke"
' Returns: Dad joke from icanhazdadjoke
```

### Search Dad Jokes
```vbs
PARAM search_term AS string LIKE "cat"
' Returns: Dad jokes containing search term
```

### Bored Activity
```vbs
DESCRIPTION "Get a random activity suggestion"
' Returns: Activity suggestion with details
```

### Bored Activity by Type
```vbs
PARAM activity_type AS "education", "recreational", "social", "diy", "charity", "cooking", "relaxation", "music", "busywork"
' Returns: Activity of specific type
```

### Random Useless Fact
```vbs
DESCRIPTION "Get a random useless but true fact"
' Returns: Useless fact
```

### Random Fun Fact
```vbs
DESCRIPTION "Get a random fun fact"
' Returns: Fun fact
```

### Kanye West Quote
```vbs
DESCRIPTION "Get a random Kanye West quote"
' Returns: Kanye quote
```

### Advice Slip
```vbs
DESCRIPTION "Get a random piece of advice"
' Returns: Random advice
```

### Search Advice
```vbs
PARAM query AS string LIKE "love"
' Returns: Advice containing query word
```

### Corporate Buzzword
```vbs
DESCRIPTION "Get random corporate buzzwords"
' Returns: Corporate buzzword phrase
```

### Yo Momma Joke
```vbs
DESCRIPTION "Get a random Yo Momma joke"
' Returns: Yo Momma joke
```

### Random Quote
```vbs
DESCRIPTION "Get a random inspirational quote"
' Returns: Quote with author
```

### Quote by Author
```vbs
PARAM author AS string LIKE "einstein"
' Returns: Quote by specific author
```

### Programming Quote
```vbs
DESCRIPTION "Get a random programming quote"
' Returns: Programming-related quote
```

### Zen Quote
```vbs
DESCRIPTION "Get a random Zen/Stoicism quote"
' Returns: Zen quote
```

### Affirmation
```vbs
DESCRIPTION "Get a random positive affirmation"
' Returns: Daily affirmation
```

### Random Trivia
```vbs
DESCRIPTION "Get a random trivia question"
' Returns: Trivia question with answer
```

### Multiple Trivia Questions
```vbs
PARAM amount AS integer LIKE 5
' Returns: Multiple trivia questions
```

### Excuse Generator
```vbs
DESCRIPTION "Get a random excuse"
' Returns: Random excuse
```

### Insult Generator
```vbs
DESCRIPTION "Get a random insult (clean)"
' Returns: Random insult
```

### Compliment Generator
```vbs
DESCRIPTION "Get a random compliment"
' Returns: Random compliment
```

## 🍽️ Food & Drink APIs

### Random Coffee Image
```vbs
DESCRIPTION "Get a random coffee image"
' Returns: Coffee image URL
```

### Random Food Dish
```vbs
DESCRIPTION "Get a random food dish image"
' Returns: Food dish image
```

### Random Food by Category
```vbs
PARAM category AS "biryani", "burger", "butter-chicken", "dessert", "dosa", "idly", "pasta", "pizza", "rice", "samosa"
' Returns: Food image from category
```

### Random Meal Recipe
```vbs
DESCRIPTION "Get a random meal recipe"
' Returns: Full recipe with ingredients
```

### Search Meal by Name
```vbs
PARAM meal_name AS string LIKE "chicken"
' Returns: Meals matching search
```

### Random Cocktail Recipe
```vbs
DESCRIPTION "Get a random cocktail recipe"
' Returns: Cocktail recipe with ingredients
```

### Search Cocktail by Name
```vbs
PARAM cocktail_name AS string LIKE "margarita"
' Returns: Cocktails matching search
```

### Search Cocktail by Ingredient
```vbs
PARAM ingredient AS string LIKE "vodka"
' Returns: Cocktails with ingredient
```

### Fruit Information
```vbs
PARAM fruit_name AS string LIKE "apple"
' Returns: Nutritional information
```

### All Fruits List
```vbs
DESCRIPTION "Get a list of all fruits"
' Returns: Array of fruits
```

### Fruits by Family
```vbs
PARAM family AS string LIKE "Rosaceae"
' Returns: Fruits from specific family
```

### Random Taco Recipe
```vbs
DESCRIPTION "Get a random taco recipe"
' Returns: Taco recipe components
```

### PunkAPI Beer Info
```vbs
DESCRIPTION "Get a random beer recipe"
' Returns: Beer details and recipe
```

### Search Beer by Name
```vbs
PARAM beer_name AS string LIKE "punk"
' Returns: Beers matching search
```

### High ABV Beers
```vbs
PARAM min_abv AS number LIKE 8.0
' Returns: Beers with high alcohol content
```

### Bacon Ipsum Text
```vbs
PARAM paragraphs AS integer LIKE 3
' Returns: Bacon-themed lorem ipsum
```

## 🔧 Data Utility & Geocoding APIs

### Generate UUID
```vbs
DESCRIPTION "Generate a random UUID v4"
' Returns: UUID string
```

### Generate Multiple UUIDs
```vbs
PARAM count AS integer LIKE 5
' Returns: Array of UUIDs
```

### Get My IP Address
```vbs
DESCRIPTION "Get your current public IP"
' Returns: IP address string
```

### Get IP Geolocation
```vbs
PARAM ip_address AS string LIKE "8.8.8.8"
' Returns: Country, city, coordinates, ISP
```

### Check if Number is Even
```vbs
PARAM number AS integer LIKE 42
' Returns: Boolean (humor API)
```

### Random Data Generator
```vbs
DESCRIPTION "Generate random test data"
' Returns: User profile data
```

### Generate Lorem Ipsum
```vbs
PARAM paragraphs AS integer LIKE 3
' Returns: Lorem ipsum text
```

### QR Code Generator
```vbs
PARAM text AS string LIKE "https://pragmatismo.com.br"
PARAM size AS integer LIKE 200
' Returns: QR code image
```

### Barcode Generator
```vbs
PARAM barcode_data AS string LIKE "1234567890"
PARAM format AS "code128", "ean13", "upca", "code39"
' Returns: Barcode image
```

### Country Information
```vbs
PARAM country AS string LIKE "brazil"
' Returns: Detailed country data
```

### All Countries List
```vbs
DESCRIPTION "Get a list of all countries"
' Returns: Array of 250+ countries
```

### Countries by Region
```vbs
PARAM region AS "africa", "americas", "asia", "europe", "oceania"
' Returns: Countries in region
```

### Currency Converter
```vbs
PARAM amount AS number LIKE 100
PARAM from_currency AS string LIKE "USD"
PARAM to_currency AS string LIKE "EUR"
' Returns: Converted amount
```

### Timezone Info
```vbs
PARAM timezone AS string LIKE "America/New_York"
' Returns: Current time in timezone
```

### All Timezones List
```vbs
DESCRIPTION "Get all timezones"
' Returns: Array of 400+ timezones
```

### Public Holidays
```vbs
PARAM country_code AS string LIKE "US"
PARAM year AS integer LIKE 2024
' Returns: List of public holidays
```

### Number Facts
```vbs
PARAM number AS integer LIKE 42
' Returns: Interesting number fact
```

### Random Number Fact
```vbs
DESCRIPTION "Get a random number fact"
' Returns: Random number fact
```

### Date Facts
```vbs
PARAM month AS integer LIKE 3
PARAM day AS integer LIKE 14
' Returns: Historical facts about date
```

### Math Fact
```vbs
PARAM number AS integer LIKE 1729
' Returns: Mathematical fact
```

### Yes or No Decision
```vbs
DESCRIPTION "Get a random Yes/No answer"
' Returns: Yes or No with GIF
```

### Postcode Lookup UK
```vbs
PARAM postcode AS string LIKE "SW1A1AA"
' Returns: UK postcode information
```

### Brazilian CEP Lookup
```vbs
PARAM cep AS string LIKE "01310-100"
' Returns: Brazilian postal code data
```

### JSON Placeholder Post
```vbs
DESCRIPTION "Get sample post data"
' Returns: Test post data
```

### Random User Generator
```vbs
DESCRIPTION "Generate random user data"
' Returns: Realistic user profile
```

### Multiple Random Users
```vbs
PARAM count AS integer LIKE 5
' Returns: Array of user profiles
```

## 🚀 Usage Examples

### Example 1: Weather Bot
```vbs
TALK "Where would you like to check the weather?"
HEAR city AS NAME

REM Get coordinates (you could use geocoding API)
lat = 52.52
lon = 13.41

REM Get weather data
weather_url = "https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lon + "&current_weather=true"
weather = GET weather_url

TALK "Current temperature in " + city + ": " + weather.current_weather.temperature + "°C"
TALK "Wind speed: " + weather.current_weather.windspeed + " km/h"
```

### Example 2: Daily Motivation Bot
```vbs
REM Get random quote
quote_data = GET "https://api.quotable.io/random"

REM Get affirmation
affirmation = GET "https://www.affirmations.dev/"

TALK "🌟 Daily Motivation:"
TALK ""
TALK "Quote of the Day:"
TALK '"' + quote_data.content + '"'
TALK "— " + quote_data.author
TALK ""
TALK "💖 Affirmation:"
TALK affirmation.affirmation
```

### Example 3: Random Pet Image Bot
```vbs
HEAR choice AS "Cat", "Dog", "Fox", "Duck"

IF choice = "Cat" THEN
    image_url = "https://cataas.com/cat"
ELSE IF choice = "Dog" THEN
    dog_data = GET "https://random.dog/woof.json"
    image_url = dog_data.url
ELSE IF choice = "Fox" THEN
    fox_data = GET "https://randomfox.ca/floof/"
    image_url = fox_data.image
ELSE IF choice = "Duck" THEN
    duck_data = GET "https://random-d.uk/api/random"
    image_url = duck_data.url
END IF

TALK "Here's your random " + choice + " image!"
file = DOWNLOAD image_url
SEND FILE file
```

### Example 4: Recipe Finder Bot
```vbs
TALK "What are you in the mood for?"
HEAR food AS "Meal", "Cocktail", "Beer"

IF food = "Meal" THEN
    meal = GET "https://www.themealdb.com/api/json/v1/1/random.php"
    recipe = meal.meals[0]
    TALK "🍳 " + recipe.strMeal
    TALK recipe.strInstructions
    
ELSE IF food = "Cocktail" THEN
    cocktail = GET "https://www.thecocktaildb.com/api/json/v1/1/random.php"
    drink = cocktail.drinks[0]
    TALK "🍹 " + drink.strDrink
    TALK drink.strInstructions
    
ELSE IF food = "Beer" THEN
    beer_data = GET "https://api.punkapi.com/v2/beers/random"
    beer = beer_data[0]
    TALK "🍺 " + beer.name
    TALK beer.description
END IF
```

### Example 5: Travel Information Bot
```vbs
TALK "Which country would you like to know about?"
HEAR country AS NAME

country_url = "https://restcountries.com/v3.1/name/" + country
country_data = GET country_url

IF country_data AND UBOUND(country_data) > 0 THEN
    info = country_data[0]
    
    TALK "🌍 " + info.name.common
    TALK "Capital: " + info.capital[0]
    TALK "Population: " + info.population
    TALK "Region: " + info.region
    TALK "Languages: " + JOIN(info.languages)
    TALK "Currency: " + JOIN(info.currencies)
    
    REM Get public holidays
    holidays_url = "https://date.nager.at/api/v3/PublicHolidays/2024/" + info.cca2
    holidays = GET holidays_url
    
    TALK ""
    TALK "🎉 Upcoming Holidays:"
    FOR EACH holiday IN holidays
        TALK "• " + holiday.date + " - " + holiday.name
    END FOR
END IF
```

## 📚 API Sources

All APIs in this package are from the [public-apis](https://github.com/public-apis/public-apis) repository and require no authentication.

### Categories Covered:
- ☁️ Weather & Environment
- 🐾 Animals & Pets
- 😄 Entertainment & Humor
- 🍽️ Food & Drink
- 🌍 Geography & Location
- 📊 Data & Utilities
- 💱 Currency & Finance
- 🎲 Random Generators
- 📚 Facts & Trivia

## ⚠️ Important Notes

1. **No API Keys Required**: All keywords use free, no-auth APIs
2. **Rate Limits**: Some APIs may have rate limits on free tier
3. **Availability**: APIs are third-party services and availability may vary
4. **Production Use**: For production apps, consider APIs with authentication for better reliability
5. **Terms of Service**: Always respect the terms of service of each API

## 🔧 Customization

You can easily extend these keywords or create your own:

```vbs
REM Template for new API keyword
PARAM your_param AS string LIKE "example"
DESCRIPTION "What your keyword does"

api_url = "https://api.example.com/endpoint?param=" + your_param
data = GET api_url

IF data THEN
    TALK "Success!"
    TALK data.result
    RETURN data
ELSE
    TALK "❌ Error fetching data"
    RETURN NULL
END IF
```

## 🤝 Contributing

To add more API keywords:
1. Find a free, no-auth API from [public-apis](https://github.com/public-apis/public-apis)
2. Create a `.bas` or `.bas` file in the appropriate category
3. Follow the existing keyword pattern
4. Test thoroughly
5. Update this README

## 📄 License

This template follows the General Bots license. Individual APIs have their own terms of service.

## 🌟 Credits

- [public-apis](https://github.com/public-apis/public-apis) - Comprehensive list of public APIs
- [7Timer!](http://www.7timer.info/) - Weather forecasting
- [Open-Meteo](https://open-meteo.com/) - Weather API
- [TheMealDB](https://www.themealdb.com/) - Meal recipes
- [TheCocktailDB](https://www.thecocktaildb.com/) - Cocktail recipes
- And many more amazing free API providers!

---

**General Bots**: Your Prompt Engineering Gets Done. 🤖
