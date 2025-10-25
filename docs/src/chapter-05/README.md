# Chapter 05: gbdialog Reference

This chapter covers the BASIC scripting language used in .gbdialog files to create conversational flows, integrate tools, and manage bot behavior.

## BASIC Language Overview

GeneralBots uses a specialized BASIC dialect designed for conversational AI. The language provides:

- **Simple Syntax**: English-like commands that are easy to understand
- **Conversation Focus**: Built-in primitives for dialog management
- **Tool Integration**: Seamless calling of external functions
- **AI Integration**: Direct access to LLM capabilities
- **Data Manipulation**: Variables, loops, and conditionals

## Language Characteristics

- **Case Insensitive**: `TALK`, `talk`, and `Talk` are equivalent
- **Line-Oriented**: Each line represents one command or statement
- **Dynamic Typing**: Variables automatically handle different data types
- **Sandboxed Execution**: Safe runtime environment with resource limits

## Basic Concepts

### Variables
Store and manipulate data:
```basic
SET user_name = "John"
SET item_count = 5
SET price = 19.99
```

### Control Flow
Make decisions and repeat actions:
```basic
IF user_role = "admin" THEN
    TALK "Welcome administrator!"
ELSE
    TALK "Welcome user!"
END IF

FOR EACH item IN shopping_cart
    TALK "Item: " + item
NEXT item
```

### User Interaction
Communicate with users:
```basic
TALK "Hello! What's your name?"
HEAR user_name
TALK "Nice to meet you, " + user_name
```

The following sections provide detailed reference for each keyword and feature available in the BASIC scripting language.
