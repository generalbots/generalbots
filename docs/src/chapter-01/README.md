# Chapter 01: Run and Talk

Getting started with BotServer is incredibly simple: **just run it!**

## Quick Start

```bash
# Download and run
./botserver

# Or build from source
cargo run
```

That's it! The bootstrap process handles everything automatically.

## What You'll Learn

This chapter covers everything you need to get started:

1. **[Installation](./installation.md)** - How the automatic bootstrap works
2. **[First Conversation](./first-conversation.md)** - Start chatting with your bot
3. **[Understanding Sessions](./sessions.md)** - How conversations are managed

## The Bootstrap Magic

When you first run BotServer, it automatically:

- ✅ Detects your operating system
- ✅ Installs PostgreSQL database
- ✅ Installs MinIO object storage
- ✅ Installs Valkey cache
- ✅ Generates secure credentials
- ✅ Creates default bots
- ✅ Starts the web server

**No manual configuration needed!** Everything just works.

## Your First Bot

After bootstrap completes (2-5 minutes), open your browser to:

```
http://localhost:8080
```

You'll see the default bot ready to chat! Just start talking - the LLM handles everything.

## The Magic Formula

```
📚 Documents (.gbkb/) + 🔧 Tools (.bas) + 🤖 LLM = ✨ Intelligent Bot
```

**No programming required!** Just:
1. Drop documents in `.gbkb/` folders
2. Create simple tools as `.bas` files (optional)
3. Start chatting - the LLM does the rest!

## Example: Student Enrollment Bot

### 1. Add Course Documents

```
edu.gbai/
  edu.gbkb/
    policies/
      enrollment-policy.pdf
      course-catalog.pdf
```

### 2. Create Enrollment Tool

`edu.gbdialog/enrollment.bas`:

```bas
PARAM name AS string     LIKE "John Smith"        DESCRIPTION "Student name"
PARAM email AS string    LIKE "john@example.com"  DESCRIPTION "Email"
PARAM course AS string   LIKE "Computer Science"  DESCRIPTION "Course"

DESCRIPTION "Processes student enrollment"

SAVE "enrollments.csv", name, email, course, NOW()
TALK "Welcome to " + course + ", " + name + "!"
```

### 3. Just Chat!

```
User: I want to enroll in computer science
Bot: I'll help you enroll! What's your name?
User: John Smith
Bot: Thanks John! What's your email?
User: john@example.com
Bot: [Executes enrollment.bas]
     Welcome to Computer Science, John Smith!
```

The LLM automatically:
- ✅ Understands user wants to enroll
- ✅ Calls the enrollment tool
- ✅ Collects required parameters
- ✅ Executes when ready
- ✅ Answers questions from your documents

## Key Concepts

### Tools = Just `.bas` Files

A **tool** is simply a `.bas` file that the LLM discovers and calls automatically.

### Knowledge = Just Documents

Drop PDFs, Word docs, or text files in `.gbkb/` - instant searchable knowledge base!

### Sessions

Each conversation is a **session** that persists:
- User identity (authenticated or anonymous)
- Conversation history
- Context and variables
- Active tools and knowledge bases

Sessions automatically save to PostgreSQL and cache in Redis for performance.

## Next Steps

- **[Installation](./installation.md)** - Understand the bootstrap process
- **[First Conversation](./first-conversation.md)** - Try out your bot
- **[Understanding Sessions](./sessions.md)** - Learn about conversation state
- **[About Packages](../chapter-02/README.md)** - Create your own bots

## Philosophy

BotServer follows these principles:

1. **Just Run It** - Bootstrap handles everything
2. **Just Chat** - No complex dialog flows needed
3. **Just Add Content** - Documents + tools = intelligent bot
4. **LLM Does the Work** - No IF/THEN logic required
5. **Production Ready** - Built for real-world use

Ready to get started? Head to [Installation](./installation.md)!