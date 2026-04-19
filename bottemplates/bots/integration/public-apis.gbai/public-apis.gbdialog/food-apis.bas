REM General Bots: Food & Drink APIs - Free Food Data Integration
REM Based on public-apis list - No authentication required

REM ============================================
REM FOOD KEYWORD - Random Coffee Image
REM ============================================
DESCRIPTION "Get a random coffee image"

coffee_data = GET "https://coffee.alexflipnote.dev/random.json"

image_url = coffee_data.file

TALK "☕ Random Coffee Image:"
TALK image_url

file = DOWNLOAD image_url
SEND FILE file

RETURN image_url

REM ============================================
REM FOOD KEYWORD - Random Food Dish
REM ============================================
DESCRIPTION "Get a random food dish image from Foodish"

food_data = GET "https://foodish-api.herokuapp.com/api/"

image_url = food_data.image

TALK "🍽️ Random Food Dish:"
TALK image_url

file = DOWNLOAD image_url
SEND FILE file

RETURN image_url

REM ============================================
REM FOOD KEYWORD - Random Food by Category
REM ============================================
PARAM category AS "biryani", "burger", "butter-chicken", "dessert", "dosa", "idly", "pasta", "pizza", "rice", "samosa"
DESCRIPTION "Get a random food image from a specific category"

food_url = "https://foodish-api.herokuapp.com/api/images/" + category

food_data = GET food_url

image_url = food_data.image

TALK "🍽️ Random " + category + ":"
TALK image_url

file = DOWNLOAD image_url
SEND FILE file

RETURN image_url

REM ============================================
REM FOOD KEYWORD - Random Meal Recipe
REM ============================================
DESCRIPTION "Get a random meal recipe from TheMealDB"

meal_data = GET "https://www.themealdb.com/api/json/v1/1/random.php"

IF meal_data.meals AND UBOUND(meal_data.meals) > 0 THEN
    meal = meal_data.meals[0]

    TALK "🍳 Random Meal Recipe: " + meal.strMeal
    TALK "🌍 Category: " + meal.strCategory + " | Area: " + meal.strArea
    TALK ""
    TALK "📝 Instructions:"
    TALK meal.strInstructions
    TALK ""
    TALK "📷 Image: " + meal.strMealThumb

    IF meal.strYoutube THEN
        TALK "🎥 Video: " + meal.strYoutube
    END IF

    TALK ""
    TALK "🥘 Ingredients:"

    REM Extract ingredients
    FOR i = 1 TO 20
        ingredient = meal["strIngredient" + i]
        measure = meal["strMeasure" + i]

        IF ingredient <> "" AND ingredient <> NULL THEN
            TALK "• " + measure + " " + ingredient
        END IF
    NEXT i

    file = DOWNLOAD meal.strMealThumb
    SEND FILE file

    RETURN meal
ELSE
    TALK "❌ Could not fetch meal recipe"
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Search Meal by Name
REM ============================================
PARAM meal_name AS string LIKE "chicken"
DESCRIPTION "Search for meals by name"

search_url = "https://www.themealdb.com/api/json/v1/1/search.php?s=" + meal_name

meal_data = GET search_url

IF meal_data.meals THEN
    TALK "🔍 Found meals matching '" + meal_name + "':"
    TALK ""

    counter = 0
    FOR EACH meal IN meal_data.meals
        IF counter < 5 THEN
            TALK "🍽️ " + meal.strMeal
            TALK "   Category: " + meal.strCategory + " | Area: " + meal.strArea
            TALK "   ID: " + meal.idMeal
            TALK ""
        END IF
        counter = counter + 1
    END FOR

    IF counter > 5 THEN
        TALK "... and " + (counter - 5) + " more meals"
    END IF

    RETURN meal_data.meals
ELSE
    TALK "❌ No meals found for: " + meal_name
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Random Cocktail Recipe
REM ============================================
DESCRIPTION "Get a random cocktail recipe from TheCocktailDB"

cocktail_data = GET "https://www.thecocktaildb.com/api/json/v1/1/random.php"

IF cocktail_data.drinks AND UBOUND(cocktail_data.drinks) > 0 THEN
    drink = cocktail_data.drinks[0]

    TALK "🍹 Random Cocktail: " + drink.strDrink
    TALK "🏷️ Category: " + drink.strCategory
    TALK "🥃 Glass: " + drink.strGlass
    TALK ""
    TALK "📝 Instructions:"
    TALK drink.strInstructions
    TALK ""
    TALK "🍸 Ingredients:"

    REM Extract ingredients
    FOR i = 1 TO 15
        ingredient = drink["strIngredient" + i]
        measure = drink["strMeasure" + i]

        IF ingredient <> "" AND ingredient <> NULL THEN
            IF measure <> "" AND measure <> NULL THEN
                TALK "• " + measure + " " + ingredient
            ELSE
                TALK "• " + ingredient
            END IF
        END IF
    NEXT i

    TALK ""
    TALK "📷 Image: " + drink.strDrinkThumb

    file = DOWNLOAD drink.strDrinkThumb
    SEND FILE file

    RETURN drink
ELSE
    TALK "❌ Could not fetch cocktail recipe"
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Search Cocktail by Name
REM ============================================
PARAM cocktail_name AS string LIKE "margarita"
DESCRIPTION "Search for cocktails by name"

search_url = "https://www.thecocktaildb.com/api/json/v1/1/search.php?s=" + cocktail_name

cocktail_data = GET search_url

IF cocktail_data.drinks THEN
    TALK "🔍 Found cocktails matching '" + cocktail_name + "':"
    TALK ""

    FOR EACH drink IN cocktail_data.drinks
        TALK "🍹 " + drink.strDrink
        TALK "   Category: " + drink.strCategory + " | Glass: " + drink.strGlass
        TALK "   Alcoholic: " + drink.strAlcoholic
        TALK ""
    END FOR

    RETURN cocktail_data.drinks
ELSE
    TALK "❌ No cocktails found for: " + cocktail_name
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Search Cocktail by Ingredient
REM ============================================
PARAM ingredient AS string LIKE "vodka"
DESCRIPTION "Search for cocktails by ingredient"

search_url = "https://www.thecocktaildb.com/api/json/v1/1/filter.php?i=" + ingredient

cocktail_data = GET search_url

IF cocktail_data.drinks THEN
    TALK "🔍 Found " + UBOUND(cocktail_data.drinks) + " cocktails with " + ingredient + ":"
    TALK ""

    counter = 0
    FOR EACH drink IN cocktail_data.drinks
        IF counter < 10 THEN
            TALK "🍹 " + drink.strDrink + " (ID: " + drink.idDrink + ")"
        END IF
        counter = counter + 1
    END FOR

    IF counter > 10 THEN
        TALK "... and " + (counter - 10) + " more cocktails"
    END IF

    RETURN cocktail_data.drinks
ELSE
    TALK "❌ No cocktails found with ingredient: " + ingredient
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Fruit Information
REM ============================================
PARAM fruit_name AS string LIKE "apple"
DESCRIPTION "Get nutritional information about a fruit"

fruit_url = "https://fruityvice.com/api/fruit/" + fruit_name

fruit_data = GET fruit_url

IF fruit_data.name THEN
    TALK "🍎 Fruit Information: " + fruit_data.name
    TALK "🏷️ Family: " + fruit_data.family
    TALK "🌳 Genus: " + fruit_data.genus
    TALK "🆔 ID: " + fruit_data.id
    TALK ""
    TALK "📊 Nutritional Information (per 100g):"
    TALK "• Calories: " + fruit_data.nutritions.calories
    TALK "• Carbohydrates: " + fruit_data.nutritions.carbohydrates + "g"
    TALK "• Protein: " + fruit_data.nutritions.protein + "g"
    TALK "• Fat: " + fruit_data.nutritions.fat + "g"
    TALK "• Sugar: " + fruit_data.nutritions.sugar + "g"

    RETURN fruit_data
ELSE
    TALK "❌ Fruit not found: " + fruit_name
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - All Fruits List
REM ============================================
DESCRIPTION "Get a list of all available fruits"

fruits_data = GET "https://fruityvice.com/api/fruit/all"

IF fruits_data THEN
    TALK "🍓 Available Fruits (" + UBOUND(fruits_data) + " total):"
    TALK ""

    counter = 0
    FOR EACH fruit IN fruits_data
        IF counter < 20 THEN
            TALK "• " + fruit.name + " (" + fruit.family + ")"
        END IF
        counter = counter + 1
    END FOR

    IF counter > 20 THEN
        TALK "... and " + (counter - 20) + " more fruits"
    END IF

    RETURN fruits_data
ELSE
    TALK "❌ Could not fetch fruits list"
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Fruits by Family
REM ============================================
PARAM family AS string LIKE "Rosaceae"
DESCRIPTION "Get fruits from a specific family"

family_url = "https://fruityvice.com/api/fruit/family/" + family

fruits_data = GET family_url

IF fruits_data THEN
    TALK "🍎 Fruits from " + family + " family:"
    TALK ""

    FOR EACH fruit IN fruits_data
        TALK "• " + fruit.name + " (Genus: " + fruit.genus + ")"
    END FOR

    RETURN fruits_data
ELSE
    TALK "❌ No fruits found for family: " + family
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Random Taco Recipe
REM ============================================
DESCRIPTION "Get a random taco recipe from TacoFancy"

taco_data = GET "http://taco-randomizer.herokuapp.com/random/"

IF taco_data THEN
    TALK "🌮 Random Taco Recipe:"
    TALK ""

    IF taco_data.base_layer THEN
        TALK "🫓 Base Layer: " + taco_data.base_layer.name
        TALK taco_data.base_layer.recipe
        TALK ""
    END IF

    IF taco_data.mixin THEN
        TALK "🥗 Mixin: " + taco_data.mixin.name
        TALK taco_data.mixin.recipe
        TALK ""
    END IF

    IF taco_data.condiment THEN
        TALK "🧂 Condiment: " + taco_data.condiment.name
        TALK taco_data.condiment.recipe
        TALK ""
    END IF

    IF taco_data.seasoning THEN
        TALK "🌶️ Seasoning: " + taco_data.seasoning.name
        TALK taco_data.seasoning.recipe
        TALK ""
    END IF

    IF taco_data.shell THEN
        TALK "🌮 Shell: " + taco_data.shell.name
        TALK taco_data.shell.recipe
    END IF

    RETURN taco_data
ELSE
    TALK "❌ Could not fetch taco recipe"
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - PunkAPI Beer Info
REM ============================================
DESCRIPTION "Get a random beer recipe from PunkAPI"

beer_data = GET "https://api.punkapi.com/v2/beers/random"

IF beer_data AND UBOUND(beer_data) > 0 THEN
    beer = beer_data[0]

    TALK "🍺 Beer Information: " + beer.name
    TALK "📝 Tagline: " + beer.tagline
    TALK ""
    TALK "📊 Details:"
    TALK "• ABV: " + beer.abv + "%"
    TALK "• IBU: " + beer.ibu
    TALK "• EBC: " + beer.ebc
    TALK "• First Brewed: " + beer.first_brewed
    TALK ""
    TALK "📖 Description:"
    TALK beer.description
    TALK ""

    IF beer.food_pairing THEN
        TALK "🍽️ Food Pairing:"
        FOR EACH pairing IN beer.food_pairing
            TALK "• " + pairing
        END FOR
        TALK ""
    END IF

    IF beer.brewers_tips THEN
        TALK "💡 Brewer's Tips:"
        TALK beer.brewers_tips
    END IF

    IF beer.image_url THEN
        TALK ""
        TALK "📷 Image: " + beer.image_url
        file = DOWNLOAD beer.image_url
        SEND FILE file
    END IF

    RETURN beer
ELSE
    TALK "❌ Could not fetch beer information"
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Search Beer by Name
REM ============================================
PARAM beer_name AS string LIKE "punk"
DESCRIPTION "Search for beers by name"

search_url = "https://api.punkapi.com/v2/beers?beer_name=" + beer_name

beer_data = GET search_url

IF beer_data AND UBOUND(beer_data) > 0 THEN
    TALK "🔍 Found " + UBOUND(beer_data) + " beer(s) matching '" + beer_name + "':"
    TALK ""

    FOR EACH beer IN beer_data
        TALK "🍺 " + beer.name
        TALK "   " + beer.tagline
        TALK "   ABV: " + beer.abv + "% | IBU: " + beer.ibu
        TALK "   First Brewed: " + beer.first_brewed
        TALK ""
    END FOR

    RETURN beer_data
ELSE
    TALK "❌ No beers found for: " + beer_name
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - High ABV Beers
REM ============================================
PARAM min_abv AS number LIKE 8.0
DESCRIPTION "Get beers with ABV higher than specified"

abv_url = "https://api.punkapi.com/v2/beers?abv_gt=" + min_abv

beer_data = GET abv_url

IF beer_data THEN
    TALK "🍺 Beers with ABV > " + min_abv + "%:"
    TALK ""

    counter = 0
    FOR EACH beer IN beer_data
        IF counter < 10 THEN
            TALK "🍺 " + beer.name + " - " + beer.abv + "% ABV"
            TALK "   " + beer.tagline
            TALK ""
        END IF
        counter = counter + 1
    END FOR

    IF counter > 10 THEN
        TALK "... and " + (counter - 10) + " more beers"
    END IF

    RETURN beer_data
ELSE
    TALK "❌ Could not fetch high ABV beers"
    RETURN NULL
END IF

REM ============================================
REM FOOD KEYWORD - Bacon Ipsum Text
REM ============================================
PARAM paragraphs AS integer LIKE 3
DESCRIPTION "Generate bacon-themed lorem ipsum text"

bacon_url = "https://baconipsum.com/api/?type=meat-and-filler&paras=" + paragraphs

bacon_text = GET bacon_url

IF bacon_text THEN
    TALK "🥓 Bacon Ipsum Text:"
    TALK ""

    FOR EACH paragraph IN bacon_text
        TALK paragraph
        TALK ""
    END FOR

    RETURN bacon_text
ELSE
    TALK "❌ Could not generate bacon ipsum"
    RETURN NULL
END IF
