# LLM Keyword

**Syntax**

```
LLM "prompt text"
```

**Parameters**

- `"prompt text"` – The text that will be sent to the configured Large Language Model (LLM) provider.

**Description**

`LLM` forwards the supplied prompt to the LLM service defined in the application configuration (`.gbot/config.csv`). The LLM processes the prompt and returns a response string, which is then made available to the script as the result of the keyword.

The keyword runs the LLM call in a background thread with its own Tokio runtime to avoid blocking the main engine. If the LLM provider returns an error or times out, a runtime error is raised.

**Example**

```basic
SET topic = "GeneralBots platform"
TALK "Generating summary for " + topic + "..."
SET summary = LLM "Summarize the following: " + topic
TALK summary
```

The script asks the LLM to summarize the topic and then outputs the generated summary.

**Implementation Notes**

- The prompt is wrapped in a standard instruction that tells the model to act as a BASIC keyword assistant.
- The keyword returns the raw response text; any formatting must be handled by the script (e.g., using `FORMAT`).
- Network errors, timeouts, or provider failures result in a runtime error.
