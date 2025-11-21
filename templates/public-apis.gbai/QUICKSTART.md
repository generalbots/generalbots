# Quick Start Guide - Public APIs for General Bots 🚀

Get started with 70+ free API keywords in under 5 minutes!

## 📦 Installation

1. Copy the entire `public-apis.gbai` folder to your General Bots templates directory:
   ```
   /templates/public-apis.gbai/
   ```

2. Restart your General Bots instance or reload templates

3. Done! All keywords are now available 🎉

## 🎯 Your First API Call

### Example 1: Get a Random Cat Image

```vbs
DESCRIPTION "Show me a random cat picture"

cat_url = "https://cataas.com/cat"
file = DOWNLOAD cat_url
SEND FILE file

RETURN cat_url
```

**Test it:**
- User: "Show me a cat"
- Bot: *sends random cat image*

### Example 2: Weather Check

```vbs
TALK "What's your location? (format: lat,lon)"
HEAR location AS string

coordinates = SPLIT(location, ",")
lat = coordinates[0]
lon = coordinates[1]

weather_url = "https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lon + "&current_weather=true"
weather = GET weather_url

current = weather.current_weather

TALK "🌡️ Temperature: " + current.temperature + "°C"
TALK "💨 Wind Speed: " + current.windspeed + " km/h"
```

### Example 3: Random Joke

```vbs
DESCRIPTION "Tell me a joke"

SET HEADER "Accept" = "application/json"
joke = GET "https://icanhazdadjoke.com/"

TALK "😄 " + joke.joke

RETURN joke.joke
```

## 🔥 Most Popular Keywords

### Animals 🐾
```vbs
REM Random dog image
dog_data = GET "https://random.dog/woof.json"
file = DOWNLOAD dog_data.url
SEND FILE file

REM Cat fact
cat_fact = GET "https://catfact.ninja/fact"
TALK cat_fact.fact

REM Random fox
fox = GET "https://randomfox.ca/floof/"
file = DOWNLOAD fox.image
SEND FILE file
```

### Entertainment 😄
```vbs
REM Chuck Norris joke
joke = GET "https://api.chucknorris.io/jokes/random"
TALK joke.value

REM Random advice
advice = GET "https://api.adviceslip.com/advice"
TALK advice.slip.advice

REM Kanye quote
kanye = GET "https://api.kanye.rest/"
TALK kanye.quote
```

### Food & Drink 🍽️
```vbs
REM Random meal recipe
meal = GET "https://www.themealdb.com/api/json/v1/1/random.php"
recipe = meal.meals[0]
TALK recipe.strMeal
TALK recipe.strInstructions

REM Random cocktail
cocktail = GET "https://www.thecocktaildb.com/api/json/v1/1/random.php"
drink = cocktail.drinks[0]
TALK drink.strDrink
TALK drink.strInstructions
```

### Utilities 🔧
```vbs
REM Generate UUID
uuid = GET "https://www.uuidgenerator.net/api/version4"
TALK "🔑 " + uuid

REM Get my IP
ip = GET "https://api.ipify.org?format=json"
TALK "🌐 Your IP: " + ip.ip

REM Generate QR Code
qr_url = "https://api.qrserver.com/v1/create-qr-code/?size=300x300&data=Hello"
file = DOWNLOAD qr_url
SEND FILE file
```

## 🎨 Building Your First Bot

### Interactive Recipe Bot

```vbs
TALK "Welcome to Recipe Bot! 🍳"
TALK "What are you hungry for?"

HEAR choice AS "Meal", "Cocktail", "Dessert"

IF choice = "Meal" THEN
    meal = GET "https://www.themealdb.com/api/json/v1/1/random.php"
    recipe = meal.meals[0]
    
    TALK "🍽️ How about: " + recipe.strMeal
    TALK ""
    TALK "Category: " + recipe.strCategory
    TALK "Origin: " + recipe.strArea
    TALK ""
    TALK "📝 Instructions:"
    TALK recipe.strInstructions
    
    file = DOWNLOAD recipe.strMealThumb
    SEND FILE file
    
ELSE IF choice = "Cocktail" THEN
    cocktail = GET "https://www.thecocktaildb.com/api/json/v1/1/random.php"
    drink = cocktail.drinks[0]
    
    TALK "🍹 Try this: " + drink.strDrink
    TALK ""
    TALK "Glass: " + drink.strGlass
    TALK ""
    TALK "🍸 Instructions:"
    TALK drink.strInstructions
    
    file = DOWNLOAD drink.strDrinkThumb
    SEND FILE file
END IF

TALK ""
TALK "Enjoy your meal! 😋"
```

### Daily Motivation Bot

```vbs
TALK "🌅 Good morning! Here's your daily motivation:"
TALK ""

REM Get inspirational quote
quote = GET "https://api.quotable.io/random"
TALK "✨ Quote:"
TALK '"' + quote.content + '"'
TALK "— " + quote.author
TALK ""

REM Get affirmation
affirmation = GET "https://www.affirmations.dev/"
TALK "💖 Affirmation:"
TALK affirmation.affirmation
TALK ""

REM Get activity suggestion
activity = GET "https://www.boredapi.com/api/activity"
TALK "💡 Activity Suggestion:"
TALK activity.activity
TALK ""

TALK "Have a great day! 🌟"
```

### Pet Picture Gallery Bot

```vbs
TALK "🐾 Welcome to Pet Picture Gallery!"
TALK "Which animal would you like to see?"

HEAR animal AS "Cat", "Dog", "Fox", "Duck", "Bear"

TALK "Getting a random " + animal + " for you..."

IF animal = "Cat" THEN
    url = "https://cataas.com/cat"
ELSE IF animal = "Dog" THEN
    data = GET "https://random.dog/woof.json"
    url = data.url
ELSE IF animal = "Fox" THEN
    data = GET "https://randomfox.ca/floof/"
    url = data.image
ELSE IF animal = "Duck" THEN
    data = GET "https://random-d.uk/api/random"
    url = data.url
ELSE IF animal = "Bear" THEN
    url = "https://placebear.com/400/300"
END IF

file = DOWNLOAD url
SEND FILE file

TALK "Isn't it adorable? 😍"
TALK ""
TALK "Want another one?"

HEAR again AS BOOLEAN

IF again THEN
    TALK "Coming right up! 🎉"
    REM Repeat the process
END IF
```

## 🧪 Testing Your Keywords

### Method 1: Direct Testing
```vbs
REM Create a test dialog file: test.gbdialog/test-apis.vbs

TALK "Testing Weather API..."
weather = GET "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&current_weather=true"
TALK "✅ Weather API works!"

TALK "Testing Joke API..."
joke = GET "https://api.chucknorris.io/jokes/random"
TALK "✅ Joke API works!"

TALK "All tests passed! 🎉"
```

### Method 2: Interactive Testing
Talk to your bot:
- "Tell me a joke"
- "Show me a cat picture"
- "What's the weather?"
- "Give me a recipe"

## 💡 Pro Tips

### 1. Error Handling
Always check if data exists:
```vbs
data = GET "https://api.example.com/endpoint"

IF data THEN
    TALK "Success!"
    TALK data.result
ELSE
    TALK "❌ Could not fetch data"
END IF
```

### 2. Rate Limiting
Add delays between multiple API calls:
```vbs
TALK "Fetching data..."
data1 = GET "https://api1.example.com"
WAIT 1
data2 = GET "https://api2.example.com"
```

### 3. Image Downloads
Always download before sending:
```vbs
image_url = "https://example.com/image.jpg"
file = DOWNLOAD image_url
SEND FILE file
```

### 4. Header Setting
Some APIs need specific headers:
```vbs
SET HEADER "Accept" = "application/json"
SET HEADER "User-Agent" = "GeneralBots/1.0"

data = GET "https://api.example.com"
```

## 📚 Learning Path

### Beginner (Day 1)
- ✅ Try 5 simple keywords (cat image, joke, quote)
- ✅ Understand GET requests
- ✅ Learn TALK and SEND FILE

### Intermediate (Day 2-3)
- ✅ Build interactive bot with HEAR
- ✅ Combine multiple APIs
- ✅ Add error handling

### Advanced (Day 4-7)
- ✅ Create multi-step conversations
- ✅ Parse complex JSON responses
- ✅ Build production-ready bots

## 🆘 Troubleshooting

### Problem: API returns error
**Solution:** Check if API is online:
```vbs
REM Add debug output
data = GET "https://api.example.com"
TALK "Raw response: " + data
```

### Problem: Image not showing
**Solution:** Verify URL and download:
```vbs
TALK "Image URL: " + image_url
file = DOWNLOAD image_url
IF file THEN
    SEND FILE file
ELSE
    TALK "❌ Could not download image"
END IF
```

### Problem: JSON parsing error
**Solution:** Check if field exists:
```vbs
data = GET "https://api.example.com"
IF data.field THEN
    TALK data.field
ELSE
    TALK "Field not found"
END IF
```

## 🎓 Next Steps

1. **Explore all categories**: Check `README.md` for full keyword list
2. **Combine APIs**: Mix weather + location + activities
3. **Build workflows**: Create multi-step conversations
4. **Share your bots**: Contribute back to community

## 🔗 Useful Resources

- [Full Documentation](README.md) - Complete API reference
- [Keywords Checklist](KEYWORDS_CHECKLIST.md) - Implementation status
- [Public APIs List](https://github.com/public-apis/public-apis) - Find more APIs
- [General Bots Docs](https://github.com/GeneralBots/BotServer) - Platform documentation

## 🤝 Community

- Found a bug? Open an issue
- Have a suggestion? Submit a PR
- Need help? Ask in discussions

## ⚡ Quick Reference Card

```vbs
REM Basic API Call
data = GET "https://api.example.com/endpoint"

REM With Parameters
data = GET "https://api.example.com?param=" + value

REM Download Image
file = DOWNLOAD url
SEND FILE file

REM Error Handling
IF data THEN
    TALK data.result
ELSE
    TALK "Error"
END IF

REM Set Headers
SET HEADER "Accept" = "application/json"

REM User Input
HEAR variable AS TYPE
HEAR name AS NAME
HEAR choice AS "Option1", "Option2"

REM Loops
FOR EACH item IN array
    TALK item
END FOR
```

---

**Ready to build amazing bots?** Start with a simple keyword and grow from there! 🚀

**Need help?** Check the examples in this guide or refer to the full README.md

**Have fun coding!** 🎉