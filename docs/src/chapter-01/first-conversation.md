# First Conversation

After BotServer starts, you can immediately start chatting with your bot. No programming required!

## Just Start Talking

Open your browser to:
```
http://localhost:8080
```

And start chatting:

```
You: Hi!
Bot: Hello! How can I help you today?

You: I want to enroll in a course
Bot: I'll help you with enrollment. Let me collect your information...
     [Bot automatically calls enrollment.bas tool]

You: What documents do you have?
Bot: [Searches .gbkb/ folders and answers from your documents]
```

**That's it!** The LLM handles everything automatically.

## How It Works

### 1. Drop Documents in `.gbkb/`

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

The bot automatically:
- ✅ Indexes all documents
- ✅ Creates vector embeddings
- ✅ Searches when users ask questions
- ✅ Provides accurate answers from your content

### 2. Create Tools as `.bas` Files

Create `enrollment.bas`:

```bas
PARAM name AS string          LIKE "John Smith"           DESCRIPTION "Student full name"
PARAM email AS string         LIKE "john@example.com"     DESCRIPTION "Contact email"
PARAM course AS string        LIKE "Computer Science"     DESCRIPTION "Course to enroll in"

DESCRIPTION "Enrollment tool - collects student information and processes enrollment"

SAVE "enrollments.csv", name, email, course, NOW()
TALK "Enrollment complete! Welcome to " + course + ", " + name
```

The LLM automatically:
- ✅ Discovers this tool
- ✅ Knows when to call it (when users want to enroll)
- ✅ Collects required parameters through conversation
- ✅ Executes the tool when ready

### 3. The LLM Does Everything Else

You **DON'T** need to write:
- ❌ IF/THEN logic
- ❌ Intent detection
- ❌ Dialog flows
- ❌ State machines
- ❌ Complex routing

The LLM automatically:
- Understands user intent
- Calls appropriate tools
- Searches knowledge bases
- Maintains conversation context
- Handles follow-up questions

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

```
📚 Documents (.gbkb/) + 🔧 Tools (.bas) + 🤖 LLM = ✨ Intelligent Bot
```

No complex programming needed!

## Creating Your First Bot

### Step 1: Create Structure

```bash
mkdir -p mybot.gbai/mybot.gbdialog
mkdir -p mybot.gbai/mybot.gbkb/docs
mkdir -p mybot.gbai/mybot.gbot
```

### Step 2: Add Documents

Drop any documents in `.gbkb/`:
- PDFs
- Word documents  
- Text files
- Markdown files

### Step 3: Create a Tool (Optional)

Create `mybot.gbdialog/my-tool.bas`:

```bas
PARAM user_name AS string
PARAM request AS string

DESCRIPTION "Handles user requests"

result = CALL "/api/process", user_name, request
TALK "Done! " + result
```

### Step 4: Start Chatting

Restart BotServer and chat! The LLM will:
- Answer questions from your documents
- Call your tools when appropriate
- Handle the entire conversation naturally

## No Programming Required

Traditional chatbots require complex logic:

```bas
' ❌ OLD WAY - DON'T DO THIS!
IF user_input CONTAINS "enroll" THEN
  TALK "What's your name?"
  HEAR name
  TALK "What's your email?"
  HEAR email
  ' ... lots more code ...
ENDIF
```

With BotServer:

```bas
' ✅ NEW WAY - Just create the tool!
PARAM name AS string
PARAM email AS string
DESCRIPTION "Enrollment tool"
SAVE "enrollments.csv", name, email
```

The LLM handles all the conversation logic!

## What Can You Build?

### Customer Support Bot
- Add product manuals to `.gbkb/`
- Create `create-ticket.bas` tool
- LLM answers questions and creates support tickets

### HR Assistant
- Add employee handbook to `.gbkb/`
- Create `leave-request.bas` tool
- LLM explains policies and processes leave requests

### Education Platform
- Add course materials to `.gbkb/`
- Create `enrollment.bas` and `submit-assignment.bas` tools
- LLM teaches content and manages student tasks

### Sales Assistant
- Add product catalogs to `.gbkb/`
- Create `create-quote.bas` tool
- LLM answers product questions and generates quotes

## Advanced Features

### Dynamic Tool Loading

The LLM can load tools based on context:

```bas
' In start.bas - minimal setup
USE_KB "general"  ' Load general knowledge base
' Tools are auto-discovered from .gbdialog/ folder
```

### Multi-Language Support

The LLM handles multiple languages automatically:

```
User: Olá, quero me inscrever no curso
Bot: Claro! Vou ajudá-lo com a inscrição...

User: 我想了解计算机科学课程
Bot: 我很乐意帮您了解计算机科学课程...
```

### Context Awareness

The LLM maintains conversation context:

```
User: I want to enroll
Bot: I'll help you enroll. What's your name?
User: Actually, first tell me about the prerequisites
Bot: Of course! The Computer Science program requires...
```

## Tips for Success

### 1. Organize Documents Clearly

```
mybot.gbkb/
  policies/      # Policy documents
  products/      # Product information
  faqs/          # Frequently asked questions
  tutorials/     # How-to guides
```

### 2. Name Tools Descriptively

Good tool names:
- `enrollment.bas`
- `create-ticket.bas`
- `schedule-meeting.bas`

The LLM understands what each tool does from its name and description.

### 3. Use Tool Descriptions

Always add descriptions to tools:

```bas
DESCRIPTION "This tool processes student enrollment for courses"
```

### 4. Let the LLM Work

Don't try to control every aspect of the conversation. Let the LLM:
- Rephrase responses naturally
- Handle unexpected questions
- Maintain conversation flow

## Next Steps

- [Understanding Sessions](./sessions.md) - How conversations persist
- [About Packages](../chapter-02/README.md) - Package structure
- [Tool Definition](../chapter-08/tool-definition.md) - Creating tools
- [Knowledge Base](../chapter-03/README.md) - Document management

Remember: **Just add documents and tools - the LLM does the rest!** 🚀