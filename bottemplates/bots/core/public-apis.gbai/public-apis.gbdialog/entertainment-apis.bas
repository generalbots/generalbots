REM General Bots: Entertainment APIs - Jokes, Quotes, and Fun Content
REM Based on public-apis list - No authentication required

REM ============================================
REM ENTERTAINMENT KEYWORD - Chuck Norris Joke
REM ============================================
DESCRIPTION "Get a random Chuck Norris joke"

chuck_joke = GET "https://api.chucknorris.io/jokes/random"

TALK "😄 Chuck Norris Joke:"
TALK chuck_joke.value

RETURN chuck_joke.value

REM ============================================
REM ENTERTAINMENT KEYWORD - Chuck Norris Categories
REM ============================================
DESCRIPTION "Get available Chuck Norris joke categories"

categories = GET "https://api.chucknorris.io/jokes/categories"

TALK "📋 Chuck Norris Joke Categories:"

FOR EACH category IN categories
    TALK "• " + category
END FOR

RETURN categories

REM ============================================
REM ENTERTAINMENT KEYWORD - Chuck Norris Joke by Category
REM ============================================
PARAM category AS string LIKE "dev"
DESCRIPTION "Get a random Chuck Norris joke from a specific category"

joke_url = "https://api.chucknorris.io/jokes/random?category=" + category

chuck_joke = GET joke_url

TALK "😄 Chuck Norris " + category + " Joke:"
TALK chuck_joke.value

RETURN chuck_joke.value

REM ============================================
REM ENTERTAINMENT KEYWORD - Dad Joke
REM ============================================
DESCRIPTION "Get a random dad joke from icanhazdadjoke"

SET HEADER "Accept" = "application/json"

dad_joke = GET "https://icanhazdadjoke.com/"

TALK "👨 Dad Joke:"
TALK dad_joke.joke

RETURN dad_joke.joke

REM ============================================
REM ENTERTAINMENT KEYWORD - Search Dad Jokes
REM ============================================
PARAM search_term AS string LIKE "cat"
DESCRIPTION "Search for dad jokes containing a specific term"

SET HEADER "Accept" = "application/json"

search_url = "https://icanhazdadjoke.com/search?term=" + search_term

results = GET search_url

TALK "🔍 Found " + results.total_jokes + " dad jokes about '" + search_term + "':"

counter = 0
FOR EACH joke IN results.results
    IF counter < 5 THEN
        TALK ""
        TALK "😄 " + joke.joke
    END IF
    counter = counter + 1
END FOR

IF results.total_jokes > 5 THEN
    TALK ""
    TALK "... and " + (results.total_jokes - 5) + " more jokes!"
END IF

RETURN results.results

REM ============================================
REM ENTERTAINMENT KEYWORD - Bored Activity
REM ============================================
DESCRIPTION "Get a random activity suggestion when bored"

activity = GET "https://www.boredapi.com/api/activity"

TALK "💡 Activity Suggestion:"
TALK activity.activity
TALK ""
TALK "📊 Type: " + activity.type
TALK "👥 Participants: " + activity.participants
TALK "💰 Price: " + activity.price

IF activity.link THEN
    TALK "🔗 Link: " + activity.link
END IF

RETURN activity

REM ============================================
REM ENTERTAINMENT KEYWORD - Bored Activity by Type
REM ============================================
PARAM activity_type AS "education", "recreational", "social", "diy", "charity", "cooking", "relaxation", "music", "busywork"
DESCRIPTION "Get a random activity suggestion of a specific type"

activity_url = "https://www.boredapi.com/api/activity?type=" + activity_type

activity = GET activity_url

TALK "💡 " + activity_type + " Activity Suggestion:"
TALK activity.activity
TALK ""
TALK "👥 Participants: " + activity.participants
TALK "💰 Price level: " + activity.price

RETURN activity

REM ============================================
REM ENTERTAINMENT KEYWORD - Random Useless Fact
REM ============================================
DESCRIPTION "Get a random useless but true fact"

fact = GET "https://uselessfacts.jsph.pl/random.json?language=en"

TALK "🤓 Random Useless Fact:"
TALK fact.text

RETURN fact.text

REM ============================================
REM ENTERTAINMENT KEYWORD - Random Fun Fact
REM ============================================
DESCRIPTION "Get a random fun fact"

fun_fact = GET "https://uselessfacts.jsph.pl/api/v2/facts/random"

TALK "🎉 Random Fun Fact:"
TALK fun_fact.text

RETURN fun_fact.text

REM ============================================
REM ENTERTAINMENT KEYWORD - Kanye West Quote
REM ============================================
DESCRIPTION "Get a random Kanye West quote"

kanye = GET "https://api.kanye.rest/"

TALK "🎤 Kanye West says:"
TALK '"' + kanye.quote + '"'

RETURN kanye.quote

REM ============================================
REM ENTERTAINMENT KEYWORD - Advice Slip
REM ============================================
DESCRIPTION "Get a random piece of advice"

advice = GET "https://api.adviceslip.com/advice"

TALK "💭 Random Advice:"
TALK advice.slip.advice

RETURN advice.slip.advice

REM ============================================
REM ENTERTAINMENT KEYWORD - Search Advice
REM ============================================
PARAM query AS string LIKE "love"
DESCRIPTION "Search for advice containing a specific word"

search_url = "https://api.adviceslip.com/advice/search/" + query

results = GET search_url

IF results.total_results > 0 THEN
    TALK "💭 Found " + results.total_results + " advice about '" + query + "':"

    counter = 0
    FOR EACH slip IN results.slips
        IF counter < 5 THEN
            TALK ""
            TALK "• " + slip.advice
        END IF
        counter = counter + 1
    END FOR

    IF results.total_results > 5 THEN
        TALK ""
        TALK "... and " + (results.total_results - 5) + " more pieces of advice!"
    END IF

    RETURN results.slips
ELSE
    TALK "❌ No advice found for: " + query
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Corporate Buzzword
REM ============================================
DESCRIPTION "Get random corporate buzzwords"

buzzword = GET "https://corporatebs-generator.sameerkumar.website/"

TALK "💼 Corporate Buzzword Generator:"
TALK buzzword.phrase

RETURN buzzword.phrase

REM ============================================
REM ENTERTAINMENT KEYWORD - Yo Momma Joke
REM ============================================
DESCRIPTION "Get a random Yo Momma joke"

joke = GET "https://api.yomomma.info/"

TALK "😂 Yo Momma Joke:"
TALK joke.joke

RETURN joke.joke

REM ============================================
REM ENTERTAINMENT KEYWORD - Random Quote
REM ============================================
DESCRIPTION "Get a random inspirational quote"

quote_data = GET "https://api.quotable.io/random"

quote_text = quote_data.content
author = quote_data.author

TALK "✨ Inspirational Quote:"
TALK '"' + quote_text + '"'
TALK "— " + author

RETURN quote_data

REM ============================================
REM ENTERTAINMENT KEYWORD - Quote by Author
REM ============================================
PARAM author AS string LIKE "einstein"
DESCRIPTION "Get a random quote by a specific author"

quote_url = "https://api.quotable.io/random?author=" + author

quote_data = GET quote_url

IF quote_data.content THEN
    TALK "✨ Quote by " + quote_data.author + ":"
    TALK '"' + quote_data.content + '"'

    RETURN quote_data
ELSE
    TALK "❌ No quotes found for author: " + author
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Quote of the Day
REM ============================================
DESCRIPTION "Get the quote of the day"

qotd = GET "https://api.quotable.io/quotes/random?tags=inspirational"

IF UBOUND(qotd) > 0 THEN
    quote = qotd[0]

    TALK "🌟 Quote of the Day:"
    TALK '"' + quote.content + '"'
    TALK "— " + quote.author

    RETURN quote
ELSE
    TALK "❌ Could not fetch quote of the day"
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Programming Quote
REM ============================================
DESCRIPTION "Get a random programming quote"

quote = GET "https://programming-quotes-api.herokuapp.com/quotes/random"

TALK "💻 Programming Quote:"
TALK '"' + quote.en + '"'
TALK "— " + quote.author

RETURN quote

REM ============================================
REM ENTERTAINMENT KEYWORD - Zen Quote
REM ============================================
DESCRIPTION "Get a random Zen/Stoicism quote"

quote = GET "https://zenquotes.io/api/random"

IF UBOUND(quote) > 0 THEN
    zen_quote = quote[0]

    TALK "🧘 Zen Quote:"
    TALK '"' + zen_quote.q + '"'
    TALK "— " + zen_quote.a

    RETURN zen_quote
ELSE
    TALK "❌ Could not fetch Zen quote"
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Affirmation
REM ============================================
DESCRIPTION "Get a random positive affirmation"

affirmation = GET "https://www.affirmations.dev/"

TALK "💖 Daily Affirmation:"
TALK affirmation.affirmation

RETURN affirmation.affirmation

REM ============================================
REM ENTERTAINMENT KEYWORD - Random Trivia
REM ============================================
DESCRIPTION "Get a random trivia question"

trivia = GET "https://opentdb.com/api.php?amount=1"

IF trivia.results AND UBOUND(trivia.results) > 0 THEN
    question = trivia.results[0]

    TALK "🎯 Trivia Question:"
    TALK "Category: " + question.category
    TALK "Difficulty: " + question.difficulty
    TALK ""
    TALK question.question
    TALK ""
    TALK "Correct Answer: " + question.correct_answer

    IF question.incorrect_answers THEN
        TALK ""
        TALK "Other Options:"
        FOR EACH wrong IN question.incorrect_answers
            TALK "• " + wrong
        END FOR
    END IF

    RETURN question
ELSE
    TALK "❌ Could not fetch trivia question"
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Multiple Trivia Questions
REM ============================================
PARAM amount AS integer LIKE 5
DESCRIPTION "Get multiple trivia questions"

trivia_url = "https://opentdb.com/api.php?amount=" + amount

trivia = GET trivia_url

IF trivia.results THEN
    TALK "🎯 " + amount + " Trivia Questions:"
    TALK ""

    counter = 1
    FOR EACH question IN trivia.results
        TALK counter + ". " + question.question
        TALK "   Category: " + question.category + " | Difficulty: " + question.difficulty
        TALK "   Answer: " + question.correct_answer
        TALK ""
        counter = counter + 1
    END FOR

    RETURN trivia.results
ELSE
    TALK "❌ Could not fetch trivia questions"
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Excuse Generator
REM ============================================
DESCRIPTION "Get a random excuse"

excuse = GET "https://excuser-three.vercel.app/v1/excuse"

IF excuse AND UBOUND(excuse) > 0 THEN
    excuse_obj = excuse[0]

    TALK "🤷 Random Excuse:"
    TALK excuse_obj.excuse
    TALK ""
    TALK "Category: " + excuse_obj.category

    RETURN excuse_obj
ELSE
    TALK "❌ Could not generate excuse"
    RETURN NULL
END IF

REM ============================================
REM ENTERTAINMENT KEYWORD - Insult Generator
REM ============================================
DESCRIPTION "Get a random insult (clean)"

insult = GET "https://evilinsult.com/generate_insult.php?lang=en&type=json"

TALK "😈 Random Insult:"
TALK insult.insult

RETURN insult.insult

REM ============================================
REM ENTERTAINMENT KEYWORD - Compliment Generator
REM ============================================
DESCRIPTION "Get a random compliment"

compliment = GET "https://complimentr.com/api"

TALK "💝 Random Compliment:"
TALK compliment.compliment

RETURN compliment.compliment
