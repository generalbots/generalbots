# Chapter 06: BASIC + LLM - The Perfect Match

## Why BASIC?

In 1964, John Kemeny and Thomas Kurtz created BASIC with a revolutionary idea: programming should be for everyone. Today, General Bots brings this philosophy to the AI era.

**With BASIC + LLM, you write:**

```basic
TALK "What's your name?"
HEAR name
poem = LLM "Create a heartfelt poem for " + name
TALK poem
```

**Not 50 lines of boilerplate.**

## The Core Keywords

Just SEVEN main keywords power everything:

| Keyword | Purpose | Example |
|---------|---------|---------|
| **TALK** | Output | `TALK "Hello!"` |
| **HEAR** | Input | `HEAR name AS NAME` |
| **USE KB** | Knowledge | `USE KB "docs"` |
| **USE TOOL** | Functions | `USE TOOL "weather"` |
| **GET** | Data | `GET "api/users"` |
| **IF/THEN** | Logic | `IF age >= 18 THEN ...` |
| **FOR/NEXT** | Loops | `FOR i = 1 TO 10 ...` |

## Your First Tool

In the LLM world, you write tools that AI can use:

```basic
' enrollment.bas - An LLM-callable tool
PARAM name AS STRING LIKE "John Smith" DESCRIPTION "Full name"
PARAM email AS STRING LIKE "john@example.com" DESCRIPTION "Email"
PARAM course AS STRING LIKE "Introduction to AI" DESCRIPTION "Course"

DESCRIPTION "Enrolls a student in a course"

SAVE "enrollments.csv", name, email, course, NOW()
TALK "Enrolled " + name + " in " + course
```

The LLM handles the conversation. You define the action.

## Everyone Can Program

**You don't need:**
- A computer science degree
- Years of experience
- Understanding of algorithms

**You just need:**
- An idea
- 10 minutes to learn BASIC
- Creativity

## Getting Started

| Time | Goal |
|------|------|
| Minute 1 | `TALK "Hello, world!"` |
| Minute 5 | Add HEAR and LLM |
| Day 1 | Production-ready bot |

## Documentation Guide

| Document | Purpose |
|----------|---------|
| [Basics](./basics.md) | Core LLM-first concepts |
| [Keywords Reference](./keywords.md) | Complete keyword list |
| [Templates](./templates.md) | Real-world examples |
| [Universal Messaging](./universal-messaging.md) | Multi-channel support |

### Keyword Categories

- **Core:** [TALK](./keyword-talk.md), [HEAR](./keyword-hear.md)
- **Context:** [SET CONTEXT](./keyword-set-context.md), [USE KB](./keyword-use-kb.md)
- **Memory:** [GET/SET BOT MEMORY](./keyword-get-bot-memory.md), [GET/SET USER MEMORY](./keyword-get-user-memory.md)
- **Data:** [GET](./keyword-get.md), [SAVE](./keyword-save.md), [FIND](./keyword-find.md)
- **HTTP:** [POST](./keyword-post.md), [PUT](./keyword-put.md), [DELETE](./keyword-delete-http.md)
- **Files:** [READ](./keyword-read.md), [WRITE](./keyword-write.md), [UPLOAD](./keyword-upload.md)

## The Philosophy

BASIC in General Bots isn't about controlling conversation flow - it's about providing tools and context that LLMs use intelligently.

**Write simple tools. Let AI handle the complexity.**

---

*"The beauty of BASIC lies not in what it can do, but in who it enables to do it."*