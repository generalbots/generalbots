# Chapter 06: BASIC + LLM - The Perfect Match

## Why BASIC? Because Everyone Can Code

In 1964, John Kemeny and Thomas Kurtz created BASIC (Beginner's All-purpose Symbolic Instruction Code) at Dartmouth College with a revolutionary idea: programming should be for everyone, not just computer scientists. They wanted students from all disciplines - humanities, arts, sciences - to experience the power of computing. Today, General Bots brings this philosophy to the AI era. We chose BASIC not despite its simplicity, but because of it.

**The truth about modern programming:** Most frameworks are overengineered. Most diagrams are unnecessary. Most technical complexity serves no real purpose except to exclude people.

With BASIC + LLM, you write:
```basic
TALK "What's your name?"
HEAR name
poem = LLM "Create a beautiful, heartfelt poem using the name " + name + " that celebrates this person's uniqueness"
TALK "Hello, " + name + "! I wrote something special for you:"
TALK poem
```

Not this:
```javascript
const bot = new BotFramework.ActivityHandler();
bot.onMessage(async (context, next) => {
  await context.sendActivity(MessageFactory.text("What's your name?"));
  // 50 more lines of boilerplate...
});
```

## No Spaghetti. No Diagrams. Just Conversation.

Traditional programming creates spaghetti code with complex flows, state machines, and architectural diagrams. But human conversation doesn't need diagrams. Neither should bot programming.

**BASIC + LLM means:**
- Write code like you speak
- No abstract concepts to master
- No frameworks to learn
- No dependencies to manage
- Just a FEW keywords that do EVERYTHING

## The Magic: LLM Fills the Gaps

Here's the revolutionary insight: LLMs understand context. You don't need to program every detail. Write the skeleton, let AI handle the flesh.

```basic
TALK "Tell me about your dream"
HEAR dream
insight = LLM "Provide a thoughtful, encouraging response about this dream: " + dream
TALK insight
```

Traditional programming would require date parsers, validation logic, error handling. With BASIC + LLM, the intelligence is built-in.

## Everyone Is Invited to Program

**You don't need:**
- A computer science degree
- Years of experience
- Understanding of algorithms
- Knowledge of design patterns

**You just need:**
- An idea
- 10 minutes to learn BASIC
- Creativity

### Real People Writing Real Code

- **Teachers** creating educational assistants
- **Doctors** building diagnostic helpers
- **Lawyers** automating document review
- **Artists** making interactive experiences
- **Students** learning by doing
- **Retirees** solving real problems

## The Core Keywords - That's All

Just SEVEN main keywords power everything:

### 1. TALK - Output to User
```basic
TALK "Hello, world!"
TALK "The answer is: " + answer
```

### 2. HEAR - Input from User
```basic
HEAR name
HEAR age AS INTEGER
HEAR confirm AS BOOLEAN
```

### 3. USE KB - Knowledge Base
```basic
USE KB "company-docs"
' Now the bot knows everything in those documents
```

### 4. USE TOOL - Enable Functions
```basic
USE TOOL "weather"
USE TOOL "calculator"
' LLM decides when to use them
```

### 5. GET - Access Data
```basic
user_data = GET "api/user/profile"
weather = GET "weather/london"
```

### 6. IF/THEN/ELSE - Logic
```basic
IF age >= 18 THEN
  TALK "Welcome!"
ELSE
  TALK "Sorry, adults only"
END IF
```

### 7. FOR/NEXT - Loops
```basic
FOR i = 1 TO 10
  TALK "Number: " + i
NEXT
```

That's it. Seven keywords. Infinite possibilities.



## Breaking the Barriers

### From Consumer to Creator

Most people consume technology but never create it. BASIC changes this:

**Monday:** Never programmed before
**Tuesday:** Writing first TALK/HEAR script  
**Wednesday:** Adding knowledge bases
**Thursday:** Integrating tools
**Friday:** Deploying production bot

### Real Examples from Real People

**Maria, 62, Retired Teacher:**
```basic
' My first bot helps students learn with encouragement
TALK "What's your name, dear student?"
HEAR name
TALK "Let's practice multiplication, " + name + "!"
x = RANDOM(1, 10)
y = RANDOM(1, 10)
TALK "What is " + x + " times " + y + "?"
HEAR answer AS NUMBER
correct = x * y
IF answer = correct THEN
  praise = LLM "Create an encouraging message for a student named " + name + " who just got a math problem correct"
  TALK praise
ELSE
  comfort = LLM "Gently encourage " + name + " after a wrong answer, explaining that " + x + " times " + y + "Correct! Well done!"
ELSE
  TALK "Not quite. The answer is " + correct
END IF
```

**João, 45, Small Business Owner:**
```basic
' Customer service bot for my restaurant
USE KB "menu"
USE TOOL "reservations"
TALK "Welcome to João's Kitchen!"
TALK "I can help with our menu or reservations."
```

## The Democratization Movement

### It's Not About Being Easy - It's About Being Possible

Complex languages aren't "better" - they're exclusionary. When programming is hard, only few can participate. When it's simple, everyone can contribute.

### Your Voice Matters

Every person who learns BASIC brings unique perspective:
- Different problems to solve
- Different ways of thinking
- Different communities to serve

### Join the Revolution

1. **Start Today** - Download General Bots, write your first script
2. **Share Your Creation** - Every bot inspires others
3. **Teach Someone** - Pass the knowledge forward
4. **Build Something Real** - Solve actual problems

## Advanced Power, Simple Syntax

Don't let simplicity fool you. BASIC can:

### Web Automation
```basic
URL "https://example.com"
CLICK "Login"
TYPE "email" "user@example.com"
```

### API Integration
```basic
USE TOOL "payment-api"
TALK "Processing your payment..."
```

### Enterprise Scale
```basic
customers = GET "database/customers"
FOR EACH customer IN customers
  SEND EMAIL TO customer
NEXT
```

## No Technical Debt

Traditional programming accumulates technical debt:
- Dependencies need updating
- Frameworks become obsolete  
- Code becomes unmaintainable

BASIC scripts remain readable forever. A script from today will make sense in 10 years.

## The Future Is Conversational

Programming is evolving from writing instructions to having conversations. BASIC + LLM is the bridge:

```basic
' The future of programming
TALK "Build me a customer dashboard"
HEAR requirements
solution = ANSWER requirements WITH TOOLS
TALK "Done! " + solution
```

## Start Your Journey Now

### Minute 1: Hello World
```basic
TALK "Hello, beautiful world!"
TALK "I'm here to listen and help."
```

### Minute 5: Interactive & Emotional
```basic
TALK "What's your name?"
HEAR name
poem = LLM "Write a touching 2-line poem that includes the name " + name + " and makes them feel special and valued"
TALK poem
TALK "It's truly wonderful to meet you, " + name
```

### Day 1: Production Ready
```basic
USE KB "support"
USE TOOL "ticket-system"
USE TOOL "email"
TALK "I'm your support assistant. How can I help?"
```

## Your First LLM Tool - Complete Example

In the LLM world, you don't write complex menu systems. You write tools that the AI can use intelligently. Here's a real enrollment tool:

```basic
' enrollment.bas - An LLM-callable tool
' The LLM collects the information naturally through conversation

PARAM name AS string        LIKE "John Smith"           DESCRIPTION "Full name of the person"
PARAM email AS string       LIKE "john@example.com"     DESCRIPTION "Email address"
PARAM course AS string      LIKE "Introduction to AI"    DESCRIPTION "Course to enroll in"

DESCRIPTION "Enrolls a student in a course. The LLM will collect all required information through natural conversation before calling this tool."

' The actual tool logic is simple
SAVE "enrollments.csv", name, email, course, NOW()
TALK "Successfully enrolled " + name + " in " + course

' That's it! The LLM handles:
' - Collecting information naturally
' - Validating inputs
' - Confirming with the user
' - Error handling
' - All the conversation flow
```

This is the power of BASIC + LLM: You define WHAT (the tool), the LLM handles HOW (the conversation).

## Why We Believe in You

Every person who learns BASIC proves that programming isn't just for the "technical" people. It's for everyone with ideas, problems to solve, and creativity to share.

**You don't need permission to be a programmer.**
**You already are one.**
**You just need to start.**

## Join the Community

The BASIC revolution isn't just about code - it's about people:

- **No question is too simple**
- **Every contribution matters**
- **Beginners teach us most**
- **Your perspective is unique**

## Learn More - Real Stories, Real Code

Visit our blog for inspiration and practical examples:

- **[BASIC for Everyone: Making AI Accessible](https://pragmatismo.com.br/blog/basic-for-everyone)** - The philosophy behind our choice of BASIC
- **[BASIC LLM Tools](https://pragmatismo.com.br/blog/basic-llm-tools)** - How to create tools that LLMs can use
- **[MCP is the new API](https://pragmatismo.com.br/blog/mcp-is-the-new-api)** - Understanding modern tool integration
- **[No Forms, Just Conversation](https://pragmatismo.com.br/blog/no-forms)** - Why conversational UI is the future
- **[Beyond Chatbots](https://pragmatismo.com.br/blog/beyond-chatbots)** - Building real business solutions

Read stories from people just like you who discovered they could code.

## Final Thought

BASIC has always been about democratization. From mainframes to personal computers, from computers to smartphones, and now from AI to everyone. General Bots continues this 60-year tradition, bringing BASIC to the age of artificial intelligence.

The question isn't whether you can learn to program.
The question is: what will you create?

**Start writing. The world is waiting for your bot.**


*"The beauty of BASIC lies not in what it can do, but in who it enables to do it."*

## Next Step

Continue to [BASIC Keywords Reference](./keywords.md) when you're ready for the complete reference.

### Additional Documentation

- [Script Execution Flow](./script-execution-flow.md) - Entry points, config injection, and lifecycle
- [Prompt Blocks](./prompt-blocks.md) - BEGIN SYSTEM PROMPT & BEGIN TALK documentation
- [Keywords Reference](./keywords.md) - Complete keyword list
- [Basics](./basics.md) - Core concepts for LLM-first development

---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>
