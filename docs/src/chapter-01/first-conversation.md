# First Conversation

## Starting a Session
After the server is running, open a web browser at `http://localhost:8080` to begin. The system will automatically create a new session and load the default dialog (`start.bas`).

## Basic Interaction
The default greeting demonstrates the `TALK` keyword:
```basic
TALK "Welcome to GeneralBots! How can I assist you today?"
```

## Example Dialog Flow
Here's a more complete example showing question handling:
```basic
TALK "Hello! I'm your GeneralBots assistant."
HEAR user_input

IF user_input CONTAINS "help" THEN
    TALK "I can help with:"
    TALK "- Answering questions"
    TALK "- Finding information"
    TALK "- Running tools"
ELSE IF user_input CONTAINS "time" THEN
    CALL GET_CURRENT_TIME
    TALK "The current time is " + time_result
ELSE
    TALK "I didn't understand. Try asking for 'help'."
ENDIF
```

## Key Features to Try
1. **Basic Responses**: Simple question/answer
2. **Conditional Logic**: Different responses based on input
3. **Tool Integration**: Calling external functions (`CALL` keyword)
4. **Variables**: Storing and using conversation state

## Troubleshooting
- If you don't get a response, check:
  - The server is running (`botserver status`)
  - The LLM service is available (`botserver status llm`)
  - There are no errors in the console

