# First Conversation

After botserver starts, you can immediately start chatting with your bot. No programming required!

## Just Start Talking

Open your browser to `http://localhost:9000` and start chatting:

```
You: Hi!
Bot: Hello! How can I help you today?

You: I want to enroll in a course
Bot: I'll help you with enrollment. Let me collect your information...
     [Bot automatically calls enrollment.bas tool]

You: What documents do you have?
Bot: [Searches .gbkb/ folders and answers from your documents]
```

That's it! The LLM handles everything automatically.

## How It Works

### Drop Documents in `.gbkb/`

```
mybot.gbai/
  mybot.gbkb/
    policies/
      enrollment-policy.pdf
      course-catalog.pdf
    faqs/
      student-faq.docx
      payment-guide.txt
```

The bot automatically indexes all documents, creates vector embeddings, searches when users ask questions, and provides accurate answers from your content.

### Create Tools as `.bas` Files

Create `enrollment.bas`:

```basic
PARAM name AS string          LIKE "John Smith"           DESCRIPTION "Student full name"
PARAM email AS string         LIKE "john@example.com"     DESCRIPTION "Contact email"
PARAM course AS string        LIKE "Computer Science"     DESCRIPTION "Course to enroll in"

DESCRIPTION "Enrollment tool - collects student information and processes enrollment"

SAVE "enrollments.csv", name, email, course, NOW()
TALK "Enrollment complete! Welcome to " + course + ", " + name
```

The LLM automatically discovers this tool, knows when to call it based on user intent, collects required parameters through natural conversation, and executes the tool when all information is gathered.

### The LLM Does Everything Else

You don't need to write IF/THEN logic, intent detection, dialog flows, state machines, or complex routing. The LLM automatically understands user intent, calls appropriate tools, searches knowledge bases, maintains conversation context, and handles follow-up questions naturally.

## Real Example Conversation

With `enrollment.bas` tool and course documents in `.gbkb/`:

```
User: Hello, I'd like to know about your computer science program

Bot: I'd be happy to help you learn about our Computer Science program! 
     Based on our course catalog, the program offers a comprehensive 
     4-year curriculum covering software development, algorithms, 
     databases, and AI. Would you like to know about specific courses 
     or the enrollment process?

User: Yes, I want to enroll

Bot: Great! I'll help you enroll in the Computer Science program. 
     Let me collect your information. What's your full name?

User: John Smith

Bot: Thank you, John. What email address should we use for contact?

User: john.smith@gmail.com

Bot: Perfect! I'm enrolling you in Computer Science now...
     [Executes enrollment.bas tool]
     Enrollment complete! Welcome to Computer Science, John Smith!
     You'll receive a confirmation at john.smith@gmail.com.

User: What are the payment options?

Bot: [Searches payment-guide.txt from .gbkb/]
     According to our payment guide, we offer three options:
     1. Full payment with 10% discount
     2. Monthly installments over 12 months
     3. Semester-based payments
     Which option would you prefer?
```

## The Magic Formula

Documents in `.gbkb/` combined with tools in `.bas` files and the LLM create an intelligent bot. No complex programming needed!

## Creating Your First Bot

Start by creating the directory structure for your bot with folders for dialog scripts, knowledge base documents, and bot configuration. Add your documents to the `.gbkb/` directory including PDFs, Word documents, text files, and Markdown files. Optionally create tools as `.bas` files to handle specific actions like processing forms or calling APIs. Then restart botserver and start chatting. The LLM will answer questions from your documents, call your tools when appropriate, and handle the entire conversation naturally.

```bash
mkdir -p mybot.gbai/mybot.gbdialog
mkdir -p mybot.gbai/mybot.gbkb/docs
mkdir -p mybot.gbai/mybot.gbot
```

Example tool in `mybot.gbdialog/my-tool.bas`:

```basic
PARAM user_name AS string
PARAM request AS string

DESCRIPTION "Handles user requests"

result = CALL "/api/process", user_name, request
TALK "Done! " + result
```

## No Programming Required

Traditional chatbots require complex logic with IF/THEN statements, intent detection, and multi-step dialog management. With botserver, you simply create the tool with parameters and a description, and the LLM handles all the conversation logic automatically.

Traditional approach (don't do this):

```basic
' Complex multi-step dialog
IF intent = "enrollment" THEN
    TALK "Let me help you enroll. What's your name?"
    HEAR name
    TALK "What's your email?"
    HEAR email
    ' ... lots more code ...
ENDIF
```

botserver approach (just create the tool):

```basic
' In enrollment.bas - becomes a tool automatically
PARAM name AS string
PARAM email AS string
DESCRIPTION "Collects enrollment information"

SAVE "enrollments.csv", name, email
TALK "Successfully enrolled " + name
```

## What Can You Build?

A customer support bot uses product manuals in `.gbkb/` and a `create-ticket.bas` tool, allowing the LLM to answer questions and create support tickets automatically.

An HR assistant combines the employee handbook in `.gbkb/` with a `leave-request.bas` tool so the LLM can explain policies and process leave requests.

An education platform stores course materials in `.gbkb/` and provides `enrollment.bas` and `submit-assignment.bas` tools, enabling the LLM to teach content and manage student tasks.

A sales assistant uses product catalogs in `.gbkb/` with a `create-quote.bas` tool, allowing the LLM to answer product questions and generate quotes.

## Advanced Features

### Dynamic Tool Loading

The LLM can load tools based on context. In `start.bas`, you simply specify which knowledge bases to use, and tools in `.gbdialog/` are auto-discovered. The LLM handles the conversation naturally without explicit HEAR statements.

### Multi-Language Support

The LLM handles multiple languages automatically. Users can write in Portuguese, Chinese, or any other language, and the bot responds appropriately in the same language.

### Context Awareness

The LLM maintains conversation context throughout the interaction. If a user starts to enroll but then asks about prerequisites, the bot handles the tangent and can return to the enrollment process afterward.

## Tips for Success

Organize documents clearly by creating folders for policies, products, FAQs, and tutorials within your `.gbkb/` directory. This helps the LLM find relevant information quickly.

Name tools descriptively with names like `enrollment.bas`, `create-ticket.bas`, and `schedule-meeting.bas`. The LLM understands what each tool does from its name and description.

Always add descriptions to tools using the DESCRIPTION keyword. A good description like "This tool processes student enrollment for courses" helps the LLM know when to use the tool.

Let the LLM work without trying to control every aspect of the conversation. Allow it to rephrase responses naturally, handle unexpected questions, and maintain conversation flow on its own.

## Next Steps

The Quick Start guide walks you through building your first bot. The Packages chapter explains the package structure in detail. The Tool Definition documentation covers creating sophisticated tools. The Knowledge Base chapter describes document management and indexing.

Remember: Just add documents and tools, and the LLM does the rest!